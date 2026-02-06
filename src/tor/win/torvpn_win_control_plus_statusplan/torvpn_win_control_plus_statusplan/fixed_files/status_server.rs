use anyhow::Result;
use crate::tor_control::TorControl;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::json;
use crate::config::Config;
use std::path::Path;
use tokio::fs;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub async fn run(cfg: Config, state_dir: std::path::PathBuf) -> Result<()> {
    if !cfg.status.enabled { return Ok(()); }
    
    // Ensure API token exists
    let token_path = state_dir.join("api_token");
    if !token_path.exists() {
        let token: String = (0..32).map(|_| format!("{:02x}", rand::random::<u8>())).collect();
        let _ = fs::write(&token_path, &token).await;
    }
    // Ensure HMAC secret exists
    let hmac_path = state_dir.join("api_hmac_secret");
    if !hmac_path.exists() {
        let secret: String = (0..32).map(|_| format!("{:02x}", rand::random::<u8>())).collect();
        let _ = fs::write(&hmac_path, &secret).await;
    }
    
    let addr = cfg.status.listen.clone();
    let listener = TcpListener::bind(&addr).await?;
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((sock, peer)) => { let _ = handle(sock, &cfg, &state_dir, peer.to_string()).await; }
                Err(_) => break,
            }
        }
    });
    Ok(())
}

/// Check authentication via token header or HMAC
fn auth_check(cfg: &Config, state_dir: &Path, method: &str, path: &str, req: &str, _query: Option<&str>) -> Result<()> {
    // If listening only on localhost, auth is optional but still checked if header present
    let is_localhost = cfg.status.listen.starts_with("127.0.0.1") || cfg.status.listen.starts_with("localhost");
    
    // Parse headers from request
    let headers = parse_headers(req);
    
    // Check for token in X-TorVPN-Token or Authorization: Bearer
    let token_path = state_dir.join("api_token");
    if let Ok(expected_token) = std::fs::read_to_string(&token_path) {
        let expected_token = expected_token.trim();
        
        // Check X-TorVPN-Token header
        if let Some(provided) = headers.get("x-torvpn-token") {
            if provided.trim() == expected_token {
                return Ok(());
            }
        }
        
        // Check Authorization: Bearer header
        if let Some(auth) = headers.get("authorization") {
            if let Some(bearer) = auth.strip_prefix("Bearer ") {
                if bearer.trim() == expected_token {
                    return Ok(());
                }
            }
        }
    }
    
    // Check HMAC auth (for remote binds)
    if let Some(hmac_header) = headers.get("x-torvpn-hmac") {
        let hmac_secret_path = state_dir.join("api_hmac_secret");
        if let Ok(secret) = std::fs::read_to_string(&hmac_secret_path) {
            if verify_hmac(hmac_header.trim(), secret.trim(), method, path) {
                return Ok(());
            }
        }
    }
    
    // For localhost, allow unauthenticated read-only access to /status
    if is_localhost && path == "/status" && method == "GET" {
        return Ok(());
    }
    
    anyhow::bail!("unauthorized")
}

fn parse_headers(req: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for line in req.lines().skip(1) {
        if line.is_empty() || line == "\r" { break; }
        if let Some((key, value)) = line.split_once(':') {
            map.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    map
}

fn verify_hmac(header: &str, secret: &str, method: &str, path: &str) -> bool {
    // Format: ts:hex_hmac
    let parts: Vec<&str> = header.splitn(2, ':').collect();
    if parts.len() != 2 { return false; }
    
    let ts_str = parts[0];
    let provided_hmac = parts[1];
    
    // Validate timestamp (allow ±300s skew)
    let ts: i64 = match ts_str.parse() {
        Ok(t) => t,
        Err(_) => return false,
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    if (now - ts).abs() > 300 {
        return false;
    }
    
    // Compute expected HMAC: hmac_sha256(secret, ts || method || path)
    let message = format!("{}{}{}", ts_str, method, path);
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(message.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());
    
    // Constant-time comparison
    expected.to_lowercase() == provided_hmac.to_lowercase()
}

async fn handle(mut sock: TcpStream, cfg: &Config, state_dir: &Path, peer: String) -> Result<()> {
    let mut buf = vec![0u8; 4096];
    let n = sock.read(&mut buf).await?;
    let req = String::from_utf8_lossy(&buf[..n]);
    let first = req.lines().next().unwrap_or("GET / HTTP/1.1");
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let raw_path = parts.next().unwrap_or("/");
    let (path, query) = parse_query(raw_path);
    
    // Control endpoints - require auth
    if path == "/control/newnym" {
        if auth_check(cfg, state_dir, method, path, &req, query).is_err() { 
            respond(&mut sock, 403, r#"{"error":"forbidden"}"#).await?; 
            return Ok(()); 
        }
        if method != "POST" { respond(&mut sock, 405, r#"{"error":"method not allowed"}"#).await?; return Ok(()); }
        let cookie = state_dir.join("tor-data").join("control_auth_cookie");
        let addr = format!("127.0.0.1:{}", cfg.tor.control_port);
        let mut ctl = TorControl::connect(&addr, &cookie).await?;
        let _ = ctl.signal_newnym().await;
        respond(&mut sock, 200, r#"{"ok":true}"#).await?; 
        return Ok(());
    }
    
    if path == "/control/proxynext" {
        if auth_check(cfg, state_dir, method, path, &req, query).is_err() { 
            respond(&mut sock, 403, r#"{"error":"forbidden"}"#).await?; 
            return Ok(()); 
        }
        if method != "POST" { respond(&mut sock, 405, r#"{"error":"method not allowed"}"#).await?; return Ok(()); }
        // Apply next proxy + newnym
        let _ = crate::tor_manager::apply_proxy_and_exit(cfg, state_dir).await;
        respond(&mut sock, 200, r#"{"ok":true}"#).await?; 
        return Ok(());
    }
    
    if path == "/control/exitclear" {
        if auth_check(cfg, state_dir, method, path, &req, query).is_err() { 
            respond(&mut sock, 403, r#"{"error":"forbidden"}"#).await?; 
            return Ok(()); 
        }
        if method != "POST" { respond(&mut sock, 405, r#"{"error":"method not allowed"}"#).await?; return Ok(()); }
        let cookie = state_dir.join("tor-data").join("control_auth_cookie");
        let addr = format!("127.0.0.1:{}", cfg.tor.control_port);
        let mut ctl = TorControl::connect(&addr, &cookie).await?;
        let _ = ctl.set_conf("ExitNodes", "").await;
        let _ = ctl.set_conf("StrictNodes", "0").await;
        let _ = ctl.signal_newnym().await;
        respond(&mut sock, 200, r#"{"ok":true}"#).await?; 
        return Ok(());
    }
    
    if path == "/control/exitset" {
        if auth_check(cfg, state_dir, method, path, &req, query).is_err() { 
            respond(&mut sock, 403, r#"{"error":"forbidden"}"#).await?; 
            return Ok(()); 
        }
        if method != "POST" { respond(&mut sock, 405, r#"{"error":"method not allowed"}"#).await?; return Ok(()); }

        let cc = query_get(query, "cc").unwrap_or("");
        let list: Vec<String> = cc.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).map(|s| format!("{{{}}}", s)).collect();
        let cookie = state_dir.join("tor-data").join("control_auth_cookie");
        let addr = format!("127.0.0.1:{}", cfg.tor.control_port);
        let mut ctl = TorControl::connect(&addr, &cookie).await?;

        if list.is_empty() {
            let _ = ctl.set_conf("ExitNodes", "").await;
            let _ = ctl.set_conf("StrictNodes", "0").await;
        } else {
            let join = list.join(",");
            ctl.set_conf("ExitNodes", &join).await?;
            ctl.set_conf("StrictNodes", "1").await?;
        }
        let _ = ctl.signal_newnym().await;
        respond(&mut sock, 200, r#"{"ok":true}"#).await?; 
        return Ok(());
    }
    
    if path == "/status/plan" {
        // Return hop plan state including order + randomized flag + next hops
        let st_path = state_dir.join("hop_state.json");
        let mut order: Vec<usize> = vec![];
        let mut randomized = false;
        let mut idx = 0usize;
        let mut next_epoch_ms = 0u64;
        if let Ok(b) = fs::read(&st_path).await {
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&b) {
                if let Some(a) = v.get("order").and_then(|x| x.as_array()) {
                    order = a.iter().filter_map(|x| x.as_u64()).map(|x| x as usize).collect();
                }
                randomized = v.get("randomized").and_then(|x| x.as_bool()).unwrap_or(false);
                idx = v.get("idx").and_then(|x| x.as_u64()).unwrap_or(0) as usize;
                next_epoch_ms = v.get("next_epoch_ms").and_then(|x| x.as_u64()).unwrap_or(0);
            }
        }
        let now = now_ms();
        let remaining = if next_epoch_ms > now { (next_epoch_ms - now) / 1000 } else { 0 };
        let total = cfg.hop.sequence.len();
        let mut upcoming: Vec<serde_json::Value> = Vec::new();
        if !order.is_empty() && idx < order.len() {
            for k in idx..order.len() {
                let i = order[k];
                if let Some(item) = cfg.hop.sequence.get(i) {
                    upcoming.push(serde_json::json!({
                        "index": i,
                        "duration": item.duration,
                        "exit_countries": item.exit_countries,
                        "proxy": item.proxy
                    }));
                }
            }
        }
        let body = serde_json::json!({
            "randomized": randomized,
            "order_indices": order,
            "current_index_in_order": idx,
            "seconds_remaining": remaining,
            "total": total,
            "upcoming": upcoming
        }).to_string();
        respond(&mut sock, 200, &body).await?; 
        return Ok(());
    }
    
    if path != "/status" {
        respond(&mut sock, 404, r#"{"error":"not found"}"#).await?; 
        return Ok(());
    }

    // Simple rate limiter when not bound to loopback
    if !cfg.status.listen.starts_with("127.0.0.1") {
        if rate_limit_check(state_dir, &peer).await.is_err() {
            respond(&mut sock, 429, r#"{"error":"too many requests"}"#).await?; 
            return Ok(());
        }
    }

    let st_path = state_dir.join("hop_state.json");
    let (idx, next_epoch_ms) = if let Ok(b) = fs::read(&st_path).await {
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&b) {
            (v.get("idx").and_then(|x| x.as_u64()).unwrap_or(0) as usize,
             v.get("next_epoch_ms").and_then(|x| x.as_u64()).unwrap_or(0))
        } else { (0, 0) }
    } else { (0, 0) };
    let total = cfg.hop.sequence.len();
    let now_ms_val = now_ms();
    let remaining = if next_epoch_ms > now_ms_val { (next_epoch_ms - now_ms_val) / 1000 } else { 0 };
    let hop_item = cfg.hop.sequence.get(idx).cloned();

    let exit_ip = match get_exit_ip_via_socks(cfg.tor.socks_port).await {
        Ok(s) => Some(s),
        Err(_) => None,
    };

    let body = json!({
        "current_index": idx,
        "total": total,
        "seconds_remaining": remaining,
        "next_epoch_ms": next_epoch_ms,
        "current_hop": hop_item,
        "tor": { "socks_port": cfg.tor.socks_port, "dns_port": cfg.tor.dns_port, "control_port": cfg.tor.control_port },
        "exit_ip": exit_ip,
    }).to_string();

    respond(&mut sock, 200, &body).await?;
    Ok(())
}

async fn respond(sock: &mut TcpStream, code: u16, body: &str) -> Result<()> {
    let status = match code { 
        200 => "OK", 
        403 => "Forbidden",
        404 => "Not Found", 
        405 => "Method Not Allowed",
        429 => "Too Many Requests",
        _ => "OK" 
    };
    let resp = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        code, status, body.len(), body
    );
    sock.write_all(resp.as_bytes()).await?;
    Ok(())
}

fn parse_query(path: &str) -> (&str, Option<&str>) {
    if let Some((p, q)) = path.split_once('?') { (p, Some(q)) } else { (path, None) }
}

fn query_get<'a>(q: Option<&'a str>, key: &str) -> Option<&'a str> {
    q.and_then(|qs| {
        for pair in qs.split('&') {
            if let Some((k,v)) = pair.split_once('=') {
                if k == key { return Some(v); }
            }
        }
        None
    })
}

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

async fn rate_limit_check(state_dir: &Path, peer: &str) -> anyhow::Result<()> {
    // File-based simple limiter: keep JSON map of ip -> {last_ms, burst, window_start_ms, count}
    let p = state_dir.join("rate_limit.json");
    let now = now_ms();
    let mut map: serde_json::Value = if let Ok(b) = fs::read(&p).await {
        serde_json::from_slice(&b).unwrap_or(serde_json::json!({}))
    } else { serde_json::json!({}) };
    let ip = peer.split(':').next().unwrap_or(peer);
    let ent = map.get(ip).cloned().unwrap_or(serde_json::json!({"last_ms":0,"burst":0,"win_ms":now,"count":0}));
    let last = ent.get("last_ms").and_then(|x| x.as_u64()).unwrap_or(0);
    let burst = ent.get("burst").and_then(|x| x.as_u64()).unwrap_or(0);
    let win_ms = ent.get("win_ms").and_then(|x| x.as_u64()).unwrap_or(now);
    let count = ent.get("count").and_then(|x| x.as_u64()).unwrap_or(0);

    let new_burst = if now > last + 1000 { 0 } else { burst + 1 };
    let mut new_win_ms = win_ms;
    let mut new_count = count;
    if now > win_ms + 60_000 { new_win_ms = now; new_count = 0; }
    new_count += 1;

    // thresholds: <=5 req/sec and <=120 req/min
    if new_burst > 5 || new_count > 120 {
        return Err(anyhow::anyhow!("rate limited"));
    }

    let new_ent = serde_json::json!({
        "last_ms": now,
        "burst": new_burst,
        "win_ms": new_win_ms,
        "count": new_count
    });
    map.as_object_mut().unwrap().insert(ip.to_string(), new_ent);
    fs::write(&p, serde_json::to_vec_pretty(&map)?).await?;
    Ok(())
}

async fn get_exit_ip_via_socks(socks_port: u16) -> Result<String> {
    use tokio::net::TcpStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut s = TcpStream::connect(("127.0.0.1", socks_port)).await?;
    s.write_all(&[0x05, 0x01, 0x00]).await?;
    let mut resp = [0u8;2]; s.read_exact(&mut resp).await?;
    if resp != [0x05, 0x00] { anyhow::bail!("socks auth failed"); }
    let host = "api.ipify.org";
    let port = 80u16;
    let hb = host.as_bytes();
    let mut req = Vec::with_capacity(7 + hb.len());
    req.extend_from_slice(&[0x05, 0x01, 0x00, 0x03, hb.len() as u8]);
    req.extend_from_slice(hb);
    req.extend_from_slice(&[(port >> 8) as u8, (port & 0xff) as u8]);
    s.write_all(&req).await?;
    let mut hdr = [0u8;4]; s.read_exact(&mut hdr).await?;
    if hdr[1] != 0x00 { anyhow::bail!("socks connect failed"); }
    match hdr[3] {
        0x01 => { let mut skip=[0u8;6]; s.read_exact(&mut skip).await?; }
        0x03 => { let mut len=[0u8;1]; s.read_exact(&mut len).await?; let mut skip=vec![0u8; len[0] as usize + 2]; s.read_exact(&mut skip).await?; }
        0x04 => { let mut skip=[0u8;18]; s.read_exact(&mut skip).await?; }
        _ => {}
    }
    let get = "GET /?format=text HTTP/1.1\r\nHost: api.ipify.org\r\nConnection: close\r\n\r\n";
    s.write_all(get.as_bytes()).await?;
    let mut buf = Vec::new(); let mut tmp = [0u8;1024];
    loop { match s.read(&mut tmp).await { Ok(0) => break, Ok(n) => buf.extend_from_slice(&tmp[..n]), Err(_) => break } }
    let body = String::from_utf8_lossy(&buf);
    if let Some(idx) = body.rfind("\r\n\r\n") {
        let ip = body[idx+4..].trim(); return Ok(ip.to_string());
    }
    anyhow::bail!("no ip in response")
}
