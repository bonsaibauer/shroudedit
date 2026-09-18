#pragma once
#include "shroudtopia_world.h"
#include <algorithm>
#include <cmath>
#include <limits>

namespace shroudedit {
// Inclusive endpoint cells, including signed coordinates and negative origins.
inline Result GridRegionFromPoints(const WorldGridSpec* spec, WorldVec3 a, WorldVec3 b,
    std::size_t budget, WorldGridRegion* result) {
    if (!spec || !result || spec->struct_size < sizeof(*spec) || result->struct_size < sizeof(*result) || !budget)
        return RESULT_INVALID_ARGUMENT;
    const double origin[] = {spec->origin.x, spec->origin.y, spec->origin.z};
    const double step[] = {spec->cell_size.x, spec->cell_size.y, spec->cell_size.z};
    const double first[] = {a.x,a.y,a.z}, second[] = {b.x,b.y,b.z};
    WorldGridRegion region{sizeof(region)};
    region.cell_count = 1;
    double lo[3]{}, hi[3]{};
    for (int i=0; i<3; ++i) {
        if (!std::isfinite(first[i]) || !std::isfinite(second[i]) || !std::isfinite(origin[i]) ||
            !std::isfinite(step[i]) || step[i] <= 0) return RESULT_INVALID_ARGUMENT;
        const double left = std::floor((std::min(first[i],second[i])-origin[i])/step[i]);
        const double right = std::floor((std::max(first[i],second[i])-origin[i])/step[i]);
        // Exact integer domain of doubles also leaves room for the exclusive upper cell.
        if (!std::isfinite(left) || !std::isfinite(right) || left < -0x1p52 || right >= 0x1p52 || right < left)
            return RESULT_INVALID_ARGUMENT;
        const double count = right-left+1;
        if (count > INT32_MAX || count > static_cast<double>(budget/region.cell_count)) return RESULT_INVALID_ARGUMENT;
        region.minimum[i] = static_cast<std::int64_t>(left);
        region.dimensions[i] = static_cast<std::size_t>(count);
        region.cell_count *= region.dimensions[i];
        lo[i] = origin[i]+left*step[i]; hi[i] = origin[i]+(right+1)*step[i];
        if (!std::isfinite(lo[i]) || !std::isfinite(hi[i]) || hi[i] <= lo[i]) return RESULT_INVALID_ARGUMENT;
    }
    region.bounds = {sizeof(WorldBounds),{lo[0],lo[1],lo[2]},{hi[0],hi[1],hi[2]}};
    *result = region;
    return RESULT_OK;
}
}
