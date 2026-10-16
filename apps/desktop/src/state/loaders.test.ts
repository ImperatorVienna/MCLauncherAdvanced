import { expect, test } from "vitest";
import { parseInstallerCatalog, loaderKinds, loaderName } from "./loaders";
const xml = (...versions: string[]) => `<metadata><versions>${versions.map(v=>`<version>${v}</version>`).join("")}</versions></metadata>`;
test("Forge catalogs match exact game versions and order numeric versions", () => {
    expect(parseInstallerCatalog("forge","1.21.1",xml("1.21-51.0.1","1.21.1-52.0.2","1.21.1-52.0.10"))).toEqual([
        {version:"52.0.10",stable:true},{version:"52.0.2",stable:true},
    ]);
    expect(parseInstallerCatalog("forge","1.12.2",xml("1.12.2-14.23.5.2860"))).toEqual([]);
    expect(parseInstallerCatalog("forge","26.3",xml("26.3-66.0.11","26.2-65.0.0"))[0].version).toBe("66.0.11");
});
test("NeoForge catalogs distinguish old, new, patch and snapshot versions", () => {
    const catalog=xml("21.1.172","21.1.173-beta","26.3.0.77","26.1.2.117","26.1.0.0-alpha.1+snapshot-1");
    expect(parseInstallerCatalog("neoforge","1.21.1",catalog)).toEqual([{version:"21.1.173-beta",stable:false},{version:"21.1.172",stable:true}]);
    expect(parseInstallerCatalog("neoforge","26.3",catalog)).toEqual([{version:"26.3.0.77",stable:true}]);
    expect(parseInstallerCatalog("neoforge","26.1.2",catalog)).toEqual([{version:"26.1.2.117",stable:true}]);
    expect(parseInstallerCatalog("neoforge","26.1",catalog)).toEqual([]);
    expect(parseInstallerCatalog("neoforge","1.21",xml("21.0.167"))).toEqual([{version:"21.0.167",stable:true}]);
    expect(()=>parseInstallerCatalog("forge","1.21.1","invalid")).toThrow();
    expect(loaderKinds.map(loaderName)).toEqual(["Fabric","Quilt","Forge","NeoForge"]);
});
