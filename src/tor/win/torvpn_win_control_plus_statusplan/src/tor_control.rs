use anyhow::{Result, bail, Context};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;
use hex::encode as hex_encode;

pub struct TorControl {
    stream: TcpStream,
}

impl TorControl {
    pub async fn connect(control_addr: &str, cookie_path: &Path) -> Result<Self> {
        let stream = TcpStream::connect(control_addr).await
            .with_context(|| format!("connect {}", control_addr))?;
        let mut ctl = TorControl { stream };
        ctl.authenticate(cookie_path).await?;
        Ok(ctl)
    }

    async fn authenticate(&mut self, cookie_path: &Path) -> Result<()> {
        let cookie = tokio::fs::read(cookie_path).await
            .with_context(|| format!("reading control cookie at {}", cookie_path.display()))?;
        let cookie_hex = hex_encode(cookie);
        self.send_cmd(format!("AUTHENTICATE {}\r\n", cookie_hex).as_str()).await?;
        Ok(())
    }

    pub async fn signal_newnym(&mut self) -> Result<()> {
        self.send_cmd("SIGNAL NEWNYM\r\n").await
    }

    pub async fn get_info(&mut self, key: &str) -> Result<String> {
        self.query_value(format!("GETINFO {}\r\n", key).as_str()).await
    }

    pub async fn set_conf(&mut self, key: &str, value: &str) -> Result<()> {
        self.send_cmd(format!("SETCONF {}={}\r\n", key, value).as_str()).await
    }

    pub async fn circuits(&mut self) -> Result<String> {
        self.query_value("GETINFO circuit-status\r\n").await
    }

    pub async fn health_summary(&mut self) -> Result<String> {
        let keys = ["status/bootstrap-phase", "net/listeners/socks", "net/listeners/dns", "status/circuit-established", "traffic/read", "traffic/written"];
        let mut out = String::new();
        for k in keys {
            if let Ok(v) = self.get_info(k).await {
                out.push_str(&format!("{}={}\n", k, v));
            }
        }
        Ok(out)
    }

    async fn send_cmd(&mut self, cmd: &str) -> Result<()> {
        self.stream.write_all(cmd.as_bytes()).await?;
        let mut reader = BufReader::new(&mut self.stream);
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 { bail!("control: EOF"); }
            if line.starts_with("250 ") || line == "250 OK\r\n" {
                break;
            }
            if line.starts_with("5") || line.starts_with("4") {
                bail!("control error: {}", line.trim());
            }
            if line.trim() == "250 OK" { break; }
        }
        Ok(())
    }

    async fn query_value(&mut self, cmd: &str) -> Result<String> {
        self.stream.write_all(cmd.as_bytes()).await?;
        let mut reader = BufReader::new(&mut self.stream);
        let mut buf = String::new();
        loop {
            let mut line = String::new();
            let n = reader.read_line(&mut line).await?;
            if n == 0 { bail!("control: EOF"); }
            if line.starts_with("250-") {
                buf.push_str(&line["250-".len()..]);
            } else if line.starts_with("250 ") {
                let rest = &line["250 ".len()..];
                if !rest.trim().is_empty() {
                    buf.push_str(rest);
                }
                break;
            } else if line.starts_with("5") || line.starts_with("4") {
                bail!("control error: {}", line.trim());
            }
        }
        Ok(buf.trim().to_string())
    }
}

/// Best-effort discovery of the Tor ControlPort cookie file.
///
/// Order:
/// 1) TORVPN_COOKIE_PATH
/// 2) TORVPN_TORRC_PATH (parse CookieAuthFile or DataDirectory)
/// 3) state_dir/common locations
pub async fn discover_control_cookie(state_dir: &Path) -> Result<PathBuf> {
    if let Ok(p) = std::env::var("TORVPN_COOKIE_PATH") {
        let pb = PathBuf::from(p);
        if tokio::fs::metadata(&pb).await.is_ok() {
            return Ok(pb);
        }
    }

    // If the user is launching Tor externally, let them point us at the torrc.
    if let Ok(torrc) = std::env::var("TORVPN_TORRC_PATH") {
        let torrc = PathBuf::from(torrc);
        if tokio::fs::metadata(&torrc).await.is_ok() {
            if let Some(p) = cookie_from_torrc(&torrc).await? {
                return Ok(p);
            }
        }
    }

    let candidates = [
        state_dir.join("tor-data").join("control_auth_cookie"),
        state_dir.join("control_auth_cookie"),
        state_dir.join("data").join("control_auth_cookie"),
    ];
    for c in &candidates {
        if tokio::fs::metadata(c).await.is_ok() {
            return Ok(c.clone());
        }
    }

    // Last resort: return the most likely default, but with a helpful error.
    Err(anyhow::anyhow!(
        "could not locate Tor control_auth_cookie; tried TORVPN_COOKIE_PATH, TORVPN_TORRC_PATH, and common paths under {}",
        state_dir.display()
    ))
}

async fn cookie_from_torrc(torrc_path: &Path) -> Result<Option<PathBuf>> {
    let bytes = tokio::fs::read(torrc_path).await
        .with_context(|| format!("reading torrc at {}", torrc_path.display()))?;
    let text = String::from_utf8_lossy(&bytes);

    let mut data_dir: Option<PathBuf> = None;
    let mut cookie_file: Option<PathBuf> = None;

    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() { continue; }
        let mut it = line.split_whitespace();
        let key = it.next().unwrap_or("");
        let rest = it.collect::<Vec<_>>().join(" ");
        if key.eq_ignore_ascii_case("DataDirectory") {
            if !rest.is_empty() { data_dir = Some(PathBuf::from(rest.clone())); }
        }
        if key.eq_ignore_ascii_case("CookieAuthFile") {
            if !rest.is_empty() { cookie_file = Some(PathBuf::from(rest)); }
        }
    }

    if let Some(p) = cookie_file {
        return Ok(Some(resolve_relative(torrc_path, &p)));
    }
    if let Some(dd) = data_dir {
        let dd = resolve_relative(torrc_path, &dd);
        return Ok(Some(dd.join("control_auth_cookie")));
    }
    Ok(None)
}

fn resolve_relative(torrc_path: &Path, p: &Path) -> PathBuf {
    if p.is_absolute() {
        return p.to_path_buf();
    }
    torrc_path.parent().unwrap_or_else(|| Path::new(".")).join(p)
}
