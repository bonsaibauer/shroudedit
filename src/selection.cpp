#include "shroudedit/selection.h"

#include <algorithm>
#include <cmath>
#include <limits>
#include <stdexcept>

namespace shroudedit {
    namespace {
        std::int32_t FloorToCell(double value) {
            if (!std::isfinite(value) || value < static_cast<double>((std::numeric_limits<std::int32_t>::min)()) ||
                value >= static_cast<double>((std::numeric_limits<std::int32_t>::max)())) {
                throw std::out_of_range("selection coordinate cannot be represented as a cell index");
            }
            return static_cast<std::int32_t>(std::floor(value));
        }

        std::uint64_t Extent(std::int32_t minimum, std::int32_t maximumExclusive) {
            if (maximumExclusive <= minimum) throw std::invalid_argument("invalid cell region");
            return static_cast<std::uint64_t>(static_cast<std::int64_t>(maximumExclusive) - minimum);
        }
    }

    std::uint64_t CellRegion::cell_count() const {
        const auto x = Extent(minimum.x, maximum_exclusive.x);
        const auto y = Extent(minimum.y, maximum_exclusive.y);
        const auto z = Extent(minimum.z, maximum_exclusive.z);
        if (x > (std::numeric_limits<std::uint64_t>::max)() / y || x * y > (std::numeric_limits<std::uint64_t>::max)() / z) {
            throw std::overflow_error("cell region is too large");
        }
        return x * y * z;
    }

    void Selection::set_a(Vec3 point) {
        if (!std::isfinite(point.x) || !std::isfinite(point.y) || !std::isfinite(point.z)) throw std::invalid_argument("point must be finite");
        a_ = point;
    }
    void Selection::set_b(Vec3 point) {
        if (!std::isfinite(point.x) || !std::isfinite(point.y) || !std::isfinite(point.z)) throw std::invalid_argument("point must be finite");
        b_ = point;
    }
    void Selection::clear() { a_.reset(); b_.reset(); }

    std::optional<Region> Selection::region() const {
        if (!complete()) return std::nullopt;
        return Region{
            { (std::min)(a_->x, b_->x), (std::min)(a_->y, b_->y), (std::min)(a_->z, b_->z) },
            { (std::max)(a_->x, b_->x), (std::max)(a_->y, b_->y), (std::max)(a_->z, b_->z) }
        };
    }

    std::optional<CellRegion> Selection::cell_region() const {
        if (!complete()) return std::nullopt;
        const Int3 first{FloorToCell(a_->x), FloorToCell(a_->y), FloorToCell(a_->z)};
        const Int3 second{FloorToCell(b_->x), FloorToCell(b_->y), FloorToCell(b_->z)};
        return CellRegion{
            {(std::min)(first.x, second.x), (std::min)(first.y, second.y), (std::min)(first.z, second.z)},
            {(std::max)(first.x, second.x) + 1, (std::max)(first.y, second.y) + 1, (std::max)(first.z, second.z) + 1}
        };
    }
}
