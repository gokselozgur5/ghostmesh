use ghostmesh_net::discovery::{discover_peers, DEFAULT_DISCOVERY_ADDR};
use std::time::Duration;

pub fn run_discover(timeout_ms: u64, json: bool) {
    let timeout = Duration::from_millis(timeout_ms);
    match discover_peers(DEFAULT_DISCOVERY_ADDR, timeout) {
        Ok(peers) if json => {
            let out = serde_json::to_string_pretty(&peers).unwrap_or_else(|_| "[]".to_string());
            println!("{out}");
        }
        Ok(peers) if peers.is_empty() => println!("No peers discovered."),
        Ok(peers) => {
            for p in peers {
                println!("{}  {}  tcp:{}", p.peer_id, p.addr, p.tcp_port);
            }
        }
        Err(e) => {
            eprintln!("Discovery failed: {e}");
            std::process::exit(1);
        }
    }
}
