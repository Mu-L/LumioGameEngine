[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$VoxelRoot,
    [string]$NativeCoreRoot = '',
    [string]$ServerRoot = '',
    [string]$RuntimeRoot = ''
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$voxelRoot = (Resolve-Path $VoxelRoot).Path
if (-not [string]::IsNullOrWhiteSpace($ServerRoot)) { $env:LumioServerRoot = (Resolve-Path $ServerRoot).Path }
if (-not [string]::IsNullOrWhiteSpace($RuntimeRoot)) { $env:LumioRuntimeRoot = (Resolve-Path $RuntimeRoot).Path }
$runArgs = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $root 'eng\dev-run.ps1'), '-VoxelRoot', $voxelRoot)
if (-not [string]::IsNullOrWhiteSpace($NativeCoreRoot)) { $runArgs += @('-NativeCoreRoot', (Resolve-Path $NativeCoreRoot).Path) }
$previous = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
$output = & powershell @runArgs 2>&1
$code = $LASTEXITCODE
$ErrorActionPreference = $previous
if ($code -ne 0) { throw "dev-run.ps1 failed with exit code $code`n$output" }
$lines = $output | ForEach-Object { $_.ToString() }
$manifestLine = $lines | Where-Object { $_ -like 'NATIVE_MANIFEST=*' } | Select-Object -Last 1
if (-not $manifestLine) { throw 'Native build did not report its actual workspace manifest.' }
$nativeManifest = $manifestLine.Substring('NATIVE_MANIFEST='.Length)
$sdkManifest = Join-Path (Split-Path -Parent $nativeManifest) 'modules\sdk-native\Cargo.toml'
$manifestText = Get-Content -LiteralPath $sdkManifest -Raw
if ($manifestText -notmatch [regex]::Escape($voxelRoot.Replace('\', '/'))) { throw "Wrong VoxelRoot in $sdkManifest" }
$serverLine = $lines | Where-Object { $_ -match '^SERVER SERVER_READY ' } | Select-Object -First 1
$clientLine = $lines | Where-Object { $_ -match '^CLIENT ENGINE_NATIVE ' } | Select-Object -First 1
if (-not $serverLine -or -not $clientLine) { throw "Missing host loading evidence.`n$output" }
if ($lines -notcontains 'VERIFICATION_STATUS=PASS') { throw 'Verification did not complete successfully.' }
Write-Output "DEV_RUN_VOXEL_ROOT_REGRESSION=manifest:$sdkManifest voxelRoot:$voxelRoot"
Write-Output $serverLine
Write-Output $clientLine
Write-Output 'DEV_RUN_PROCESSES_CLEANED=true'
