#ifdef NDEBUG
#undef NDEBUG
#endif
#include <windows.h>
#include "shroudedit/building_tile.h"
#include <nlohmann/json.hpp>
#include <cassert>
#include <filesystem>
#include <iostream>
#include <map>
#include <string>

using nlohmann::json;
namespace {
StringView view(const std::string& s) { return {s.data(),s.size()}; }
std::string copy(StringView s) { return {s.data,s.size}; }
constexpr char Owner[]="mod.shroudedit";
struct Resource { std::string type; json data; };
std::map<std::string,Resource> assets;
unsigned counter=0;
Result (CALL* engineList)(StringView,AssetVisitor,void*)=nullptr;
Result CALL list(StringView,const StringView type,AssetVisitor visitor,void* context) {
    if (engineList) return engineList(type,visitor,context);
    for (const auto& [guid,r]:assets) if (r.type==copy(type)) {
        const AssetId key{sizeof(key),view(guid),view(r.type),0};
        if (visitor(&key,context)!=RESULT_OK) return RESULT_CALLBACK_FAILED;
    }
    return RESULT_OK;
}
Result CALL get(StringView,const AssetId* key,char* out,std::size_t size,std::size_t* required) {
    const auto bytes=assets.at(copy(key->guid)).data.dump(); *required=bytes.size();
    if (!out) return RESULT_OK;
    if (size<bytes.size()) return RESULT_INVALID_ARGUMENT;
    std::memcpy(out,bytes.data(),bytes.size()); return RESULT_OK;
}
Result CALL update(StringView,const AssetId* key,StringView text) {
    assets.at(copy(key->guid)).data=json::parse(copy(text)); return RESULT_OK;
}
Result CALL create(StringView,StringView type,StringView text,AssetVisitor visitor,void* context) {
    const auto guid="generated-"+std::to_string(++counter);
    assets.emplace(guid,Resource{copy(type),json::parse(copy(text))});
    const AssetId key{sizeof(key),view(guid),type,0}; return visitor(&key,context);
}
struct Read { const Api* api; std::map<std::string,json> values; };
Result CALL collect(const AssetId* key,void* context) {
    auto& read=*static_cast<Read*>(context);
    std::size_t size{};
    if (read.api->get_asset(view(Owner),key,nullptr,0,&size)!=RESULT_OK) return RESULT_CALLBACK_FAILED;
    std::string bytes(size,'\0');
    if (read.api->get_asset(view(Owner),key,bytes.data(),size,&size)!=RESULT_OK) return RESULT_CALLBACK_FAILED;
    try { read.values.emplace(copy(key->guid),json::parse(bytes)); return RESULT_OK; }
    catch (...) { return RESULT_CALLBACK_FAILED; }
}
auto read(const Api& api,const char* type) {
    Read result{&api,{}};
    assert(api.list_assets(view(Owner),view(type),collect,&result)==RESULT_OK);
    return result.values;
}
void verify(Api& api) {
    const auto before=read(api,"keen::ItemInfo");
    const auto registries=read(api,"keen::ItemRegistryResource");
    const auto blueprints=read(api,"keen::VoxelBlueprintItemRegistryResource");
    assert(shroudedit::stage_building_test_tile(api,view(Owner))==RESULT_OK);
    const auto after=read(api,"keen::ItemInfo");
    assert(after.size()==before.size()+1);
    std::string added;
    json original;
    for (const auto& [id,data]:before) if (data.value("debugName","")=="Blueprint_Voxel_Block_Wall_Straight_4m") original=data;
    for (const auto& [id,data]:after) {
        if (before.contains(id)) { assert(before.at(id)==data); continue; }
        added=id;
        assert(data.at("objectId")==id);
        assert(data.at("itemId").at("value")==0x53450001);
        assert(data.at("debugName")=="ShroudEdit_Test_Wall_4m");
        auto expectedItem=original;
        expectedItem["objectId"]=id;
        expectedItem["itemId"]["value"]=0x53450001;
        expectedItem["debugName"]="ShroudEdit_Test_Wall_4m";
        assert(data==expectedItem);
    }
    const auto registry=read(api,"keen::ItemRegistryResource");
    assert(registry.size()==1);
    auto expected=registries.begin()->second;
    expected["itemRefs"].push_back(added);
    assert(registry.begin()->second==expected);
    const auto bp=read(api,"keen::VoxelBlueprintItemRegistryResource");
    assert(bp.begin()->second.at("blueprintItems").size()==blueprints.begin()->second.at("blueprintItems").size()+1);
    auto expectedBlueprints=blueprints.begin()->second;
    json expectedEntry;
    for (const auto& entry:expectedBlueprints.at("blueprintItems")) if (entry.at("itemId")==original.at("itemId")) expectedEntry=entry;
    expectedEntry["itemId"]["value"]=0x53450001;
    expectedBlueprints["blueprintItems"].push_back(expectedEntry);
    assert(bp.begin()->second==expectedBlueprints);
    assert(shroudedit::stage_building_test_tile(api,view(Owner))==RESULT_CONFLICT);
    assert(read(api,"keen::ItemInfo")==after);
}
}
int wmain(int argc,wchar_t** argv) {
    Api api{}; api.list_assets=list; api.get_asset=get; api.create_asset=create; api.update_asset=update;
    if (argc==1) {
        assets["wall"]={"keen::ItemInfo",{{"debugName","Blueprint_Voxel_Block_Wall_Straight_4m"},{"itemId",{{"value",42}}},{"objectId","wall"}}};
        assets["registry"]={"keen::ItemRegistryResource",{{"itemRefs",json::array({"wall"})}}};
        assets["blueprints"]={"keen::VoxelBlueprintItemRegistryResource",{{"blueprintItems",json::array({{{"itemId",{{"value",42}}},{"data",json::array({255})}}})}}};
        const auto baseline=assets;
        assets["collision"]={"keen::ItemInfo",{{"debugName","Unrelated_Item"},{"itemId",{{"value",0x53450001}}}}};
        assert(shroudedit::stage_building_test_tile(api,view(Owner))==RESULT_CONFLICT);
        assert(assets.size()==baseline.size()+1);
        assets=baseline;
        assets["blueprints"].data["blueprintItems"]=json::array();
        assert(shroudedit::stage_building_test_tile(api,view(Owner))==RESULT_NOT_FOUND);
        assert(assets.size()==baseline.size());
        assets=baseline;
        verify(api);
        assets.clear();
        assert(shroudedit::stage_building_test_tile(api,view(Owner))==RESULT_NOT_FOUND);
        std::cout << "Building tile registration, original preservation and collision tests passed.\n";
        return 0;
    }
    assert(argc==3);
    const auto fixture=std::filesystem::canonical(argv[2]);
    const auto root=std::filesystem::canonical(SHROUDTOPIA_FIXTURE_ROOT);
    const auto relative=fixture.lexically_relative(root);
    if (relative.empty() || relative=="." || *relative.begin()=="..") return 2;
    auto dll=LoadLibraryW(argv[1]); assert(dll);
    auto symbol=[&](const char* name) { auto p=GetProcAddress(dll,name); assert(p); return p; };
    auto open=reinterpret_cast<Result(CALL*)(StringView,StringView)>(symbol("ShroudtopiaAssetsOpen"));
    engineList=reinterpret_cast<decltype(engineList)>(symbol("ShroudtopiaListAssets"));
    api.get_asset=reinterpret_cast<decltype(api.get_asset)>(symbol("ShroudtopiaGetAssetJson"));
    api.create_asset=reinterpret_cast<decltype(api.create_asset)>(symbol("ShroudtopiaCreateAssetJson"));
    api.update_asset=reinterpret_cast<decltype(api.update_asset)>(symbol("ShroudtopiaUpdateAssetJson"));
    auto reset=reinterpret_cast<Result(CALL*)(StringView)>(symbol("ShroudtopiaResetAssets"));
    auto save=reinterpret_cast<Result(CALL*)()>(symbol("ShroudtopiaSaveAssets"));
    auto close=reinterpret_cast<void(CALL*)()>(symbol("ShroudtopiaAssetsClose"));
    const auto path=fixture.u8string();
    assert(open({reinterpret_cast<const char*>(path.data()),path.size()},view("enshrouded"))==RESULT_OK);
    verify(api);
    assert(save()==RESULT_OK);
    assert(reset(view(Owner))==RESULT_OK);
    assert(save()==RESULT_OK);
    close(); FreeLibrary(dll);
    std::cout << "Real game resource registration, serialization and owner rollback passed.\n";
}
