$ErrorActionPreference = "Stop"
$ROOT = Split-Path -Parent $MyInvocation.MyCommand.Path
$EXE  = Join-Path $ROOT "target\release\torvpn_win.exe"
$PRO  = Join-Path $ROOT "profiles\default_win.toml"
$LOGD = Join-Path $ROOT "logs"
$LOG  = Join-Path $LOGD "start.log"
$STATE= Join-Path $ROOT "state"

New-Item -ItemType Directory -Force $LOGD,$STATE | Out-Null
if (!(Test-Path $EXE)) { throw "missing exe: $EXE" }
if (!(Test-Path $PRO)) { throw "missing profile: $PRO" }

$env:RUST_BACKTRACE="full"
$env:TORVPN_STATE_DIR=$STATE

# Cookie path:
# - If Tor is launched by torvpn, cookie will appear under $STATE\tor-data\...
# - If you run an external Tor instance, you can keep using C:\tools\torvpn-state
$cookie1 = Join-Path $STATE "tor-data\control_auth_cookie"
$cookie2 = "C:\tools\torvpn-state\control_auth_cookie"
if (Test-Path $cookie1) { $env:TORVPN_COOKIE_PATH = $cookie1 }
elseif (Test-Path $cookie2) { $env:TORVPN_COOKIE_PATH = $cookie2 }

"[$(Get-Date -Format s)] start: $EXE" | Add-Content -Encoding ASCII $LOG
try { & $EXE --profile $PRO start 2>&1 | Tee-Object -FilePath $LOG -Append; exit $LASTEXITCODE }
catch { "EXCEPTION: $($_.Exception)" | Add-Content -Encoding ASCII $LOG; throw }
