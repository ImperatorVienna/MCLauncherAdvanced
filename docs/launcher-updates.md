# Launcher updates

The fixed source is `ImperatorVienna/MCLauncherAdvanced` on GitHub. This document
specifies a future release; it does not publish anything or require an existing
release to test the implementation.

## User flow

Packaged builds check once at startup without blocking startup or moving focus.
Settings can repeat the check. An inaccessible/missing stable release (HTTP 404)
is distinct from an up-to-date installed version. Network, rate-limit and invalid
metadata failures are errors, not a successful check.

A newer stable release displays a red sidebar notice. What's new shows the stable
release body as inert text and a green shortcut to Settings; release content never
executes HTML. Startup checks do not move focus. The frontend shares one update
state across Settings, announcements, the sidebar and the task tray.

The user chooses **Download launcher update** and can cancel the transfer. After
verification the launcher automatically applies the update and restarts. A single
writer guard spans download, verification and handoff, without an unlocked gap.
Other page controls are inert during this operation; keyboard and controller focus
moves to Cancel and returns to the originating action on cancellation/failure.
Checking does not acquire the game/files lock; downloading cannot begin during a
game session or another file operation. English and Chinese UI text are provided.

Development binaries cannot replace themselves. Stable builds only accept newer
stable SemVer releases, never a downgrade or prerelease. Unsupported version
strings cannot be used for stable update selection.

## Publishing contract

- Publish a non-draft, non-prerelease GitHub Release tagged `v1.0.0` (or `1.0.0`).
  The version must match the built application's version, including Cargo and
  Tauri configuration. GitHub's latest stable release is authoritative.
- Upload assets using these exact names:
  - `mclauncher-advanced_1.0.0-linux-x86_64.AppImage`
  - `mclauncher-advanced_1.0.0-win-x86_64.exe`
- Finish uploading before publishing. Each asset must have `state: uploaded`,
  a nonzero size, and a GitHub API `digest` of `sha256:<64 lowercase hex digits>`.
  A missing digest blocks updates; it is never bypassed. Check this metadata
  after upload before announcing a release.
- A release without an asset for the running platform reports an error rather
  than selecting another architecture. Internal `rN-*` packages are not selected
  by this stable channel.
- No token is needed for public releases. GitHub's anonymous API rate limit still
  applies. No GitHub credential is stored in the application.

The download URL must point to this repository/tag/exact asset name. Downloads
use HTTPS, allowlisted redirects, a size limit and streaming SHA-256 checks.
GitHub's digest provides transfer integrity under the repository/HTTPS trust
model; it is not an independent publisher signature. No update URL or path is
accepted from frontend IPC.

## Replacement and recovery

The verified file is staged on the same filesystem as the running executable.
It is hashed again before handoff. A helper readiness handshake completes before
the old app exits. The helper waits for that process to end; Windows retries the
rename briefly for teardown/antivirus handles. On a failed staged-file move it
restores the previous executable. Windows also restores it if process creation
fails. The previous binary stays beside the new one as `<filename>.previous`;
an older backup is replaced only when applying another update.

The replacement uses the new release asset filename. Startup refreshes the
portable script or remembered EXE name; the desktop shortcut still targets the
stable script. An existing destination file blocks the update without overwriting
it. Game directories, Java and user settings
are not moved. Restart preserves inherited Steam identifiers; Linux clears the
old AppImage mount variables before starting the replacement.

The helper writes `.launcher-update-*/apply.log` next to the launcher. If the new
application starts but later fails, the updater does not infer application health
or automatically revert data. With the launcher closed, the retained `.previous`
file can be renamed to the original executable name for manual recovery. After successful startup the complete helper directory is removed, including
logs and temporary files. Failed updates retain diagnostics. Interrupted staged downloads
never replace the installed executable.

## Validation

Tests cover stable version ordering, target selection, missing digest, wrong
repository, prerelease exclusion, verification and cancellation in the shared
downloader, Linux replacement with quoted/spaced filenames, retained backup and
Steam environment, rollback after failed rename, and rejected symlinks. Frontend
tests cover no-release messaging and disabled/focused action transitions.
Linux and Windows x86_64 source compilation is checked separately.

A real GitHub Release end-to-end upgrade cannot yet be exercised because there
are no releases. Windows helper replacement/restart still requires real Windows
acceptance testing; cross-compilation does not replace that test. The first two
published test versions can be used for that final acceptance without changing
this implementation.
