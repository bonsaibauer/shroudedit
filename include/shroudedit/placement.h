#pragma once

#include "shroudedit/blueprint.h"

#include <cstdint>
#include <vector>

namespace shroudedit {
    struct PlannedProp {
        std::uint64_t local_id{};
        std::string template_id;
        Transform transform{};
    };

    struct PlannedGridChannel {
        std::string id;
        Int3 dimensions{};
        std::vector<std::uint32_t> values;
        std::vector<Coverage> coverage;
        Vec3 origin{}; // World-space minimum after rotation.
        Vec3 cell_size{1, 1, 1};
    };

    struct PlacementPlan {
        Vec3 target_anchor{};
        Vec3 region_minimum{};
        Vec3 region_maximum{};
        UpAxis up_axis{UpAxis::Z};
        std::uint8_t quarter_turns{};
        std::vector<PlannedProp> props;
        std::vector<PlannedGridChannel> channels;
    };

    [[nodiscard]] Vec3 rotate_quarter_turns(Vec3 value, UpAxis axis, std::uint8_t quarterTurns);
    [[nodiscard]] PlacementPlan make_placement_plan(const Blueprint& blueprint, Vec3 targetAnchor, std::uint8_t quarterTurns);
}
