import { expect, test } from "vitest";
import { channelOf, familyOf } from "./versionCatalog";
const v = (id: string, type: "snapshot" | "release" = "snapshot") => ({id,type,releaseTime:""});
test("separate pre-releases and RCs from snapshots and group version families",()=>{
  expect(channelOf(v("1.21.1","release"))).toBe("release");
  for (const id of ["26.3-pre-1", "26.3-rc-1", "1.21-pre1","1.21-rc1","1.14 Pre-Release 1"]) expect(channelOf(v(id))).toBe("preview");
  expect(channelOf(v("26.3-snapshot-1"))).toBe("snapshot");
  expect(familyOf(v("26.3-snapshot-1"))).toBe("26.x");
  expect(familyOf(v("1.21.1"))).toBe("1.21.x");
  expect(familyOf(v("23w31a"))).toBe("snapshots-2023");
});
