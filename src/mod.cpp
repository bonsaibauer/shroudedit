#include <windows.h>
#include "shroudtopia.h"
#include "shroudedit/blueprint_codec.h"
#include "shroudedit/editor.h"
#include "shroudedit/service.h"
#include "shroudedit/building_tile.h"
#include "shroudedit/library.h"
#include <array>
#include <algorithm>
#include <atomic>
#include <cmath>
#include <cstring>
#include <filesystem>
#include <mutex>
#include <memory>
#include <sstream>
#include <string>
#include <vector>

namespace {
constexpr char ModId[] = "mod.shroudedit";
constexpr char BlueprintContract[] = "shroudedit.blueprints";
const Api* host = nullptr; HMODULE module = nullptr; std::mutex mutex;
shroudedit::Editor editor{ModId}; bool active = false;
std::filesystem::path library; std::vector<Registration> registrations;
std::unique_ptr<shroudedit::BlueprintLibrary> catalog;
bool selectionTool = true;
Registration serviceRegistration = 0;
std::atomic<std::uint32_t> pendingHotkeys{};
std::atomic<std::uint32_t> registeredHotkeys{};
HANDLE hotkeyThread = nullptr, hotkeyStop = nullptr, hotkeyReady = nullptr;
enum HotkeyBit : std::uint32_t { UndoKey=1, MarkKey=2, ClearKey=4, PlaceKey=8, CopyKey=16 };
StringView View(const char* v) { return {v, std::strlen(v)}; }
StringView View(const std::string& v) { return {v.data(), v.size()}; }
void Log(LogLevel level, const std::string& message) { if (host) host->log(View(ModId), level, View(message)); }
DWORD WINAPI HotkeyLoop(void*) {
    struct Binding { int id; UINT key; std::uint32_t bit; };
    constexpr Binding bindings[]{{1,VK_F5,UndoKey},{2,VK_F6,MarkKey},{3,VK_F7,ClearKey},
                                 {4,VK_F8,PlaceKey},{5,VK_F9,CopyKey}};
    MSG message{}; PeekMessageW(&message,nullptr,WM_USER,WM_USER,PM_NOREMOVE);
    std::uint32_t mask{};
    for (const auto& binding:bindings)
        if (RegisterHotKey(nullptr,binding.id,MOD_NOREPEAT,binding.key)) mask|=binding.bit;
    registeredHotkeys.store(mask,std::memory_order_release);
    SetEvent(hotkeyReady);
    for (;;) {
        const auto wait=MsgWaitForMultipleObjects(1,&hotkeyStop,FALSE,INFINITE,QS_ALLINPUT);
        if (wait==WAIT_OBJECT_0) break;
        while (PeekMessageW(&message,nullptr,0,0,PM_REMOVE)) {
            if (message.message!=WM_HOTKEY) continue;
            DWORD process{}; GetWindowThreadProcessId(GetForegroundWindow(),&process);
            if (process!=GetCurrentProcessId()) continue;
            for (const auto& binding:bindings)
                if (message.wParam==binding.id) pendingHotkeys.fetch_or(binding.bit,std::memory_order_release);
        }
    }
    for (const auto& binding:bindings) if (mask&binding.bit) UnregisterHotKey(nullptr,binding.id);
    return 0;
}
void StartHotkeys() {
    if (hotkeyThread) return;
    pendingHotkeys.store(0); registeredHotkeys.store(0);
    hotkeyStop=CreateEventW(nullptr,TRUE,FALSE,nullptr);
    hotkeyReady=CreateEventW(nullptr,TRUE,FALSE,nullptr);
    if (!hotkeyStop || !hotkeyReady) return;
    hotkeyThread=CreateThread(nullptr,0,HotkeyLoop,nullptr,0,nullptr);
    if (hotkeyThread) WaitForSingleObject(hotkeyReady,2000);
}
void StopHotkeys() {
    if (hotkeyStop) SetEvent(hotkeyStop);
    if (hotkeyThread) { WaitForSingleObject(hotkeyThread,2000); CloseHandle(hotkeyThread); }
    if (hotkeyStop) CloseHandle(hotkeyStop);
    if (hotkeyReady) CloseHandle(hotkeyReady);
    hotkeyThread=nullptr; hotkeyStop=nullptr; hotkeyReady=nullptr;
    pendingHotkeys.store(0); registeredHotkeys.store(0);
}
const WorldApi* World() {
    if (!host) return nullptr;
    const ServiceRequest request{sizeof(request), View(WorldServiceId), WorldServiceMajor, WorldServiceMinor};
    const void* value = nullptr;
    if (host->find_service(&request, &value) != RESULT_OK || !value) return nullptr;
    const auto* world = static_cast<const WorldApi*>(value);
    return world->struct_size >= sizeof(WorldApi) && world->version == WorldApiVersion ? world : nullptr;
}
bool Point(std::istream& input, shroudedit::Vec3& p) { return static_cast<bool>(input >> p.x >> p.y >> p.z) && std::isfinite(p.x) && std::isfinite(p.y) && std::isfinite(p.z); }
bool End(std::istream& input) { input >> std::ws; return input.eof(); }
std::string Status() {
    std::ostringstream text;
    text<<"Selection="<<(editor.selection.complete()?"ready":"incomplete")
        <<", tool="<<(selectionTool?"selection":"blueprint")
        <<", blueprint="<<(editor.clipboard()?"loaded":"none")
        <<", plan="<<(editor.preview()?"ready":"none")
        <<", world="<<(World()?"available":"unavailable")
        <<", recovery="<<(editor.recovery_required()?"required":"none");
    if (editor.selection.a()) { const auto p=*editor.selection.a(); text<<"\nA XYZ: "<<p.x<<' '<<p.y<<' '<<p.z; }
    if (editor.selection.b()) { const auto p=*editor.selection.b(); text<<"\nB XYZ: "<<p.x<<' '<<p.y<<' '<<p.z; }
    return text.str();
}
enum class Action { A, B, Clear, Capture, Save, Load, Preview, Paste, Undo, Status, Copy, Tiles, Tile, Mark, Place };
Result CursorPoint(shroudedit::Vec3& point) {
    const auto* world = World();
    if (!world || !world->get_cursor) return RESULT_NOT_AVAILABLE;
    CursorSnapshot cursor{sizeof(cursor)};
    const auto result = world->get_cursor(View(ModId), &cursor);
    if (result != RESULT_OK) return result;
    // The current client reports flag value 1 while exposing a valid, finite
    // building cursor. Treat the flags as opaque until their semantics have
    // been isolated; the native provider already rejects malformed transforms.
    point = {cursor.primary.position.x,cursor.primary.position.y,cursor.primary.position.z};
    return RESULT_OK;
}
Result MarkCursorPoint() {
    if (!selectionTool) return RESULT_PERMISSION_DENIED;
    shroudedit::Vec3 point;
    const auto result=CursorPoint(point);
    if (result!=RESULT_OK) return result;
    const bool pointA=!editor.selection.a() || editor.selection.complete();
    if (pointA) { editor.selection.clear(); editor.selection.set_a(point); }
    else editor.selection.set_b(point);
    std::ostringstream message;
    message<<(pointA ? "Selection A" : "Selection B")<<" = "<<point.x<<' '<<point.y<<' '<<point.z;
    Log(LOG_INFO,message.str());
    return RESULT_OK;
}
Result CALL Command(StringView arguments, void* context) {
    std::scoped_lock lock(mutex); if (!active) return RESULT_PERMISSION_DENIED;
    if ((arguments.size && !arguments.data) || !context) return RESULT_INVALID_ARGUMENT;
    const auto action = *static_cast<const Action*>(context);
    try {
        std::istringstream input(arguments.data ? std::string(arguments.data, arguments.size) : std::string{});
        if (action == Action::Tiles) {
            std::size_t page=1;
            if (!End(input) && (!(input >> page) || !End(input) || !page)) return RESULT_INVALID_ARGUMENT;
            catalog->refresh();
            const auto tiles=catalog->tiles(page-1);
            std::string text="Blueprint page "+std::to_string(page)+"/"+std::to_string(catalog->pages());
            for (std::size_t i=0;i<tiles.size();++i) text+="\n"+std::to_string(i+1)+": "+tiles[i].label+(tiles[i].enabled ? "" : " [disabled: "+tiles[i].error+"]");
            Log(LOG_INFO,text); return RESULT_OK;
        }
        if (action == Action::Tile) {
            std::size_t page=0,index=0;
            if (!(input >> page >> index) || !End(input) || !page || !index) return RESULT_INVALID_ARGUMENT;
            const auto tiles=catalog->tiles(page-1);
            if (index>tiles.size()) return RESULT_NOT_FOUND;
            const auto& tile=tiles[index-1];
            if (!tile.enabled) return RESULT_NOT_AVAILABLE;
            if (tile.kind==shroudedit::TileKind::Selection) selectionTool=true;
            else { editor.set_blueprint(catalog->load(tile.key)); selectionTool=false; }
            Log(LOG_INFO,selectionTool ? "Selection tool active; placing is blocked." : "Blueprint selected: "+tile.key);
            return RESULT_OK;
        }
        if (action == Action::Mark) {
            if (!End(input)) return RESULT_INVALID_ARGUMENT;
            return MarkCursorPoint();
        }
        if (action == Action::A || action == Action::B) {
            shroudedit::Vec3 p;
            if (End(input)) { const auto result=CursorPoint(p); if (result != RESULT_OK) return result; }
            else if (!Point(input,p) || !End(input)) return RESULT_INVALID_ARGUMENT;
            if (action == Action::A) editor.selection.set_a(p); else editor.selection.set_b(p);
            Log(LOG_INFO,"Selection point set."); return RESULT_OK;
        }
        if (action == Action::Clear) { if (!End(input)) return RESULT_INVALID_ARGUMENT; editor.selection.clear(); Log(LOG_INFO,"Selection cleared."); return RESULT_OK; }
        if (action == Action::Save || action == Action::Load) { std::string name; if (!(input >> name) || !End(input)) return RESULT_INVALID_ARGUMENT; (void)catalog->path(name); if (action == Action::Save) { if (!editor.clipboard()) return RESULT_NOT_FOUND; const auto result=catalog->save_new(name,*editor.clipboard()); if (result!=RESULT_OK) return result; } else { editor.set_blueprint(catalog->load(name)); selectionTool=false; } Log(LOG_INFO,action == Action::Save ? "Blueprint saved; library updated." : "Blueprint loaded."); return RESULT_OK; }
        if (action == Action::Preview) { shroudedit::Vec3 p; int turns = 0; if (!Point(input,p) || !(input >> turns) || !End(input) || turns < 0 || turns > 3) return RESULT_INVALID_ARGUMENT; editor.prepare(p,static_cast<std::uint8_t>(turns)); Log(LOG_INFO,"Placement plan prepared."); return RESULT_OK; }
        if (action == Action::Status) { if (!End(input)) return RESULT_INVALID_ARGUMENT; Log(LOG_INFO,Status()); return RESULT_OK; }
        const auto* world = World(); if (!world) { Log(LOG_WARNING,"World service unavailable; live capture/paste cannot run."); return RESULT_NOT_AVAILABLE; }
        if (action == Action::Copy) {
            std::string name,axis,scope,id;
            if (!(input >> name >> axis >> scope) || (axis!="x" && axis!="y" && axis!="z") || (scope!="props" && scope!="grid" && scope!="all")) return RESULT_INVALID_ARGUMENT;
            const auto path=catalog->path(name);
            if (std::filesystem::exists(std::filesystem::symlink_status(path))) return RESULT_CONFLICT;
            std::vector<std::string> grids; while (input >> id) grids.push_back(id);
            if ((scope=="props" && !grids.empty()) || (scope!="props" && grids.empty())) return RESULT_INVALID_ARGUMENT;
            const auto axisValue=axis=="x" ? shroudedit::UpAxis::X : axis=="y" ? shroudedit::UpAxis::Y : shroudedit::UpAxis::Z;
            const auto captured=editor.capture(*world,scope!="grid",grids,axisValue);
            if (captured!=RESULT_OK) return captured;
            for (const auto& channel:editor.clipboard()->channels)
                if (std::find(channel.coverage.begin(),channel.coverage.end(),shroudedit::Coverage::Unknown)!=channel.coverage.end())
                    return RESULT_NOT_AVAILABLE; // Do not publish an incomplete area as a usable tile.
            const auto saved=catalog->save_new(name,*editor.clipboard());
            if (saved==RESULT_OK) Log(LOG_INFO,"Copied area saved as blueprint tile: "+name);
            return saved;
        }
        if (action == Action::Place) {
            int turns=0;
            if (!End(input) && (!(input >> turns) || !End(input) || turns<0 || turns>3)) return RESULT_INVALID_ARGUMENT;
            if (selectionTool) return RESULT_PERMISSION_DENIED;
            shroudedit::Vec3 p; const auto result=CursorPoint(p); if (result!=RESULT_OK) return result;
            editor.prepare(p,static_cast<std::uint8_t>(turns));
            const auto placed=editor.paste(*world,shroudedit::PasteMode::Additive);
            return placed.result;
        }
        if (action==Action::Paste && selectionTool) return RESULT_PERMISSION_DENIED;
        if (action == Action::Capture) { std::string axis,scope; if (!(input >> axis >> scope) || (axis != "x" && axis != "y" && axis != "z") || (scope != "props" && scope != "grid" && scope != "all")) return RESULT_INVALID_ARGUMENT; std::vector<std::string> grids; std::string id; while (input >> id) grids.push_back(id); if ((scope == "props" && !grids.empty()) || (scope != "props" && grids.empty())) return RESULT_INVALID_ARGUMENT; const auto up = axis == "x" ? shroudedit::UpAxis::X : axis == "y" ? shroudedit::UpAxis::Y : shroudedit::UpAxis::Z; return editor.capture(*world,scope != "grid",grids,up); }
        shroudedit::EditResult result; if (action == Action::Undo) { if (!End(input)) return RESULT_INVALID_ARGUMENT; result = editor.undo(*world); } else { std::string mode; if (!(input >> mode) || !End(input) || (mode != "add" && mode != "replace-cells")) return RESULT_INVALID_ARGUMENT; result = editor.paste(*world,mode == "add" ? shroudedit::PasteMode::Additive : shroudedit::PasteMode::ReplaceCells); } return result.result;
    } catch (const std::invalid_argument& error) { Log(LOG_ERROR,error.what()); return RESULT_INVALID_ARGUMENT; } catch (const std::exception& error) { Log(LOG_ERROR,error.what()); return RESULT_INTERNAL_ERROR; }
}
int __cdecl ValidateBlueprintFile(const char* path, ShroudEditBlueprintValidationV1* result) { if (!path || !result || result->struct_size < sizeof(*result)) return 1; try { const auto blueprint=shroudedit::load_blueprint(std::filesystem::u8path(path)); result->error_count=static_cast<std::uint32_t>(shroudedit::validate(blueprint).size()); return result->error_count?2:0; } catch (...) { result->error_count=1; return 3; } }
ShroudEditBlueprintServiceV1 blueprintService{sizeof(blueprintService),ValidateBlueprintFile};
struct CommandSpec { const char* id; const char* help; Action action; };
std::array commands{CommandSpec{"shroudedit.select_a","x y z",Action::A},CommandSpec{"shroudedit.select_b","x y z",Action::B},CommandSpec{"shroudedit.selection.clear","Clear selection",Action::Clear},CommandSpec{"shroudedit.capture","<x|y|z> <props|grid|all> [grid IDs]",Action::Capture},CommandSpec{"shroudedit.save","<blueprint name>",Action::Save},CommandSpec{"shroudedit.load","<blueprint name>",Action::Load},CommandSpec{"shroudedit.preview","x y z <0..3>",Action::Preview},CommandSpec{"shroudedit.paste","<add|replace-cells>",Action::Paste},CommandSpec{"shroudedit.undo","Undo last edit",Action::Undo},CommandSpec{"shroudedit.status","Report editor status",Action::Status}};
std::array tileCommands{CommandSpec{"shroudedit.copy","<name> <x|y|z> <props|grid|all> [grid IDs]",Action::Copy},CommandSpec{"shroudedit.tiles","[page starting at 1]",Action::Tiles},CommandSpec{"shroudedit.tile","<page> <tile> (both starting at 1)",Action::Tile},CommandSpec{"shroudedit.mark","Mark next A/B point at cursor",Action::Mark},CommandSpec{"shroudedit.place","[0..3 turns] Place selected blueprint at cursor, additive",Action::Place}};
Result CALL Load(const Api* api, void*) {
    if (!api || api->api_version != API_VERSION) return RESULT_VERSION_MISMATCH; host=api; wchar_t path[32768]{}; const auto size=GetModuleFileNameW(module,path,32768); if (!size || size >= 32768) return RESULT_INTERNAL_ERROR; library=std::filesystem::path(std::wstring(path,size)).parent_path()/L"blueprints";
    double cells=16777216, props=1000000; if (api->get_mod_setting_number(View(ModId),View("maximumCells"),cells,&cells) != RESULT_OK || api->get_mod_setting_number(View(ModId),View("maximumProps"),props,&props) != RESULT_OK || !std::isfinite(cells) || !std::isfinite(props) || cells < 1 || cells > 16777216 || props < 1 || props > 1000000 || std::floor(cells) != cells || std::floor(props) != props) return RESULT_INVALID_ARGUMENT;
    editor=shroudedit::Editor(ModId,{static_cast<std::uint64_t>(cells),static_cast<std::uint32_t>(props)});
    catalog=std::make_unique<shroudedit::BlueprintLibrary>(library,editor.limits());
    catalog->refresh(); selectionTool=true;
    const ServiceDescriptor service{sizeof(service),View(BlueprintContract),SHROUDEDIT_BLUEPRINT_SERVICE_MAJOR,SHROUDEDIT_BLUEPRINT_SERVICE_MINOR,&blueprintService};
    return api->register_service(View(ModId),&service,&serviceRegistration);
}
Result CALL Activate(const Api* api, void*) {
    if (!api) return RESULT_INVALID_ARGUMENT;
    std::scoped_lock lock(mutex);
    if (active) return RESULT_CONFLICT;
    std::vector<CommandSpec*> allCommands;
    for (auto& command:commands) allCommands.push_back(&command);
    for (auto& command:tileCommands) allCommands.push_back(&command);
    for (auto* spec:allCommands) {
        auto& command=*spec;
        const CommandDescriptor descriptor{sizeof(descriptor),View(command.id),View(command.help),Command,&command.action};
        Registration registration{};
        const auto result=api->register_command(View(ModId),&descriptor,&registration);
        if (result != RESULT_OK) {
            for (auto handle:registrations) api->release_registration(handle);
            registrations.clear();
            return result;
        }
        registrations.push_back(registration);
    }
    std::uint8_t testTile=0;
    auto tileResult=api->get_mod_setting_bool ? api->get_mod_setting_bool(View(ModId),View("diagnosticBuildTile"),0,&testTile) : RESULT_OK;
    if (tileResult != RESULT_OK) {
        for (auto handle:registrations) api->release_registration(handle);
        registrations.clear();
        Log(LOG_ERROR,"Building test tile failed; activation cancelled.");
        return tileResult;
    }
    // Game resources load before on_activate completes. Never rewrite KFC files
    // here: tools/install-building-test.ps1 prepares the diagnostic tile offline.
    if (testTile) Log(LOG_DEBUG,"Prelaunch building test profile expected: item 1397030913. No runtime asset writes; visibility still requires in-game verification.");
    active=true;
    StartHotkeys();
    Log(LOG_INFO,"ShroudEdit ready. F6 marks A/B, F7 resets selection and current blueprint, F9 copies, F8 places the current blueprint, F5 undoes; F10 opens Debug Console.");
    const auto hotkeys=registeredHotkeys.load(std::memory_order_acquire);
    if (hotkeys!=(UndoKey|MarkKey|ClearKey|PlaceKey|CopyKey))
        Log(LOG_WARNING,"One or more ShroudEdit hotkeys could not be registered; commands remain available.");
    Log(LOG_DEBUG,Status());
    return RESULT_OK;
}
Result CALL Update(const Api*, void*, double delta) {
    std::scoped_lock lock(mutex);
    const auto pressed=pendingHotkeys.exchange(0,std::memory_order_acq_rel);
    if (pressed&MarkKey) {
        if (!selectionTool) { selectionTool=true; editor.selection.clear(); }
        const auto result=MarkCursorPoint();
        if (result!=RESULT_OK) Log(LOG_WARNING,"F6 failed: result="+std::to_string(result));
    }
    if (pressed&ClearKey) {
        if (editor.recovery_required()) {
            Log(LOG_WARNING,"F7 reset refused: an incomplete edit must be recovered with F5 first.");
        } else {
            editor.reset();
            selectionTool=true;
            Log(LOG_INFO,"F7 full reset completed: selection, current blueprint, placement plan and undo history cleared.");
        }
        return RESULT_OK; // Reset has priority over any other hotkey queued in this update.
    }
    if (pressed&CopyKey) {
        const auto* world=World();
        auto result=world ? editor.capture(*world,true,{"voxel"},shroudedit::UpAxis::Y) : RESULT_NOT_AVAILABLE;
        std::string name;
        if (result==RESULT_OK) {
            std::size_t occupied=0;
            for (const auto& channel:editor.clipboard()->channels)
                occupied+=static_cast<std::size_t>(std::count(channel.coverage.begin(),channel.coverage.end(),shroudedit::Coverage::Occupied));
            const auto props=editor.clipboard()->props.size();
            Log(LOG_INFO,"F9 capture contents: occupied voxels="+std::to_string(occupied)+" props="+std::to_string(props));
            if (!occupied && !props) {
                Log(LOG_WARNING,"F9 captured only empty cells. Check selection and world context; F8 will not place this empty blueprint.");
            }
            for (std::size_t index=1;index<=999999;++index) {
                const auto candidate="capture-"+std::to_string(index);
                if (!std::filesystem::exists(std::filesystem::symlink_status(catalog->path(candidate)))) { name=candidate; break; }
            }
            result=name.empty() ? RESULT_CONFLICT : catalog->save_new(name,*editor.clipboard());
        }
        if (result==RESULT_OK) selectionTool=false;
        const auto message=result==RESULT_OK ? "Saved world blueprint (voxels + props) and selected it for placement: "+name : "F9 copy failed: result="+std::to_string(result);
        Log(result==RESULT_OK ? LOG_INFO : LOG_WARNING,message);
    }
    if (pressed&PlaceKey) {
        const auto* world=World();
        shroudedit::Vec3 point;
        auto result=world ? CursorPoint(point) : RESULT_NOT_AVAILABLE;
        shroudedit::EditResult placed{result};
        if (result==RESULT_OK) {
            try { editor.prepare(point,0); placed=editor.paste(*world,shroudedit::PasteMode::ReplaceAll); }
            catch (const std::invalid_argument&) { placed.result=RESULT_INVALID_ARGUMENT; }
            catch (...) { placed.result=RESULT_INTERNAL_ERROR; }
        }
        const auto message=placed.result==RESULT_OK ?
            "F8 placed blueprint at "+std::to_string(point.x)+" "+std::to_string(point.y)+" "+std::to_string(point.z) :
            "F8 placement failed: result="+std::to_string(placed.result)+
                (placed.recovery_required ? " (rollback incomplete; use undo before continuing)" : " (no successful placement; applied changes rolled back)");
        Log(placed.result==RESULT_OK ? LOG_INFO : LOG_WARNING,message);
    }
    if (pressed&UndoKey) {
        const auto* world=World();
        const auto undone=world ? editor.undo(*world) : shroudedit::EditResult{RESULT_NOT_AVAILABLE};
        Log(undone.result==RESULT_OK ? LOG_INFO : LOG_WARNING,
            undone.result==RESULT_OK ? "F5 undo completed." : "F5 undo failed: result="+std::to_string(undone.result));
    }
    (void)delta;
    return RESULT_OK;
}
Result CALL Deactivate(const Api* api, void*) { StopHotkeys(); std::scoped_lock lock(mutex); if (editor.recovery_required()) return RESULT_INTERNAL_ERROR; active=false; selectionTool=true; for (const auto registration:registrations) api->release_registration(registration); registrations.clear(); editor.reset(); return RESULT_OK; }
Result CALL Unload(const Api* api, void*) {
    const auto result=Deactivate(api,nullptr);
    if (result != RESULT_OK) return result;
    std::scoped_lock lock(mutex);
    if (serviceRegistration) api->release_registration(serviceRegistration);
    serviceRegistration=0;
    host=nullptr;
    return RESULT_OK;
}
}
extern "C" __declspec(dllexport) Result CALL CreateMod(std::uint32_t version, ModDescriptor* descriptor) { if (version != API_VERSION) return RESULT_VERSION_MISMATCH; if (!descriptor || descriptor->struct_size < sizeof(*descriptor)) return RESULT_INVALID_ARGUMENT; *descriptor={sizeof(*descriptor),View(ModId),nullptr,Load,Activate,Update,Deactivate,Unload}; return RESULT_OK; }
BOOL APIENTRY DllMain(HMODULE handle,DWORD reason,LPVOID) { if (reason == DLL_PROCESS_ATTACH) { module=handle; DisableThreadLibraryCalls(handle); } return TRUE; }
