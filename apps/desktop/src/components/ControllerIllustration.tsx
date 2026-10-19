import { ControllerBrandSymbol } from "./ControllerBrandSymbol";
import { FunctionSymbol, PlayStationSymbol } from "./ControllerSymbol";
import { useId } from "react";
import { t } from "../i18n";
import { controllerGeometry as geometry, type ControllerAppearance } from "./controllerAppearance";

const controlBorder = "#888888";
// Shared with the three-line menu key: all monochrome function controls.
const functionBackground = "#1b2228";
const functionForeground = "#c6d0d6";

/** Original scalable illustration; purely presentational, without input mapping. */
export function ControllerIllustration({style}:{style:ControllerAppearance}) {
    const id=useId().replace(/:/g, ""), shell=`${id}-shell`, rubber=`${id}-rubber`, cap=`${id}-cap`;
    const ps=style.symmetric, xbox=style.family==="Xbox", nintendo=style.family==="Nintendo";
    const palette=ps ? ["#f5fbff","#e2f0fc","#a9cdee"] : xbox ? ["#587965","#355642","#213c2c"] : nintendo ? ["#a76859","#744338","#462b29"] : ["#647f9a","#415970","#293b51"];
    const leftStick=ps ? geometry.lowerLeft : geometry.upperLeft;
    function stick(x:number,y:number) {
        return <g transform={`translate(${x} ${y})`}>
            <circle r={geometry.radius} fill="#131619" stroke={controlBorder} strokeWidth="1.5"/>
            <circle cy="2" r="22" fill="#080a0c"/>
            <circle r="20" fill={`url(#${rubber})`} stroke={controlBorder} strokeWidth="1"/>
            <circle r="15.5" fill="none" stroke={controlBorder} strokeWidth="1" strokeDasharray="1 2.5"/>
            <circle r="12" fill="#252b30" stroke={controlBorder}/>
            <path d="M-11 -7 Q0 -15 11 -7" fill="none" stroke="#737b82" strokeOpacity=".45"/>
        </g>;
    }
    const faces=[{label:ps?"△":nintendo?"X":xbox?"Y":"",color:ps?"#8bdbc1":xbox?"#e6c568":"#e8edf0"},
        {label:ps?"□":nintendo?"Y":xbox?"X":"",color:ps?"#db9dcc":xbox?"#83b9ec":"#e8edf0"},
        {label:style.east,color:ps?"#efa6aa":xbox?"#ee9990":"#e8edf0"},
        {label:style.south,color:ps?"#9bbde9":xbox?"#b7df91":"#e8edf0"}];
    return <svg className="controller-model" viewBox="0 0 360 250" role="img" aria-label={t("Controller button layout")}>
        <defs>
            <linearGradient id={shell} x1="0" y1="0" x2="0.2" y2="1"><stop stopColor={palette[0]}/><stop offset=".45" stopColor={palette[1]}/><stop offset="1" stopColor={palette[2]}/></linearGradient>
            <radialGradient id={rubber} cx=".4" cy=".25" r=".8"><stop stopColor="#505960"/><stop offset=".7" stopColor="#2b3238"/><stop offset="1" stopColor="#181e23"/></radialGradient>
            <linearGradient id={cap} x2="0" y2="1"><stop stopColor="#41494f"/><stop offset="1" stopColor="#151a1e"/></linearGradient>
        </defs>
        <ellipse cx="180" cy="227" rx="135" ry="10" fill="#000" opacity=".2"/>
        <path d="M81 40 Q54 40 43 65 Q23 113 18 184 Q15 218 37 223 Q55 228 73 203 L111 156 Q180 176 249 156 L287 203 Q305 228 323 223 Q345 218 342 184 Q337 113 317 65 Q306 40 279 40 Z" fill={`url(#${shell})`} stroke={controlBorder} strokeWidth="1.5"/>
        <path d="M47 112 Q34 163 32 195 Q32 211 42 211 M313 112 Q326 163 328 195 Q328 211 318 211" fill="none" stroke={ps?"#1764af":"#0e1317"} strokeWidth="8" strokeLinecap="round" opacity=".35"/>
        <path d="M74 45 Q180 39 286 45" fill="none" stroke="#d6e0e7" strokeOpacity=".25"/>
        <g fill={`url(#${cap})`} stroke={controlBorder} strokeWidth="1">
            {geometry.shoulders.map(({x})=><rect key={x} x={x} y="32" width="59" height="18" rx="6"/>)}
        </g>
        <g className="controller-shoulder-label">{geometry.shoulders.map(({labelX},i)=><text key={labelX} x={labelX} y="45">{i===0?style.left:style.right}</text>)}</g>
        {ps && <rect x="131" y="55" width="98" height="40" rx="8" fill="#215b94" stroke={controlBorder}/>}
        {!ps && !nintendo && <g transform="translate(180 65)">
            <circle r="10" fill={functionBackground} stroke={controlBorder}/>
            {xbox && <g style={{color:functionForeground}}><ControllerBrandSymbol brand="Xbox"/></g>}
        </g>}
        <g transform={`translate(${(ps?geometry.upperLeft:geometry.lowerLeft).join(" ")})`}>
            <path d="M-8 -23 H8 V-8 H23 V8 H8 V23 H-8 V8 H-23 V-8 H-8 Z" fill={`url(#${cap})`} stroke={controlBorder} strokeWidth="1" strokeLinejoin="round"/>
            <path d="M-3 -16 L0 -19 L3 -16 M16 -3 L19 0 L16 3 M3 16 L0 19 L-3 16 M-16 3 L-19 0 L-16 -3" fill="none" stroke={functionForeground} strokeWidth="1"/>
        </g>
        {stick(leftStick[0],leftStick[1])}{stick(...geometry.lowerRight)}
        {faces.map(({label,color},i)=><g key={i} transform={`translate(${geometry.faceButtons[i].join(" ")})`}>
            <circle cy="1.5" r="12" fill="#090d10"/>
            <circle r="11" fill={`url(#${cap})`} stroke={controlBorder} strokeWidth="1"/>
            {ps ? <g style={{color}}><PlayStationSymbol position={i}/></g> : <text x="0" y="0" textAnchor="middle" dominantBaseline="central" style={{fill:color,fontSize:13,fontWeight:600}}>{label}</text>}
        </g>)}
        {nintendo ? <g fill={functionBackground} stroke={controlBorder} strokeWidth="1">
            <circle cx="151" cy="72" r="8"/><circle cx="209" cy="72" r="8"/>
            <g stroke="none" style={{color:functionForeground}}><g transform="translate(151 72)"><FunctionSymbol kind="minus"/></g><g transform="translate(209 72)"><FunctionSymbol kind="plus"/></g></g>
            <rect x="157" y="91" width="18" height="18" rx="3"/><g transform="translate(166 100)" style={{color:functionForeground}}><FunctionSymbol kind="capture"/></g>
            <circle cx="194" cy="100" r="9"/><g transform="translate(194 100)" style={{color:functionForeground}}><FunctionSymbol kind="home"/></g>
        </g> : <g>
            {([-1,1] as const).map(side=><g key={side} transform={`translate(${180+side*(ps?62:27)} ${ps?65:94})`}>
                <rect x="-8" y="-7" width="16" height="14" rx="4" fill={functionBackground} stroke={controlBorder}/>
                <g transform={side===1 ? "scale(.5)" : undefined} style={{color:functionForeground}}>
                    <FunctionSymbol kind={side===1 ? "menu" : ps ? "create" : "view"}/>
                </g>
            </g>)}
            {xbox && <g transform="translate(180 111)">
                <rect x="-11" y="-7" width="22" height="14" rx="7" fill={functionBackground} stroke={controlBorder}/>
                <g style={{color:functionForeground}}><FunctionSymbol kind="share"/></g>
            </g>}
        </g>}
        {ps && <g transform="translate(180 120)" style={{color:functionForeground}}>
            <rect x="-14" y="-11" width="28" height="22" rx="5" fill={functionBackground} stroke={controlBorder}/>
            <ControllerBrandSymbol brand="PlayStation"/>
        </g>}
        <circle cx="180" cy={ps?143:132} r="3" fill="#afce9b" opacity=".8"/>
        <path d="M165 154 H195" stroke="#121a20" strokeWidth="2" strokeLinecap="round"/>
    </svg>;
}
