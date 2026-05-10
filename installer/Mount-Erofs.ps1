<#
.SYNOPSIS
  Mount an erofs image as a Windows drive letter via WinFsp.

.DESCRIPTION
  Thin PowerShell wrapper around `erofs.exe mount`. Accepts an optional
  image path; if omitted, opens an Open-File dialog so the user can pick
  one. Picks the first free drive letter (Z:..D:) and runs the mount in
  the foreground -- Ctrl-C unmounts.

  Installed alongside erofs.exe in `%ProgramFiles%\erofs-win-driver\`.

.PARAMETER ImagePath
  Path to an erofs image (.img) or whole-disk image. Optional.

.PARAMETER DriveLetter
  Force a specific drive letter (e.g. 'X:'). Optional. Default: first free.

.PARAMETER Part
  1-indexed partition number for whole-disk images. Optional.

.PARAMETER ReadOnly
  Mount read-only. Default is read-write -- pass this switch when you
  want to be sure nothing on the disk gets touched (forensics, damaged
  filesystem inspection, etc).

.EXAMPLE
  Mount-Erofs.ps1 C:\images\rootfs.img
  Mount-Erofs.ps1 -ImagePath disk.img -DriveLetter Y: -Part 1
  Mount-Erofs.ps1 -ImagePath disk.img -ReadOnly  # safe inspection mode
  Mount-Erofs.ps1                                # opens file picker
#>

[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$ImagePath,

    [Parameter()]
    [string]$DriveLetter,

    [Parameter()]
    [int]$Part,

    [Parameter()]
    [switch]$ReadOnly
)

$ErrorActionPreference = 'Stop'

# ----------------------------------------------------------------------------
# Locate erofs.exe. Installed copy lives next to this script in Program Files;
# fall back to PATH for dev runs.
# ----------------------------------------------------------------------------
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$exeNextToScript = Join-Path $scriptDir 'erofs.exe'

if (Test-Path $exeNextToScript) {
    $erofsExe = $exeNextToScript
} else {
    $cmd = Get-Command erofs.exe -ErrorAction SilentlyContinue
    if (-not $cmd) {
        Write-Error "erofs.exe not found next to this script ($scriptDir) or on PATH."
        exit 1
    }
    $erofsExe = $cmd.Source
}

# ----------------------------------------------------------------------------
# If no image path was supplied, pop the Open-File dialog.
# ----------------------------------------------------------------------------
if (-not $ImagePath) {
    Add-Type -AssemblyName System.Windows.Forms
    $dlg = New-Object System.Windows.Forms.OpenFileDialog
    $dlg.Title = 'Select an erofs image to mount'
    $dlg.Filter = 'Disk images (*.img;*.iso;*.bin;*.raw)|*.img;*.iso;*.bin;*.raw|All files (*.*)|*.*'
    $dlg.CheckFileExists = $true
    if ($dlg.ShowDialog() -ne [System.Windows.Forms.DialogResult]::OK) {
        Write-Host 'Cancelled.'
        exit 0
    }
    $ImagePath = $dlg.FileName
}

if (-not (Test-Path $ImagePath)) {
    Write-Error "Image not found: $ImagePath"
    exit 1
}

# ----------------------------------------------------------------------------
# Pick a drive letter if the user didn't supply one. Walk Z..D and grab the
# first letter that isn't currently a PSDrive.
# ----------------------------------------------------------------------------
function Get-FreeDriveLetter {
    $used = @(Get-PSDrive -PSProvider FileSystem | ForEach-Object { $_.Name.ToUpper() })
    foreach ($c in [char[]]'ZYXWVUTSRQPONMLKJIHGFED') {
        if ($used -notcontains "$c") { return "${c}:" }
    }
    throw 'No free drive letter available (D..Z all in use).'
}

if (-not $DriveLetter) {
    $DriveLetter = Get-FreeDriveLetter
}

# Normalise: 'X' -> 'X:'.
if ($DriveLetter -notmatch ':$') { $DriveLetter += ':' }

# ----------------------------------------------------------------------------
# Run the mount in the foreground. Ctrl-C in the console unmounts.
# ----------------------------------------------------------------------------
$args = @('mount', $ImagePath, '--drive', $DriveLetter)
if ($PSBoundParameters.ContainsKey('Part')) {
    $args += @('--part', "$Part")
}
if ($ReadOnly) {
    $args += '--ro'
}

$mode = if ($ReadOnly) { 'read-only' } else { 'read-write' }
Write-Host "Mounting $ImagePath at $DriveLetter ($mode) ..."
Write-Host "Ctrl-C in this window to unmount."
Write-Host ""

& $erofsExe @args
exit $LASTEXITCODE
