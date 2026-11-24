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
    assert!(stdout.contains("sim-sync: E2EE handshake, CRDT sync & convergence verified successfully."));
}

#[test]
fn test_cli_pack_and_verify() {
    let temp_dir = std::env::temp_dir();
    let sample_file = temp_dir.join("ghostmesh_sample.txt");
    let manifest_file = temp_dir.join("ghostmesh_sample.json");

    std::fs::write(&sample_file, b"Ghostmesh CLI e2e test payload with BLAKE3 chunks.")
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
