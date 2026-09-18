#include "shroudedit/blueprint.h"

#include <cmath>
#include <limits>
#include <unordered_set>

namespace shroudedit {
    namespace {
        bool Finite(Vec3 value) { return std::isfinite(value.x) && std::isfinite(value.y) && std::isfinite(value.z); }
        bool Finite(Quaternion value) {
            return std::isfinite(value.x) && std::isfinite(value.y) && std::isfinite(value.z) && std::isfinite(value.w);
        }

        std::uint64_t CellCount(Int3 dimensions, bool& valid) {
            valid = dimensions.x > 0 && dimensions.y > 0 && dimensions.z > 0;
            if (!valid) return 0;
            const auto x = static_cast<std::uint64_t>(dimensions.x);
            const auto y = static_cast<std::uint64_t>(dimensions.y);
            const auto z = static_cast<std::uint64_t>(dimensions.z);
            if (x > (std::numeric_limits<std::uint64_t>::max)() / y || x * y > (std::numeric_limits<std::uint64_t>::max)() / z) {
                valid = false;
                return 0;
            }
            return x * y * z;
        }
    }

    std::vector<ValidationError> validate(const Blueprint& blueprint, const ValidationLimits& limits) {
        std::vector<ValidationError> errors;
        if (blueprint.schema_major != 1) errors.push_back({"schema_major", "unsupported blueprint schema major"});
        if (blueprint.schema_minor > 1) errors.push_back({"schema_minor", "unsupported blueprint schema minor"});
        if (blueprint.id.empty()) errors.push_back({"id", "blueprint ID is required"});
        if (blueprint.id.size() > limits.maximum_string_bytes || blueprint.name.size() > limits.maximum_string_bytes) {
            errors.push_back({"metadata", "string size exceeds validation limit"});
        }
        if (!Finite(blueprint.extent) || blueprint.extent.x < 0 || blueprint.extent.y < 0 || blueprint.extent.z < 0) {
            errors.push_back({"extent", "extent must contain finite non-negative values"});
        }
        if (!Finite(blueprint.anchor)) errors.push_back({"anchor", "anchor must be finite"});
        if (blueprint.up_axis != UpAxis::X && blueprint.up_axis != UpAxis::Y && blueprint.up_axis != UpAxis::Z) {
            errors.push_back({"up_axis", "up axis is invalid"});
        }
        if (blueprint.props.size() > limits.maximum_props) errors.push_back({"props", "prop count exceeds validation limit"});

        std::unordered_set<std::uint64_t> propIds;
        for (std::size_t i = 0; i < blueprint.props.size(); ++i) {
            const auto& prop = blueprint.props[i];
            const auto path = "props[" + std::to_string(i) + "]";
            if (prop.local_id == 0 || !propIds.insert(prop.local_id).second) errors.push_back({path + ".local_id", "local ID must be unique and non-zero"});
            if (prop.template_id.empty() || prop.template_id.size() > limits.maximum_string_bytes) errors.push_back({path + ".template_id", "template ID is invalid"});
            if (!Finite(prop.transform.position) || !Finite(prop.transform.scale) || !Finite(prop.transform.rotation)) {
                errors.push_back({path + ".transform", "transform values must be finite"});
            }
            const auto& q = prop.transform.rotation;
            const double norm = q.x*q.x + q.y*q.y + q.z*q.z + q.w*q.w;
            if (!std::isfinite(norm) || norm < 1e-12) errors.push_back({path + ".rotation", "quaternion must have a non-zero norm"});
        }

        std::unordered_set<std::string> channelIds;
        std::uint64_t totalCells = 0;
        if (blueprint.channels.size() > 1024) errors.push_back({"channels", "too many channels"});
        for (std::size_t i = 0; i < blueprint.channels.size(); ++i) {
            const auto& channel = blueprint.channels[i];
            const auto path = "channels[" + std::to_string(i) + "]";
            if (!Finite(channel.origin) || !Finite(channel.cell_size) || channel.cell_size.x <= 0 ||
                channel.cell_size.y <= 0 || channel.cell_size.z <= 0)
                errors.push_back({path, "grid origin and positive cell size must be finite"});
            if (channel.id.size() > limits.maximum_string_bytes) errors.push_back({path, "channel ID too long"});
            if (blueprint.schema_minor == 0 && (channel.origin != Vec3{} || channel.cell_size != Vec3{1, 1, 1}))
                errors.push_back({path, "grid metadata requires schema 1.1"});
            if (channel.id.empty() || !channelIds.insert(channel.id).second) errors.push_back({path + ".id", "channel ID must be unique and non-empty"});
            bool dimensionsValid = false;
            const auto cells = CellCount(channel.dimensions, dimensionsValid);
            if (cells > limits.maximum_cells - totalCells) errors.push_back({path, "total cell budget exceeded"});
            else totalCells += cells;
            if (!dimensionsValid || cells > limits.maximum_cells) errors.push_back({path + ".dimensions", "cell dimensions are invalid or exceed the limit"});
            if (dimensionsValid && (channel.values.size() != cells || channel.coverage.size() != cells)) {
                errors.push_back({path, "value and coverage counts must match the channel dimensions"});
            }
            for (const auto coverage : channel.coverage) {
                if (coverage != Coverage::Unknown && coverage != Coverage::Empty && coverage != Coverage::Occupied) {
                    errors.push_back({path + ".coverage", "coverage value is invalid"});
                    break;
                }
            }
        }
        return errors;
    }
}
