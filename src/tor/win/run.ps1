# Start Tor only when you run VPN
Start-Process `
  -FilePath "C:\tools\tor-expert\tor\tor.exe" `
  -WorkingDirectory "C:\tools\tor-expert\tor" `
  -ArgumentList @('-f','C:\tools\torvpn-state\torrc') `
  -WindowStyle Hidden

Start-Sleep -Seconds 1

& "D:\code\0-ultra-stealth-tor-vpn\src\tor\win\torvpn_win_control_plus_statusplan\target\release\torvpn_win.exe" `
  --profile "D:\code\0-ultra-stealth-tor-vpn\src\tor\win\torvpn_win_control_plus_statusplan\profiles\default_win.toml" `
  health
