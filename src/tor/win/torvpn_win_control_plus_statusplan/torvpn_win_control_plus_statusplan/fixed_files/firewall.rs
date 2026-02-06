use anyhow::Result;
use tokio::process::Command;
use crate::config::Config;

pub async fn apply_rules(cfg: &Config) -> Result<()> {
    // Apply strict outbound block, allowing only Tor process and Wintun adapter traffic.
    let script = include_str!("../scripts/fw-apply.ps1");
    let args_json = serde_json::json!({
        "AdapterHint": cfg.tun.interface,
        "TorPath": cfg.tor.tor_path_hint.as_deref().unwrap_or("")
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
        eprintln!("[firewall] apply_rules exited with: {:?}", status.code());
    }
    Ok(())
}

pub async fn teardown_rules() -> Result<()> {
    let script = include_str!("../scripts/fw-teardown.ps1");
    let status = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy").arg("Bypass")
        .arg("-Command")
        .arg(script)
        .status().await?;
    
    if !status.success() {
        eprintln!("[firewall] teardown_rules exited with: {:?}", status.code());
    }
    Ok(())
}
