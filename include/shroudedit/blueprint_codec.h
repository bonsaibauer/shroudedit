#pragma once

#include "shroudedit/blueprint.h"

#include <filesystem>

namespace shroudedit {
    void save_blueprint(const Blueprint& blueprint, const std::filesystem::path& path, const ValidationLimits& limits = {}, bool replace = true);
    [[nodiscard]] Blueprint load_blueprint(const std::filesystem::path& path, const ValidationLimits& limits = {});
}
