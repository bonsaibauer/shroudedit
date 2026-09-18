#include "shroudedit/editor.h"
#include <algorithm>
#include <cmath>
#include <stdexcept>
#include <unordered_set>

namespace shroudedit {
namespace {
    StringView View(const std::string& text) { return {text.data(), text.size()}; }
    WorldVec3 Api(Vec3 v) { return {v.x, v.y, v.z}; }
    bool Near(double a, double b) { return std::abs(a - b) <= 1e-7; }
    bool Same(const WorldTransform& a, const WorldTransform& b) {
        return Near(a.position.x,b.position.x) && Near(a.position.y,b.position.y) && Near(a.position.z,b.position.z) &&
            Near(a.rotation.x,b.rotation.x) && Near(a.rotation.y,b.rotation.y) && Near(a.rotation.z,b.rotation.z) && Near(a.rotation.w,b.rotation.w) &&
            Near(a.scale.x,b.scale.x) && Near(a.scale.y,b.scale.y) && Near(a.scale.z,b.scale.z);
    }
    bool Ready(const WorldApi& world) { return world.struct_size >= sizeof(world) && world.version == WorldApiVersion; }
    WorldTransform Api(const Transform& t) {
        return {sizeof(WorldTransform), Api(t.position), {t.rotation.x,t.rotation.y,t.rotation.z,t.rotation.w}, Api(t.scale)};
    }
    bool Aligned(double value, double origin, double step) {
        if (!std::isfinite(step) || step <= 0 || !std::isfinite(origin) || !std::isfinite(value)) return false;
        const auto coordinate = (value-origin)/step;
        return std::isfinite(coordinate) && Near(coordinate, std::round(coordinate));
    }
}
Editor::Editor(std::string owner, ValidationLimits limits) : owner_(std::move(owner)), limits_(limits) {}
StringView Editor::owner() const { return View(owner_); }
void Editor::reset() { selection.clear(); clipboard_.reset(); preview_.reset(); history_.reset(); recovery_ = false; }
void Editor::set_blueprint(Blueprint blueprint) {
    const auto errors = validate(blueprint, limits_);
    if (!errors.empty()) throw std::invalid_argument(errors.front().message);
    if (recovery_) throw std::runtime_error("restore the incomplete edit before loading another blueprint");
    clipboard_ = std::move(blueprint); preview_.reset();
}
void Editor::prepare(Vec3 target, std::uint8_t turns) {
    if (!clipboard_) throw std::invalid_argument("no blueprint loaded");
    if (recovery_) throw std::runtime_error("restore the incomplete edit first");
    preview_ = make_placement_plan(*clipboard_, target, turns);
}
Result Editor::capture(const WorldApi& world, bool props, const std::vector<std::string>& grids, UpAxis up_axis) {
    if (!Ready(world)) return RESULT_VERSION_MISMATCH;
    if (recovery_ || !selection.complete() || (!props && grids.empty())) return RESULT_INVALID_ARGUMENT;
    if ((props && !world.query_entities) || (!grids.empty() && (!world.get_grid_spec || !world.read_grid_region))) return RESULT_NOT_AVAILABLE;
    const auto region = *selection.region();
    const WorldBounds bounds{sizeof(bounds), Api(region.minimum), Api(region.maximum)};
    Blueprint result;
    result.id = "shroudedit.capture"; result.name = "Captured volume"; result.up_axis = up_axis;
    result.extent = {region.maximum.x-region.minimum.x, region.maximum.y-region.minimum.y, region.maximum.z-region.minimum.z};
    if (!std::isfinite(result.extent.x) || !std::isfinite(result.extent.y) || !std::isfinite(result.extent.z)) return RESULT_INVALID_ARGUMENT;
    if (props) {
        // Voxel selections may be one cell thick. Query props over the same
        // inclusive voxel bounds instead of rejecting equal A/B coordinates.
        WorldBounds propBounds=bounds;
        if (!grids.empty()) {
            WorldGridSpec spec{sizeof(spec)};
            auto status=world.get_grid_spec(owner(),View(grids.front()),&spec);
            if (status!=RESULT_OK) return status;
            WorldGridRegion cells{sizeof(cells)};
            status=GridRegionFromPoints(&spec,Api(*selection.a()),Api(*selection.b()),limits_.maximum_cells,&cells);
            if (status!=RESULT_OK) return status;
            propBounds=cells.bounds;
        } else if (result.extent.x <= 0 || result.extent.y <= 0 || result.extent.z <= 0) return RESULT_INVALID_ARGUMENT;
        size_t count = 0;
        auto status = world.query_entities(owner(), &propBounds, nullptr, 0, &count);
        if (status != RESULT_OK) return status;
        if (count > limits_.maximum_props) return RESULT_INVALID_ARGUMENT;
        if (count) {
            std::vector<WorldEntity> entities(count);
            for (auto& entity : entities) { entity.struct_size = sizeof(entity); entity.transform.struct_size = sizeof(entity.transform); }
            status = world.query_entities(owner(), &propBounds, entities.data(), entities.size(), &count);
            if (status != RESULT_OK) return status;
            if (count > entities.size()) return RESULT_INTERNAL_ERROR;
            for (size_t i = 0; i < count; ++i) {
                const auto& entity = entities[i];
                if (entity.kind == WorldEntityKind::Other) continue;
                if (entity.kind != WorldEntityKind::Prop || entity.struct_size < sizeof(entity)) return RESULT_NOT_AVAILABLE;
                if (!entity.template_id.data || !entity.template_id.size || entity.template_id.size > limits_.maximum_string_bytes)
                    return RESULT_INVALID_ARGUMENT;
                const auto& t = entity.transform;
                // Copy template strings before any subsequent API call invalidates them.
                result.props.push_back({i+1, std::string(entity.template_id.data,entity.template_id.size),
                    {{t.position.x-region.minimum.x,t.position.y-region.minimum.y,t.position.z-region.minimum.z},
                    {t.rotation.x,t.rotation.y,t.rotation.z,t.rotation.w},{t.scale.x,t.scale.y,t.scale.z}}});
            }
        }
    }
    size_t remaining = static_cast<size_t>(limits_.maximum_cells);
    for (const auto& id : grids) {
        WorldGridSpec spec{sizeof(spec)};
        auto status = world.get_grid_spec(owner(), View(id), &spec);
        if (status != RESULT_OK) return status;
        WorldGridRegion cells{sizeof(cells)};
        status = GridRegionFromPoints(&spec, Api(*selection.a()), Api(*selection.b()), remaining, &cells);
        if (status != RESULT_OK) return status;
        remaining -= cells.cell_count;
        GridEdit edit; edit.id = id; edit.bounds = cells.bounds;
        status = read_grid(world, edit, cells.cell_count);
        if (status != RESULT_OK) return status;
        GridChannel channel;
        channel.id = id;
        channel.dimensions = {static_cast<int32_t>(cells.dimensions[0]), static_cast<int32_t>(cells.dimensions[1]), static_cast<int32_t>(cells.dimensions[2])};
        channel.origin = {cells.bounds.minimum.x-region.minimum.x,cells.bounds.minimum.y-region.minimum.y,cells.bounds.minimum.z-region.minimum.z};
        channel.cell_size = {spec.cell_size.x,spec.cell_size.y,spec.cell_size.z};
        channel.values = std::move(edit.before);
        for (const auto coverage : edit.before_coverage) channel.coverage.push_back(static_cast<Coverage>(coverage));
        result.channels.push_back(std::move(channel));
    }
    if (!validate(result, limits_).empty()) return RESULT_INVALID_ARGUMENT;
    clipboard_ = std::move(result); preview_.reset();
    return RESULT_OK;
}

Result Editor::read_grid(const WorldApi& world, GridEdit& edit, size_t count) {
    edit.before.assign(count, 0); edit.before_coverage.assign(count, WorldCellState::Unknown);
    size_t actual = 0;
    const auto status = world.read_grid_region(owner(), View(edit.id), &edit.bounds,
        edit.before.data(), edit.before_coverage.data(), count, &actual);
    if (status != RESULT_OK) return status;
    if (actual != count) return RESULT_INTERNAL_ERROR;
    for (size_t i=0;i<count;++i) {
        const auto coverage = edit.before_coverage[i];
        if (coverage < WorldCellState::Unknown || coverage > WorldCellState::Occupied) return RESULT_INTERNAL_ERROR;
        if (coverage != WorldCellState::Occupied) edit.before[i] = 0;
    }
    return RESULT_OK;
}

EditResult Editor::paste(const WorldApi& world, PasteMode mode) {
    if (!Ready(world)) return {RESULT_VERSION_MISMATCH};
    if (!preview_ || recovery_) return {RESULT_INVALID_ARGUMENT};
    if (mode==PasteMode::Additive && preview_->props.empty() &&
        std::none_of(preview_->channels.begin(),preview_->channels.end(),[](const auto& channel) {
            return std::find(channel.coverage.begin(),channel.coverage.end(),Coverage::Occupied)!=channel.coverage.end();
        })) return {RESULT_NOT_FOUND};
    const bool replaceAll=mode==PasteMode::ReplaceAll;
    if (((!preview_->props.empty() || replaceAll) && (!world.spawn_entity || !world.destroy_entity || !world.get_entity_transform)) ||
        (replaceAll && !world.query_entities) ||
        (!preview_->channels.empty() && (!world.get_grid_spec || !world.read_grid_region || !world.write_grid_region))) return {RESULT_NOT_AVAILABLE};
    History prepared;
    prepared.props.reserve(preview_->props.size());
    if (replaceAll) {
        auto minimum=preview_->region_minimum,maximum=preview_->region_maximum;
        for (const auto& channel:preview_->channels) {
            minimum.x=(std::min)(minimum.x,channel.origin.x); minimum.y=(std::min)(minimum.y,channel.origin.y);
            minimum.z=(std::min)(minimum.z,channel.origin.z);
            maximum.x=(std::max)(maximum.x,channel.origin.x+channel.dimensions.x*channel.cell_size.x);
            maximum.y=(std::max)(maximum.y,channel.origin.y+channel.dimensions.y*channel.cell_size.y);
            maximum.z=(std::max)(maximum.z,channel.origin.z+channel.dimensions.z*channel.cell_size.z);
        }
        const WorldBounds target{sizeof(target),Api(minimum),Api(maximum)};
        std::size_t count{};
        auto status=world.query_entities(owner(),&target,nullptr,0,&count);
        if (status!=RESULT_OK) return {status};
        if (count>limits_.maximum_props) return {RESULT_INVALID_ARGUMENT};
        std::vector<WorldEntity> entities(count);
        for (auto& entity:entities) { entity.struct_size=sizeof(entity); entity.transform.struct_size=sizeof(entity.transform); }
        if (count) status=world.query_entities(owner(),&target,entities.data(),entities.size(),&count);
        if (status!=RESULT_OK) return {status};
        if (count>entities.size()) return {RESULT_INTERNAL_ERROR};
        for (std::size_t i=0;i<count;++i) {
            const auto& entity=entities[i];
            if (entity.kind==WorldEntityKind::Other) continue;
            if (entity.kind!=WorldEntityKind::Prop || !entity.handle.id || !entity.template_id.data ||
                !entity.template_id.size || entity.template_id.size>limits_.maximum_string_bytes) return {RESULT_NOT_AVAILABLE};
            prepared.removed.push_back({entity.handle,std::string(entity.template_id.data,entity.template_id.size),entity.transform,false});
        }
    }
    for (const auto& channel : preview_->channels) {
        WorldGridSpec spec{sizeof(spec)};
        auto status = world.get_grid_spec(owner(), View(channel.id), &spec);
        if (status != RESULT_OK) return {status};
        if (!Near(spec.cell_size.x,channel.cell_size.x) || !Near(spec.cell_size.y,channel.cell_size.y) || !Near(spec.cell_size.z,channel.cell_size.z) ||
            !Aligned(channel.origin.x,spec.origin.x,spec.cell_size.x) || !Aligned(channel.origin.y,spec.origin.y,spec.cell_size.y) ||
            !Aligned(channel.origin.z,spec.origin.z,spec.cell_size.z)) return {RESULT_INVALID_ARGUMENT};
        GridEdit edit; edit.id = channel.id;
        edit.bounds = {sizeof(WorldBounds), Api(channel.origin),
            {channel.origin.x+channel.dimensions.x*channel.cell_size.x,
             channel.origin.y+channel.dimensions.y*channel.cell_size.y,
             channel.origin.z+channel.dimensions.z*channel.cell_size.z}};
        status = read_grid(world, edit, channel.values.size());
        if (status != RESULT_OK) return {status};
        edit.after = edit.before; edit.after_coverage = edit.before_coverage;
        for (size_t i = 0; i < channel.values.size(); ++i) {
            if (channel.coverage[i] == Coverage::Unknown || (mode == PasteMode::Additive && channel.coverage[i] == Coverage::Empty)) continue;
            if (edit.before_coverage[i] == WorldCellState::Unknown) return {RESULT_NOT_FOUND};
            // "Additive" means empty source cells do not erase the target.
            // Occupied source cells must still be allowed to replace existing
            // terrain/building voxels, otherwise walls cannot join floors or
            // existing structures.
            edit.after[i] = channel.coverage[i] == Coverage::Empty ? 0 : channel.values[i];
            edit.after_coverage[i] = static_cast<WorldCellState>(channel.coverage[i]);
        }
        prepared.grids.push_back(std::move(edit));
    }
    // One bounded history entry. All target grids were read before the first write.
    history_ = std::move(prepared);
    auto fail = [&](Result error) { auto restored = restore(world, false); restored.result = error; return restored; };
    for (auto& prop:history_->removed) {
        const auto status=world.destroy_entity(owner(),prop.handle);
        if (status==RESULT_OK) prop.applied=true;
        else {
            WorldTransform current{sizeof(current)};
            prop.applied=world.get_entity_transform(owner(),prop.handle,&current)==RESULT_NOT_FOUND;
            return fail(status);
        }
    }
    for (auto& edit : history_->grids) {
        edit.applied = true; // A failed write can still have modified part of the region.
        const auto status = world.write_grid_region(owner(), View(edit.id), &edit.bounds, edit.after.data(), edit.after_coverage.data(), edit.after.size());
        if (status != RESULT_OK) return fail(status);
    }
    for (const auto& prop : preview_->props) {
        const auto transform = Api(prop.transform);
        WorldEntityHandle handle{};
        const auto status = world.spawn_entity(owner(), View(prop.template_id), &transform, &handle);
        if (status != RESULT_OK) return fail(status);
        history_->props.push_back({handle,transform});
    }
    const EditResult result{RESULT_OK,history_->props.size(),history_->grids.size(),false};
    preview_.reset();
    return result;
}

EditResult Editor::restore(const WorldApi& world, bool check_current) {
    if (!history_) return {RESULT_NOT_FOUND};
    if (check_current) {
        // Refuse undo when another edit changed any tracked target.
        for (const auto& prop : history_->props) {
            WorldTransform current{sizeof(current)};
            const auto status = world.get_entity_transform(owner(), prop.handle, &current);
            if (status != RESULT_OK) return {status,0,0,recovery_};
            if (!Same(current,prop.transform)) return {RESULT_CONFLICT,0,0,recovery_};
        }
        for (const auto& prop:history_->removed) if (prop.applied) {
            WorldTransform current{sizeof(current)};
            const auto status=world.get_entity_transform(owner(),prop.handle,&current);
            if (status==RESULT_OK) return {RESULT_CONFLICT,0,0,recovery_};
            if (status!=RESULT_NOT_FOUND) return {status,0,0,recovery_};
        }
        for (const auto& edit : history_->grids) {
            if (!edit.applied) continue;
            GridEdit current; current.id = edit.id; current.bounds = edit.bounds;
            const auto status = read_grid(world,current,edit.after.size());
            if (status != RESULT_OK) return {status,0,0,recovery_};
            for (size_t i=0;i<edit.after.size();++i) {
                const bool after = current.before[i] == edit.after[i] && current.before_coverage[i] == edit.after_coverage[i];
                const bool before = current.before[i] == edit.before[i] && current.before_coverage[i] == edit.before_coverage[i];
                if (!after && !(recovery_ && before)) return {RESULT_CONFLICT,0,0,recovery_};
            }
        }
    }
    while (!history_->props.empty()) {
        const auto status = world.destroy_entity(owner(),history_->props.back().handle);
        if (status != RESULT_OK) { recovery_ = true; return {status,history_->props.size(),0,true}; }
        history_->props.pop_back();
    }
    for (auto it = history_->grids.rbegin(); it != history_->grids.rend(); ++it) {
        if (!it->applied) continue;
        const auto status = world.write_grid_region(owner(),View(it->id),&it->bounds,it->before.data(),it->before_coverage.data(),it->before.size());
        if (status != RESULT_OK) { recovery_ = true; return {status,0,0,true}; }
        it->applied = false;
    }
    for (auto it=history_->removed.rbegin();it!=history_->removed.rend();++it) {
        if (!it->applied) continue;
        WorldEntityHandle restored{};
        const auto status=world.spawn_entity(owner(),View(it->template_id),&it->transform,&restored);
        if (status!=RESULT_OK) { recovery_=true; return {status,0,0,true}; }
        it->applied=false;
    }
    history_.reset(); recovery_ = false;
    return {};
}
EditResult Editor::undo(const WorldApi& world) {
    if (!Ready(world)) return {RESULT_VERSION_MISMATCH};
    if (!history_) return {RESULT_NOT_FOUND};
    if ((!history_->props.empty() && (!world.get_entity_transform || !world.destroy_entity)) ||
        (!history_->removed.empty() && (!world.get_entity_transform || !world.spawn_entity)) ||
        (!history_->grids.empty() && (!world.read_grid_region || !world.write_grid_region))) return {RESULT_NOT_AVAILABLE};
    return restore(world, true);
}
}
