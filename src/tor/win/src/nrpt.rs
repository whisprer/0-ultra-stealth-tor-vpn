use anyhow::Result;
use tokio::process::Command;
use crate::config::Config;

pub async fn apply_dns_lock(cfg: &Config) -> Result<()> {
    // If DNSPort is disabled, skip NRPT/firewall DNS enforcement.
    // (Windows commonly has 5353 in use, and Tor's DNSPort isn't required
    // when users only proxy apps via SOCKS.)
    if cfg.tor.dns_port == 0 {
        return Ok(());
    }
    let script = include_str!("../scripts/dns-apply.ps1");
    let args_json = serde_json::json!({
        "AdapterHint": cfg.tun.interface,
        "DnsLoopback": "127.0.0.1",
        "TorDnsPort": cfg.tor.dns_port
    }).to_string();

    // FIX: Embed JSON as a variable assignment, then run the script
    // PowerShell -Command treats multiple args as separate statements, not params
    let full_command = format!(
        "$jsonArgs = '{}'; {}",
        args_json.replace("'", "''"),  // Escape single quotes for PS
        script
    );

    let status = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy").arg("Bypass")
        .arg("-Command")
        .arg(&full_command)
        .status().await?;
    
    if !status.success() {
        eprintln!("[nrpt] apply_dns_lock exited with: {:?}", status.code());
    }
    Ok(())
}

pub async fn teardown_dns_lock() -> Result<()> {
    let script = include_str!("../scripts/dns-teardown.ps1");
    let status = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy").arg("Bypass")
        .arg("-Command")
        .arg(script)
        .status().await?;
    
    if !status.success() {
        eprintln!("[nrpt] teardown_dns_lock exited with: {:?}", status.code());
    }
    Ok(())
}
