use std::process::Command;

#[test]
fn test_taskfile_exists() {
    let output = Command::new("ls")
        .arg("Taskfile.yml")
        .output()
        .expect("failed to execute process");
    
    assert!(output.status.success());
}

#[test]
fn test_task_list() {
    let output = Command::new("task")
        .arg("--list")
        .output()
        .expect("failed to execute process");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("build"));
    assert!(stdout.contains("test"));
    assert!(stdout.contains("lint"));
    assert!(stdout.contains("format"));
}
