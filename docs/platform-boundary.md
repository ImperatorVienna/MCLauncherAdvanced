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
exclusion and exit coordination. The frontend owns translated status, explicit
restart interaction and stable focus identity. `launcher-platform::updater`
locates the packaged executable and starts the host-specific replacement helper;
it has no knowledge of GitHub, release versions or frontend pages.

Linux uses a shell helper; Windows uses an encoded PowerShell helper. Both wait
for the old process to terminate, use the new filename and retain a `.previous` backup, and
restart in the portable directory with inherited Steam variables. Detailed release
requirements and validation limits are in `launcher-updates.md`.
