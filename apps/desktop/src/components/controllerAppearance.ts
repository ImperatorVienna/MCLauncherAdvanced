// Presentation only: native input continues to use physical button positions.
export function controllerStyle(name:string) {
    if (/dualsense|dualshock|playstation|\bps[345]\b/i.test(name)) return {family:"PlayStation",south:"×",east:"○",left:"L1",right:"R1",symmetric:true};
    if (/nintendo|switch.*pro|joy-con/i.test(name)) return {family:"Nintendo",south:"B",east:"A",left:"L",right:"R",symmetric:false};
    if (/xbox|xinput|x-box/i.test(name)) return {family:"Xbox",south:"A",east:"B",left:"LB",right:"RB",symmetric:false};
    return {family:"",south:"",east:"",left:"L",right:"R",symmetric:false};
}
export type ControllerAppearance = ReturnType<typeof controllerStyle>;
export const controllerGeometry = {
    radius: 26,
    upperLeft: [82, 94], lowerLeft: [134, 130], lowerRight: [226, 130],
    faceButtons: [[278, 71], [255, 94], [301, 94], [278, 117]],
    shoulders: [{x:60, labelX:89.5}, {x:241, labelX:270.5}],
} as const;
