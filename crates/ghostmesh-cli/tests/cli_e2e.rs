use std::process::Command;

#[test]
fn test_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_ghostmesh-cli"))
        .output()
        .expect("run ghostmesh binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ghostmesh CLI"));
    assert!(stdout.contains("keygen"));
    assert!(stdout.contains("sim-sync"));
}

#[test]
fn test_cli_keygen() {
    let output = Command::new(env!("CARGO_BIN_EXE_ghostmesh-cli"))
        .arg("keygen")
        .output()
        .expect("run ghostmesh keygen");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("peer_id:"));
    assert!(stdout.contains("verifying_key:"));
}

#[test]
fn test_cli_sim_sync() {
    let output = Command::new(env!("CARGO_BIN_EXE_ghostmesh-cli"))
        .arg("sim-sync")
        .output()
        .expect("run ghostmesh sim-sync");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Alice peer_id:"));
    assert!(stdout.contains("Bob peer_id:"));
    assert!(
        stdout.contains("sim-sync: E2EE handshake, CRDT sync & convergence verified successfully.")
    );
}

#[test]
fn test_cli_pack_and_verify() {
    let temp_dir = std::env::temp_dir();
    let sample_file = temp_dir.join("ghostmesh_sample.txt");
    let manifest_file = temp_dir.join("ghostmesh_sample.json");

    std::fs::write(
        &sample_file,
        b"Ghostmesh CLI e2e test payload with BLAKE3 chunks.",
    )
    .expect("write sample");

    let pack_output = Command::new(env!("CARGO_BIN_EXE_ghostmesh-cli"))
        .args(["pack", sample_file.to_str().unwrap()])
        .output()
        .expect("run ghostmesh pack");
    assert!(pack_output.status.success());

    std::fs::write(&manifest_file, &pack_output.stdout).expect("write manifest");

    let verify_output = Command::new(env!("CARGO_BIN_EXE_ghostmesh-cli"))
        .args([
            "verify",
            manifest_file.to_str().unwrap(),
            sample_file.to_str().unwrap(),
        ])
        .output()
        .expect("run ghostmesh verify");
    assert!(verify_output.status.success());
    let verify_stdout = String::from_utf8_lossy(&verify_output.stdout);
    assert!(verify_stdout.contains("OK: File matches manifest root hash"));

    let _ = std::fs::remove_file(&sample_file);
    let _ = std::fs::remove_file(&manifest_file);
}

#[test]
fn test_cli_live_demo() {
    let output = Command::new(env!("CARGO_BIN_EXE_ghostmesh-cli"))
        .arg("live-demo")
        .output()
        .expect("run ghostmesh live-demo");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("LAN Discovery Beacon: peer="));
    assert!(stdout.contains("CRDT Sync: node_b network='ghostmesh-live'"));
    assert!(stdout.contains("File Transfer: verified"));
}

#[test]
fn test_cli_tui_demo_and_live() {
    let demo = Command::new(env!("CARGO_BIN_EXE_ghostmesh-cli"))
        .args(["tui", "--demo", "--ticks", "2"])
        .output()
        .expect("run ghostmesh tui --demo");
    assert!(demo.status.success());
    let demo_out = String::from_utf8_lossy(&demo.stdout);
    assert!(demo_out.contains("\x1b[?25h"));

    let live = Command::new(env!("CARGO_BIN_EXE_ghostmesh-cli"))
        .args(["tui", "--ticks", "2"])
        .output()
        .expect("run ghostmesh tui live");
    assert!(live.status.success());
    let live_out = String::from_utf8_lossy(&live.stdout);
    assert!(live_out.contains("[LIVE]"));
    assert!(live_out.contains("\x1b[?25h"));
}

#[test]
<<<<<<< HEAD
fn test_cli_discover_json() {
    let out = Command::new(env!("CARGO_BIN_EXE_ghostmesh-cli"))
        .args(["discover", "--timeout-ms", "80", "--json"])
        .output()
        .expect("run ghostmesh discover --json");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "[]");
}

#[test]
fn cli_init_and_trust_persist_identity() {
    let dir = std::env::temp_dir().join(format!("ghostmesh_init_e2e_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let first = Command::new(env!("CARGO_BIN_EXE_ghostmesh-cli"))
        .arg("init")
        .env("GHOSTMESH_CONFIG_DIR", &dir)
        .output()
        .expect("run init");
    assert!(first.status.success());
    let first_out = String::from_utf8_lossy(&first.stdout);
    assert!(first_out.contains("initialized peer_id:"));
    assert!(dir.join("identity.key").exists());

    let second = Command::new(env!("CARGO_BIN_EXE_ghostmesh-cli"))
        .arg("init")
        .env("GHOSTMESH_CONFIG_DIR", &dir)
        .output()
        .expect("run init again");
    assert!(second.status.success());
    assert!(String::from_utf8_lossy(&second.stdout).contains("loaded peer_id:"));

    let peer_hex = "a".repeat(64);
    let trust = Command::new(env!("CARGO_BIN_EXE_ghostmesh-cli"))
        .args(["trust", &peer_hex])
        .env("GHOSTMESH_CONFIG_DIR", &dir)
        .output()
        .expect("run trust");
    assert!(trust.status.success());
    assert!(String::from_utf8_lossy(&trust.stdout).contains("trusted:"));
    assert!(dir.join("trusted_peers.json").exists());

    let _ = std::fs::remove_dir_all(&dir);
}
