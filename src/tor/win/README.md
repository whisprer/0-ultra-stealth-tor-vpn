# TorVPN (Windows MVP)

This is a Windows-only MVP that boots a local Tor instance and gives you:

- a **SOCKS proxy** at `127.0.0.1:9050`
- a **Tor control** port at `127.0.0.1:9051` (cookie auth)
- helper commands to apply exit/proxy policy and request NEWNYM
- wrappers (`start.cmd` / `stop.cmd`) so you can run it from **any** folder without “where am I?” path pain

## What this is (and is not)

- ✅ Great for: routing *specific apps* through Tor (browser, tooling) via SOCKS.
- ⚠️ Full “all traffic in Windows is forced through Tor” is **not** automatic in this MVP. That requires system routing / adapter work. We *do* apply a conservative firewall policy to reduce leaks, but you still need to point apps at the SOCKS proxy (or add full-tunnel routing later).

## Quick start

From this folder, build once:

```powershell
cargo build --release
```

Then:

```powershell
.\start.cmd
```

Stop everything and restore firewall/NRPT:

```powershell
.\stop.cmd
```

### Run from any directory

The wrappers resolve paths relative to themselves, so you can do:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File "D:\code\...\torvpn_win_control_plus_statusplan\start.ps1"
```

or simply:

```powershell
D:\code\...\torvpn_win_control_plus_statusplan\start.cmd
```

## Verifying Tor is up

After `start.cmd`, you should see listeners:

```powershell
netstat -ano | findstr ":9050"
netstat -ano | findstr ":9051"
```

And the built-in health check:

```powershell
.\target\release\torvpn_win.exe --profile .\profiles\default_win.toml health
```

If you see an auth error like **“cookie did not match expected value”**, set the cookie path:

```powershell
$env:TORVPN_COOKIE_PATH = "C:\tools\torvpn-state\control_auth_cookie"
```

…but if you’re using `start.cmd` and letting torvpn launch Tor, you usually **don’t** need to set this manually (the program defaults to the cookie under the state dir).

## Testing that your apparent IP changed

### Easiest: Tor Browser

Install **Tor Browser**, run it, and open an IP-check site. That proves Tor itself is working.

### CLI tests (Windows)

**Is curl Linux-only?** No — modern Windows includes `curl.exe`.

Direct test via SOCKS (DNS through SOCKS as well):

```powershell
curl.exe --socks5-hostname 127.0.0.1:9050 https://api.ipify.org?format=text
```

If you don’t have curl in PATH, you can still call it explicitly if present (e.g. `C:\Windows\System32\curl.exe`).

## Using an app through the SOCKS proxy

You must configure the app to use:

- **SOCKS5 host:** `127.0.0.1`
- **SOCKS5 port:** `9050`
- Prefer “proxy DNS through SOCKS” (or “SOCKS v5 remote DNS”) if the app offers it.

### Firefox example

Settings → Network Settings → Manual proxy configuration:

- SOCKS Host: `127.0.0.1`
- Port: `9050`
- SOCKS v5
- Enable remote DNS (Firefox: `network.proxy.socks_remote_dns = true`)

## Rotation / new exits

Your binary supports:

- `newnym` — ask Tor for new circuits
- `proxy-next` — rotate to next upstream proxy (if configured) **and** request NEWNYM
- `exit-set` — set exit countries + enable StrictNodes
- `exit-clear` — clear exit country preferences
- `apply-policy` — apply proxy/exit policy from your profile

Examples:

```powershell
.\target\release\torvpn_win.exe --profile .\profiles\default_win.toml newnym

.\target\release\torvpn_win.exe --profile .\profiles\default_win.toml exit-set us,de

.\target\release\torvpn_win.exe --profile .\profiles\default_win.toml apply-policy

.\target\release\torvpn_win.exe --profile .\profiles\default_win.toml proxy-next
```

Important notes:

- NEWNYM is rate-limited by Tor. If you spam it, Tor will ignore you for a bit.
- “Exit-set” only affects **new** circuits; existing connections may stay on old circuits.

## Profiles and paths

This project reads an optional profile TOML:

```text
profiles\default_win.toml
```

In that profile, you can point to Tor and tun2socks binaries using Windows-style absolute paths. Example (already in the default profile):

```toml
[tor]
tor_path_hint = "C:/tools/tor-expert/tor/tor.exe"

[tun2socks]
binary = "C:/tools/tun2socks/tun2socks-windows-amd64-v3.exe"
```

### DNSPort on Windows

By default the config uses `dns_port = 5353`. Windows often has mDNS/Bonjour on that port, and Tor may fail with `WSAEACCES`.

If Tor complains about DNSPort bind failures, set:

```toml
[tor]
dns_port = 0
```

When `dns_port = 0`:

- Tor will not open a DNSPort
- torvpn will skip NRPT DNS locking

## State directory

Wrappers set:

- `TORVPN_STATE_DIR = <repo>\state`

The state dir stores:

- Tor DataDirectory (`state\tor-data\...`)
- `state\pids.json` (Tor + tun2socks PIDs)

You can also set it manually:

```powershell
$env:TORVPN_STATE_DIR = "D:\some\other\folder\torvpn-state"
```

## Logs

- `logs\start.log`
- `logs\stop.log`

If a window disappears too fast, check those logs.

---

### FAQ

**Why does `health` fail with connection refused (9051)?**

Tor isn’t listening on 9051. Start Tor first (`start.cmd`), then re-run `health`.

**I started Tor manually and got cookie auth failures.**

Your torvpn binary reads the cookie from either:

- `TORVPN_COOKIE_PATH` (if set)
- or `TORVPN_STATE_DIR\tor-data\control_auth_cookie`

If you start Tor outside of torvpn, make sure torvpn points at the correct cookie.
