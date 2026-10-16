# Local skins

The account skin page is a local library, independent of game instances.
Offline accounts are for launch testing and do not support custom skins.
No Java agent or local authentication/skin server is installed or launched.
When an offline account is selected, the skin page displays only an unsupported-account message; the library and its controls are hidden. The skin library implementation is retained for future Microsoft accounts. Applying
a skin is disabled until Microsoft authentication and official upload are
implemented. Historical local account bindings are ignored by the UI.
The sidebar uses a neutral avatar for offline accounts.

The desktop stores a revisioned, atomically written library at
~/.minecraft/launcher-skins.json. Browser previews use separate localStorage.
PNG data is embedded in the library so deletion cannot leave orphan image files.
Deleting a custom skin also removes all its account bindings. Builtin names and files cannot
be deleted or edited; import a PNG to create an editable entry.

Imports are bounded to 128 KiB and 64x64 or legacy 64x32 pixels. The frontend
decodes and normalizes imports to static PNG. Rust decodes the PNG again before
saving. Invalid or corrupt stores are not silently overwritten. Up to 100
custom skins are supported.

The main preview uses skinview3d 3.4.2 (MIT), loaded only on the skins page.
Library thumbnails and sidebar heads use 2D canvas. WebGL failures fall back to
a flat preview. Rotation buttons and the outer-layer checkbox participate in
the existing navigation system; dragging also rotates the model.

## Builtin asset provenance

The nine PNGs in apps/desktop/src/assets/skins were extracted, unmodified, from
the user's installed Minecraft 26.3 client JAR, under
assets/minecraft/textures/entity/player/. Alex uses slim/alex.png; the other
eight use wide/<name>.png. Minecraft artwork belongs to Mojang/Microsoft.
Each builtin also includes the opposite arm variant as <name>-alternate.png.
Per-card model choices are saved immediately; builtin choices use builtinModels
and switch to the corresponding official texture, while custom choices update model.
These assets are separate from the MIT-licensed renderer. Renderer, utility,
and Three.js licenses are included in the application resources.

Browser previews expose a separate "Microsoft account (test)" selection at the
top of the account dropdown. It unlocks the skin page for UI testing only.
It is session-only, never written to accounts.json, and not offered in Tauri.
Editing/deleting that test entry is disabled; selecting an offline account
restores the unsupported-skins message. Official skin upload remains disabled.
