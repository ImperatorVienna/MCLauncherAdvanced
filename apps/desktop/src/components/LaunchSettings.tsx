import { useEffect, useRef, useState } from "react";
import { useAvailableMemory } from "../state/useAvailableMemory";
import { t } from "../i18n";
import { Field } from "./Field";
import { Checkbox } from "./Checkbox";
import { SelectMenu } from "./SelectMenu";

import { defaultLaunchSettings, windowSizes, validLaunchSettings, type LaunchSettings as Settings } from "../state/launchSettings";

export function LaunchSettings({ busy, saving = false, value = defaultLaunchSettings, onSave }: { busy: boolean; saving?: boolean; value?: Settings; onSave: (settings: Settings) => Promise<boolean> }) {
  const [memory, setMemory] = useState(value.maxMemoryMib === null ? "" : String(Math.round(value.maxMemoryMib / 1024 * 10) / 10));

  const availableGB = useAvailableMemory();
  const exceedsMemory = availableGB !== undefined && memory !== "" && Number(memory) > availableGB;

  const [mode, setMode] = useState<Settings["displayMode"]>(value.displayMode);
  const [resolution, setResolution] = useState(`${value.width} × ${value.height}`);
  const [rememberMaximized, setRememberMaximized] = useState(value.rememberMaximized);
  const [width, height] = resolution.split(" × ").map(Number);
  const settings: Settings = { maxMemoryMib: memory === "" ? null : Math.ceil(Number(memory) * 1024), displayMode: mode, width, height, rememberMaximized };
  const invalid = !validLaunchSettings(settings) || exceedsMemory || (memory !== "" && availableGB === undefined);
  const serialized = JSON.stringify(settings);
  const lastAttempt = useRef(serialized);
  const pending = useRef(false);
  const [completed, setCompleted] = useState(0);
  useEffect(() => {
    if (busy || saving || invalid || pending.current || lastAttempt.current === serialized) return;
    lastAttempt.current = serialized;
    pending.current = true;
    void onSave(JSON.parse(serialized) as Settings).finally(() => {
      pending.current = false;
      setCompleted(n => n + 1);
    });
  }, [serialized, busy, saving, invalid, onSave, completed]);
  return <section className="launch-settings" aria-label={t("Game launch settings")}>
    <Field immediate id="launch-memory" scope="main" label={t("Maximum memory")} unit="GB" value={memory} onChange={setMemory} invalid={exceedsMemory} describedBy={exceedsMemory ? "memory-limit-warning" : undefined} disabled={busy || availableGB === undefined || availableGB < 0.1} type="number" min={0.1} max={availableGB} step={0.1} placeholder={t("Leave blank for automatic")}/>
    {exceedsMemory && <p id="memory-limit-warning" className="memory-warning" role="alert">{t("Maximum memory exceeds the available limit of {amount} GB. Enter a smaller value.", { amount: availableGB })}</p>}
    <span>{t("Display mode")}</span>
    <SelectMenu id="launch-display" scope="main" compact showIcons={false} disabled={busy}
      options={["inherit", "windowed", "fullscreen"]} value={mode} label={t("Display mode")}
      labelForValue={v => v === "inherit" ? t("Use game settings") : v === "windowed" ? t("Windowed") : t("Fullscreen")} onChange={v => setMode(v as Settings["displayMode"])}/>
    {mode === "windowed" && <>
      <span>{t("Window size")}</span>
      <div className="launch-resolution">
        <SelectMenu id="launch-resolution" scope="main" compact showIcons={false} disabled={busy}
          options={windowSizes}
          value={resolution} label={t("Window size")} onChange={setResolution}/>
        <Checkbox id="launch-remember-maximized" scope="main" checked={rememberMaximized} disabled={busy} onClick={() => setRememberMaximized(v => !v)}>
          {t("Remember last maximized state")}
        </Checkbox>
      </div>
    </>}
  </section>;
}
