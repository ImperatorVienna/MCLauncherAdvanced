# Portable directory layout

The actual launcher executable's parent directory is the storage root. Linux AppImage release builds use APPIMAGE, not the mounted executable location or the process working directory. Development binaries use their own parent directory. There is no relocation, legacy locator, migration, or copying from the user's home directory.

```
<launcher>/
  MinecraftLauncher.AppImage
  launch.sh
  icon.png
  session.lock
  launcher-library.json
  launcher-settings.json
  accounts.json
  launcher-skins.json
  webview/
  runtimes/java/
  logs/
  .minecraft/
    libraries/
    assets/
    versions/<configuration name>/
```

Each AppImage startup refreshes launch.sh and the user's applications/minecraft.launcher.advanced.desktop entry. The desktop entry is named MC Launcher Advanced and points to the script's absolute path. The script resolves the executable relative to its own directory, uses exec, and preserves arguments and environment (including Steam identifiers). The lock is per launcher directory. Moving a folder requires running the AppImage once and re-adding its entry to Steam. Existing Steam shortcuts are not edited.

Self-update uses the new release asset filename in the same directory and appends `.previous` to the old executable. Downloads are staged in a unique `.launcher-update-*` directory beside it and SHA-256 verified before replacement. Failed updates retain `apply.log`; successful replacement startup removes its complete staging directory; see [launcher updates](launcher-updates.md).

Managed Java selections are serialized relative to the launcher directory and resolved to absolute paths for the UI and process commands. Startup probes selected Java paths; completed negative probes clear selections without choosing another runtime. Probe infrastructure errors preserve selections. All configurations, including Forge/NeoForge, can have no Java selected. Their installation dialog requires Java; their loader-change button is disabled without Java. The existing installer validates the selected runtime before executing processors.

Startup discovers .minecraft/versions/<name>/<name>.json, creates configuration records named after the existing directories, and assigns persistent random icons from the shared catalog. It does not rewrite game metadata during discovery. Existing records are not duplicated or re-randomized. Invalid metadata and unsafe paths are skipped. Recognized loader families are Fabric, Quilt, Forge, and NeoForge. Unverified external installations require the normal repair pipeline; merely finding a directory does not mark it complete.

Launching an incomplete configuration with an account and Java selected runs the existing installation/repair pipeline with shared progress and cancellation, then launches only on success. Shared assets and libraries are verified/reused. The configuration directory is always the game directory. Root-level saves, mods, resource packs, shaders, and game settings are not copied or inherited. There is no migration mode.

## Windows portable entry

Windows release startup writes `launch.bat` and `launcher-executable.txt` beside
the main EXE, and creates `MC Launcher Advanced.lnk` in the Windows Desktop known
folder (including redirected desktops). No Start menu entry is registered.
The shortcut targets the BAT, never the versioned/renameable EXE.

The BAT sets its own directory as data for an embedded PowerShell command. It
first validates the remembered executable's ProductName resource. If renamed or
missing, it scans only sibling EXEs for the exact `MC Launcher Advanced` product
name, rejecting reparse points and previous/temporary/download files. This is
application identification, not a signature/authenticity check. With multiple
candidates it fails explicitly; running the desired EXE directly refreshes the
remembered filename. The EXE inherits the environment (including Steam IDs),
and uses the portable directory as its working directory. The BAT currently
accepts no forwarded command-line arguments. It waits for the launcher process
to exit before ending, preserving the lifetime of the Steam entry process.

All generated paths are passed as data rather than embedded in PowerShell code;
the BAT disables delayed expansion and uses UTF-16 encoded static PowerShell.
PowerShell and WScript.Shell must be available. No execution-policy setting is
changed. Moving the directory requires running the main EXE once to refresh the
desktop shortcut. Renaming/removing the BAT itself invalidates the shortcut.
Self-update uses a PowerShell helper that waits for the old EXE to exit before replacement. The r32 Windows test build enables game installation with native adapters;
real-device validation remains pending. See platform-boundary.md.

Linux registration migrates the two legacy application entries to the single visible identity entry. Desktop symlinks targeting the old entries are retargeted before those entries are removed. Windows registration is unchanged.
