import { Checkbox } from "./Checkbox";
import { useEffect, useRef, useState } from "react";
import type { SkinViewer } from "skinview3d";
import type { DisplaySkin } from "../state/skins";
import { Control } from "./Control";
import { SkinPortrait } from "./SkinPortrait";
import { t } from "../i18n";
function RotationArrow({ left = false }: { left?: boolean }) {
    return <svg viewBox="0 0 9 13" aria-hidden="true" focusable="false" shapeRendering="crispEdges">
        <g transform={left ? "translate(9 0) scale(-1 1)" : undefined}>
            <path className="skin-arrow-outline" fill="#080808" d="M0 0H2V1H3V2H4V3H5V4H6V5H7V6H8V7H7V8H6V9H5V10H4V11H3V12H2V13H0Z"/>
            <path fill="#888" d="M1 1H2V2H3V3H4V4H5V5H6V6H7V7H6V8H5V9H4V10H3V11H2V12H1Z"/>
            <path fill="#fff" d="M1 1H2V2H3V3H4V4H5V5H6V6H5V5H4V4H3V3H2V2H1Z"/>
            <path fill="#444" d="M1 11H2V10H3V9H4V8H5V7H6V6H7V7H6V8H5V9H4V10H3V11H2V12H1Z"/>
        </g>
    </svg>;
}
export function SkinPreview({ skin }: { skin: DisplaySkin }) {
    const canvas = useRef<HTMLCanvasElement>(null), box = useRef<HTMLDivElement>(null), viewer = useRef<SkinViewer | null>(null);
    const [failed, setFailed] = useState(false), [outer, setOuter] = useState(true);
    const outerRef = useRef(outer); outerRef.current = outer;
    useEffect(() => {
        let live = true; let resize: ResizeObserver | undefined; let renderer: SkinViewer | undefined;
        setFailed(false);
        void import("skinview3d").then(async ({SkinViewer}) => {
            if (!live || !canvas.current || !box.current) return;
            renderer = new SkinViewer({canvas:canvas.current,width:box.current.clientWidth,height:320,pixelRatio:Math.min(devicePixelRatio,2)});
            viewer.current = renderer; renderer.controls.enablePan = false; renderer.controls.enableZoom = false;
            await renderer.loadSkin(skin.url,{model:skin.model});
            if (!live) return;
            renderer.playerObject.skin.setOuterLayerVisible(outerRef.current);
            renderer.playerObject.rotation.y = 0.35;
            resize = new ResizeObserver(() => { if (renderer && box.current) renderer.width = box.current.clientWidth; }); resize.observe(box.current);
        }).catch(() => { if (live) setFailed(true); });
        return () => { live = false; resize?.disconnect(); renderer?.dispose(); viewer.current = null; };
    }, [skin.url,skin.model]);
    useEffect(() => { viewer.current?.playerObject.skin.setOuterLayerVisible(outer); }, [outer]);
    const rotate = (amount:number) => { if (viewer.current) viewer.current.playerObject.rotation.y += amount; };
    return <div className="skin-preview">
        <div ref={box} className="skin-preview-canvas">{failed ? <><SkinPortrait skin={skin}/><p>{t("3D preview is unavailable; showing a flat preview.")}</p></> : <canvas ref={canvas} aria-label={t("Rotate the skin by dragging")}/>}</div>
        <div className="skin-preview-tools">
            <Control className="skin-rotate-arrow" id="skin-rotate-left" aria-label={t("Rotate left")} disabled={failed} onClick={() => rotate(-Math.PI/4)}><RotationArrow left/></Control>
            <Control className="skin-rotate-arrow" id="skin-rotate-right" aria-label={t("Rotate right")} disabled={failed} onClick={() => rotate(Math.PI/4)}><RotationArrow/></Control>
            <Checkbox id="skin-outer" checked={outer} disabled={failed} onClick={() => setOuter(!outer)}>{t("Outer layer")}</Checkbox>
        </div>
    </div>;
}
