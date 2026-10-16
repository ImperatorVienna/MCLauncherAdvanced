# Local instance content management

The instance mods, resource packs and shaders tabs share one frontend and backend.
All paths are resolved from the stored instance ID. No global game folders are imported.
Opening a tab creates its missing `mods`, `resourcepacks` or `shaderpacks` directory
when no game or file task is running. Refresh and window focus rescan external changes.

Only mods have enable/disable checkboxes. Resource and shader packs have file lists and deletion, with no backup or disabled-folder mechanism. Recognized content:
- Mods: `.jar` files.
- Resource packs: `.zip` files or directories containing `pack.mcmeta`.
- Shader packs: `.zip` files or directories containing `shaders`.

Disabling a mod moves its file into the instance's
`.launcher-disabled/mods/`; enabling restores it to its original name.
No bytes are rewritten, archives are not extracted, and settings such as
`options.txt`, Iris/OptiFine configuration and shader parameters are untouched.
Players choose pack stacking order and their active shader inside the game.
A checked mod still needs a compatible loader and dependencies. The backend rejects enable/disable requests for resource and shader packs.

Linux `renameat2(RENAME_NOREPLACE)` prevents overwriting concurrent destination
files. Duplicate names in both locations block toggling. Symlink entries are
excluded, paths are scoped, and mutations hold the storage lock and check backend
busy state. Scans during running tasks are read-only. Frontend scope remounting
discards late results after switching instances/tabs; file names keep focus IDs
stable across toggles. Browser preview never pretends to access local files.

In-game selection editing and automatic resolution of existing dependency conflicts remain outside this stage.

All three lists have a red Delete button for permanent removal of the displayed
file or unpacked pack directory, without backups. Disabled mods can also be
deleted. Conflicts block deletion until resolved. Deletion uses the same scoped
busy guard as toggles and returns focus to a remaining delete button or Rescan.
Download buttons navigate to the matching sidebar browser. The browser reuses
the global instance selector; its version and mod loader are not editable filters.
Modrinth is currently the sole entry in the Settings source dropdown.

Native catalog and file operations re-resolve the instance from storage. Search
uses the exact game version and, for mods, the exact loader. Resource/shader packs
use game-version filters because their loader tags describe pack consumers such
as Iris, not Fabric/Forge. The source's version catalog determines unsupported
versions; an empty search is distinct from unsupported or network failure.
Browser previews use the public API but never write local files.

Downloads reuse the install task/progress/cancel system. Required dependencies
skip projects already recognized locally, regardless of version or enabled
state. Provider version pins select the download only for missing dependencies;
existing dependencies are never upgraded, downgraded or enabled automatically.
All downloads are staged and hashed before commit. This presence-only policy
does not verify compatibility between installed mods.

Mod updates identify local files through SHA-1, not filenames. Only newer versions
for the same project and exact game/loader yield an Update button before Delete.
Unrecognized files remain untouched. Updates recheck original bytes, preserve
enabled/disabled state, stage downloads, and use non-overwriting moves. Failed
commit restores the original; if restoration itself fails its recovery directory
is retained and reported. No permanent update backup is created.

Focus: changing configuration keeps focus on the shared selector. Page Back
restores the management Download control. Pending controls keep existing focus
but cannot activate; after leaving them they are not navigable until enabled.
Late requests cannot change a new instance's list. Completed update/deletion
moves focus from removed controls to a remaining file action or Rescan.
Live tests exercise catalog search, verified download, hash identification of a
renamed disabled mod, and updating it while preserving its disabled state.

All content updates (mods, resource packs and shader packs) use the new upstream
filename. Updates never rewrite options.txt, shader configuration or sidecar
parameter files. Players reselect resource/shader packs and adjust their settings
inside the game when needed. Mod enabled/disabled state is still preserved by
installing into the corresponding directory. File verification, conflict checks
and commit rollback remain shared across all content types.

Rescans retain checked update suggestions whose local filename and hash still
match, updating their enabled state after toggles. Changed or removed files lose
only their own suggestion. This local validation makes no catalog requests;
failed updates retain retry actions, and unrelated focused update buttons stay
in place. These rules apply equally to all three content kinds.

Download catalogs identify local archives by hash for the selected instance,
including disabled mods. A project with no recognized archive offers Download;
a recognized project offers Update only for a newer compatible upstream release,
otherwise Installed is green and noninteractive. Duplicate recognized versions
are not arbitrarily chosen for replacement. Unknown hashes/unpacked folders
cannot provide reliable project identity; file collision checks still apply.
Identification failure does not enable download actions as if nothing existed.
Catalog status is refreshed after file tasks and when the window regains focus;
late requests cannot update a new instance/page. Pagination waits for status and
focuses the first/last actionable result, falling back to search when none exist.
This logic belongs in the content service and frontend, not host adapters.
