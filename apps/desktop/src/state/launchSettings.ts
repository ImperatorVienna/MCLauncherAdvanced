export type LaunchSettings = {
  maxMemoryMib: number | null;
  displayMode: "inherit" | "windowed" | "fullscreen";
  width: number;
  height: number;
  rememberMaximized: boolean;
};
export const defaultLaunchSettings: LaunchSettings = {
  maxMemoryMib: null, displayMode: "inherit", width: 1280, height: 720, rememberMaximized: false,
};
export function validLaunchSettings(s: LaunchSettings): boolean {
  return !!s && (s.maxMemoryMib === null || (Number.isInteger(s.maxMemoryMib) && s.maxMemoryMib >= 103 && s.maxMemoryMib <= 4294967295))
    && ["inherit", "windowed", "fullscreen"].includes(s.displayMode)
    && Number.isInteger(s.width) && s.width >= 320 && s.width <= 3840
    && Number.isInteger(s.height) && s.height >= 200 && s.height <= 2400
    && typeof s.rememberMaximized === "boolean";
}
export const windowSizes = ["640 × 480", "800 × 600", "854 × 480", "960 × 540", "1024 × 576", "1024 × 768", "1152 × 864", "1280 × 720", "1280 × 800", "1280 × 960", "1280 × 1024", "1366 × 768", "1440 × 900", "1600 × 900", "1600 × 1200", "1680 × 1050", "1920 × 1080", "1920 × 1200", "1920 × 1440", "2048 × 1152", "2560 × 1080", "2560 × 1440", "2560 × 1600", "2880 × 1800", "3200 × 1800", "3440 × 1440", "3840 × 1600", "3840 × 2160", "3840 × 2400"];
