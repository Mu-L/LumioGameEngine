[CmdletBinding()]
param(
    [switch]$KeepRunning,
    [switch]$Verify,
    [string]$VoxelRoot = '',
    [string]$NativeCoreRoot = ''
)
$ErrorActionPreference = 'Stop'
$nodeArgs = @((Join-Path $PSScriptRoot 'dev-run.mjs'))
if ($KeepRunning) { $nodeArgs += '--keep-running' }
if ($Verify) { $nodeArgs += '--verify' }
if (-not [string]::IsNullOrWhiteSpace($VoxelRoot)) { $nodeArgs += @('--voxel-root', $VoxelRoot) }
if (-not [string]::IsNullOrWhiteSpace($NativeCoreRoot)) { $nodeArgs += @('--native-core-root', $NativeCoreRoot) }
& node @nodeArgs
exit $LASTEXITCODE
