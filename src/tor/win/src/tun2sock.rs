use anyhow::{anyhow, Context};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct Tun2SocksCfg {
    pub enabled: bool,
    pub binary: PathBuf,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Tun2SocksRuntime {
    pub pid_file: PathBuf,
    pub stdout_log: PathBuf,
    pub stderr_log: PathBuf,
}

fn write_pid(pid_file: &Path, pid: u32) -> anyhow::Result<()> {
    if let Some(p) = pid_file.parent() {
        fs::create_dir_all(p).ok();
    }
    fs::write(pid_file, pid.to_string()).context("write tun2socks pid")?;
    Ok(())
}

fn read_pid(pid_file: &Path) -> Option<u32> {
    fs::read_to_string(pid_file).ok()?.trim().parse::<u32>().ok()
}

fn replace_placeholders(arg: &str) -> String {
    // Keep replacements minimal + deterministic
    // Extend later as needed.
    arg.replace("{socks}", "socks5://127.0.0.1:9050")
}

pub fn spawn_tun2socks(cfg: &Tun2SocksCfg, rt: &Tun2SocksRuntime) -> anyhow::Result<u32> {
    if !cfg.enabled {
        return Err(anyhow!("tun2socks disabled"));
    }
    if !cfg.binary.exists() {
        return Err(anyhow!("tun2socks binary not found: {}", cfg.binary.display()));
    }

    if let Some(p) = rt.stdout_log.parent() {
        fs::create_dir_all(p).ok();
    }

    let out = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&rt.stdout_log)
        .context("open tun2socks stdout log")?;

    let err = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&rt.stderr_log)
        .context("open tun2socks stderr log")?;

    let args: Vec<String> = cfg.args.iter().map(|a| replace_placeholders(a)).collect();

    let mut child: Child = Command::new(&cfg.binary)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(out))
        .stderr(Stdio::from(err))
        .spawn()
        .with_context(|| format!("spawn tun2socks: {}", cfg.binary.display()))?;

    let pid = child.id();
    if pid == 0 {
        return Err(anyhow!("tun2socks returned pid=0"));
    }

    // Detach: drop child handle; PID will remain for Stop.
    std::mem::drop(child);

    write_pid(&rt.pid_file, pid)?;
    Ok(pid)
}

pub async fn wait_tun2socks_stable(pid: u32) -> anyhow::Result<()> {
    // Give it a moment to initialize and crash early if args are wrong.
    sleep(Duration::from_millis(800)).await;

    // Check it still exists (Windows-friendly: tasklist)
    let out = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid)])
        .output()
        .context("tasklist")?;

    let s = String::from_utf8_lossy(&out.stdout);
    if !s.contains(&pid.to_string()) {
        return Err(anyhow!("tun2socks exited early (pid {})", pid));
    }
    Ok(())
}

pub fn stop_tun2socks(rt: &Tun2SocksRuntime) -> anyhow::Result<()> {
    let Some(pid) = read_pid(&rt.pid_file) else {
        return Ok(());
    };

    // taskkill /T /F to ensure child process tree ends
    let status = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .status()
        .context("taskkill tun2socks")?;

    // Remove pid file regardless; if taskkill failed, stop will report error.
    let _ = fs::remove_file(&rt.pid_file);

    if !status.success() {
        return Err(anyhow!("failed to taskkill tun2socks pid {}", pid));
    }
    Ok(())
}
