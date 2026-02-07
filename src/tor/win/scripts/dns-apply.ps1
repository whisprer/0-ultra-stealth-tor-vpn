# dns-apply.ps1 - Apply DNS lock via NRPT + firewall for TorVPN
# $jsonArgs should be set before this script runs (as JSON string)

if ($jsonArgs -and ($jsonArgs -is [string])) { 
    $argsObj = $jsonArgs | ConvertFrom-Json 
} else { 
    $argsObj = $null 
}

$AdapterHint = if ($argsObj) { $argsObj.AdapterHint } else { "torvpn" }
$DnsLoopback = if ($argsObj) { $argsObj.DnsLoopback } else { "127.0.0.1" }
$TorDnsPort  = if ($argsObj) { $argsObj.TorDnsPort } else { 5353 }

$group = "TORVPN"

# NRPT: Route all namespaces to 127.0.0.1 (Tor's DNSPort)
# Remove old TORVPN rules
Get-DnsClientNrptRule -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -eq "TORVPN Default" } | Remove-DnsClientNrptRule -Force -ErrorAction SilentlyContinue | Out-Null

# Create default-catch NRPT rule for all domains (".")
Add-DnsClientNrptRule -Namespace "." -NameServers $DnsLoopback -DisplayName "TORVPN Default" -ErrorAction SilentlyContinue | Out-Null
Write-Host "[+] NRPT rule added: all DNS -> $DnsLoopback"

# Set Wintun adapter DNS server explicitly to loopback as belt-and-suspenders
$adapter = Get-NetAdapter | Where-Object { 
    $_.InterfaceDescription -like "*Wintun*" -or $_.Name -like "*$AdapterHint*" 
} | Select-Object -First 1

if ($adapter) {
    Set-DnsClientServerAddress -InterfaceAlias $adapter.Name -ServerAddresses $DnsLoopback -ErrorAction SilentlyContinue | Out-Null
    Write-Host "[+] Set DNS on adapter: $($adapter.Name)"
} else {
    Write-Host "[!] Wintun adapter not found yet (may appear after tun2socks starts)"
}

# Firewall DNS hardening:
# 1) Block outbound UDP/TCP 53 globally
# 2) Allow DNS to 127.0.0.1:53 (loopback) for local resolver

# Clean old DNS rules
Get-NetFirewallRule -Group $group -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -like "TORVPN DNS*" } | Remove-NetFirewallRule -ErrorAction SilentlyContinue | Out-Null

New-NetFirewallRule -DisplayName "TORVPN DNS Block UDP 53" -Group $group -Direction Outbound -Action Block -Enabled True -Protocol UDP -RemotePort 53 | Out-Null
New-NetFirewallRule -DisplayName "TORVPN DNS Block TCP 53" -Group $group -Direction Outbound -Action Block -Enabled True -Protocol TCP -RemotePort 53 | Out-Null

New-NetFirewallRule -DisplayName "TORVPN DNS Allow Loopback 53 UDP" -Group $group -Direction Outbound -Action Allow -Enabled True -Protocol UDP -RemoteAddress 127.0.0.1 -RemotePort 53 | Out-Null
New-NetFirewallRule -DisplayName "TORVPN DNS Allow Loopback 53 TCP" -Group $group -Direction Outbound -Action Allow -Enabled True -Protocol TCP -RemoteAddress 127.0.0.1 -RemotePort 53 | Out-Null

# Ensure Tor DNSPort specifically allowed (127.0.0.1:TorDnsPort)
New-NetFirewallRule -DisplayName "TORVPN DNS Allow Tor DNSPort UDP" -Group $group -Direction Outbound -Action Allow -Enabled True -Protocol UDP -RemoteAddress 127.0.0.1 -RemotePort $TorDnsPort | Out-Null
New-NetFirewallRule -DisplayName "TORVPN DNS Allow Tor DNSPort TCP" -Group $group -Direction Outbound -Action Allow -Enabled True -Protocol TCP -RemoteAddress 127.0.0.1 -RemotePort $TorDnsPort | Out-Null

Write-Host "[+] DNS lock applied (port 53 blocked except loopback, Tor DNS on :$TorDnsPort)"
