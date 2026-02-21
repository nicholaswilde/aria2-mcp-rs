use std::path::Path;
use std::process::Command;

#[test]
fn test_taskfile_exists() {
    assert!(Path::new("Taskfile.yml").exists());
}

#[test]
fn test_task_list() {
    let output = match Command::new("task").arg("--list").output() {
        Ok(output) => output,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("'task' command not found, skipping test");
            return;
        }
        Err(e) => panic!("failed to execute process: {}", e),
    };

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("build"));
    assert!(stdout.contains("test"));
    assert!(stdout.contains("lint"));
    assert!(stdout.contains("fmt") || stdout.contains("format"));
}
