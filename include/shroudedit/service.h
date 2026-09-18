#pragma once

#include "shroudedit/blueprint.h"

#include <cstddef>
#include <cstdint>

#define SHROUDEDIT_BLUEPRINT_SERVICE_MAJOR 1u
#define SHROUDEDIT_BLUEPRINT_SERVICE_MINOR 0u

extern "C" {
    typedef struct ShroudEditBlueprintValidationV1 {
        std::size_t struct_size;
        std::uint32_t error_count;
    } ShroudEditBlueprintValidationV1;

    typedef struct ShroudEditBlueprintServiceV1 {
        std::size_t struct_size;
        int (__cdecl* validate_file)(const char* path_utf8, ShroudEditBlueprintValidationV1* result);
    } ShroudEditBlueprintServiceV1;
}
