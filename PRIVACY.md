# Privacy and local data

MC Launcher Advanced is maintained by ImperatorVienna in an individual capacity. Contact: imperator.vienna@outlook.com. This document describes the current public desktop build; it does not replace the privacy policies of external services.

## Sign-in and credentials

Microsoft sign-in takes place in the system browser using a device code. The launcher does not ask for or store your Microsoft password. The native process exchanges tokens with Microsoft, Xbox Live and Minecraft Services. The interface receives the temporary user-facing sign-in code and public account summaries, not access or refresh tokens.

The launcher stores Minecraft profile names and UUIDs, Xbox user IDs when returned, access and refresh tokens, expiry times and the selected account in `accounts.json` beside the launcher executable. This file is not encrypted. New writes use owner-only file permissions on Linux and a restricted file ACL on Windows. These protections do not protect against software running as you or an administrator, and may not survive copying to another filesystem.

Selecting Delete in account management removes that account's locally stored credentials. It does not delete your Microsoft account or revoke sessions at Microsoft. To remove all local sign-in data, close the launcher and delete `accounts.json`. Backups and other copies must be removed separately. Browser cookies and Microsoft permissions are managed through your browser and Microsoft account settings.

## Other local data

Game configurations, Java selections, launcher preferences, skins and logs are stored in the portable launcher directory. The embedded WebView also has local storage/cache under `webview/`. Games, mods and resource packs are stored under `.minecraft/`. Removing a launcher account does not remove these files. To remove all launcher data, close the launcher and delete its folder after backing up any worlds or other files you want to keep.

Launcher logs record operation status. Game and installer logs are produced by the game or installer and can contain local paths, player names, server addresses or other personal information. Review and redact logs before sharing them. Do not share `accounts.json`, access/refresh tokens or sign-in codes.

## External connections

- Microsoft, Xbox Live and Minecraft Services: device-code sign-in, token exchange/refresh, entitlement and profile requests.
- Mojang/Minecraft services: game metadata and files, official news, article content and images.
- GitHub: automatic stable-release checks at startup and requested launcher updates; some download providers also distribute files through GitHub.
- Modrinth: content searches, file-hash identification, update checks and downloads. Identification sends file hashes, not the complete local mod or pack files; searches include the query and applicable game version/loader filters.
- Eclipse Adoptium and loader distribution services: requested Java, Fabric, Quilt, Forge and NeoForge metadata and downloads.

These services and their distribution networks receive normal request information, including your IP address and request metadata. Opening an external article, project page or sign-in page invokes the system browser and is subject to that browser's settings and the destination's policies. Steam integration, when enabled, requests the local Steam client's virtual keyboard; controller detection reads local device/input data.

The launcher currently has no dedicated analytics, advertising or automatic crash-report upload service. Logs are not automatically submitted to the maintainer. This does not describe telemetry performed independently by Minecraft, mods, Microsoft, Steam or other external software.

## Current authentication limitation

Minecraft API access for this application is not approved. The implemented flow and simulated tests do not establish successful real-account authentication or complete subscription/restriction handling. No subscription status is inferred solely from an empty entitlement list.
