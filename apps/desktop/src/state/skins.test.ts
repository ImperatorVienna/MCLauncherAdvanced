import { describe, expect, it } from "vitest";
import { emptySkins, validateSkins, removeSkin, setSkinModel, displaySkins } from "./skins";
describe("local skins", () => {
    const skin = {id:"12345678-1234-1234-1234-123456789abc",name:"Custom",model:"default" as const,png:"YWJj"};
    it("validates builtins and rejects dangling assignments and duplicate IDs", () => {
        expect(() => validateSkins({...emptySkins(),assignments:{Steve:"builtin-alex"}})).not.toThrow();
        expect(() => validateSkins({...emptySkins(),assignments:{Steve:"missing"}})).toThrow();
        expect(() => validateSkins({...emptySkins(),skins:[skin,skin]})).toThrow();
    });
    it("deleting a skin clears all its bindings but preserves other accounts", () => {
        const data = {...emptySkins(),skins:[skin],assignments:{Steve:skin.id,Alex:skin.id,Other:"builtin-ari"}};
        const next=removeSkin(data,skin.id);
        expect(next.skins).toEqual([]);
        expect(next.assignments).toEqual({Other:"builtin-ari"});
        expect(data.skins).toHaveLength(1);
    });
    it("switches builtin textures and custom models independently", () => {
        const original = {...emptySkins(), skins:[skin]};
        const changed = setSkinModel(original,"builtin-steve","slim");
        expect(() => validateSkins(changed)).not.toThrow();
        const before = displaySkins(original).find(s => s.id === "builtin-steve")!;
        const after = displaySkins(changed).find(s => s.id === "builtin-steve")!;
        expect(after.model).toBe("slim");
        expect(after.url).toBeTruthy();
        expect(after.url).not.toBe(before.url);
        expect(setSkinModel(changed,skin.id,"slim").skins[0].model).toBe("slim");
        expect(() => validateSkins({...original,builtinModels:{missing:"slim"}})).toThrow();
    });
    it("rejects unbounded and malformed imported records", () => {
        for (const patch of [{name:" "},{name:"a".repeat(65)},{png:"x".repeat(174769)},{model:"unknown"}]) {
            expect(() => validateSkins({...emptySkins(),skins:[{...skin,...patch} as typeof skin]})).toThrow();
        }
    });
});
