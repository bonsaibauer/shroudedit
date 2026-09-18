#ifdef NDEBUG
#undef NDEBUG
#endif
#include <windows.h>
#include "shroudtopia.h"
#include "shroudtopia_world.h"
#include "shroudedit/blueprint_codec.h"
#include <cassert>
#include <cstring>
#include <filesystem>
#include <iostream>
#include <map>
#include <string>

namespace {
std::map<std::string, CommandDescriptor> commands;
std::map<Registration, std::string> handles;
Registration next = 1; std::string lastLog;
Registration serviceHandle = 0;
bool cursorAvailable = false;
int spawnCount=0;
Result CALL Query(StringView,const WorldBounds*,WorldEntity* out,size_t capacity,size_t* count) {
    *count=1; if (!capacity) return RESULT_OK;
    assert(out && capacity>=1);
    static const char id[]="template.table";
    out[0]={sizeof(*out),{1,1,0},{id,sizeof(id)-1},{sizeof(WorldTransform),{.5,.5,.5},{0,0,0,1},{1,1,1}},WorldEntityKind::Prop};
    return RESULT_OK;
}
Result CALL Spawn(StringView,StringView,const WorldTransform*,WorldEntityHandle* out) { *out={static_cast<uint64_t>(++spawnCount),1,0}; return RESULT_OK; }
Result CALL Destroy(StringView,WorldEntityHandle) { return RESULT_OK; }
Result CALL Transform(StringView,WorldEntityHandle,WorldTransform*) { return RESULT_NOT_FOUND; }
Result CALL Grid(StringView,StringView,WorldGridSpec* out) { *out={sizeof(*out),{},{0,0,0},{1,1,1},32,32,32}; return RESULT_OK; }
Result CALL UnknownGrid(StringView,StringView,const WorldBounds*,std::uint32_t* cells,WorldCellState* states,size_t capacity,size_t* count) {
    *count=capacity;
    for (size_t i=0;i<capacity;++i) { cells[i]=0; states[i]=WorldCellState::Unknown; }
    return RESULT_OK;
}
Result CALL Cursor(StringView,CursorSnapshot* value) {
    if (!cursorAvailable) return RESULT_NOT_AVAILABLE;
    *value={sizeof(*value)};
    value->primary.position={1,2,3};
    return RESULT_OK;
}
WorldApi world=[] { WorldApi value{sizeof(value),WorldApiVersion}; value.get_cursor=Cursor; value.query_entities=Query; value.spawn_entity=Spawn; value.destroy_entity=Destroy; value.get_entity_transform=Transform; return value; }();
StringView View(const std::string& value) { return {value.data(),value.size()}; }
std::string Copy(StringView value) { return {value.data,value.size}; }
Result CALL RegisterService(StringView,const ServiceDescriptor*,Registration* out) { *out=next++; serviceHandle=*out; return RESULT_OK; }
Result CALL FindService(const ServiceRequest*,const void** out) { *out=cursorAvailable ? &world : nullptr; return cursorAvailable ? RESULT_OK : RESULT_NOT_FOUND; }
Result CALL RegisterCommand(StringView,const CommandDescriptor* descriptor,Registration* out) { const auto id=Copy(descriptor->command_id); commands.emplace(id,*descriptor); *out=next++; handles.emplace(*out,id); return RESULT_OK; }
Result CALL Release(Registration registration) { if (registration==serviceHandle) serviceHandle=0; if (const auto found=handles.find(registration); found != handles.end()) { commands.erase(found->second); handles.erase(found); } return RESULT_OK; }
Result CALL Number(StringView,StringView,double fallback,double* out) { *out=fallback; return RESULT_OK; }
Result CALL Log(StringView,LogLevel,StringView text) { lastLog=Copy(text); return RESULT_OK; }
Result Call(const std::string& id,const std::string& args="") { const auto& command=commands.at(id); return command.callback(View(args),command.user_data); }
}
int wmain(int argc,wchar_t** argv) {
    assert(argc==2); const auto dllPath=std::filesystem::absolute(argv[1]); const auto module=LoadLibraryW(dllPath.c_str()); assert(module);
    using Create=Result (CALL*)(std::uint32_t,ModDescriptor*); const auto create=reinterpret_cast<Create>(GetProcAddress(module,"CreateMod")); assert(create);
    ModDescriptor mod{sizeof(mod)}; assert(create(API_VERSION,&mod)==RESULT_OK);
    Api api{}; api.struct_size=sizeof(api); api.api_version=API_VERSION; api.register_service=RegisterService; api.find_service=FindService; api.register_command=RegisterCommand; api.release_registration=Release; api.get_mod_setting_number=Number; api.log=Log;
    assert(mod.on_load(&api,nullptr)==RESULT_OK && commands.empty());
    assert(mod.on_activate(&api,nullptr)==RESULT_OK && commands.size()==15);
    assert(Call("shroudedit.tile","1 1")==RESULT_OK);
    assert(Call("shroudedit.mark")==RESULT_NOT_AVAILABLE);
    assert(Call("shroudedit.select_a")==RESULT_NOT_AVAILABLE);
    cursorAvailable=true;
    assert(Call("shroudedit.select_a")==RESULT_OK);
    cursorAvailable=false;
    assert(Call("shroudedit.select_a","0 0 0 trailing")==RESULT_INVALID_ARGUMENT); assert(Call("shroudedit.select_a","0 0 0")==RESULT_OK); assert(Call("shroudedit.select_b","2 2 2")==RESULT_OK);
    assert(Call("shroudedit.capture","z props")==RESULT_NOT_AVAILABLE); assert(Call("shroudedit.load","../../savegame")==RESULT_INVALID_ARGUMENT);
    shroudedit::Blueprint blueprint; blueprint.id="test.library"; blueprint.extent={1,1,1}; blueprint.props.push_back({1,"template.table",{{.5,.5,.5},{},{1,1,1}}});
    const auto name="test-"+std::to_string(GetCurrentProcessId()); const auto library=dllPath.parent_path()/"blueprints"; const auto input=library/(name+".seblueprint"), output=library/(name+"-saved.seblueprint"); const auto existed=std::filesystem::exists(library); std::filesystem::create_directories(library); shroudedit::save_blueprint(blueprint,input);
    assert(Call("shroudedit.load",name)==RESULT_OK); assert(Call("shroudedit.preview","10 20 30 1")==RESULT_OK); assert(Call("shroudedit.status")==RESULT_OK && lastLog.find("plan=ready") != std::string::npos); assert(Call("shroudedit.save",name+"-saved")==RESULT_OK); assert(shroudedit::load_blueprint(output).props.size()==1); assert(Call("shroudedit.paste","add")==RESULT_NOT_AVAILABLE);
    assert(Call("shroudedit.save",name+"-saved")==RESULT_CONFLICT);
    cursorAvailable=true;
    assert(Call("shroudedit.tile","1 1")==RESULT_OK);
    assert(Call("shroudedit.place")==RESULT_PERMISSION_DENIED && spawnCount==0);
    assert(Call("shroudedit.paste","add")==RESULT_PERMISSION_DENIED && spawnCount==0);
    assert(Call("shroudedit.copy",name+"-copy z props")==RESULT_OK);
    const auto copied=library/(name+"-copy.seblueprint");
    assert(shroudedit::load_blueprint(copied).name==name+"-copy");
    world.query_entities=nullptr;
    assert(Call("shroudedit.copy",name+"-failed z props")==RESULT_NOT_AVAILABLE);
    assert(!std::filesystem::exists(library/(name+"-failed.seblueprint")));
    world.query_entities=Query;
    world.get_grid_spec=Grid; world.read_grid_region=UnknownGrid;
    assert(Call("shroudedit.copy",name+"-unknown z grid terrain")==RESULT_NOT_AVAILABLE);
    assert(!std::filesystem::exists(library/(name+"-unknown.seblueprint")));
    assert(Call("shroudedit.tiles")==RESULT_OK && lastLog.find(name+"-copy")!=std::string::npos);
    assert(Call("shroudedit.load",name+"-copy")==RESULT_OK);
    assert(Call("shroudedit.mark")==RESULT_PERMISSION_DENIED);
    assert(Call("shroudedit.place","1")==RESULT_OK && spawnCount==1);
    assert(Call("shroudedit.tile","1 1")==RESULT_OK);
    assert(Call("shroudedit.mark")==RESULT_OK);
    assert(Call("shroudedit.mark")==RESULT_OK);
    assert(Call("shroudedit.tile","0 1")==RESULT_INVALID_ARGUMENT);
    assert(mod.on_deactivate(&api,nullptr)==RESULT_OK && commands.empty() && serviceHandle);
    assert(mod.on_activate(&api,nullptr)==RESULT_OK && commands.size()==15 && serviceHandle);
    assert(mod.on_unload(&api,nullptr)==RESULT_OK && commands.empty() && !serviceHandle);
    FreeLibrary(module); std::filesystem::remove(input); std::filesystem::remove(output); std::filesystem::remove(copied); if (!existed) std::filesystem::remove(library); std::cout << "ShroudEdit DLL lifecycle, commands and library tests passed.\n";
}
