# Embedded in launch.bat; all paths are data, never interpolated into commands.
$ErrorActionPreference = 'Stop'
try {
    $root = [IO.Path]::GetFullPath($env:MCLAUNCHER_ROOT)
    function IsLauncher([IO.FileInfo]$file) {
        if (-not $file.Exists -or $file.Extension -ine '.exe' -or ($file.Attributes -band [IO.FileAttributes]::ReparsePoint)) { return $false }
        if ($file.Name -match '(?i)(\.previous|\.tmp|\.partial|\.download)(\.|$)') { return $false }
        try { return [Diagnostics.FileVersionInfo]::GetVersionInfo($file.FullName).ProductName -ceq 'MC Launcher Advanced' } catch { return $false }
    }
    $selected = $null
    $record = Join-Path $root 'launcher-executable.txt'
    if ([IO.File]::Exists($record)) {
        $name = [IO.File]::ReadAllText($record).Trim()
        if ($name -and [IO.Path]::GetFileName($name) -ceq $name) {
            $candidate = [IO.FileInfo]::new((Join-Path $root $name))
            if (IsLauncher $candidate) { $selected = $candidate }
        }
    }
    if ($null -eq $selected) {
        $matches = @(Get-ChildItem -LiteralPath $root -File -Filter '*.exe' | Where-Object { IsLauncher $_ })
        if ($matches.Count -ne 1) { throw "Expected one launcher EXE in this folder; found $($matches.Count). Run the desired launcher EXE directly once to refresh the entry." }
        $selected = $matches[0]
    }
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $selected.FullName
    $start.WorkingDirectory = $root
    $start.UseShellExecute = $false
    $process = [Diagnostics.Process]::Start($start)
    $process.WaitForExit()
    exit $process.ExitCode
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 1
}
