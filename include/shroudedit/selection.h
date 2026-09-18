#pragma once

#include "shroudedit/types.h"

#include <optional>

namespace shroudedit {
    struct Region {
        Vec3 minimum{};
        Vec3 maximum{};
    };

    struct CellRegion {
        Int3 minimum{};
        Int3 maximum_exclusive{};
        [[nodiscard]] std::uint64_t cell_count() const;
    };

    class Selection {
    public:
        void set_a(Vec3 point);
        void set_b(Vec3 point);
        void clear();

        [[nodiscard]] const std::optional<Vec3>& a() const { return a_; }
        [[nodiscard]] const std::optional<Vec3>& b() const { return b_; }
        [[nodiscard]] bool complete() const { return a_.has_value() && b_.has_value(); }
        [[nodiscard]] std::optional<Region> region() const;
        [[nodiscard]] std::optional<CellRegion> cell_region() const;

    private:
        std::optional<Vec3> a_;
        std::optional<Vec3> b_;
    };
}
