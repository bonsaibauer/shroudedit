#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shroudedit/editor.h"
#include "shroudedit/blueprint_codec.h"
#include <cassert>
#include <cmath>
#include <cstring>
#include <iostream>
#include <limits>
#include <map>

using namespace shroudedit;
namespace {
std::vector<uint32_t> cells;
std::vector<WorldCellState> coverage;
std::map<uint64_t,WorldTransform> props;
uint64_t nextId=100;
int spawnCalls=0,failSpawn=0,writeCalls=0,failWrite=0;
bool failDestroy=false;
bool targetPresent=true;
Result CALL Query(StringView,const WorldBounds*,WorldEntity* buffer,size_t capacity,size_t* required) {
    *required=targetPresent ? 1 : 0;
    if (!targetPresent) return RESULT_OK;
    if (!capacity) return RESULT_OK;
    if (!buffer || capacity<1) return RESULT_INVALID_ARGUMENT;
    static const char id[]="template.table";
    buffer[0]={sizeof(*buffer),{10,1,1},{id,sizeof(id)-1},
        {sizeof(WorldTransform),{1,2,3},{0,0,0,1},{2,3,4}},WorldEntityKind::Prop};
    return RESULT_OK;
}
Result CALL Get(StringView,WorldEntityHandle handle,WorldTransform* out) {
    if (handle.id==10 && targetPresent) { *out={sizeof(*out),{1,2,3},{0,0,0,1},{2,3,4}}; return RESULT_OK; }
    if (!props.contains(handle.id)) return RESULT_NOT_FOUND;
    *out=props.at(handle.id); return RESULT_OK;
}
Result CALL Spawn(StringView,StringView,const WorldTransform* transform,WorldEntityHandle* handle) {
    if (++spawnCalls==failSpawn) return RESULT_NOT_FOUND;
    *handle={nextId++,1,1}; props[handle->id]=*transform; return RESULT_OK;
}
Result CALL Destroy(StringView,WorldEntityHandle handle) {
    if (failDestroy) return RESULT_INTERNAL_ERROR;
    if (handle.id==10 && targetPresent) { targetPresent=false; return RESULT_OK; }
    return props.erase(handle.id)?RESULT_OK:RESULT_NOT_FOUND;
}
Result CALL Grid(StringView,StringView,WorldGridSpec* grid) {
    *grid={sizeof(*grid),{}, {0,0,0},{1,1,1},16,16,16}; return RESULT_OK;
}
Result CALL Read(StringView,StringView,const WorldBounds*,uint32_t* out,WorldCellState* covered,size_t capacity,size_t* required) {
    *required=cells.size();
    if (capacity<cells.size()) return RESULT_INVALID_ARGUMENT;
    std::copy(cells.begin(),cells.end(),out); std::copy(coverage.begin(),coverage.end(),covered); return RESULT_OK;
}
Result CALL Write(StringView,StringView,const WorldBounds*,const uint32_t* data,const WorldCellState* mask,size_t count) {
    if (count!=cells.size()) return RESULT_INVALID_ARGUMENT;
    ++writeCalls;
    for (size_t i=0;i<count;++i) {
        if (mask[i]!=WorldCellState::Unknown) { cells[i]=data[i]; coverage[i]=mask[i]; }
        if (writeCalls==failWrite) return RESULT_INTERNAL_ERROR;
    }
    return RESULT_OK;
}
WorldApi world{sizeof(world),WorldApiVersion,Query,Get,Spawn,Destroy,nullptr,Grid,Read,Write};
void Reset() {
    cells={0,0,44}; coverage={WorldCellState::Empty,WorldCellState::Empty,WorldCellState::Occupied};
    props.clear(); spawnCalls=0; failSpawn=0; writeCalls=0; failWrite=0;
    failDestroy=false;
    targetPresent=true;
}
Blueprint House() {
    Blueprint b; b.id="test.house"; b.extent={3,1,1};
    b.props.push_back({1,"template.table",{{.5,.5,.5},{},{2,3,4}}});
    b.channels.push_back({"terrain",{3,1,1},{7,0,999},{Coverage::Occupied,Coverage::Empty,Coverage::Unknown}});
    return b;
}
}
int main() {
    WorldGridSpec spec{sizeof(spec),{}, {-10,0,0},{.5,1,2},16,16,16};
    WorldGridRegion bounds{sizeof(bounds)};
    assert(GridRegionFromPoints(&spec,{-9.1,-.1,1},{-10.1,-2.1,1},100,&bounds)==RESULT_OK);
    assert(bounds.minimum[0]==-1 && bounds.dimensions[0]==3 && bounds.cell_count==9);
    assert(bounds.bounds.minimum.x==-10.5 && bounds.bounds.maximum.x==-9);
    assert(GridRegionFromPoints(&spec,{0,0,0},{1000,1000,1000},10,&bounds)==RESULT_INVALID_ARGUMENT);
    assert(GridRegionFromPoints(&spec,{1e300,0,0},{1e300,0,0},100,&bounds)==RESULT_INVALID_ARGUMENT);
    assert(GridRegionFromPoints(&spec,{0,0,0},{0,0,0},0,&bounds)==RESULT_INVALID_ARGUMENT);
    spec.cell_size.x=0;
    assert(GridRegionFromPoints(&spec,{0,0,0},{1,1,1},100,&bounds)==RESULT_INVALID_ARGUMENT);

    Reset(); Editor e("mod.test");
    e.selection.set_a({0,0,0}); e.selection.set_b({3,4,5});
    assert(e.capture(world,true,{},UpAxis::Z)==RESULT_OK);
    assert(e.clipboard()->props[0].template_id=="template.table");
    assert((e.clipboard()->props[0].transform.scale==Vec3{2,3,4}));
    e.selection.set_b({2,0,0}); cells={7,0,99}; coverage={WorldCellState::Occupied,WorldCellState::Empty,WorldCellState::Unknown};
    assert(e.capture(world,false,{"terrain"},UpAxis::Z)==RESULT_OK);
    assert(e.clipboard()->channels[0].coverage[2]==Coverage::Unknown);
    assert(e.clipboard()->channels[0].dimensions.x==3);
    assert(e.capture(world,true,{"terrain"},UpAxis::Z)==RESULT_OK); // A thin wall is valid with prop capture enabled.
    assert(e.clipboard()->channels[0].dimensions.y==1 && e.clipboard()->channels[0].dimensions.z==1);

    Reset(); auto voxelOnly=House(); voxelOnly.props.clear();
    e.set_blueprint(voxelOnly); e.prepare({10,20,30},0); failSpawn=1;
    assert(e.paste(world,PasteMode::Additive).result==RESULT_OK);
    assert(cells[0]==7 && spawnCalls==0); // Entity failures must not affect a voxel-only blueprint.
    assert(e.undo(world).result==RESULT_OK && cells[0]==0);

    Reset(); e.set_blueprint(House()); e.prepare({10,20,30},0);
    assert(e.paste(world,PasteMode::ReplaceAll).result==RESULT_OK);
    assert(!targetPresent && cells[0]==7 && props.size()==1);
    assert(e.undo(world).result==RESULT_OK);
    assert(cells[0]==0 && props.size()==1); // Removed target prop was restored with a fresh handle.

    Reset(); auto empty=House(); empty.props.clear();
    empty.channels[0].values={0,0,0};
    empty.channels[0].coverage={Coverage::Empty,Coverage::Empty,Coverage::Empty};
    e.set_blueprint(empty); e.prepare({10,20,30},0);
    assert(e.paste(world,PasteMode::Additive).result==RESULT_NOT_FOUND);
    assert(writeCalls==0 && spawnCalls==0);
    Reset(); e.set_blueprint(House()); e.prepare({10,20,30},0);
    assert(e.paste(world,PasteMode::Additive).result==RESULT_OK);
    assert(cells[0]==7 && cells[2]==44 && props.size()==1);
    assert(e.undo(world).result==RESULT_OK);
    assert(cells[0]==0 && cells[2]==44 && props.empty());

    e.prepare({10,20,30},0); cells[0]=8; coverage[0]=WorldCellState::Occupied;
    assert(e.paste(world,PasteMode::Additive).result==RESULT_OK);
    assert(cells[0]==7 && props.size()==1 && writeCalls==3);
    assert(e.undo(world).result==RESULT_OK && cells[0]==8 && props.empty());
    e.prepare({10,20,30},0);
    assert(e.paste(world,PasteMode::ReplaceCells).result==RESULT_OK);
    cells[0]=66;
    assert(e.undo(world).result==RESULT_CONFLICT); // Do not undo another edit.
    assert(cells[0]==66 && props.size()==1);
    cells[0]=7;
    assert(e.undo(world).result==RESULT_OK && cells[0]==8);

    Reset(); e.set_blueprint(House()); e.prepare({0,0,0},0); failWrite=1;
    const auto failed=e.paste(world,PasteMode::ReplaceCells);
    assert(failed.result==RESULT_INTERNAL_ERROR && !failed.recovery_required);
    assert(cells[0]==0 && cells[2]==44 && !e.has_history());

    Reset(); auto house=House(); house.props.push_back({2,"template.chair",{{1,0,0},{},{1,1,1}}});
    e.set_blueprint(house); e.prepare({0,0,0},0); failSpawn=2;
    assert(e.paste(world,PasteMode::ReplaceCells).result==RESULT_NOT_FOUND);
    assert(props.empty() && cells[0]==0 && !e.has_history());

    Reset(); e.prepare({0,0,0},0); failSpawn=2; failDestroy=true;
    const auto incomplete=e.paste(world,PasteMode::ReplaceCells);
    assert(incomplete.recovery_required && e.recovery_required() && props.size()==1);
    cells[0]=88; failDestroy=false;
    assert(e.undo(world).result==RESULT_CONFLICT && e.recovery_required());
    cells[0]=7;
    assert(e.undo(world).result==RESULT_OK && props.empty() && cells[0]==0 && !e.recovery_required());

    const auto rotated=make_placement_plan(House(),{10,20,30},1);
    assert((rotated.channels[0].origin==Vec3{9,20,30}));
    assert((rotated.props[0].transform.position==Vec3{9.5,20.5,30.5}));
    assert((rotated.channels[0].dimensions==Int3{1,3,1}));
    auto anisotropic=House(); anisotropic.channels[0].origin={2,3,4}; anisotropic.channels[0].cell_size={.5,2,3};
    const auto plan=make_placement_plan(anisotropic,{0,0,0},1);
    assert((plan.channels[0].cell_size==Vec3{2,.5,3}));
    const auto path=std::filesystem::temp_directory_path()/"shroudedit-grid-metadata.seblueprint";
    save_blueprint(anisotropic,path);
    const auto loaded=load_blueprint(path); std::filesystem::remove(path);
    assert(loaded.channels[0].origin==anisotropic.channels[0].origin && loaded.channels[0].cell_size==anisotropic.channels[0].cell_size);
    bool rejected=false;
    try { e.selection.set_a({std::numeric_limits<double>::quiet_NaN(),0,0}); } catch (...) { rejected=true; }
    assert(rejected);
    auto limited=House(); limited.channels.push_back(limited.channels.front()); limited.channels.back().id="building";
    assert(!validate(limited,ValidationLimits{3}).empty());
    std::cout<<"ShroudEdit API workflow tests passed (test world, not Enshrouded).\n";
}
