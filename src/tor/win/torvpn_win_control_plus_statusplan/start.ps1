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
$env:TORVPN_COOKIE_PATH="C:\tools\torvpn-state\control_auth_cookie"

"[$(Get-Date -Format s)] start: $EXE" | Add-Content -Encoding ASCII $LOG
try { & $EXE --profile $PRO start 2>&1 | Tee-Object -FilePath $LOG -Append; exit $LASTEXITCODE }
catch { "EXCEPTION: $($_.Exception)" | Add-Content -Encoding ASCII $LOG; throw }
