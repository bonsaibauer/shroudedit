#pragma once

#include <cstdint>

namespace shroudedit {
    struct Vec3 {
        double x{};
        double y{};
        double z{};

        friend bool operator==(const Vec3&, const Vec3&) = default;
    };

    struct Int3 {
        std::int32_t x{};
        std::int32_t y{};
        std::int32_t z{};

        friend bool operator==(const Int3&, const Int3&) = default;
    };

    struct Quaternion {
        double x{};
        double y{};
        double z{};
        double w{1.0};

        friend bool operator==(const Quaternion&, const Quaternion&) = default;
    };

    struct Transform {
        Vec3 position{};
        Quaternion rotation{};
        Vec3 scale{1.0, 1.0, 1.0};
    };

    enum class UpAxis : std::uint8_t { X = 0, Y = 1, Z = 2 };
}
