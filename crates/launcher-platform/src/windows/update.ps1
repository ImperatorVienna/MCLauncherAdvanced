$ErrorActionPreference = 'Stop'
try {
    $old = Get-Process -Id $oldPid -ErrorAction SilentlyContinue
    [IO.File]::WriteAllText($ready, 'ready')
    if ($old -and -not $old.WaitForExit(120000)) { throw 'Launcher did not exit' }
    if ($destination -ine $current -and (Test-Path -LiteralPath $destination)) { throw 'New launcher filename already exists' }
    # Antivirus and WebView teardown can briefly keep handles open after exit.
    $moved = $false
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        try {
            if ([IO.File]::Exists($backup)) { [IO.File]::Delete($backup) }
            [IO.File]::Move($current, $backup)
            $moved = $true
            break
        } catch { Start-Sleep -Milliseconds 500 }
    }
    if (-not $moved) { throw 'Cannot replace launcher executable' }
    $installed = $false
    try {
        [IO.File]::Move($staged, $destination)
        $installed = $true
        $start = New-Object Diagnostics.ProcessStartInfo
        $start.FileName = $destination
        $start.WorkingDirectory = [IO.Path]::GetDirectoryName($destination)
        $start.UseShellExecute = $false
        $start.EnvironmentVariables['MCLAUNCHER_UPDATE_DIRECTORY'] = [IO.Path]::GetDirectoryName($staged)
        $child = [Diagnostics.Process]::Start($start)
    } catch {
        if ($installed -and [IO.File]::Exists($destination)) { [IO.File]::Delete($destination) }
        [IO.File]::Move($backup, $current)
        throw
    }
    $child.WaitForExit()
    exit $child.ExitCode
} catch {
    [IO.File]::AppendAllText((Join-Path ([IO.Path]::GetDirectoryName($staged)) 'apply.log'), $_.Exception.Message + [Environment]::NewLine)
    exit 1
}
