use std::process::Command;

#[test]
fn test_version_flag_long() {
    let binary_path = env!("CARGO_BIN_EXE_aria2-mcp-rs");
    let output = Command::new(binary_path)
        .arg("--version")
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&format!("aria2-mcp-rs {}", env!("CARGO_PKG_VERSION"))));
}

#[test]
fn test_version_flag_short() {
    let binary_path = env!("CARGO_BIN_EXE_aria2-mcp-rs");
    let output = Command::new(binary_path)
        .arg("-V")
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&format!("aria2-mcp-rs {}", env!("CARGO_PKG_VERSION"))));
}
