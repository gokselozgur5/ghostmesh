use ghostmesh_core::identity::{PeerId, PeerIdentity};
use rand_core::OsRng;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn config_dir() -> Result<PathBuf, String> {
    if let Ok(dir) = env::var("GHOSTMESH_CONFIG_DIR") {
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }
    let home = env::var("HOME").map_err(|_| "HOME environment variable is not set".to_string())?;
    Ok(PathBuf::from(home).join(".config").join("ghostmesh"))
}

pub fn init_identity(dir: &Path, force: bool) -> Result<(PeerId, bool), String> {
    fs::create_dir_all(dir).map_err(|e| format!("create config dir: {e}"))?;
    let key_path = dir.join("identity.key");
    if key_path.exists() && !force {
        let hex = fs::read_to_string(&key_path).map_err(|e| format!("read identity: {e}"))?;
        let seed = decode_seed_hex(&hex)?;
        return Ok((PeerIdentity::from_seed(&seed).peer_id(), false));
    }
    let mut rng = OsRng;
    let id = PeerIdentity::generate(&mut rng);
    let hex: String = id.seed_bytes().iter().map(|b| format!("{b:02x}")).collect();
    fs::write(&key_path, format!("{hex}\n")).map_err(|e| format!("write identity: {e}"))?;
    set_private_perms(&key_path)?;
    Ok((id.peer_id(), true))
}

pub fn trust_peer(dir: &Path, peer_hex: &str) -> Result<bool, String> {
    let peer_id = PeerId::from_hex(peer_hex).map_err(|e| format!("invalid peer_id: {e}"))?;
    fs::create_dir_all(dir).map_err(|e| format!("create config dir: {e}"))?;
    let path = dir.join("trusted_peers.json");
    let mut peers: BTreeSet<String> = if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| format!("read trust store: {e}"))?;
        serde_json::from_str(&raw).map_err(|e| format!("parse trust store: {e}"))?
    } else {
        BTreeSet::new()
    };
    let inserted = peers.insert(peer_id.to_hex());
    let json = serde_json::to_string_pretty(&peers).map_err(|e| format!("encode trust store: {e}"))?;
    fs::write(&path, format!("{json}\n")).map_err(|e| format!("write trust store: {e}"))?;
    Ok(inserted)
}

fn decode_seed_hex(hex: &str) -> Result<[u8; 32], String> {
    let s = hex.trim();
    if s.len() != 64 {
        return Err("invalid identity.key length".to_string());
    }
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
            .map_err(|e| format!("invalid identity.key hex: {e}"))?;
    }
    Ok(out)
}

#[cfg(unix)]
fn set_private_perms(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|e| format!("set permissions: {e}"))
}

#[cfg(not(unix))]
fn set_private_perms(_path: &Path) -> Result<(), String> {
    Ok(())
}
