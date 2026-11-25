pub mod discover;
pub mod monitor;
pub mod ops;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        ops::print_usage();
        return;
    }

    match args[1].as_str() {
        "keygen" => ops::run_keygen(),
        "sim-sync" => ops::run_sim_sync(),
        "pack" => {
            if args.len() < 3 {
                eprintln!("Usage: ghostmesh pack <file>");
                std::process::exit(1);
            }
            ops::run_pack(&args[2]);
        }
        "verify" => {
            if args.len() < 4 {
                eprintln!("Usage: ghostmesh verify <manifest.json> <source_file>");
                std::process::exit(1);
            }
            ops::run_verify(&args[2], &args[3]);
        }
        "live-demo" => ops::run_live_demo(),
        "listen" => {
            if args.len() < 3 {
                eprintln!("Usage: ghostmesh listen <addr>");
                std::process::exit(1);
            }
            ops::run_listen(&args[2]);
        }
        "send" => {
            if args.len() < 4 {
                eprintln!("Usage: ghostmesh send <addr> <file>");
                std::process::exit(1);
            }
            ops::run_send(&args[2], &args[3]);
        }
        "tui" | "monitor" => run_tui_cmd(&args[2..]),
        "discover" => run_discover_cmd(&args[2..]),
        _ => {
            ops::print_usage();
            std::process::exit(1);
        }
    }
}

fn run_tui_cmd(args: &[String]) {
    let demo = args.iter().any(|a| a == "--demo");
    let ticks = args
        .windows(2)
        .find(|w| w[0] == "--ticks")
        .and_then(|w| w[1].parse::<u64>().ok());
    if demo {
        monitor::run_monitor(ticks);
    } else {
        monitor::run_live_monitor(ticks);
    }
}

fn run_discover_cmd(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let timeout_ms = args
        .windows(2)
        .find(|w| w[0] == "--timeout-ms")
        .and_then(|w| w[1].parse::<u64>().ok())
        .unwrap_or(1500);
    discover::run_discover(timeout_ms, json);
}
