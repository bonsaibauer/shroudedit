#pragma once
#include "shroudtopia.h"

namespace shroudedit {
// Diagnostic duplicate of a known game shape. Not a captured WorldEdit area.
// On success mutations are staged for save_assets; caller owns rollback.
Result stage_building_test_tile(const Api& api, StringView owner);
}
