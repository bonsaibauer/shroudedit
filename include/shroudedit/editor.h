#pragma once

#include "shroudedit/placement.h"
#include "shroudedit/selection.h"
#include "shroudedit/world_api.h"
#include <optional>
#include <string>
#include <vector>

namespace shroudedit {
    enum class PasteMode { Additive, ReplaceCells, ReplaceAll };
    struct EditResult {
        Result result{RESULT_OK};
        std::size_t spawned{};
        std::size_t grids_written{};
        bool recovery_required{};
    };

    // No game memory or hooks: every live operation uses Shroudtopia's API.
    class Editor {
    public:
        explicit Editor(std::string owner, ValidationLimits limits = {});
        Selection selection;
        [[nodiscard]] const std::optional<Blueprint>& clipboard() const { return clipboard_; }
        [[nodiscard]] const std::optional<PlacementPlan>& preview() const { return preview_; }
        [[nodiscard]] bool has_history() const { return history_.has_value(); }
        [[nodiscard]] bool recovery_required() const { return recovery_; }
        [[nodiscard]] const ValidationLimits& limits() const { return limits_; }
        void reset();
        void set_blueprint(Blueprint blueprint);
        void prepare(Vec3 target, std::uint8_t turns);
        Result capture(const WorldApi& world, bool props, const std::vector<std::string>& grids, UpAxis up_axis);
        EditResult paste(const WorldApi& world, PasteMode mode);
        EditResult undo(const WorldApi& world);
    private:
        struct GridEdit {
            std::string id;
            WorldBounds bounds{};
            std::vector<std::uint32_t> before, after;
            std::vector<WorldCellState> before_coverage, after_coverage;
            bool applied{};
        };
        struct CreatedProp { WorldEntityHandle handle; WorldTransform transform; };
        struct RemovedProp { WorldEntityHandle handle; std::string template_id; WorldTransform transform; bool applied{}; };
        struct History { std::vector<CreatedProp> props; std::vector<RemovedProp> removed; std::vector<GridEdit> grids; };
        std::string owner_;
        ValidationLimits limits_;
        std::optional<Blueprint> clipboard_;
        std::optional<PlacementPlan> preview_;
        std::optional<History> history_;
        bool recovery_{};
        StringView owner() const;
        Result read_grid(const WorldApi&, GridEdit&, std::size_t);
        EditResult restore(const WorldApi&, bool check_current);
    };
}
