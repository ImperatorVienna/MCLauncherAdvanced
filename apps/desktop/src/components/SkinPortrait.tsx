import { useEffect, useRef } from "react";
import type { DisplaySkin } from "../state/skins";
/** Small 2D previews avoid a WebGL context for every library tile. */
export function SkinPortrait({ skin, head = false }: { skin: DisplaySkin; head?: boolean }) {
    const canvas = useRef<HTMLCanvasElement>(null);
    useEffect(() => {
        let live = true; const img = new Image();
        img.onload = () => {
            if (!live || !canvas.current) return;
            const c = canvas.current; c.width = head ? 8 : 16; c.height = head ? 8 : 32;
            const ctx = c.getContext("2d"); if (!ctx) return; ctx.imageSmoothingEnabled = false;
            const part = (sx:number,sy:number,w:number,h:number,x:number,y:number) => ctx.drawImage(img,sx,sy,w,h,x,y,w,h);
            part(8,8,8,8,head ? 0 : 4,0);
            if (img.height === 64) part(40,8,8,8,head ? 0 : 4,0);
            if (head) return;
            part(20,20,8,12,4,8); part(4,20,4,12,4,20);
            const arm = skin.model === "slim" ? 3 : 4;
            part(44,20,arm,12,4-arm,8);
            part(img.height === 64 ? 36 : 44,img.height === 64 ? 52 : 20,arm,12,12,8);
            part(img.height === 64 ? 20 : 4,img.height === 64 ? 52 : 20,4,12,8,20);
            if (img.height === 64) { part(20,36,8,12,4,8); part(44,36,arm,12,4-arm,8); part(52,52,arm,12,12,8); part(4,36,4,12,4,20); part(4,52,4,12,8,20); }
        };
        img.src = skin.url; return () => { live = false; };
    }, [skin.url,skin.model,head]);
    return <canvas className={head ? "skin-avatar" : "skin-portrait"} ref={canvas} role="img" aria-label={skin.name}/>;
}
