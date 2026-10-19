$ErrorActionPreference = 'Stop'
$root = $env:MCLAUNCHER_ROOT
$desktop = [Environment]::GetFolderPath('DesktopDirectory')
if (-not $desktop) { throw 'The desktop directory is unavailable' }
$shortcut = (New-Object -ComObject WScript.Shell).CreateShortcut((Join-Path $desktop 'MC Launcher Advanced.lnk'))
$shortcut.TargetPath = Join-Path $root 'launch.bat'
$shortcut.WorkingDirectory = $root
$shortcut.Description = 'A third-party powered launcher for Minecraft Java Edition with an interface similar to the official.'
$shortcut.IconLocation = (Join-Path $root 'icon.ico') + ',0'
$shortcut.WindowStyle = 7
$shortcut.Save()
