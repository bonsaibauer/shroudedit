cmake_minimum_required(VERSION 3.24)
project(shroudedit VERSION 0.2.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(SHROUDTOPIA_API_DIR "" CACHE PATH "Path to the Shroudtopia repository containing api/include")
if(NOT EXISTS "${SHROUDTOPIA_API_DIR}/api/shroudtopia.h")
    message(FATAL_ERROR "Set SHROUDTOPIA_API_DIR to the Shroudtopia repository")
endif()

add_library(shroudedit_core STATIC
    src/blueprint.cpp
    src/blueprint_codec.cpp
    src/placement.cpp
    src/selection.cpp
    src/editor.cpp
    src/library.cpp
)
target_include_directories(shroudedit_core PUBLIC include)
target_include_directories(shroudedit_core PUBLIC "${SHROUDTOPIA_API_DIR}/api")
target_compile_features(shroudedit_core PUBLIC cxx_std_20)

if(MSVC)
    target_compile_options(shroudedit_core PRIVATE /W4 /permissive-)
else()
    target_compile_options(shroudedit_core PRIVATE -Wall -Wextra -Wpedantic)
endif()

option(SHROUDEDIT_BUILD_MOD "Build the Shroudtopia native mod DLL" ON)

if(WIN32 AND SHROUDEDIT_BUILD_MOD)
    add_executable(prepare_building_tile tools/prepare-building-tile.cpp src/building_tile.cpp)
    target_link_libraries(prepare_building_tile PRIVATE shroudedit_core)
    target_include_directories(prepare_building_tile PRIVATE "${SHROUDTOPIA_API_DIR}/third-party/nlohmann-json/single_include")
    target_compile_definitions(prepare_building_tile PRIVATE SHROUDTOPIA_FIXTURE_ROOT="${SHROUDTOPIA_API_DIR}/build")
    add_library(shroudedit SHARED src/mod.cpp src/building_tile.cpp)
    target_link_libraries(shroudedit PRIVATE shroudedit_core)
    target_include_directories(shroudedit PRIVATE "${SHROUDTOPIA_API_DIR}/api")
    target_include_directories(shroudedit PRIVATE "${SHROUDTOPIA_API_DIR}/third-party/nlohmann-json/single_include")
    target_compile_definitions(shroudedit PRIVATE WIN32_LEAN_AND_MEAN NOMINMAX)
    if(MSVC)
        # Remove the wall-clock PE timestamp so identical source inputs produce
        # an identical release DLL on the same toolchain.
        target_link_options(shroudedit PRIVATE /Brepro)
    endif()
    set_target_properties(shroudedit PROPERTIES OUTPUT_NAME "mod.shroudedit")
    install(TARGETS shroudedit RUNTIME DESTINATION mods/mod.shroudedit LIBRARY DESTINATION mods/mod.shroudedit)
    install(FILES mod.json README.md LICENSE VALIDATED-BUILD.md DESTINATION mods/mod.shroudedit)
endif()

include(CTest)
if(BUILD_TESTING)
    add_executable(shroudedit_library_tests tests/library_tests.cpp)
    target_link_libraries(shroudedit_library_tests PRIVATE shroudedit_core)
    add_test(NAME shroudedit_library_tests COMMAND shroudedit_library_tests)
    add_executable(shroudedit_core_tests tests/core_tests.cpp)
    target_link_libraries(shroudedit_core_tests PRIVATE shroudedit_core)
    add_test(NAME shroudedit_core_tests COMMAND shroudedit_core_tests)
    add_executable(shroudedit_editor_tests tests/editor_tests.cpp)
    target_link_libraries(shroudedit_editor_tests PRIVATE shroudedit_core)
    add_test(NAME shroudedit_editor_tests COMMAND shroudedit_editor_tests)
    if(TARGET shroudedit)
        add_executable(shroudedit_building_tile_tests tests/building_tile_tests.cpp src/building_tile.cpp)
        target_link_libraries(shroudedit_building_tile_tests PRIVATE shroudedit_core)
        target_include_directories(shroudedit_building_tile_tests PRIVATE "${SHROUDTOPIA_API_DIR}/third-party/nlohmann-json/single_include")
        target_compile_definitions(shroudedit_building_tile_tests PRIVATE WIN32_LEAN_AND_MEAN NOMINMAX SHROUDTOPIA_FIXTURE_ROOT="${SHROUDTOPIA_API_DIR}/build")
        add_test(NAME shroudedit_building_tile_tests COMMAND shroudedit_building_tile_tests)
        add_executable(shroudedit_mod_tests tests/mod_tests.cpp)
        target_link_libraries(shroudedit_mod_tests PRIVATE shroudedit_core)
        target_compile_definitions(shroudedit_mod_tests PRIVATE WIN32_LEAN_AND_MEAN NOMINMAX)
        add_dependencies(shroudedit_mod_tests shroudedit)
        add_test(NAME shroudedit_mod_tests COMMAND shroudedit_mod_tests $<TARGET_FILE:shroudedit>)
    endif()
endif()

