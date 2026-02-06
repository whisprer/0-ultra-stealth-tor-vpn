use anyhow::{Result, bail, Context};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use std::path::Path;
use hex::encode as hex_encode;

pub struct TorControl {
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
}

impl TorControl {
    pub async fn connect(control_addr: &str, cookie_path: &Path) -> Result<Self> {
        let stream = TcpStream::connect(control_addr).await
            .with_context(|| format!("connect {}", control_addr))?;
        
        // FIX: Use into_split() instead of try_clone() - Tokio async streams can't be cloned
        let (read_half, write_half) = stream.into_split();
        let reader = BufReader::new(read_half);
        let writer = BufWriter::new(write_half);
        
        let mut ctl = TorControl { reader, writer };
        ctl.authenticate(cookie_path).await?;
        Ok(ctl)
    }

    async fn authenticate(&mut self, cookie_path: &Path) -> Result<()> {
        let cookie = tokio::fs::read(cookie_path).await?;
        let cookie_hex = hex_encode(cookie);
        self.send_cmd(&format!("AUTHENTICATE {}\r\n", cookie_hex)).await?;
        Ok(())
    }

    pub async fn signal_newnym(&mut self) -> Result<()> {
        self.send_cmd("SIGNAL NEWNYM\r\n").await
    }

    pub async fn get_info(&mut self, key: &str) -> Result<String> {
        self.query_value(&format!("GETINFO {}\r\n", key)).await
    }

    pub async fn set_conf(&mut self, key: &str, value: &str) -> Result<()> {
        self.send_cmd(&format!("SETCONF {}={}\r\n", key, value)).await
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
        self.writer.write_all(cmd.as_bytes()).await?;
        self.writer.flush().await?;
        
        let mut line = String::new();
        loop {
            line.clear();
            let n = self.reader.read_line(&mut line).await?;
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
        self.writer.write_all(cmd.as_bytes()).await?;
        self.writer.flush().await?;
        
        let mut buf = String::new();
        loop {
            let mut line = String::new();
            let n = self.reader.read_line(&mut line).await?;
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
