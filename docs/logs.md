# Logs

Instance management exposes Open instance directory and a yellow View logs button.
The Settings page exposes yellow View launcher logs. Game failure messages offer
a yellow link to the affected run (using its instance ID, not the selected
instance) and Copy error details. All log-view/open-file actions use yellow
StandardButton controls.

One read-only dialog lists up to 200 newest files, refreshes, opens the original
file via the system application, and copies displayed text. It supports keyboard
Tab/Escape and controller focus/Back; controller scrolling targets the log content
while the dialog is open. An open dropdown retains input priority. Closing the
dialog is allowed while a game or transfer is running and restores entry focus.
Old requests cannot replace a newly selected file. Contents are rendered as text,
never HTML. Browser preview shows an empty log list and cannot access local files.

Game stdout/stderr, Minecraft logs and newly generated Forge/NeoForge installer
logs belong to versions/<instance directory>/logs. Each installer attempt gets a
timestamp, so retries do not overwrite earlier attempts. Crash reports are listed
from that instance's crash-reports directory. Existing historical logs under the
old shared launcher-logs directory are left intact: their owning instance cannot
be inferred reliably when multiple configurations use the same loader version.

Launcher operational events (startup, instance-store errors, installation, loader
updates, game session lifecycle and close requests) go to
<launcher>/logs/launcher-<timestamp>.log. It keeps the latest 10
launcher-owned sessions with a 5 MiB per-session write cap. It does not capture
arbitrary stdout, full launch arguments or credentials. Game logs are not pruned.

IPC resolves a known instance ID or the fixed launcher directory. It rejects
arbitrary paths, symlinks, traversal and unsupported file extensions. Plain logs
show at most the last 512 KiB; gzip archives show at most the first 512 KiB after
bounded decompression. Large files display a notice directing users to the
original file. Copy copies only what is displayed. No logs are uploaded.
