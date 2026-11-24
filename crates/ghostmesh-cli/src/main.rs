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
        _ => {
            ops::print_usage();
            std::process::exit(1);
        }
    }
}
