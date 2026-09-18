#include "shroudedit/building_tile.h"
#include <nlohmann/json.hpp>
#include <string>
#include <vector>
#include <stdexcept>

namespace shroudedit {
namespace {
using nlohmann::json;
constexpr std::uint32_t TestId = 0x53450001;
constexpr char Template[] = "Blueprint_Voxel_Block_Wall_Straight_4m";
StringView view(const std::string& s) { return {s.data(),s.size()}; }
struct Resource {
    std::string guid, type;
    std::uint32_t part{};
    json data;
    AssetId key() const { return {sizeof(AssetId),view(guid),view(type),part}; }
};
void check(Result result) { if (result != RESULT_OK) throw result; }
struct Scan {
    const Api& api;
    StringView owner;
    std::vector<Resource> resources;
    bool items{};
    Result failure{RESULT_OK};
};
Result CALL collect(const AssetId* key, void* user) {
    auto& scan=*static_cast<Scan*>(user);
    try {
        std::size_t length{};
        check(scan.api.get_asset(scan.owner,key,nullptr,0,&length));
        if (length > 64*1024*1024) throw RESULT_INVALID_ARGUMENT;
        std::string bytes(length,'\0');
        check(scan.api.get_asset(scan.owner,key,bytes.data(),bytes.size(),&length));
        if (length > bytes.size()) throw RESULT_INTERNAL_ERROR;
        auto data=json::parse(bytes.begin(),bytes.begin()+length);
        if (scan.items) {
            if (data.at("itemId").at("value").get<std::uint32_t>() == TestId ||
                data.value("debugName","") == "ShroudEdit_Test_Wall_4m") throw RESULT_CONFLICT;
            if (data.value("debugName","") != Template) return RESULT_OK;
        }
        scan.resources.push_back({std::string(key->guid.data,key->guid.size),std::string(key->type_name.data,key->type_name.size),key->part,std::move(data)});
        return RESULT_OK;
    } catch (Result result) { scan.failure=result; return result; }
      catch (...) { scan.failure=RESULT_INVALID_ARGUMENT; return scan.failure; }
}
Resource single(const Api& api,StringView owner,const std::string& type,bool items=false) {
    Scan scan{api,owner,{},items};
    const auto result=api.list_assets(owner,view(type),collect,&scan);
    if (scan.failure != RESULT_OK) throw scan.failure;
    check(result);
    if (scan.resources.empty()) throw RESULT_NOT_FOUND;
    if (scan.resources.size()!=1) throw RESULT_CONFLICT;
    return std::move(scan.resources.front());
}
Result CALL created(const AssetId* key,void* user) {
    try {
        auto& resource=*static_cast<Resource*>(user);
        resource.guid.assign(key->guid.data,key->guid.size);
        resource.type.assign(key->type_name.data,key->type_name.size);
        resource.part=key->part;
        return RESULT_OK;
    } catch (...) { return RESULT_INTERNAL_ERROR; }
}
void update(const Api& api,StringView owner,const Resource& resource) {
    const auto key=resource.key(); const auto text=resource.data.dump();
    check(api.update_asset(owner,&key,view(text)));
}
}

Result stage_building_test_tile(const Api& api,StringView owner) {
    if (!api.list_assets || !api.get_asset || !api.create_asset || !api.update_asset)
        return RESULT_NOT_AVAILABLE;
    try {
        auto item=single(api,owner,"keen::ItemInfo",true);
        auto registry=single(api,owner,"keen::ItemRegistryResource");
        auto blueprints=single(api,owner,"keen::VoxelBlueprintItemRegistryResource");
        const auto originalId=item.data.at("itemId").at("value");
        auto& entries=blueprints.data.at("blueprintItems");
        if (!entries.is_array() || !registry.data.at("itemRefs").is_array()) return RESULT_INVALID_ARGUMENT;
        json entry; std::size_t matches=0;
        for (const auto& candidate:entries) {
            if (candidate.at("itemId").at("value")==TestId) return RESULT_CONFLICT;
            if (candidate.at("itemId").at("value")==originalId) { entry=candidate; ++matches; }
        }
        if (matches != 1) return matches ? RESULT_CONFLICT : RESULT_NOT_FOUND;
        // Preserve dimensions, compressed data, placement bounds, tags and model.
        // This tests registration independently of voxel format conversion.
        item.data["itemId"]["value"]=TestId;
        item.data["debugName"]="ShroudEdit_Test_Wall_4m";
        const auto text=item.data.dump();
        check(api.create_asset(owner,view(item.type),view(text),created,&item));
        item.data["objectId"]=item.guid;
        update(api,owner,item);
        entry["itemId"]["value"]=TestId;
        entries.push_back(std::move(entry));
        registry.data["itemRefs"].push_back(item.guid);
        update(api,owner,blueprints);
        update(api,owner,registry);
        return RESULT_OK;
    } catch (Result result) { return result; }
      catch (...) { return RESULT_INVALID_ARGUMENT; }
}
}
