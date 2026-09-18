#ifdef NDEBUG
#undef NDEBUG
#endif
#include "shroudedit/library.h"
#include "shroudedit/blueprint_codec.h"
#include <cassert>
#include <chrono>
#include <fstream>
#include <iostream>
using namespace shroudedit;
int main() {
    const auto directory=std::filesystem::temp_directory_path()/
        ("shroudedit-library-"+std::to_string(std::chrono::steady_clock::now().time_since_epoch().count()));
    assert(std::filesystem::create_directory(directory));
    BlueprintLibrary library(directory);
    library.refresh();
    assert(library.pages()==1 && library.tiles().size()==1);
    assert(library.tiles()[0].kind==TileKind::Selection);
    Blueprint b; b.id="source"; b.extent={1,1,1};
    b.props.push_back({1,"template.table",{{.5,.5,.5},{},{1,1,1}}});
    assert(library.save_new("house",b)==RESULT_OK);
    assert(library.tiles().size()==2 && library.tiles()[1].key=="house");
    assert(library.load("house").name=="house");
    assert(library.save_new("house",b)==RESULT_CONFLICT);
    bool rejected=false;
    try { (void)library.path("../savegame"); } catch (const std::invalid_argument&) { rejected=true; }
    assert(rejected);
    rejected=false;
    try { save_blueprint(b,library.path("house"),{},false); } catch (const std::runtime_error&) { rejected=true; }
    assert(rejected && library.load("house").name=="house");
    { std::ofstream file(directory/"broken.seblueprint"); file<<"not a blueprint"; }
    for (int i=0;i<126;++i) save_blueprint(b,directory/("entry-"+std::to_string(i)+".seblueprint"));
    library.refresh();
    assert(library.pages()==2 && library.tiles(0).size()==127 && library.tiles(1).size()==3);
    assert(library.tiles(1)[0].kind==TileKind::Selection);
    assert(!library.tiles(0)[1].enabled && !library.tiles(0)[1].error.empty());
    rejected=false;
    try { (void)library.tiles(2); } catch (const std::invalid_argument&) { rejected=true; }
    assert(rejected);
    // Only files created by this isolated test are removed.
    for (const auto& entry:std::filesystem::directory_iterator(directory)) std::filesystem::remove(entry.path());
    std::filesystem::remove(directory);
    std::cout<<"Library selection slot, persistence, collision, invalid file and pagination tests passed.\n";
}
