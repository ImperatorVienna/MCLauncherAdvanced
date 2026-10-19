import type { ReactNode } from "react";

/** One viewport/centering contract for artwork and HTML legends.
 * Each symbol supplies its intentional bounds (including optical alignment).
 * Artwork is centered on its parent origin; legends align through shared CSS.
 */
export function ControllerGlyph({children, viewBox, width, height=width, label}: {
    children: ReactNode; viewBox: string; width: number; height?: number; label?: string;
}) {
    return <svg x={label ? undefined : -width/2} y={label ? undefined : -height/2}
        width={width} height={height} viewBox={viewBox} preserveAspectRatio="xMidYMid meet"
        className={label ? "controller-legend-symbol" : undefined}
        role={label ? "img" : undefined} aria-label={label} aria-hidden={label ? undefined : true}>
        {children}
    </svg>;
}
