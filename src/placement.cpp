#include "shroudedit/placement.h"

#include <stdexcept>
#include <algorithm>
#include <cmath>

namespace shroudedit {
    namespace {
        Quaternion Multiply(Quaternion left, Quaternion right) {
            return {
                left.w*right.x + left.x*right.w + left.y*right.z - left.z*right.y,
                left.w*right.y - left.x*right.z + left.y*right.w + left.z*right.x,
                left.w*right.z + left.x*right.y - left.y*right.x + left.z*right.w,
                left.w*right.w - left.x*right.x - left.y*right.y - left.z*right.z
            };
        }

        Quaternion QuarterRotation(UpAxis axis, std::uint8_t turns) {
            Quaternion rotation{};
            constexpr double HalfSqrtTwo = 0.7071067811865475244;
            const Quaternion step = axis == UpAxis::X ? Quaternion{HalfSqrtTwo, 0, 0, HalfSqrtTwo}
                : axis == UpAxis::Y ? Quaternion{0, HalfSqrtTwo, 0, HalfSqrtTwo}
                : Quaternion{0, 0, HalfSqrtTwo, HalfSqrtTwo};
            for (std::uint8_t i = 0; i < turns % 4; ++i) rotation = Multiply(step, rotation);
            return rotation;
        }

        std::size_t Index(Int3 point, Int3 dimensions) {
            return static_cast<std::size_t>(point.x) + static_cast<std::size_t>(dimensions.x) *
                (static_cast<std::size_t>(point.y) + static_cast<std::size_t>(dimensions.y) * static_cast<std::size_t>(point.z));
        }

        std::pair<Int3, Int3> RotateCell(Int3 point, Int3 dimensions, UpAxis axis) {
            switch (axis) {
            case UpAxis::X: return {{point.x, dimensions.z - 1 - point.z, point.y}, {dimensions.x, dimensions.z, dimensions.y}};
            case UpAxis::Y: return {{point.z, point.y, dimensions.x - 1 - point.x}, {dimensions.z, dimensions.y, dimensions.x}};
            case UpAxis::Z: return {{dimensions.y - 1 - point.y, point.x, point.z}, {dimensions.y, dimensions.x, dimensions.z}};
            default: throw std::invalid_argument("invalid up axis");
            }
        }
    }

    Vec3 rotate_quarter_turns(Vec3 value, UpAxis axis, std::uint8_t quarterTurns) {
        quarterTurns %= 4;
        for (std::uint8_t i = 0; i < quarterTurns; ++i) {
            switch (axis) {
            case UpAxis::X: value = {value.x, -value.z, value.y}; break;
            case UpAxis::Y: value = {value.z, value.y, -value.x}; break;
            case UpAxis::Z: value = {-value.y, value.x, value.z}; break;
            default: throw std::invalid_argument("invalid up axis");
            }
        }
        return value;
    }

    PlacementPlan make_placement_plan(const Blueprint& blueprint, Vec3 targetAnchor, std::uint8_t quarterTurns) {
        if (!std::isfinite(targetAnchor.x) || !std::isfinite(targetAnchor.y) || !std::isfinite(targetAnchor.z))
            throw std::invalid_argument("target anchor must be finite");
        const auto errors = validate(blueprint);
        if (!errors.empty()) throw std::invalid_argument(errors.front().path + ": " + errors.front().message);

        PlacementPlan plan;
        plan.target_anchor=targetAnchor;
        plan.up_axis=blueprint.up_axis;
        plan.quarter_turns=static_cast<std::uint8_t>(quarterTurns % 4);
        const auto volumeNear=rotate_quarter_turns({-blueprint.anchor.x,-blueprint.anchor.y,-blueprint.anchor.z},
            blueprint.up_axis,plan.quarter_turns);
        const auto volumeFar=rotate_quarter_turns({blueprint.extent.x-blueprint.anchor.x,
            blueprint.extent.y-blueprint.anchor.y,blueprint.extent.z-blueprint.anchor.z},
            blueprint.up_axis,plan.quarter_turns);
        plan.region_minimum={targetAnchor.x+(std::min)(volumeNear.x,volumeFar.x),
            targetAnchor.y+(std::min)(volumeNear.y,volumeFar.y),targetAnchor.z+(std::min)(volumeNear.z,volumeFar.z)};
        plan.region_maximum={targetAnchor.x+(std::max)(volumeNear.x,volumeFar.x),
            targetAnchor.y+(std::max)(volumeNear.y,volumeFar.y),targetAnchor.z+(std::max)(volumeNear.z,volumeFar.z)};
        const auto planRotation = QuarterRotation(blueprint.up_axis, plan.quarter_turns);
        plan.props.reserve(blueprint.props.size());
        for (const auto& prop : blueprint.props) {
            const Vec3 relative{
                prop.transform.position.x - blueprint.anchor.x,
                prop.transform.position.y - blueprint.anchor.y,
                prop.transform.position.z - blueprint.anchor.z
            };
            const auto rotated = rotate_quarter_turns(relative, blueprint.up_axis, plan.quarter_turns);
            auto transform = prop.transform;
            transform.position = {targetAnchor.x + rotated.x, targetAnchor.y + rotated.y, targetAnchor.z + rotated.z};
            transform.rotation = Multiply(planRotation, transform.rotation);
            plan.props.push_back({prop.local_id, prop.template_id, transform});
        }

        plan.channels.reserve(blueprint.channels.size());
        for (const auto& source : blueprint.channels) {
            PlannedGridChannel target{source.id, source.dimensions, source.values, source.coverage, {}, source.cell_size};
            const Vec3 relative{source.origin.x - blueprint.anchor.x,
                source.origin.y - blueprint.anchor.y, source.origin.z - blueprint.anchor.z};
            const Vec3 far{relative.x + source.dimensions.x * source.cell_size.x,
                relative.y + source.dimensions.y * source.cell_size.y,
                relative.z + source.dimensions.z * source.cell_size.z};
            const auto nearRotated = rotate_quarter_turns(relative, blueprint.up_axis, plan.quarter_turns);
            const auto farRotated = rotate_quarter_turns(far, blueprint.up_axis, plan.quarter_turns);
            target.origin = {targetAnchor.x + (std::min)(nearRotated.x, farRotated.x),
                targetAnchor.y + (std::min)(nearRotated.y, farRotated.y),
                targetAnchor.z + (std::min)(nearRotated.z, farRotated.z)};
            for (std::uint8_t turn = 0; turn < plan.quarter_turns; ++turn) {
                const auto nextDimensions = RotateCell({0, 0, 0}, target.dimensions, blueprint.up_axis).second;
                std::vector<std::uint32_t> nextValues(target.values.size());
                std::vector<Coverage> nextCoverage(target.coverage.size(), Coverage::Unknown);
                for (std::int32_t z = 0; z < target.dimensions.z; ++z) {
                    for (std::int32_t y = 0; y < target.dimensions.y; ++y) {
                        for (std::int32_t x = 0; x < target.dimensions.x; ++x) {
                            const Int3 sourcePoint{x, y, z};
                            const auto targetPoint = RotateCell(sourcePoint, target.dimensions, blueprint.up_axis).first;
                            const auto sourceIndex = Index(sourcePoint, target.dimensions);
                            const auto targetIndex = Index(targetPoint, nextDimensions);
                            nextValues[targetIndex] = target.values[sourceIndex];
                            nextCoverage[targetIndex] = target.coverage[sourceIndex];
                        }
                    }
                }
                target.dimensions = nextDimensions;
                const auto rotatedSize = rotate_quarter_turns(target.cell_size, blueprint.up_axis, 1);
                target.cell_size = {std::abs(rotatedSize.x), std::abs(rotatedSize.y), std::abs(rotatedSize.z)};
                target.values = std::move(nextValues);
                target.coverage = std::move(nextCoverage);
            }
            plan.channels.push_back(std::move(target));
        }
        return plan;
    }
}
