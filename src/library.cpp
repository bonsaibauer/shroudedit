#include "shroudedit/library.h"
#include "shroudedit/blueprint_codec.h"
#include <algorithm>
#include <stdexcept>

namespace shroudedit {
namespace { constexpr std::size_t PerPage = 126; }
BlueprintLibrary::BlueprintLibrary(std::filesystem::path directory, ValidationLimits limits)
    : directory_(std::move(directory)), limits_(limits) {}
std::filesystem::path BlueprintLibrary::path(const std::string& key) const {
    if (key.empty() || key.size()>120 || key.find_first_not_of(
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_")!=std::string::npos)
        throw std::invalid_argument("blueprint key must contain letters, digits, '-' or '_'");
    return directory_/(key+".seblueprint");
}
void BlueprintLibrary::refresh() {
    std::vector<BlueprintTile> next;
    if (std::filesystem::exists(directory_)) {
        for (const auto& file:std::filesystem::directory_iterator(directory_)) {
            // Never follow library links into another directory or savegame.
            if (file.is_symlink() || !file.is_regular_file() || file.path().extension()!=L".seblueprint") continue;
            const auto key=file.path().stem().string();
            BlueprintTile tile{TileKind::Blueprint,key,key,{},true};
            try { const auto blueprint=load(key); if (!blueprint.name.empty()) tile.label=blueprint.name; }
            catch (const std::exception& error) { tile.enabled=false; tile.error=error.what(); }
            next.push_back(std::move(tile));
        }
    }
    std::sort(next.begin(),next.end(),[](const auto& a,const auto& b) { return a.key<b.key; });
    entries_=std::move(next);
}
std::size_t BlueprintLibrary::pages() const { return std::max(std::size_t{1},(entries_.size()+PerPage-1)/PerPage); }
std::vector<BlueprintTile> BlueprintLibrary::tiles(std::size_t page) const {
    if (page>=pages()) throw std::invalid_argument("blueprint page outside library");
    std::vector<BlueprintTile> result{{TileKind::Selection,"","Bereich markieren / XYZ",{},true}};
    const auto first=page*PerPage, last=std::min(entries_.size(),first+PerPage);
    result.insert(result.end(),entries_.begin()+first,entries_.begin()+last);
    return result;
}
Blueprint BlueprintLibrary::load(const std::string& key) const {
    const auto file=path(key);
    if (std::filesystem::is_symlink(file)) throw std::invalid_argument("blueprint links are not supported");
    return load_blueprint(file,limits_);
}
Result BlueprintLibrary::save_new(const std::string& key,const Blueprint& blueprint) {
    const auto destination=path(key);
    if (std::filesystem::exists(std::filesystem::symlink_status(destination))) return RESULT_CONFLICT;
    std::filesystem::create_directories(directory_);
    auto named=blueprint; named.id=key; named.name=key;
    // The existing codec validates and publishes using an atomic file rename.
    // This API is serialized by the mod's editor mutex (not a cross-process lock).
    save_blueprint(named,destination,limits_,false);
    refresh();
    return RESULT_OK;
}
}
