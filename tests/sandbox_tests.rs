use aria2_mcp_rs::tools::sandbox::PathSandbox;

#[test]
fn test_sandbox_valid_path() {
    let base = std::env::current_dir().unwrap();
    let sandbox = PathSandbox::new(base.clone());

    let relative = "Cargo.toml";
    let resolved = sandbox.resolve(relative).unwrap();

    assert_eq!(resolved, base.join(relative).canonicalize().unwrap());
}

#[test]
fn test_sandbox_traversal_attempt() {
    let base = std::env::current_dir().unwrap().join("src");
    let sandbox = PathSandbox::new(base);

    let result = sandbox.resolve("../Cargo.toml");
    assert!(result.is_err());
}

#[test]
fn test_sandbox_absolute_path_attempt() {
    let base = std::env::current_dir().unwrap().join("src");
    let sandbox = PathSandbox::new(base);

    let result = sandbox.resolve("/etc/passwd");
    assert!(result.is_err());
}
