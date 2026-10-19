import { LegalDialog } from "./components/LegalDialog";
import { version } from "../src-tauri/tauri.conf.json";
import { AboutPage } from "./pages/AboutPage";
import { OfficialNews, GamePatchNotes } from "./components/OfficialContent";
import { LauncherUpdateProvider, useLauncherUpdate } from "./state/launcherUpdate";
import { ContentBrowser } from "./pages/ContentBrowser";
import { type ContentKind } from "./state/contentSource";
import { SettingsPage } from "./pages/SettingsPage";
import { LogDialog } from "./components/LogDialog";
import { GameFailure } from "./components/GameFailure";
import type { LogRequest } from "./state/logs";
import { t, localizeMessage, useLocale } from "./i18n";
import { pageLabel, tabLabel } from "./state/pages";
import React, { useLayoutEffect, useMemo, useRef, useState } from "react";
import { Control, Navigation } from "./components/Control";
import { SelectMenu } from "./components/SelectMenu";
import { SidebarButton } from "./components/SidebarButton";
import { LauncherUpdates } from "./pages/LauncherUpdates";
import { FocusManager } from "./navigation/FocusManager";
import { keyboardAction } from "./input/actions";
import { useSteamSession } from "./input/steamSession";
import { isTextEntry } from "./input/textEntry";
import { installNativeController, type NativeStatus } from "./input/nativeController";
import { InstanceContent, managementPages } from "./pages/InstanceContent";
import { InstanceIcon } from "./components/InstanceIcon";
import { useLibrary, labelOf, type Instance } from "./state/library";
import { StandardButton } from "./components/StandardButton";
import { InstanceEditor } from "./components/InstanceEditor";
import { useJava } from "./state/java";
import { InstancesPage } from "./pages/InstancesPage";
import { useInstaller } from "./state/installer";
import { TransferProgress } from "./components/TransferProgress";
import { AccountsPage } from "./pages/AccountsPage";
import { useGameLaunch } from "./state/useGameLaunch";
import { useAccounts, selectedMicrosoft, accountName } from "./state/accounts";
import { SkinAvatar } from "./components/SkinAvatar";
import { SkinsPage } from "./pages/SkinsPage";
import { useSkins } from "./state/skins";
import heroLogo from "./assets/mclogo-java.png";
import "./styles/app.css";
const tabs = ["play", "instances", ...managementPages, "patch-notes"];
const accountTabs = ["accounts", "skins"];
const settingsTabs = ["settings", "about"];
export function App() { return <LauncherUpdateProvider><AppContent/></LauncherUpdateProvider>; }
function AppContent() {
    const updater=useLauncherUpdate();
    const updateLocked=useRef(false);updateLocked.current=updater.locked;
    useLocale();
    const accounts = useAccounts();
    const skins = useSkins();
    const manager = useMemo(() => new FocusManager(), []);
    const [page, setPage] = useState(import.meta.env.DEV && new URLSearchParams(window.location.search).has("transferDemo") ? "instances" : "play");
    const pageHistory = useRef<Array<{
        page: string;
        focus?: string;
    }>>([]);
    const requestedFocus = useRef<string | undefined>(undefined);
    const pageRef = useRef(page);
    const lastGamePage = useRef("play");
    const lastAccountPage = useRef("accounts");
    const currentTabs = tabs.includes(page) ? tabs : accountTabs.includes(page) ? accountTabs : settingsTabs.includes(page) ? settingsTabs : [page];
    const [modal, setModal] = useState<"create" | "edit" | "delete" | "logs" | "license" | "notices" | null>(null);
    const library = useLibrary();
    const [logRequest,setLogRequest]=useState<LogRequest>({});
    function viewLogs(request:LogRequest) {
        const item=library.data.instances.find(i=>i.id===request.instanceId);
        manager.enterModal();setLogRequest({...request,label:request.label ?? item?.name});setModal("logs");
    }
    const installer = useInstaller(library.preview, library.data.revision);
    const java = useJava(library.preview, installer.busy);
    const game = useGameLaunch(library.preview);
    const preparingGame = game.status.preparing || (import.meta.env.DEV && library.preview && new URLSearchParams(window.location.search).get("launchDemo") === "repair" && installer.busy);
    const working = library.busy || installer.busy || game.busy || updater.locked;
    const steam = useSteamSession(manager, library.preview);
    const selected = library.data.instances.find(i => i.id === library.data.selectedId);
    const instanceOptions = useMemo(() => library.data.instances.map(labelOf), [library.data.instances]);
    const installed = installer.installed.find(i => i.instanceId === selected?.id && i.minecraftVersion === selected?.minecraftVersion);
    const statusForOption = (label: string) => installer.installed.some(i => i.installed && i.instanceId === library.data.instances.find(c => labelOf(c) === label)?.id) ? t("Installed") : t("Not installed");
    const instanceLabel = selected ? labelOf(selected) : t("No instance");
    const selectInstance = (label: string) => {
        const item = library.data.instances.find(i => labelOf(i) === label);
        if (item)
            void library.commit({ ...library.data, selectedId: item.id });
    };
    const iconForOption = (label: string) => library.data.instances.find(i => labelOf(i) === label)?.icon ?? "grass";
    const busyRef = useRef(false);
    busyRef.current = working;
    async function saveInstance(item: Instance) {
        const items = library.data.instances;
        const saved = items.some(i => i.id === item.id)
            ? await library.commit({ ...library.data, instances: items.map(i => i.id === item.id ? item : i), selectedId: item.id })
            : await library.create(item);
        if (saved)
            setModal(null);
    }
    async function deleteInstance() {
        if (selected && await library.remove(selected))
            setModal(null);
    }
    const [nativeStatus, setNativeStatus] = useState<NativeStatus>({ devices: [], error: null });
    const [controllerGuide,setControllerGuide]=useState("");
    const [source, setSource] = useState("keyboard-mouse");
    const controllerInput = source === "controller";
    const main = useRef<HTMLElement>(null);
    const scroller = useRef<HTMLDivElement>(null);
    function open(kind: typeof modal) {
        library.clearError();
        manager.enterModal();
        setModal(kind);
    }
    function close() {
        if (modal === "logs" || modal === "license" || modal === "notices") { setModal(null); return; }
        if (busyRef.current)
            return;
        library.clearError();
        setModal(null);
    }
    const [browseQuery,setBrowseQuery]=useState("");
    function changePage(next: string) {
        if(updateLocked.current)return;
        if (next === pageRef.current)
            return;
        setBrowseQuery("");
        const old = pageRef.current;
        const owner = tabs.includes(old) ? "nav-java" : accountTabs.includes(old) ? "account" : settingsTabs.includes(old) ? "nav-settings" : `nav-${old}`;
        const active = manager.activeId();
        // A clicked destination sidebar button is not the previous page's focus.
        const focus = active === "account" || active?.startsWith("nav-") ? owner
            : active?.startsWith("tab-") ? `tab-${old}` : active;
        pageHistory.current.push({ page: old, focus });
        pageRef.current = next;
        setPage(next);
    }
    function browseMod(query: string) {
        changePage("mods");
        setBrowseQuery(query);
        requestedFocus.current = selected && !working ? "browse-instance" : "tab-mods";
    }
    function previousPage() {
        if(updateLocked.current)return;
        const previous = pageHistory.current.pop();
        if (!previous)
            return;
        requestedFocus.current = previous.focus;
        pageRef.current = previous.page;
        setPage(previous.page);
    }
    useLayoutEffect(() => {
        // Transient UI takes precedence; otherwise Back follows actual navigation history.
        manager.onBack = () => { if (modal)
            close();
        else
            previousPage(); };
        manager.onTab = (delta) => {
            const old = pageRef.current;
            const group = tabs.includes(old) ? tabs : accountTabs.includes(old) ? accountTabs : settingsTabs.includes(old) ? settingsTabs : [];
            if (group.length) {
                const next = group[(group.indexOf(old) + delta + group.length) % group.length];
                if (manager.activeId() === `tab-${old}`)
                    requestedFocus.current = `tab-${next}`;
                changePage(next);
            }
        };
        manager.onMenu = () => changePage("settings");
        manager.scroll = scroller.current;
        if (main.current)
            main.current.inert = !!modal;
        if (modal)
            manager.focus("modal-first");
        else
            manager.leaveModal(pageRef.current === "instances" ? "config-create" : `tab-${pageRef.current}`);
    }, [modal, manager]);
    useLayoutEffect(() => {
        if (tabs.includes(page))
            lastGamePage.current = page;
        if (accountTabs.includes(page))
            lastAccountPage.current = page;
        manager.sidebarEntryId = tabs.includes(page) ? "nav-java" : accountTabs.includes(page) ? "account" : settingsTabs.includes(page) ? "nav-settings" : `nav-${page}`;
        manager.activeTabId = `tab-${page}`;
        // Page changes must not steal focus from a control that still exists.
        if (!modal) {
            if (requestedFocus.current) {
                manager.focus(requestedFocus.current);
                requestedFocus.current = undefined;
            }
            else {
                manager.preserveFocus(tabs.includes(page) || accountTabs.includes(page) || settingsTabs.includes(page) ? `tab-${page}` : `nav-${page}`);
            }
        }
    }, [page, manager]);
    const updateReturnFocus=useRef<string | undefined>(undefined);
    useLayoutEffect(()=>{
        const elements=main.current?.querySelectorAll<HTMLElement>(".sidebar, .workspace > header, .tabs, .page-scroll");
        if(updater.locked)updateReturnFocus.current=manager.activeId();
        elements?.forEach(element=>element.toggleAttribute("inert",updater.locked));
        if(updater.locked){
            manager.focus("launcher-cancel-update");
        }else if(updateReturnFocus.current){
            manager.preserveFocus(updateReturnFocus.current);
            updateReturnFocus.current=undefined;
        }
        const trapUpdateTab=(event:KeyboardEvent)=>{
            if(updater.locked&&event.key==="Tab"){
                event.preventDefault();manager.focus("launcher-cancel-update");
            }
        };
        window.addEventListener("keydown",trapUpdateTab,true);
        return()=>{elements?.forEach(element=>element.removeAttribute("inert"));window.removeEventListener("keydown",trapUpdateTab,true);};
    },[updater.locked,manager]);
    const wasInstalling = useRef(false);
    useLayoutEffect(() => {
        if (installer.busy && !wasInstalling.current && !modal)
            manager.preserveFocus("install-cancel");
        if (!installer.busy && wasInstalling.current && !modal)
            manager.preserveFocus(pageRef.current === "instances" ? "config-install" : `tab-${pageRef.current}`);
        wasInstalling.current = installer.busy;
    }, [installer.busy, modal, manager]);
    useLayoutEffect(() => {
        if (library.preview)
            return;
        return installNativeController((action,device) => {
            setControllerGuide(device.name);
            window.dispatchEvent(new CustomEvent("launcher:input-takeover", { detail: "native" }));
            setSource("controller");
            manager.resumeControllerFocus();
            manager.dispatch(action);
        }, status => {
            setNativeStatus(status);
            if (!status.devices.length || status.error)
                setSource(old => old === "controller" ? "keyboard-mouse" : old);
        });
    }, [manager, library.preview]);
    useLayoutEffect(() => {
        document.documentElement.dataset.inputMode = controllerInput ? "gamepad" : "standard";
        return () => { delete document.documentElement.dataset.inputMode; };
    }, [source]);
    useLayoutEffect(() => {
        const pointer = () => setSource("mouse");
        const keyboard = (event: KeyboardEvent) => {
            if (event.key === "Tab")
                setSource("keyboard");
        };
        window.addEventListener("pointerdown", pointer, true);
        window.addEventListener("keydown", keyboard, true);
        return () => {
            window.removeEventListener("pointerdown", pointer, true);
            window.removeEventListener("keydown", keyboard, true);
        };
    }, []);
    useLayoutEffect(() => {
        const key = (e: KeyboardEvent) => {
            if (e.defaultPrevented || e.isComposing)
                return;
            setSource("keyboard");
            manager.trapTab(e);
            if (e.defaultPrevented)
                return;
            const target = e.target as HTMLElement;
            if (isTextEntry(target) &&
                e.key !== "Escape")
                return;
            // Keep native activation, but suppress repeated Enter/Space keydowns.
            if (e.repeat && (e.key === "Enter" || e.key === " ") && target.closest("button")) {
                e.preventDefault();
                return;
            }
            const action = keyboardAction(e);
            if (action) {
                e.preventDefault();
                if (!e.repeat ||
                    action.startsWith("Navigate") ||
                    action.startsWith("Scroll"))
                    manager.dispatch(action);
            }
        };
        window.addEventListener("keydown", key);
        return () => {
            window.removeEventListener("keydown", key);
        };
    }, [manager]);
    return (<Navigation.Provider value={manager}>
      <main className="shell" ref={main} onPointerDown={() => setSource("mouse")}>
        <aside className="sidebar">
          <SidebarButton id="account" account selected={accountTabs.includes(page)} icon={<SkinAvatar />} onClick={() => changePage(lastAccountPage.current)}>{t("Accounts")}<small>{selectedMicrosoft(accounts.data) ? <span className="account-name">{accountName(accounts.data)}</span> : t("Not signed in")}</small>
          </SidebarButton>
          <SidebarButton id="nav-java" selected={tabs.includes(page)} icon={<InstanceIcon icon={iconForOption(instanceLabel)}/>} onClick={() => changePage(lastGamePage.current)}>
            MINECRAFT:<small>JAVA EDITION</small>
          </SidebarButton>
          {["mods", "resource-packs", "shaders"].map(section => (<SidebarButton key={section} id={`nav-${section}`} selected={page === section} icon={<InstanceIcon icon={section==="mods" ? "anvil" : section==="resource-packs" ? "iron_pickaxe" : "nether_star"}/>} onClick={() => changePage(section)}>
              {pageLabel(section)}
            </SidebarButton>))}
          <div className="sidebar-bottom">
            <SidebarButton id="nav-launcher-updates" selected={page === "launcher-updates"} icon={<span className="side-symbol">▤</span>} onClick={() => changePage("launcher-updates")}>{t("What's new?")}{updater.status.version&&<span className="update-badge" role="img" aria-label={t("Launcher update available: {version}",{version:updater.status.version})}><span aria-hidden="true"/></span>}</SidebarButton>
            <SidebarButton id="nav-settings" selected={settingsTabs.includes(page)} icon={<span className="side-symbol">⚙</span>} onClick={() => changePage("settings")}>{t("Settings")}</SidebarButton>
            <p>MC Launcher Advanced v{version}</p>
          </div>
        </aside>
        <section className="workspace">
          <header>
            <h1>{tabs.includes(page) ? "MINECRAFT: JAVA EDITION" : accountTabs.includes(page) ? pageLabel("accounts") : settingsTabs.includes(page) ? pageLabel("settings") : pageLabel(page)}</h1>
            {library.preview && <span className="prototype">{t("Browser preview")}</span>}
          </header>
          {steam.error && <p role="alert" className="config-error page-notice">{steam.error}</p>}
          {currentTabs.length > 0 && <nav className="tabs" aria-label={accountTabs.includes(page) ? t("Account pages") : tabs.includes(page) ? t("Java Edition pages") : t("{value0} page", { value0: pageLabel(page) })}>
            {currentTabs.map((tab) => (<Control id={`tab-${tab}`} group="tabs" key={tab} className={page === tab ? "active" : ""} aria-current={page === tab ? "page" : undefined} onClick={() => changePage(tab)}>
                {tabLabel(tab)}
              </Control>))}
          </nav>}
          <div className={`page-scroll${page === "play" ? " play-page" : ""}`} ref={scroller}>
            {library.loading && <p className="page-notice" role="status">{t("Loading local instances\u2026")}</p>}
            {library.error && !modal && <p className="config-error page-notice" role="alert">{library.error}{t(". Reload the application and retry.")}</p>}
            {game.status.error && <GameFailure key={game.status.error+game.status.logPath} status={game.status} onViewLogs={viewLogs}/>}
            {page === "play" ? (<div className="play-layout">
                <section className="hero reference-hero" aria-label={t("Minecraft Java Edition home")}>
                  <div className="hero-brand">
                    <img className="hero-logo" src={heroLogo} alt="Minecraft Java Edition"/>
                  </div>
                </section>
                <section className="play-bar" aria-label={t("Launch instance")}>
                  <SelectMenu disabled={working || library.loading || !selected} options={instanceOptions} value={instanceLabel} onChange={selectInstance} iconForOption={iconForOption} statusForOption={statusForOption}/>
                  <Control id="play" className="play" disabled={!selected || library.loading || working} onClick={() => { if (!accounts.data.selected) changePage("accounts"); else if (!selected?.javaPath) changePage("instances"); else if (selected) void game.launch(selected.id); }}>{preparingGame ? t("Preparing game launch") : game.busy ? t("Game running") : t("Play")}<span>{preparingGame ? (installer.busy ? t("Completing game files") : selected?.name ?? "") : !accounts.data.selected ? t("Select an account first") : !selected?.javaPath ? t("Choose Java to continue") : installed?.installed ? accountName(accounts.data) : t("Missing game files will be repaired before launch")}</span>
                  </Control>
                  <div className="play-status">
                    <span className="status-dot"/>
                    {installed?.installed ? t("Game installed") : t("Game not installed")}<small>{installer.busy ? t("Installation in progress") : (accountName(accounts.data) ?? t("No account connected"))}</small>
                  </div>
                </section>
                <OfficialNews/>
              </div>) : page === "patch-notes" ? <GamePatchNotes key={selected?.minecraftVersion ?? "none"} version={selected?.minecraftVersion}/> : page === "skins" ? (!selectedMicrosoft(accounts.data) ? <section className="skins-page"><p className="skin-notice">{t("Sign in to use skins.")}</p></section> : <SkinsPage library={skins}/>) : page === "accounts" ? <AccountsPage accounts={accounts}/> : managementPages.includes(page) ? (<InstanceContent title={page} instanceId={selected?.id} selection={{options:instanceOptions,value:instanceLabel,onChange:selectInstance,iconForOption,statusForOption}} busy={working} preview={library.preview} hasLoader={!!selected?.loader} refreshKey={JSON.stringify([installer.task?.running,installer.task?.progress.phase,installer.task?.error])} onBrowse={() => changePage(page.replace("instance-",""))} onStarted={()=>void installer.refreshTask()}/>) : ["mods","resource-packs","shaders"].includes(page) ? <ContentBrowser initialQuery={browseQuery} kind={page as ContentKind} item={selected} busy={working} preview={library.preview} onStarted={()=>void installer.refreshTask()} selection={{options:instanceOptions,value:instanceLabel,onChange:selectInstance,iconForOption,statusForOption}}/> : page === "launcher-updates" ? (<LauncherUpdates onSettings={()=>{changePage("about");requestedFocus.current="launcher-update-action";}}/>) : page === "instances" ? <InstancesPage onBrowseModMenu={()=>browseMod("Mod Menu")} library={library} installer={installer} java={java} gameBusy={game.busy} onOpen={open} onViewLogs={item=>viewLogs({instanceId:item.id,label:item.name})} selection={{options:instanceOptions,value:instanceLabel,onChange:selectInstance,iconForOption,statusForOption}}/> : page === "about" ? <AboutPage onDocument={open}/> : page === "settings" ? <SettingsPage controllerSelection={controllerGuide} onControllerSelection={setControllerGuide} onBrowseControllerMod={()=>browseMod("Controlify")} steam={steam} preview={library.preview} controllerDevices={nativeStatus.devices.map(d=>d.name)} controllerError={nativeStatus.error ?? undefined} onViewLogs={()=>viewLogs({})}/> : <section className="placeholder" aria-label={pageLabel(page)}><p>{t("{value0} is not available yet.",{value0:pageLabel(page)})}</p></section>}
          </div>
          {updater.locked && <section className="transfer-tray" aria-label={t("Download launcher update")}>
            <TransferProgress phase={updater.status.phase==="restarting" ? t("Restarting the launcher…") : t("The launcher will restart automatically when the update is ready.")} completed={updater.status.completed} total={updater.status.total}/>
            <StandardButton id="launcher-cancel-update" disabled={updater.pending==="cancel_launcher_update" || updater.status.phase==="restarting"} onClick={()=>void updater.action("cancel_launcher_update")}>{t("Cancel task")}</StandardButton>
          </section>}
          {installer.busy && <section className="transfer-tray" aria-label={t(installer.task?.content ? "Content installation task" : "Game installation task")}>
            <TransferProgress phase={installer.task?.cancelling ? t("Cancelling; waiting for the current request\u2026") : `${installer.task?.version ?? ""} · ${localizeMessage(installer.task?.progress.phase || "Preparing installation")}`} completed={installer.task?.progress.completed ?? 0} total={installer.task?.progress.total ?? 0}/>
            <StandardButton id="install-cancel" disabled={!!installer.task?.cancelling} onClick={() => void installer.cancel()}>{t("Cancel task")}</StandardButton>
          </section>}
          {!installer.busy && installer.task?.content && installer.task.error && <section className="transfer-tray config-error" role="alert">{localizeMessage(installer.task.error)}</section>}

        </section>
      </main>
      {modal ? (<div className="modal-backdrop" onPointerDown={event => {
                if (event.target === event.currentTarget)
                    close();
            }}>
          <section ref={element => { manager.modalContainer = element; }} role="dialog" aria-modal="true" aria-labelledby="dialog-title" className={modal === "logs" || modal === "license" || modal === "notices" ? "dialog log-dialog" : "dialog"}>
            {modal === "license" || modal === "notices" ? <LegalDialog kind={modal} onClose={close}/> : modal === "logs" ? <LogDialog request={logRequest} onClose={close}/> : modal === "create" || modal === "edit" ? <InstanceEditor java={java} onJavaInstall={major => void installer.startJava(major)} preview={library.preview} item={modal === "edit" ? selected : undefined} busy={working} onCancelTask={() => void installer.cancel()} taskRunning={installer.busy} error={library.error || installer.error || localizeMessage(installer.task?.error)} onSave={saveInstance} onCancel={close}/> : modal === "delete" ? <>
              <h2 id="dialog-title">{t("Delete instance")}</h2>
              <p>{t("Delete instance {name}? Its directory and all saves, mods, resource packs, shaders, and settings will be permanently deleted.", {name: selected?.name ?? ""})}</p>
              <p className="java-path">{t("Directory:")}{installed?.gameDirectory ?? `.minecraft/versions/${selected?.directoryName}`}</p>
              {library.error && <p className="config-error" role="alert">{library.error}</p>}
              <div className="config-actions"><StandardButton id="modal-first" scope="modal" disabled={library.busy} onClick={close}>{t("Cancel")}</StandardButton><StandardButton id="confirm-delete" scope="modal" tone="red" disabled={library.busy} onClick={deleteInstance}>{library.busy ? t("Deleting\u2026") : t("Delete instance and all files")}</StandardButton></div>
            </> : null}
          </section>
        </div>) : null}
    </Navigation.Provider>);
}
