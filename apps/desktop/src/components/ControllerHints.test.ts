import { expect, test } from "vitest";
import { controllerStyle } from "./controllerAppearance";
test("controller labels follow physical south/east positions, with an unknown fallback",()=>{
    expect(controllerStyle("Xbox Wireless Controller")).toMatchObject({south:"A",east:"B"});
    expect(controllerStyle("PS5 Controller")).toMatchObject({south:"×",east:"○",symmetric:true});
    expect(controllerStyle("Nintendo Switch Pro Controller")).toMatchObject({south:"B",east:"A"});
    expect(controllerStyle("Unknown gamepad").family).toBe("");
});
