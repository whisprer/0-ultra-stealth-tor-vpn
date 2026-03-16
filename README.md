<!-- repo-convergence:readme-header:start -->
<!-- repo-convergence:language=FILL_ME -->
# 0-ultra-stealth-tor-vpn

<p align="center">
  <a href="https://github.com/whisprer/0-ultra-stealth-tor-vpn/releases">
    <img src="https://img.shields.io/github/v/release/whisprer/0-ultra-stealth-tor-vpn?color=4CAF50&label=release" alt="Release Version">
  </a>
  <a href="https://github.com/whisprer/0-ultra-stealth-tor-vpn/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/license-Hybrid-green.svg" alt="License">
  </a>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg" alt="Platform">
  <a href="https://github.com/whisprer/0-ultra-stealth-tor-vpn/actions">
    <img src="https://img.shields.io/badge/build-workflow%20not%20set-lightgrey.svg" alt="Build Status">
  </a>
</p>

[![GitHub](https://img.shields.io/badge/GitHub-whisprer%2F0-ultra-stealth-tor-vpn-blue?logo=github&style=flat-square)](https://github.com/whisprer/0-ultra-stealth-tor-vpn)
![Commits](https://img.shields.io/github/commit-activity/m/whisprer/0-ultra-stealth-tor-vpn?label=commits)
![Last Commit](https://img.shields.io/github/last-commit/whisprer/0-ultra-stealth-tor-vpn)
![Issues](https://img.shields.io/github/issues/whisprer/0-ultra-stealth-tor-vpn)
[![Version](https://img.shields.io/badge/version-3.1.1-blue.svg)](https://github.com/whisprer/0-ultra-stealth-tor-vpn)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-lightgrey.svg)](https://www.microsoft.com/windows)
[![Language](https://img.shields.io/badge/language-FILL_ME-blue.svg)](#)
[![Status](https://img.shields.io/badge/Status-Alpha%20Release-orange?style=flat-square)](#)

<p align="center">
  <img src="/assets/0-ultra-stealth-tor-vpn-banner.png" width="850" alt="0-ultra-stealth-tor-vpn Banner">
</p>
<!-- repo-convergence:readme-header:end -->

<p align="center">
  <a href="https://github.com/whisprer/0-ultra-stealth-tor-vpn/releases"> 
    <img src="https://img.shields.io/github/v/release/whisprer/0-ultra-stealth-tor-vpn?color=4CAF50&label=release" alt="Release Version"> 
  </a>
  <a href="https://github.com/whisprer/0-ultra-stealth-tor-vpn/actions"> 
    <img src="https://img.shields.io/github/actions/workflow/status/whisprer/0-ultra-stealth-tor-vpn/lint-and-plot.yml?label=build" alt="Build Status"> 
  </a>
</p>

![Commits](https://img.shields.io/github/commit-activity/m/whisprer/0-ultra-stealth-tor-vpn?label=commits) 
![Last Commit](https://img.shields.io/github/last-commit/whisprer/0-ultra-stealth-tor-vpn) 
![Issues](https://img.shields.io/github/issues/whisprer/0-ultra-stealth-tor-vpn) 
[![Version](https://img.shields.io/badge/version-3.1.1-blue.svg)](https://github.com/whisprer/0-ultra-stealth-tor-vpn) 
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-lightgrey.svg)](https://www.microsoft.com/windows)
[![Python](https://img.shields.io/badge/python-3.8%2B-blue.svg)](https://www.python.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

<p align="center">
  <img src="0-ultra-stealth-tor-vpn-banner.png" width="850" alt="0-Ultra-Stealth-Tor-Vpn Banner">


TorVPN — Ultra-Stealth, Tor-Routed VPN (Linux & Windows)

TorVPN is a defensive, leak-resistant VPN daemon that transparently routes system traffic through the Tor network using either transparent redirect or TUN → SOCKS tunneling.
It is designed with strict privilege separation, fail-closed networking, and active leak prevention (IPv4 and IPv6) as first-class goals.

This is not a wrapper around torify.
It is a full system VPN architecture built around Tor’s control protocol, modern firewalling, and hardened service isolation.

✨ Key Features
Core Routing Modes

Transparent mode (nftables / WFP redirect)
Redirects TCP traffic at the kernel/firewall layer into Tor’s TransPort.

TUN → SOCKS mode (tun2socks)
Full virtual interface for TCP + UDP support via SOCKS5.

Tor Control & Circuit Management
Tor ControlPort integration:
- NEWNYM (circuit rotation)
- Exit country selection
- Circuit health checks

Scheduled hop plans:
- Timed country / proxy hops
- Optional randomization (Fisher–Yates shuffle)

On-demand control via:
- CLI
- Local HTTP control API (/control/newnym, /control/exitset, /control/proxynext)
- Leak Prevention (Fail-Closed)
- IPv4 + IPv6 hardened
Linux: nftables inet table (v4 + v6)
Windows: WFP + NRPT + explicit ICMPv6 / DNS blocks

DNS pinned to loopback:
127.0.0.1 and ::1
Default-deny firewall stance
MSS clamping to avoid PMTU leaks
Privilege Separation & Sandboxing

Linux
systemd service with:
DynamicUser
Minimal capabilities only (CAP_NET_ADMIN, CAP_NET_BIND_SERVICE)
Strong sandboxing (ProtectSystem=strict, PrivateDevices, etc.)
Optional setcap for non-root manual runs

Windows
Service runs as LocalSystem only for:
Firewall (WFP)
NRPT
Adapter management
Tor + tun2socks run under a restricted non-admin local user
Spawned via CreateProcessAsUser
Credentials stored DPAPI-encrypted (machine-bound)

Proxy & Hop Chaining
Optional pre-Tor proxy chaining
SOCKS5 / HTTPS proxies
Rotate manually or via hop plan
Supports “around-the-world” routing patterns without touching Tor exits directly

Status & Observability
Local JSON status endpoint:

Current hop
Time remaining
Exit IP

Leak test commands (v4 + v6)
Structured logs suitable for journald / Windows Event Log

🚫 Explicit Non-Goals
No post-Tor “exit proxy” chaining (breaks Tor anonymity model)
No browser fingerprinting mitigation (out of scope)
No UDP tunneling in transparent mode (Tor limitation)

🖥 Supported Platforms
Platform	Status
Linux (systemd)	✅ Fully supported
Windows 10 / 11	✅ Fully supported
macOS	❌ Not currently supported

📦 Architecture Overview
[ Applications ]
        	     |
        	     v
[ Firewall / Redirect ]
        	     |
   ┌────┴─────┐
   |          			  |
[ TransPort ] [ TUN ]
   |          			  |
   └────┬─────┘
        	     v
     	      [Tor ]
        	     |
   [ Tor Network ]


Control plane:
CLI + HTTP control API
Tor ControlPort
Hop planner & scheduler

Data plane:
nftables / WFP
tun2socks
Tor SOCKS / TransPort

🔐 Security Model (High Level)
Default-deny networking
Least privilege everywhere
Tor is the only process allowed external egress
IPv6 treated as hostile by default
Control API bound to localhost, token-protected
Optional HMAC auth for remote control

🧪 Leak Testing
Built-in checks:
DNS resolution sanity
Exit IP verification via Tor SOCKS
IPv6 AAAA resolution checks
Firewall enforcement validation

🛠 Installation (High Level)
Linux
Build binary
Apply systemd service
Start daemon

Windows
Build service binary
Install service
Create restricted runtime user
Start service

(See platform-specific docs in /docs or /patch bundles.)

⚠️ Legal & Ethical Notice
TorVPN is intended for:
Privacy protection
Traffic analysis resistance
Research, education, and legitimate security use

You are responsible for complying with local laws and network policies.

🧭 Roadmap Ideas
Prometheus /metrics endpoint
Profile signing & verification
Split-tunnel support
GUI status tray (optional)
macOS support (future, maybe)

🧠 Philosophy
This project treats leaks as failures, not edge cases.
If traffic can escape, the system is considered broken.