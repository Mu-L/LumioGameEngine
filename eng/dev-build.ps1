[CmdletBinding()]
param(
    [string]$NativeCoreRoot = '',
    [string]$VoxelRoot = '',
    [ValidateSet('debug', 'release')]
    [string]$Configuration = 'debug'
)
$ErrorActionPreference = 'Stop'
$nodeArgs = @((Join-Path $PSScriptRoot 'dev-build.mjs'), '--configuration', $Configuration)
if (-not [string]::IsNullOrWhiteSpace($NativeCoreRoot)) { $nodeArgs += @('--native-core-root', $NativeCoreRoot) }
if (-not [string]::IsNullOrWhiteSpace($VoxelRoot)) { $nodeArgs += @('--voxel-root', $VoxelRoot) }
& node @nodeArgs
exit $LASTEXITCODE
