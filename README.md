# MC Launcher Advanced

A third-party launcher for Minecraft Java Edition with an interface inspired by the official launcher.

**[简体中文](README.zh-CN.md)** · Package name: `mclauncher-advanced`

This independent desktop launcher supports keyboard, mouse and controller navigation, portable storage, isolated game instances and Modrinth content management. It is not an official Minecraft product and is not affiliated with, approved by or endorsed by Mojang or Microsoft.

## Release status

The current version is **1.0.4**, available for Linux and Windows x86_64. Platform validation details are listed below.

See the [1.0.4 release notes](docs/release-notes/1.0.4.en-US.md).

Version 1.0.4 fixes installation failures for some mods supporting multiple loaders caused by dependencies exclusive to another loader, and improves dependency resolution errors in English and Simplified Chinese.

## Features

- Install, repair and launch Minecraft: Java Edition using a Java runtime explicitly selected by the player.
- Create isolated instances, recognize local version folders, and manage memory, display mode and Java separately for each instance.
- Install Fabric, Quilt, Forge and NeoForge, and change loader versions within an instance's existing loader family.
- Download Java from Eclipse Adoptium/Temurin, verify downloaded files and rescan local runtimes. The player initiates downloads and selects the runtime.
- Browse, search, install and update mods, resource packs and shader packs on Modrinth, filtered by the current instance's game version and loader. Browsing and instance management share the same instance selector.
- Scan local content, enable or disable mods, delete files and check for updates. Resource packs and shaders are enabled and configured in the game.
- Navigate with a mouse, keyboard or physical controller. Optional Steam integration enables launcher fullscreen when started through Steam and requests the Steam virtual keyboard.
- View Xbox, PlayStation, Nintendo Switch Pro and generic controller diagrams and instructions in Settings. Choose a device from the dropdown; valid controller input automatically selects the active device without changing the page or focus.
- Use English or Simplified Chinese, with English selected on first launch.
- View each instance's game and installation logs, as well as launcher logs, within the interface.
- Read five official Java Edition news items on the home page and load game release notes for the current instance's Minecraft version.
- View the version, check for updates and visit the maintainer and project pages in Settings → About. Read open-source licenses and third-party notices in separate dialogs.

The skin library supports local PNG imports, account skin previews, and uploading and applying skins. Choose Classic or Slim arms, then click Use skin to apply the change to your account. The launcher automatically checks GitHub for stable updates at startup. Settings → About also lets you check manually, download and verify an update, and restart automatically to install it. The new version keeps the release asset's filename in the original directory; the previous version is retained with a `.previous` suffix. Update cache files are removed after the new version starts successfully. See the [release contract and validation scope](docs/launcher-updates.md).

## Official news and game release notes

The home page loads five Java Edition news items from Mojang's public launcher feeds. Clicking a card opens the corresponding official Minecraft article in the browser.

The game's Release notes page shows the official article for the current instance's Minecraft version. It is separate from launcher announcements under What's new? in the sidebar. Articles must match the exact game version; notes for a different version are never substituted. A message appears when no matching article exists or a network request fails. Official news and game release notes retain their original English text regardless of the interface language; buttons and source labels follow the selected language. Unavailable news or articles do not prevent the game from launching.

Version 1.0.2 introduced the MCLA icon for the launcher window, desktop integration and browser preview, fixed Wayland application identity matching, and improved bundled fonts, article layout and English interface text. Asset notices and original license texts are displayed in English.

## Platforms and first launch

| Platform | Package | Validation status |
| --- | --- | --- |
| Linux x86_64, glibc | AppImage | Game installation and launch tested during development |
| Windows 10/11 x86_64 | Portable EXE | Earlier versions received limited Wine checks; 1.0.4 has been cross-compiled and packaged, but still requires testing on Windows |
| macOS ARM64 | Planned | No build available |

Windows requires **Microsoft Edge WebView2 Runtime**. The portable EXE does not include or automatically install it. Wine/Proton is not a substitute for Windows acceptance testing; use the native AppImage on Linux. Windows 7/8/8.1 and other processor architectures are not current targets.

1. Place the executable in a writable folder, avoiding system-protected installation directories.
2. On Linux, make the AppImage executable and run it. The system must support running AppImages.
3. On Windows, install WebView2 Runtime if it is missing, then run the EXE. First launch creates `launch.bat` beside the executable and a desktop shortcut; it does not create a Start menu entry.
4. Select an instance and Java runtime. Forge/NeoForge installation requires a valid Java selection. Other installation workflows allow an explicit No Java selected choice, but launching the game still requires Java.
5. Sign in and launch. Use installation or repair to restore missing managed game files.

For Steam, add the generated launcher entry or script as a non-Steam game. Windows shortcuts point to `launch.bat`, which can still locate the executable after it is renamed. After moving the entire folder, run the executable directly once to refresh the desktop entry, then add it to Steam again.

## Portable storage

Storage is based on the directory containing the launcher executable, not the working directory used to run the command:

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

Folders are created as needed. Saves, mods, resource packs, shaders and game settings are isolated per instance; libraries and assets are shared. The launcher does not migrate data from other launchers or inherit mods and resource packs from their shared game directories. Local version detection does not guarantee compatibility with every third-party launcher's folder layout.

Managed Java paths are stored relative to the launcher directory. System Java may need to be selected again on a different computer; rescanning checks the current computer's runtimes.

`accounts.json` contains authentication credentials protected by operating-system file permissions. It is not an encrypted export. **Do not upload it, attach it to issue reports or share it when copying the launcher to someone else.**

## Content update rules

- Compatibility filters follow the current instance. Content for unsupported game versions may need to be installed manually.
- File hashes identify content recognized by Modrinth. Unrecognized files remain manageable by filename but cannot be matched automatically to updates.
- Mods, resource packs and shaders use the new upstream filenames after an update. Re-enable packs and adjust settings in the game if necessary; the launcher does not change the game's selected resource packs.
- Missing required dependencies are downloaded, and recognized existing dependencies are retained. Updating a file does not establish full compatibility among all installed mods.
- File deletion is permanent and does not create backups.
- In-game controller support requires a controller mod such as Controlify, and shaders require a compatible shader mod. Launcher navigation does not automatically add these capabilities to the game.

## Development

The project uses Tauri 2/Rust and React/TypeScript/Vite. Install a compatible Rust toolchain, Node.js 22.12+, the pnpm version specified in `package.json`, and the [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/).

```sh
pnpm install
pnpm dev                       # Browser preview: http://127.0.0.1:1420
pnpm test                      # Frontend regression tests
pnpm build                     # Type checking and frontend build
pnpm tauri dev                 # Native desktop development
pnpm tauri build --bundles appimage  # Linux AppImage
python3 scripts/check-source-language.py
```

Browser previews use separate local data and do not perform native game installation or launch. F8 controller simulation has been removed; controller navigation uses native input adapters.

Architecture boundaries are mandatory: the frontend owns presentation, focus and navigation; core owns portable models and persistence; domain services own game and content workflows; platform adapters own operating-system integration. See [platform boundaries](docs/platform-boundary.md), [storage rules](docs/storage-and-updates.md), [installation](docs/installation.md) and [form spacing](docs/ui-reference/form-spacing.md).

## Privacy and local data

[Privacy and local data](PRIVACY.md)

## Maintainer and contact

MC Launcher Advanced is maintained and published by [ImperatorVienna](https://github.com/ImperatorVienna) as an individual. The maintainer is responsible for the project and does not represent a company or organization.

Email: [imperator.vienna@outlook.com](mailto:imperator.vienna@outlook.com).

## Issue reports and acknowledgements

Include the version or branch, operating system and architecture, game and loader versions, reproduction steps and redacted logs when reporting an issue. Do not attach access tokens, refresh tokens, device sign-in codes or account storage files.

Dependencies and bundled assets have their own notices and licenses. See `apps/desktop/src-tauri/licenses/`, the bundled font's OFL notice and [asset provenance](docs/ui-reference/README.md). This README does not grant redistribution rights to third-party artwork or Minecraft assets. Minecraft and related names belong to their respective owners.

## License

Project-owned code is licensed under [GNU GPL 3.0](LICENSE) (`GPL-3.0-only`). Third-party libraries, fonts and artwork retain their own licenses and rights; this license does not relicense Minecraft assets.

## Interface design and asset notices

The interface references the visual style of the official Minecraft launcher. It is independently implemented and optimized for keyboard, mouse and controller use. The project does not use official launcher program code obtained through reverse engineering.

Original code is released under GPL-3.0-only. Referenced Minecraft logos, textures, skins, backgrounds and official news content are excluded from that license grant. Rights to the MCLA icon's original lettering and geometry are distinguished from those of the referenced grass-block texture. Provenance does not constitute permission; unconfirmed redistribution rights are documented explicitly. See [Asset, design and third-party rights notices](ASSET-NOTICES.md).
