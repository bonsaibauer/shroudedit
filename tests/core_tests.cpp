#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shroudedit/blueprint_codec.h"
#include "shroudedit/placement.h"
#include "shroudedit/selection.h"

#include <cassert>
#include <cmath>
#include <filesystem>
#include <iostream>

using namespace shroudedit;

namespace {
    bool Near(double left, double right) { return std::abs(left - right) < 1e-9; }

    Blueprint ExampleBlueprint() {
        Blueprint blueprint;
        blueprint.id = "tests.house";
        blueprint.name = "Test house";
        blueprint.extent = {2, 2, 1};
        blueprint.anchor = {0, 0, 0};
        blueprint.up_axis = UpAxis::Z;
        blueprint.props.push_back({1, "game.prop.table", {{1, 0, 0}, {}, {1, 1, 1}}});
        blueprint.channels.push_back({"terrain", {2, 1, 1}, {17, 0}, {Coverage::Occupied, Coverage::Empty}});
        return blueprint;
    }
}

int main() {
    Selection selection;
    selection.set_a({2.9, -0.1, 5.0});
    selection.set_b({-1.2, -2.8, 5.0});
    const auto cells = selection.cell_region();
    assert(cells.has_value());
    assert((cells->minimum == Int3{-2, -3, 5}));
    assert((cells->maximum_exclusive == Int3{3, 0, 6}));
    assert(cells->cell_count() == 15);

    const auto blueprint = ExampleBlueprint();
    assert(validate(blueprint).empty());
    const auto path = std::filesystem::temp_directory_path() / "shroudedit-core-test.seblueprint";
    save_blueprint(blueprint, path);
    const auto loaded = load_blueprint(path);
    std::filesystem::remove(path);
    assert(loaded.id == blueprint.id);
    assert(loaded.channels[0].coverage[0] == Coverage::Occupied);
    assert(loaded.channels[0].coverage[1] == Coverage::Empty);

    auto point = Vec3{1, 2, 3};
    for (int i = 0; i < 4; ++i) point = rotate_quarter_turns(point, UpAxis::Z, 1);
    assert(Near(point.x, 1) && Near(point.y, 2) && Near(point.z, 3));
    const auto plan = make_placement_plan(blueprint, {10, 20, 30}, 1);
    assert(plan.props.size() == 1);
    assert(Near(plan.props[0].transform.position.x, 10));
    assert(Near(plan.props[0].transform.position.y, 21));
    assert(Near(plan.props[0].transform.position.z, 30));
    assert(Near(plan.props[0].transform.rotation.z, 0.7071067811865475));
    assert(Near(plan.props[0].transform.rotation.w, 0.7071067811865475));
    assert((plan.channels[0].dimensions == Int3{1, 2, 1}));
    assert(plan.channels[0].values[0] == 17);
    assert(plan.channels[0].coverage[1] == Coverage::Empty);

    auto invalid = blueprint;
    invalid.channels[0].coverage.clear();
    assert(!validate(invalid).empty());

    std::cout << "ShroudEdit core tests passed.\n";
    return 0;
}
