# Skins

Signed-in Microsoft accounts load their active skin from the authenticated Minecraft profile. The native application refreshes credentials when needed, downloads only HTTPS textures from textures.minecraft.net without credentials or redirects, and validates a bounded static PNG before returning image data. The frontend displays the skin in the sidebar and as an account entry in the skin library, preserving classic/slim arms. Switching accounts discards stale responses; preview failures do not block login or game launch. Profiles without an active texture retain the placeholder avatar and local preview. Users can select classic/slim arms locally, then explicitly click Use skin to upload the selected PNG and model to the authenticated account. Builtin, imported and current account skins use the same upload path. The native service validates the PNG and model, refreshes credentials as needed, sends multipart data to the fixed official endpoint, and verifies the returned account and active model before reporting success. Failed uploads retain the last confirmed account preview; concurrent clicks and stale account responses are ignored. The account entry cannot be deleted or edited as a local library record.

The local library is independent of game instances. No Java agent or local authentication/skin server is installed or launched. Historical local account bindings are ignored by the UI.

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
