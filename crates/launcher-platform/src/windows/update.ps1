$ErrorActionPreference = 'Stop'
try {
    $old = Get-Process -Id $oldPid -ErrorAction SilentlyContinue
    [IO.File]::WriteAllText($ready, 'ready')
    if ($old -and -not $old.WaitForExit(120000)) { throw 'Launcher did not exit' }
    # Antivirus and WebView teardown can briefly keep handles open after exit.
    $moved = $false
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        try {
            if ([IO.File]::Exists($backup)) { [IO.File]::Delete($backup) }
            [IO.File]::Move($destination, $backup)
            $moved = $true
            break
        } catch { Start-Sleep -Milliseconds 500 }
    }
    if (-not $moved) { throw 'Cannot replace launcher executable' }
    try {
        [IO.File]::Move($staged, $destination)
        $start = New-Object Diagnostics.ProcessStartInfo
        $start.FileName = $destination
        $start.WorkingDirectory = [IO.Path]::GetDirectoryName($destination)
        $start.UseShellExecute = $false
        $child = [Diagnostics.Process]::Start($start)
    } catch {
        if ([IO.File]::Exists($destination)) { [IO.File]::Delete($destination) }
        [IO.File]::Move($backup, $destination)
        throw
    }
    $child.WaitForExit()
    exit $child.ExitCode
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 1
}
