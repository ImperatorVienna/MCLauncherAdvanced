# Platform boundary

`launcher-core` owns instance models, validation, revisioned persistence, portable
paths and launch settings. Its storage constructor receives a base directory;
it does not detect the host, register desktop entries, execute scripts or plan
AppImage replacement. It has no platform adapter dependency.

`launcher-platform` owns OS integration and depends only on the core's generic
atomic writer (plus Linux libc). Its public functions are the boundary used by
Tauri commands and the installer. Linux-specific implementations include:

- Outer AppImage discovery, portable startup script and desktop registration.
- Executable replacement after process exit, in-place backup and restart.
- External browser/file opening, available memory and kernel version.
- Steam keyboard invocation, child environment cleanup and process groups.
- No-overwrite file moves, private account permissions and runtime extraction
  permission/link primitives; system Java search roots.

`launcher-install` retains Minecraft metadata rules, loaders, downloads, hash
verification, extraction safety and file transactions. Tauri retains application
lifecycle, UI commands and authentication. These layers call platform functions
instead of embedding Linux commands. Registration remains optional in development.

## Windows work still required

Windows adapters now implement external opening, available memory, OS version,
private credential ACLs, no-overwrite moves and installer process-tree cancellation
with Job Objects. Desktop registration generates a fixed BAT entry and a desktop
shortcut without Start menu registration. Portable storage uses the executable
directory. These adapters compile for x86_64 Windows MSVC; compilation does not
constitute a real Windows runtime test. The r32 test build enables Windows x86_64 installation. Wine smoke tests cover
memory/version queries, path conversion, no-overwrite moves and installer job
start/cancellation; real Windows game/Steam/WebView2 validation remains pending.

Java provider requests, download/checksum verification, explicit selection and
installation transactions remain in `launcher-install`. `java/provider.rs` owns
the shared Adoptium protocol; `java/archives.rs` handles ZIP and tar.gz extraction.
The platform contract supplies OS, archive format, executable name, search roots,
permissions and process mechanisms. No provider HTTP code belongs in the platform
adapter, and no OS integration belongs in the core. Managed Java paths are stored
with portable separators and resolved relative to the launcher runtime directory.

Windows acceptance still requires packaging/WebView2, process cancellation,
credential ACLs, file locking/replacement, reserved filenames and relocating the
portable folder on Windows. Java ZIP extraction and existing Linux tar extraction
have automated regression coverage. Windows Minecraft rule selection and JVM path
handling compile but still require launch validation.

Linux regression coverage includes relocating the launch script while retaining
Steam variables/arguments, desktop generation, memory parsing, portable Java
paths, content collision/rollback and installer cancellation. No game directories
are migrated and stored launcher configuration formats remain compatible.

## Persistent implementation rule

The repository's `AGENTS.md` makes these boundaries mandatory for all future
changes. Frontend rendering, localization, navigation and focus stay in the
frontend. Platform adapters supply host mechanisms and capabilities, never UI
behavior or domain workflow policy. Application orchestration connects these
layers through explicit contracts. Missing contracts must be added at the
correct boundary; implementing a feature is not an exception to layering.

## Internal release naming and scope

Internal revisions use `rN-<os>-<arch>`, for example `r31-win-x86_64` and
`r31-linux-x86_64`. The package identifier remains `mclauncher-advanced`.
Windows and Linux packaging requests currently default to x86_64. macOS is
planned for arm64 only; macOS x86_64 is excluded. Windows/Linux arm64 work is
deferred until after macOS arm64 and a subsequent request for those targets.
A platform suffix identifies a built artifact, not a promise of support before
validation. Windows test builds are explicitly pre-release until real-device validation.

## Launcher self-update

`launcher-install::self_update` owns the fixed GitHub repository, stable SemVer
selection, exact platform asset naming and required SHA-256 metadata. It reuses
the verified download service. No updater code was added to `launcher-core`.
Tauri `updater` owns startup checking, task state, cancellation, shared writer/game
exclusion and exit coordination. The frontend owns translated status, announcement navigation and automatic-update task locking and stable focus identity. `launcher-platform::updater`
locates the packaged executable and starts the host-specific replacement helper;
it has no knowledge of GitHub, release versions or frontend pages.

Linux uses a shell helper; Windows uses an encoded PowerShell helper. Both wait
for the old process to terminate, use the new filename and retain a `.previous` backup, and
restart in the portable directory with inherited Steam variables. Detailed release
requirements and validation limits are in `launcher-updates.md`.

## Official editorial content

`launcher-install::editorial` owns Mojang feed endpoints, bounded HTTP reads and
allowed information-link destinations. Tauri exposes asynchronous read-only IPC
and dispatches approved URLs to the existing platform external-open mechanism.
The frontend owns per-version selection, session caching, stale-response guards,
Java news filtering, inert article rendering, localization and navigation. No
remote HTML is inserted into the WebView; supported links become registered
controls. Only the official content image origin is allowed by CSP. The Vite
proxy is a development-only equivalent of native feed reads. Feed availability
is not required for game installation or launch. These public endpoints are
external dependencies, not a guaranteed API contract.

## Native controller acquisition

`launcher-platform::gamepad` owns Linux/Windows gilrs acquisition, physical
button/axis snapshots, device inventory and bounded transport buffering. Gilrs
selects its Linux backend or the explicitly enabled Windows WGI backend. Tauri
only supplies window-focus suspension and exposes polling IPC. Frontend routing,
neutral rearming, repeat timing, model illustrations and page/focus decisions
remain in the frontend. No controller dependency is introduced into core.

## Launcher icon identity

The next release uses the MCLA isometric mark generated by
`scripts/generate-launcher-icon.py`. Tauri embeds the PNG window icon and Windows
ICO executable resource. Windows registration points the BAT shortcut at a
portable `icon.ico` refreshed from the embedded asset at launch.

Tauri enables its GTK application ID using the unchanged application identifier.
Linux registration preserves the existing launcher desktop entry and adds a
`NoDisplay` alias named after that ID, allowing Wayland to resolve the same icon
without a duplicate menu item. The application passes identity and asset bytes;
platform adapters only write host integration files. No core or navigation code
is involved. GTK application-ID registration also participates in session-bus
application uniqueness; multiple simultaneous launcher copies need desktop testing.
Compositor title-bar/dock rendering still requires real Wayland validation.

Wayland preview verification found that GTK still advertised the executable
program name as `xdg_toplevel.app_id`, despite `enableGTKAppId`. The Tauri
`host` adapter now sets GLib's program name to the configured identifier before
GTK initialization. This Linux-only toolkit dependency stays in application host
integration; core, services and frontend remain unchanged. The desktop alias and
protocol identity now use the same identifier, independent of executable renaming.

## Authentication and privacy boundary audit (1.0.2)

Authentication, entitlement-response validation, refresh and cancellation are one
application service in `microsoft_auth.rs`. `microsoft_auth_commands.rs` owns
Tauri IPC and dispatches the approved verification address to the platform browser
adapter. The service itself has no Tauri or platform imports. `accounts.rs` owns
account persistence and renderer-safe summaries; it calls the platform's private
file-permission mechanism before writing credentials. Linux permissions and
Windows ACL implementation remain separate. No account policy belongs in core or
platform code. Privacy documents describe these actual paths and service requests.

The audit found no reverse core-to-platform dependency or frontend navigation in
platform adapters. Core's process ID for unique atomic-write names is a portable
mechanism, not process launching. OS-specific GTK identity remains in the Tauri
host adapter. Java command arguments and Minecraft OS metadata rules remain domain
logic; process environment and host lifecycle mechanisms use platform contracts.

Entitlement response structure is validated, but an empty item list alone does
not reject subscriptions. A valid profile response remains required. Mock tests
cover rejected refresh, restricted account errors, missing profiles, cancellation
and malformed entitlement structure. Subscription expiry and game-level privileges
are not claimed as fully implemented or tested; live access and an established
service contract are still needed. Do not infer these states from arbitrary item
names or introduce platform-specific authentication flows.

## Desktop clipboard

`clipboard_commands` bounds text input and dispatches to the toolkit main thread.
The isolated application `host` adapter uses GTK clipboard ownership on Linux,
including Wayland, rather than permission-gated WebView clipboard JavaScript.
This toolkit dependency stays out of core and domain services. Other hosts report
no native clipboard capability and retain the existing WebView clipboard path.
The frontend owns translated failure messages; no clipboard text is logged or
sent over the network. Actual compositor transfer remains a desktop acceptance check.

## Official account skin previews

The application authentication service reads the authenticated Minecraft profile.
The skins IPC adapter downloads an allowlisted, bounded texture on a blocking worker
and validates its PNG before sending image data to the frontend. Tokens remain native;
texture requests contain no credentials. Frontend state owns account-switch race
protection, preview selection and rendering. Core and platform gain no dependencies
or account/skin policy.

Official skin updates use the same application boundary: skins IPC validates image
size, PNG dimensions and model; the authentication service sends the multipart
upload using native credentials and validates the returned identity/model. The
frontend owns model preview, explicit apply, progress and feedback. The generic
frontend clipboard helper is shared by log copying and authorization-code copying;
the existing isolated host clipboard adapter remains unchanged.
