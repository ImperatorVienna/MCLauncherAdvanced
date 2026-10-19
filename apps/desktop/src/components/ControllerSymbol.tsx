import { ControllerGlyph } from "./ControllerGlyph";
/** Shared symbols for the controller artwork and its operation legend. */
export function PlayStationSymbol({position}:{position:number}) {
    return <g fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
        {position===0 ? <path d="M0 -8 L6.9282 4 H-6.9282 Z"/> : position===1 ? <rect x="-5.6569" y="-5.6569" width="11.3138" height="11.3138"/> : position===2 ? <circle r="8"/> : <path d="M-5.6569 -5.6569 L5.6569 5.6569 M5.6569 -5.6569 L-5.6569 5.6569"/>}
    </g>;
}
export function MenuSymbol() {
    return <path d="M-6 -4 H6 M-6 0 H6 M-6 4 H6" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round"/>;
}

/** Shared monochrome symbols, centered on the control origin. */
export function FunctionSymbol({kind}:{kind:"menu"|"plus"|"minus"|"home"|"capture"|"view"|"share"|"create"}) {
    if (kind === "menu") return <MenuSymbol/>;
    return <g fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
        {kind === "plus" ? <path d="M-4 0 H4 M0 -4 V4"/> :
         kind === "minus" ? <path d="M-4 0 H4"/> :
         kind === "home" ? <path d="M-5 -.5 L0 -4.5 L5 -.5 V4.5 H-5 Z" fill="currentColor" stroke="none"/> :
         kind === "capture" ? <circle r="3"/> :
         kind === "create" ? <path d="M0 -4 V0 M-4 -2 L-2 1 M4 -2 L2 1"/> :
         kind === "view" ? <path d="M-3 -3 H1 V1 H-3 Z M-1 -1 H3 V3 H-1"/> :
         <path d="M-5 0 V3.5 H5 V0 M0 1 V-4 M-2.5 -1.5 L0 -4 L2.5 -1.5"/>}
    </g>;
}
export function ControllerLegendSymbol({kind,label}:{kind:"cross"|"circle"|"menu"|"plus";label:string}) {
    const face=kind === "cross" || kind === "circle";
    return <ControllerGlyph width={18} viewBox={face ? "-9 -9 18 18" : "-7 -7 14 14"} label={label}>
        {face ? <PlayStationSymbol position={kind === "cross" ? 3 : 2}/> : <FunctionSymbol kind={kind}/>}
    </ControllerGlyph>;
}
