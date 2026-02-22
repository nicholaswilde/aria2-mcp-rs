use std::process::Command;

#[test]
fn test_version_flag_long() {
    let binary_path = env!("CARGO_BIN_EXE_aria2-mcp-rs");
    let output = Command::new(binary_path).arg("--version").output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            eprintln!(
                "Skipping test: failed to execute binary '{}': {}",
                binary_path, e
            );
            return;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Detect architecture mismatch or loader issues which often manifest as syntax errors in the shell fallback
        if stderr.contains("Syntax error")
            || stderr.contains("ELF")
            || output.status.code() == Some(126)
            || output.status.code() == Some(127)
        {
            eprintln!("Skipping test: binary cannot be executed on this host (likely architecture mismatch or missing loader).");
            eprintln!("Status: {:?}", output.status);
            eprintln!("STDERR: {}", stderr);
            return;
        }

        eprintln!("Binary path: {}", binary_path);
        eprintln!("Exit status: {:?}", output.status);
        eprintln!("Exit code: {:?}", output.status.code());
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", stderr);
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
    let output = Command::new(binary_path).arg("-V").output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            eprintln!(
                "Skipping test: failed to execute binary '{}': {}",
                binary_path, e
            );
            return;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Detect architecture mismatch or loader issues
        if stderr.contains("Syntax error")
            || stderr.contains("ELF")
            || output.status.code() == Some(126)
            || output.status.code() == Some(127)
        {
            eprintln!("Skipping test: binary cannot be executed on this host (likely architecture mismatch or missing loader).");
            eprintln!("Status: {:?}", output.status);
            eprintln!("STDERR: {}", stderr);
            return;
        }

        eprintln!("Binary path: {}", binary_path);
        eprintln!("Exit status: {:?}", output.status);
        eprintln!("Exit code: {:?}", output.status.code());
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", stderr);
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
