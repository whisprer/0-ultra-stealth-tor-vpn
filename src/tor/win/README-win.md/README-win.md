TorVPN (Windows) — Operator README

A Windows Tor-backed VPN launcher that starts Tor only when needed, routes traffic through Tor, and provides start  stop  health controls with logging and recovery.

This project is designed to be

deterministic (fixed state dir, no guessing)

debuggable (logs always written)

non-invasive (no system boot services)

safe to crash without killing your shell

1. What this does (mental model)

When you run start

TorVPN starts Tor (if it isn’t already running)

Tor exposes

SOCKS on 127.0.0.19050

ControlPort on 127.0.0.19051

TorVPN authenticates via Tor control cookie

TorVPN sets up routing (tun2socks etc.)

Your system traffic is routed via Tor

Logs are written to logs

When you run stop

TorVPN removes DNS  firewall  routing changes

TorVPN shuts down Tor (only the instance it started)

Your system networking returns to normal

Tor does NOT start on boot.
Tor ONLY runs while the VPN is running.

2. Directory layout (important)
torvpn_win_control_plus_statusplan
├─ target
│  └─ release
│     └─ torvpn_win.exe
├─ profiles
│  └─ default_win.toml
├─ logs
│  ├─ start.log
│  └─ stop.log
├─ start.ps1
├─ stop.ps1
└─ README.md


External dependencies (already installed on your system)

Ctoolstor-experttortor.exe
Ctoolstun2sockstun2socks-windows-amd64-v3.exe
Ctoolstorvpn-state   (Tor state + cookie)

3. One-time setup (do this once)
3.1 Build the binary

From the project directory

cargo build --release


Confirm the binary exists

dir targetreleasetorvpn_win.exe

3.2 Verify your profile

Open

profilesdefault_win.toml


Minimum required entries

[tor]
binary = Ctoolstor-experttortor.exe
torrc  = Ctoolstorvpn-statetorrc
socks_port = 9050
control_port = 9051
dns_port = 0   # IMPORTANT must be 0 on Windows

[tun2socks]
binary = Ctoolstun2sockstun2socks-windows-amd64-v3.exe


⚠️ DNSPort must be disabled (dns_port = 0)
Windows blocks Tor DNSPort binding → Tor exits → VPN fails.

4. How to START the VPN

From any directory

powershell -NoProfile -ExecutionPolicy Bypass -File `
  Dcode0-ultra-stealth-tor-vpnsrctorwintorvpn_win_control_plus_statusplanstart.ps1


What you should see

Console output from TorVPN

A new file logsstart.log

Tor running (tor.exe)

VPN routing active

If it fails

notepad logsstart.log

5. How to STOP the VPN

From any directory

powershell -NoProfile -ExecutionPolicy Bypass -File `
  Dcode0-ultra-stealth-tor-vpnsrctorwintorvpn_win_control_plus_statusplanstop.ps1


This

Removes DNS  firewall rules

Stops tun2socks

Terminates Tor (only the TorVPN instance)

Logs written to

logsstop.log

6. How to check VPN health (Tor control)

From the project directory or using absolute paths

.targetreleasetorvpn_win.exe --profile .profilesdefault_win.toml health


Expected output (example)

statusbootstrap-phase=... DONE
OK
statuscircuit-established=1
OK


If this works, Tor control auth is correct.

7. How to test if your IP is hidden (IMPORTANT)
7.1 Test via command line (recommended)

After VPN is started

curl httpsapi.ipify.org


OR

curl httpsifconfig.me


You should NOT see your real ISP IP.

To see location

curl httpsipinfo.io


Example output

{
  ip 185.xxx.xxx.xxx,
  city Frankfurt,
  country DE,
  org Tor Exit Node
}


That confirms

Your traffic is exiting via Tor

Your real IP is hidden

7.2 Browser test (extra confidence)

After starting VPN

Open a browser

Visit

httpscheck.torproject.org

httpsipleak.net

You should see

“You are using Tor”

Exit country ≠ your real country

8. How to test routing failures (sanity checks)
Tor not running
netstat -ano  findstr 9051

Is Tor alive
Get-Process tor

View Tor logs
Get-Content Ctoolstorvpn-statetor.log -Tail 100

9. Logs & debugging
File	Purpose
logsstart.log	VPN startup, panics, Rust backtrace
logsstop.log	Cleanup actions
Ctoolstorvpn-statetor.log	Tor daemon logs

Enable full Rust backtrace (already in start.ps1)

$envRUST_BACKTRACE=full

10. Common failure modes (and fixes)
❌ Tor starts then exits immediately

Check dns_port = 0

Check tor.log for bind errors

❌ 515 Authentication failed

Cookie mismatch → wrong state dir

Ensure

Ctoolstorvpn-statecontrol_auth_cookie


Do NOT move Tor’s DataDirectory manually

❌ PowerShell window disappears

Use start.ps1 (logs output)

Never double-click torvpn_win.exe directly

11. Safe shutdown reminder

Always run stop.ps1 before

reboot

switching networks

disabling VPN manually

This ensures DNS + firewall rules are restored cleanly.

12. Design philosophy (why it’s this way)

No Windows services (fragile, painful)

No boot persistence (privacy-safe)

Explicit state directory

Tor lifecycle owned by the VPN

Crash-safe shell survives even if VPN dies

This is intentional.

13. Future extensions (optional)

torvpn restart

exit country rotation

Tor control SIGNAL NEWNYM

per-app routing

non-Tor fallback mode

TL;DR (muscle memory)
# start
powershell -ep bypass -file start.ps1

# test
curl ifconfig.me

# stop
powershell -ep bypass -file stop.ps1
