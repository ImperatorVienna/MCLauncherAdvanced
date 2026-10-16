import { expect,test } from "vitest";
import { searchFacets } from "./contentSource";
import type { Instance } from "./library";
const item:Instance={id:"test",name:"Test",minecraftVersion:"1.21.1",icon:"grass",directoryName:"Test",loader:{kind:"neoforge",version:"21.1.1"}};
test("search always uses exact selected version and mod loader, not editable filters",()=>{
    expect(searchFacets(item,"mods")).toEqual([["project_type:mod"],["versions:1.21.1"],["categories:neoforge"]]);
    expect(searchFacets({...item,minecraftVersion:"24w14a"},"mods")[1]).toEqual(["versions:24w14a"]);
    expect(searchFacets(item,"shaders")).toEqual([["project_type:shader"],["versions:1.21.1"]]);
});
