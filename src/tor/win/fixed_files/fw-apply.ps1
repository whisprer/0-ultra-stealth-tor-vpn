# fw-apply.ps1 - Apply firewall kill-switch for TorVPN
# $jsonArgs should be set before this script runs (as JSON string)

if ($jsonArgs -and ($jsonArgs -is [string])) { 
    $argsObj = $jsonArgs | ConvertFrom-Json 
} else { 
    $argsObj = $null 
}

$AdapterHint = if ($argsObj) { $argsObj.AdapterHint } else { "torvpn" }
$TorPath     = if ($argsObj) { $argsObj.TorPath } else { "" }

# Create a policy group
$group = "TORVPN"

# Clean old rules from this group
Get-NetFirewallRule -Group $group -ErrorAction SilentlyContinue | Remove-NetFirewallRule -ErrorAction SilentlyContinue | Out-Null

# Block all outbound by default (for this group)
New-NetFirewallRule -DisplayName "TORVPN Block All" -Group $group -Direction Outbound -Action Block -Enabled True | Out-Null

# Allow Tor process
$torFound = $false
if (-not [string]::IsNullOrEmpty($TorPath) -and (Test-Path $TorPath)) {
    New-NetFirewallRule -DisplayName "TORVPN Allow Tor" -Group $group -Program $TorPath -Direction Outbound -Action Allow -Enabled True | Out-Null
    $torFound = $true
    Write-Host "[+] Allowed Tor at: $TorPath"
} else {
    # Try to find tor.exe in PATH
    $tor = Get-Command tor.exe -ErrorAction SilentlyContinue
    if ($tor) {
        New-NetFirewallRule -DisplayName "TORVPN Allow Tor" -Group $group -Program $tor.Path -Direction Outbound -Action Allow -Enabled True | Out-Null
        $torFound = $true
        Write-Host "[+] Allowed Tor at: $($tor.Path)"
    } else {
        Write-Host "[!] WARNING: tor.exe not found - Tor traffic may be blocked!"
    }
}

# Allow traffic through the Wintun adapter
$adapter = Get-NetAdapter | Where-Object { 
    $_.InterfaceDescription -like "*Wintun*" -or $_.Name -like "*$AdapterHint*" 
} | Select-Object -First 1

if ($adapter) {
    New-NetFirewallRule -DisplayName "TORVPN Allow Wintun" -Group $group -InterfaceAlias $adapter.Name -Direction Outbound -Action Allow -Enabled True | Out-Null
    Write-Host "[+] Allowed adapter: $($adapter.Name)"
} else {
    Write-Host "[!] WARNING: Wintun adapter not found yet (may appear after tun2socks starts)"
}

# Also allow localhost traffic (for Tor control port, DNS, etc.)
New-NetFirewallRule -DisplayName "TORVPN Allow Localhost" -Group $group -Direction Outbound -Action Allow -RemoteAddress 127.0.0.1 -Enabled True | Out-Null

Write-Host "[+] Firewall rules applied for group: $group"
