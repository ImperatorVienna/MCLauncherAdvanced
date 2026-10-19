# MC Launcher Advanced

A third-party powered launcher for Minecraft Java Edition with an interface similar to the official.

**[简体中文](README.zh-CN.md)** · Package name: `mclauncher-advanced`

An independent desktop launcher with keyboard, mouse and controller navigation, portable storage, isolated game instances and Modrinth content management. It is not an official Minecraft product and is not affiliated with or endorsed by Mojang or Microsoft.

## Release status

The current version is **1.0.2**, with Linux and Windows x86_64 packages. See the platform validation scope below.

See the [1.0.2 release notes](docs/release-notes/1.0.2.en-US.md) for this update. Game sessions require Microsoft sign-in; publishing the launcher does not imply completed Minecraft API access approval.

Microsoft device-code sign-in and the Xbox/Minecraft authentication flow are implemented. **This application has not obtained Minecraft API access approval.** Microsoft sign-in currently cannot complete the required Minecraft authentication flow. Successful Microsoft sign-in alone does not establish that Minecraft authentication or game launch has succeeded.

## Features

- Install, repair and launch Minecraft: Java Edition with an explicitly selected Java runtime.
- Create isolated instances, discover local version folders and manage per-instance memory, display and Java settings.
- Install Fabric, Quilt, Forge and NeoForge; change versions within an instance's existing loader family.
- Download Java from Eclipse Adoptium/Temurin, verify downloads and rescan installed runtimes. Downloads and runtime selection remain explicit user actions.
- Browse, search, install and update mods, resource packs and shader packs from Modrinth using the selected instance's game version and loader. The instance selector is shared with instance management.
- Scan local content, disable/enable mods, delete files and check for updates. Resource packs and shader packs are activated and configured in game.
- Navigate with mouse, keyboard or a physical controller. Optional Steam adaptation enables launcher fullscreen and requests the Steam keyboard when launched through Steam.
- Settings provides Xbox, PlayStation, Nintendo Switch Pro and generic controller diagrams with operation guides. Select a device from the dropdown; valid controller input silently selects the active device without changing the page or focus.
- English and Simplified Chinese interfaces; English is the initial default.
- Per-instance game/installer logs and launcher logs accessible through the interface.
- Five official Java Edition news items on the home page, plus game release notes matched to the selected instance’s Minecraft version.
- Settings → About shows the version, update checks, maintainer homepage and project homepage, with separate dialogs for the open-source license and third-party notices.

The skin library supports local PNG import and preview. Uploading/applying skins to a Microsoft account is **not connected yet**. The launcher checks GitHub for a stable release at startup. Settings → About also provides manual checking, verified downloading, and automatic restart to install the update. The new executable uses the release asset filename in the same directory; the previous binary is retained with a `.previous` suffix. Successful startup removes the update staging directory. See the [release contract and validation limits](docs/launcher-updates.md).

## Official news and game release notes

The home page loads five Java Edition news items from Mojang’s public launcher feed. Selecting a card opens the corresponding official Minecraft article in the browser.

The game’s “Release notes” tab shows the official article for the selected instance’s Minecraft version. It is separate from the launcher release announcements under “What’s new?” in the sidebar. Articles are matched to the exact game version; another version’s notes are never substituted. Missing articles or failed requests produce a message. Official news and game release notes use English source content regardless of the interface language; buttons and source labels follow the interface language. Unavailable news or articles do not block game launch.

Version 1.0.2 introduces the MCLA icon across the launcher window, desktop integration and browser preview, fixes Wayland application identity matching, and refines bundled fonts, article layout and English interface wording. Asset notices and original license documents are displayed in English.

## Platforms and first launch

| Platform | Package | Validation |
| --- | --- | --- |
| Linux x86_64, glibc | AppImage | Game installation and launch tested during development |
| Windows 10/11 x86_64 | Portable EXE | Earlier versions received limited Wine checks; 1.0.2 has been cross-compiled and packaged, and still requires real Windows acceptance testing |
| macOS ARM64 | Planned | No build available |

Windows requires **Microsoft Edge WebView2 Runtime**. The portable EXE does not bundle or automatically install it. Wine/Proton is not a supported substitute for Windows acceptance testing; use the native AppImage on Linux. Windows 7/8/8.1 and other CPU architectures are not current targets.

1. Put the executable in a writable folder. Avoid a system-protected installation directory.
2. On Linux, make the AppImage executable and run it. The host must support running AppImages.
3. On Windows, install WebView2 Runtime if missing, then run the EXE. First launch creates `launch.bat` beside the EXE and a desktop shortcut; no Start menu entry is created.
4. Choose an instance and Java. Forge/NeoForge installation requires a valid Java selection. Other installation flows allow an explicit “No Java selected”; launching still requires Java.
5. Sign in and launch. Missing managed game files are completed through the installation/repair flow.

For Steam, add the generated launcher entry/script as a non-Steam game. Windows shortcuts use `launch.bat`; the script discovers the launcher even if the EXE is renamed. After moving the directory, run the executable once to refresh the desktop entry and re-add it to Steam.

## Portable files

Storage is based on the launcher's executable directory, not the shell's working directory:

```text
<launcher folder>/
  <launcher executable>
  launch.sh or launch.bat
  launcher-library.json
  launcher-settings.json
  accounts.json
  webview/
  runtimes/java/
  logs/
  .minecraft/
    assets/
    libraries/
    versions/<instance folder>/
      mods/
      resourcepacks/
      shaderpacks/
      saves/
      logs/
```

Folders are created as needed. Saves, mods, packs and game settings are isolated per instance; libraries/assets are shared. The launcher does not move data from another launcher or inherit mods/packs from its shared game directory. Detection of local version folders does not guarantee compatibility with every third-party launcher layout.

Managed Java is stored relative to the portable folder. System Java may need to be reselected on another machine. Rescanning checks the current machine's runtimes.

`accounts.json` contains authentication credentials protected with OS file permissions. It is not an encrypted export. **Do not upload it, include it in bug reports, or share it when copying the launcher to another person.**

## Content updates

- Compatibility filters follow the selected instance; unsupported game versions may require manual content installation.
- File hashes identify content where Modrinth recognizes it. Unrecognized files remain manageable by filename but cannot be automatically matched to an update.
- Updates use the new upstream filename for mods, resource packs and shader packs. Re-enable updated packs and adjust their settings in game if necessary; the launcher does not rewrite game pack-selection files.
- Missing required dependencies are downloaded. Existing recognized dependencies are retained; updating a target does not establish full compatibility of every installed mod.
- File deletion is permanent; it does not create a backup.
- A controller mod such as Controlify is needed for in-game controller support. A compatible shader mod is needed to use shader packs. Launcher navigation does not add either capability to the game itself.

## Development

The project uses Tauri 2/Rust and React/TypeScript/Vite. Install a compatible Rust toolchain, Node.js 22.12+ and the pnpm version pinned in `package.json`, plus the [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/).

```sh
pnpm install
pnpm dev                       # Browser preview, http://127.0.0.1:1420
pnpm test                      # Frontend regression tests
pnpm build                     # Type check and frontend build
pnpm tauri dev                 # Native desktop development
pnpm tauri build --bundles appimage  # Linux AppImage
python3 scripts/check-source-language.py
```

Browser preview has separate local data and does not perform native game installation or launch. F8 controller simulation has been removed; controller navigation uses the native input adapter.

Architecture boundaries are mandatory: frontend owns presentation/focus/navigation; core owns portable models and persistence; domain services own game/content workflows; platform adapters own OS integration. See [platform boundaries](docs/platform-boundary.md), [storage](docs/storage-and-updates.md), [installation](docs/installation.md), and [form spacing](docs/ui-reference/form-spacing.md).

## Privacy and local data

[Privacy and local data](PRIVACY.md)

## Maintainer and contact

MC Launcher Advanced is maintained and published by [ImperatorVienna](https://github.com/ImperatorVienna) in an individual capacity. The individual maintainer is responsible for this project; it is not published on behalf of a company or organization.

Contact: [imperator.vienna@outlook.com](mailto:imperator.vienna@outlook.com).

## Reports and credits

Report the edition, OS/architecture, game and loader versions, reproduction steps and relevant redacted logs. Do not include access/refresh tokens, device codes or account storage.

Dependencies and bundled assets have their own notices and licenses; see `apps/desktop/src-tauri/licenses/`, the bundled font's OFL notice and [asset provenance](docs/ui-reference/README.md). This README does not grant redistribution rights to third-party artwork or Minecraft assets. Minecraft and related names belong to their respective owners.

## License

Project-owned code is licensed under [GNU GPL 3.0](LICENSE)
(`GPL-3.0-only`). Third-party libraries, fonts and artwork retain their own
licenses and ownership; this license does not relicense Minecraft assets.

## Interface design and asset notices

The interface draws on the visual style of the official Minecraft launcher, is independently implemented, and is optimized for keyboard, mouse and controller use. The project does not use official launcher program code obtained through reverse engineering.

Original code is licensed under GPL-3.0-only. Referenced Minecraft marks, textures, skins, backgrounds and official editorial content are excluded from that grant. The MCLA icon's original lettering and geometry are distinguished from its referenced grass-block texture. Provenance is not permission; unconfirmed redistribution permissions are recorded explicitly. See [Asset, design and third-party rights notices](ASSET-NOTICES.md).
