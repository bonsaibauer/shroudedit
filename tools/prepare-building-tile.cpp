#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include "shroudedit/building_tile.h"
#include <filesystem>
#include <iostream>
#include <string>

namespace {
Result (CALL* listResources)(StringView,AssetVisitor,void*);
Result CALL list(StringView,StringView type,AssetVisitor visitor,void* context) { return listResources(type,visitor,context); }
StringView view(const std::string& text) { return {text.data(),text.size()}; }
}
int wmain(int argc,wchar_t** argv) {
    if (argc!=3) { std::cerr << "prepare-building-tile <asset engine DLL> <isolated fixture>\n"; return 1; }
    try {
        const auto fixture=std::filesystem::canonical(argv[2]);
        const auto root=std::filesystem::canonical(SHROUDTOPIA_FIXTURE_ROOT);
        const auto relative=fixture.lexically_relative(root);
        if (relative.empty() || relative=="." || *relative.begin()=="..") return 2;
        const auto dll=LoadLibraryW(argv[1]); if (!dll) return 3;
        auto symbol=[&](const char* name) { auto p=GetProcAddress(dll,name); if (!p) throw std::runtime_error(name); return p; };
        auto open=reinterpret_cast<Result(CALL*)(StringView,StringView)>(symbol("ShroudtopiaAssetsOpen"));
        auto close=reinterpret_cast<void(CALL*)()>(symbol("ShroudtopiaAssetsClose"));
        auto save=reinterpret_cast<Result(CALL*)()>(symbol("ShroudtopiaSaveAssets"));
        listResources=reinterpret_cast<decltype(listResources)>(symbol("ShroudtopiaListAssets"));
        Api api{}; api.list_assets=list;
        api.get_asset=reinterpret_cast<decltype(api.get_asset)>(symbol("ShroudtopiaGetAssetJson"));
        api.create_asset=reinterpret_cast<decltype(api.create_asset)>(symbol("ShroudtopiaCreateAssetJson"));
        api.update_asset=reinterpret_cast<decltype(api.update_asset)>(symbol("ShroudtopiaUpdateAssetJson"));
        const auto bytes=fixture.u8string();
        auto result=open({reinterpret_cast<const char*>(bytes.data()),bytes.size()},view("enshrouded"));
        if (result==RESULT_OK) {
            result=shroudedit::stage_building_test_tile(api,view("mod.shroudedit"));
            if (result==RESULT_OK) result=save();
            close();
        }
        FreeLibrary(dll);
        std::cout << "Prelaunch building tile preparation result=" << result << '\n';
        return result==RESULT_OK ? 0 : 4;
    } catch (const std::exception& error) { std::cerr << error.what() << '\n'; return 5; }
}
