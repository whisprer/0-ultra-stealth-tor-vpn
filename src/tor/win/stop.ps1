$ErrorActionPreference = "Stop"

$ROOT    = Split-Path -Parent $MyInvocation.MyCommand.Path
$VPN_EXE = Join-Path $ROOT "target\release\torvpn_win.exe"
$PROFILE = Join-Path $ROOT "profiles\default_win.toml"

$LOGDIR = Join-Path $ROOT "logs"
$LOG    = Join-Path $LOGDIR "stop.log"
$STATE  = Join-Path $ROOT "state"

New-Item -ItemType Directory -Force $LOGDIR | Out-Null
New-Item -ItemType Directory -Force $STATE  | Out-Null

if (-not (Test-Path $VPN_EXE)) { throw "[torvpn] missing binary: $VPN_EXE" }
if (-not (Test-Path $PROFILE)) { throw "[torvpn] missing profile: $PROFILE" }

$env:RUST_BACKTRACE   = "full"
$env:TORVPN_STATE_DIR = $STATE

# Cookie path (see start.ps1 for details)
$cookie1 = Join-Path $STATE "tor-data\control_auth_cookie"
$cookie2 = "C:\tools\torvpn-state\control_auth_cookie"
if (Test-Path $cookie1) { $env:TORVPN_COOKIE_PATH = $cookie1 }
elseif (Test-Path $cookie2) { $env:TORVPN_COOKIE_PATH = $cookie2 }

$ts = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
"[$ts] [torvpn] stop requested" | Add-Content -Encoding UTF8 $LOG
"[$ts] [torvpn] exe:       $VPN_EXE" | Add-Content -Encoding UTF8 $LOG
"[$ts] [torvpn] profile:   $PROFILE" | Add-Content -Encoding UTF8 $LOG
"[$ts] [torvpn] state_dir: $STATE" | Add-Content -Encoding UTF8 $LOG

try {
  & $VPN_EXE --profile $PROFILE stop 2>&1 | Tee-Object -FilePath $LOG -Append
  exit $LASTEXITCODE
} catch {
  "[$(Get-Date -Format "yyyy-MM-dd HH:mm:ss")] EXCEPTION: $($_.Exception.ToString())" |
    Add-Content -Encoding UTF8 $LOG
  Write-Host "[torvpn] exception stopping torvpn (see $LOG)"
  exit 1
}
