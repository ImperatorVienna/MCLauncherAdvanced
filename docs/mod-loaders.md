# Mod loader installation

New instances optionally select no loader, Fabric, Quilt, Forge, or NeoForge.
Fabric/Quilt versions are queried for the selected game version from Fabric Meta
v2 or Quilt Meta v3.
The game mapping catalog gates unsupported games. Sort versions newest first
(including historical four-part Fabric versions), prefer the newest stable
version, and fall back to the newest available version if none is stable.
Changing the game or loader clears the old selection. Late results from previous
queries cannot replace current options. Errors expose retry and block install.
Default names include game, loader and loader version; manual names are preserved.

Instance.loader is optional {kind,version}. A normal metadata save cannot change
it; updates need a dedicated transaction. Installation fetches an official profile,
checks inheritance, resolves Maven libraries, verifies official SHA-1 checksums
and atomically caches dependencies. The Fabric/Quilt adapter accepts only their official repositories.
Missing file sizes are determined using a bounded, hash-checked download.
Loader arguments and dependency overrides are merged into verified vanilla
metadata without changing the base client, assets or native downloads. The final
standalone manifest and receipt identify the loader. Repair reinstalls that exact
version; launch rejects a mismatch. No Fabric API/QSL or user mods are installed.

Desktop uses Rust IPC. Browser preview queries the same public Meta APIs using
CORS, but still does not download or launch a game.

Validation: isolated official download tests cover Fabric 0.16.14 and Quilt 0.29.2
with Minecraft 1.21.1; metadata merge, launcher arguments, file checksums and
stale UI requests have automated coverage. Actual graphical game launch and
mod compatibility remain manual testing tasks.

## Existing instance management

Loader management sits between the isolated-directory description and Java controls.
Vanilla instances can add any of the four loaders; an existing loader family is fixed.
There is no cross-family conversion or instance-copy feature. Compatible versions
are fetched for the fixed game version; the newest stable is initially selected.
Choosing a version does not apply it until the player presses the action button.
Older versions remain selectable for an explicit downgrade. Installation/repair
must finish before native loader changes are enabled.

Updates share the install task's progress, cancellation and global busy lock.
Fresh verified vanilla metadata avoids carrying over previous loader arguments.
Loader libraries are staged and hash-verified in the shared cache before touching
instance metadata. A journal snapshots the manifest, receipt and instance library;
failed commits restore all three, and startup recovers interrupted commits before
opening storage. An unrecoverable write error blocks storage until recovery.
Names, directories, mods, worlds and settings are untouched. A downgrade changes
loader metadata only; it does not undo gameplay writes or guarantee mod compatibility.
Browser preview models the selection locally and does not install game files.


## Forge and NeoForge

The shared Forge-family adapter uses each official Maven metadata catalog.
Forge versions are matched by the exact game prefix. NeoForge release matching
supports the 20.x/21.x scheme (including x.0 for an unpatched game release) and
the four-component 26.x scheme. Snapshot suffixes are not treated as releases.
This adapter currently exposes modern Forge for Minecraft 1.13+; legacy Forge
1.12.2 and earlier and NeoForge's original 1.20.1 artifact are not supported.

Installation requires the Java executable explicitly selected for the instance.
There is no automatic runtime inventory fallback, download, or selection. A
missing/too-old selection reports the required major and directs the player to
the existing Java controls. That download picker still defaults to the game's
recommended Java; downloading never changes the selected runtime.

The installer JAR is downloaded over HTTPS from the official Maven repository
and checked against its SHA-1 sidecar before execution. A temporary game root
receives vanilla client metadata, the verified client JAR and an empty launcher
profile. The official headless --installClient command performs its own library
downloads and patch processors; the real launcher profile and worlds are never
passed to it. Both families share this runner, timeout, process-group cancellation,
log handling and runtime-artifact import. Logs live under versions/<instance directory>/logs, with a distinct timestamp
for each installation attempt. Temporary files are removed on normal success/failure/cancellation.

The embedded launch profile must match the base game and a supported entry point.
Runtime artifacts are verified before atomic cache import. Generated artifacts
with no remote URL are recorded with their resulting hash and installer provenance;
repair reruns the official installer before the common game download/verification
phase. Runtime Maven identity includes classifiers, preserving client and universal
artifacts. Flattening, arguments, module-path placeholders, receipts and the existing
three-file journal update are shared with the other loaders.

Validation covers real official installers for Forge 52.1.0 / NeoForge 21.1.172
on Minecraft 1.21.1 and Forge 66.0.11 / NeoForge 26.3.0.77 on Minecraft 26.3.
The network test explicitly selects test Java executables. Unit tests cover exact
catalog filtering, generated-file validation, cancellation, explicit Java selection,
launch argument/module-path expansion, family locking and manual downgrades.
Graphical game launch and user mod compatibility still require desktop testing.

Official implementation references:
- https://github.com/MinecraftForge/Installer
- https://github.com/neoforged/LegacyInstaller
- https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml
- https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml
