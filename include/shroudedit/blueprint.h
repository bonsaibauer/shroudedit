#pragma once

#include "shroudedit/types.h"

#include <cstdint>
#include <string>
#include <vector>

namespace shroudedit {
    enum class Coverage : std::uint8_t {
        Unknown = 0,
        Empty = 1,
        Occupied = 2
    };

    struct Prop {
        std::uint64_t local_id{};
        std::string template_id;
        Transform transform{};
    };

    struct GridChannel {
        std::string id;
        Int3 dimensions{};
        std::vector<std::uint32_t> values;
        std::vector<Coverage> coverage;
        Vec3 origin{}; // Relative to the captured volume's minimum.
        Vec3 cell_size{1, 1, 1};
    };

    struct Blueprint {
        std::uint16_t schema_major{1};
        std::uint16_t schema_minor{1};
        std::string id;
        std::string name;
        Vec3 extent{};
        Vec3 anchor{};
        UpAxis up_axis{UpAxis::Z};
        std::vector<Prop> props;
        std::vector<GridChannel> channels;
    };

    struct ValidationLimits {
        std::uint64_t maximum_cells{16'777'216};
        std::uint32_t maximum_props{1'000'000};
        std::uint32_t maximum_string_bytes{1'048'576};
    };

    struct ValidationError {
        std::string path;
        std::string message;
    };

    [[nodiscard]] std::vector<ValidationError> validate(
        const Blueprint& blueprint,
        const ValidationLimits& limits = {});
}
