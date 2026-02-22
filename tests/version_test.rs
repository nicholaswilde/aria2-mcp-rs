use std::process::Command;

#[test]
fn test_version_flag_long() {
    let binary_path = env!("CARGO_BIN_EXE_aria2-mcp-rs");
    let output = Command::new(binary_path)
        .arg("--version")
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        eprintln!("Binary path: {}", binary_path);
        eprintln!("Exit status: {:?}", output.status);
        eprintln!("Exit code: {:?}", output.status.code());
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(
        output.status.success(),
        "Binary exited with non-zero status: {:?}",
        output.status
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&format!("aria2-mcp-rs {}", env!("CARGO_PKG_VERSION"))),
        "STDOUT does not contain version: {}",
        stdout
    );
}

#[test]
fn test_version_flag_short() {
    let binary_path = env!("CARGO_BIN_EXE_aria2-mcp-rs");
    let output = Command::new(binary_path)
        .arg("-V")
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        eprintln!("Binary path: {}", binary_path);
        eprintln!("Exit status: {:?}", output.status);
        eprintln!("Exit code: {:?}", output.status.code());
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(
        output.status.success(),
        "Binary exited with non-zero status: {:?}",
        output.status
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&format!("aria2-mcp-rs {}", env!("CARGO_PKG_VERSION"))),
        "STDOUT does not contain version: {}",
        stdout
    );
}
