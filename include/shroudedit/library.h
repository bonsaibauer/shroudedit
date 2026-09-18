#pragma once
#include "shroudedit/editor.h"
#include <filesystem>

namespace shroudedit {
enum class TileKind { Selection, Blueprint };
struct BlueprintTile {
    TileKind kind{TileKind::Selection};
    std::string key, label, error;
    bool enabled{true};
};
// Domain catalog, not an engine adapter. Index zero on every page is the
// selection tool; remaining entries have stable file keys, not engine IDs.
class BlueprintLibrary {
public:
    explicit BlueprintLibrary(std::filesystem::path directory, ValidationLimits limits = {});
    void refresh();
    [[nodiscard]] std::vector<BlueprintTile> tiles(std::size_t page = 0) const;
    [[nodiscard]] std::size_t pages() const;
    Result save_new(const std::string& key, const Blueprint& blueprint);
    Blueprint load(const std::string& key) const;
    [[nodiscard]] std::filesystem::path path(const std::string& key) const;
private:
    std::filesystem::path directory_;
    ValidationLimits limits_;
    std::vector<BlueprintTile> entries_;
};
}
