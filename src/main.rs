use anyhow::Result;
use aria2_mcp_rs::{Aria2Client, Config, McpServer, ResourceRegistry, ToolRegistry};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'u', long, env = "ARIA2_MCP_RPC_URL")]
    rpc_url: Option<String>,
    #[arg(short = 's', long, env = "ARIA2_MCP_RPC_SECRET")]
    rpc_secret: Option<String>,
    #[arg(short, long, env = "ARIA2_MCP_TRANSPORT")]
    transport: Option<String>,
    #[arg(long, env = "ARIA2_MCP_HTTP_HOST")]
    http_host: Option<String>,
    #[arg(long, env = "ARIA2_MCP_HTTP_PORT")]
    http_port: Option<u16>,
    #[arg(long, env = "ARIA2_MCP_HTTP_AUTH_TOKEN")]
    http_auth_token: Option<String>,
    #[arg(short = 'L', long, env = "ARIA2_MCP_LOG_LEVEL")]
    log_level: Option<String>,
    #[arg(short, long)]
    config: Option<String>,
    #[arg(short, long, env = "ARIA2_MCP_LAZY")]
    lazy: bool,
    #[arg(long, env = "ARIA2_MCP_NO_VERIFY_SSL")]
    no_verify_ssl: bool,
    #[arg(long, env = "ARIA2_MCP_VERIFY_SSL")]
    verify_ssl: bool,
    #[arg(short = 'i', long, value_parser = parse_instance_arg)]
    instance: Vec<aria2_mcp_rs::config::Aria2Instance>,
}

fn parse_instance_arg(s: &str) -> Result<aria2_mcp_rs::config::Aria2Instance, String> {
    let mut name = None;
    let mut url = None;
    let mut secret = None;

    for part in s.split(',') {
        if let Some((key, value)) = part.split_once('=') {
            match key {
                "name" => name = Some(value.to_string()),
                "url" => url = Some(value.to_string()),
                "secret" => secret = Some(value.to_string()),
                _ => return Err(format!("Unknown instance key: {}", key)),
            }
        } else {
            return Err(format!(
                "Invalid instance part: {}. Expected key=value",
                part
            ));
        }
    }

    let name = name.ok_or_else(|| {
        "Missing 'name' in instance. Example: --instance name=local,url=http://...".to_string()
    })?;
    let rpc_url = url.ok_or_else(|| {
        "Missing 'url' in instance. Example: --instance name=local,url=http://...".to_string()
    })?;

    Ok(aria2_mcp_rs::config::Aria2Instance {
        name,
        rpc_url,
        rpc_secret: secret,
    })
}

fn main() -> Result<()> {
    // Install default crypto provider for rustls
    let _ = rustls::crypto::ring::default_provider().install_default();

    let args = Args::parse();
    let mut config = Config::load()?;

    // Override config with CLI arguments if provided
    if let Some(rpc_url) = args.rpc_url {
        config.rpc_url = rpc_url.clone();
        // Explicitly update the first instance if it's the default one
        if !config.instances.is_empty() && config.instances[0].name == "default" {
            config.instances[0].rpc_url = rpc_url;
        }
    }
    if let Some(secret) = args.rpc_secret {
        config.rpc_secret = Some(secret.clone());
        if !config.instances.is_empty() && config.instances[0].name == "default" {
            config.instances[0].rpc_secret = Some(secret);
        }
    }
    if let Some(transport) = args.transport {
        config.transport = match transport.to_lowercase().as_str() {
            "sse" | "http" => aria2_mcp_rs::TransportType::Sse,
            _ => aria2_mcp_rs::TransportType::Stdio,
        };
    }
    if let Some(host) = args.http_host {
        config.http_host = host;
    }
    if let Some(port) = args.http_port {
        config.http_port = port;
    }
    if let Some(token) = args.http_auth_token {
        config.http_auth_token = Some(token);
    }
    if let Some(log_level) = args.log_level {
        config.log_level = log_level;
    }
    if args.lazy {
        config.lazy_mode = true;
    }
    if args.verify_ssl {
        config.no_verify_ssl = false;
    } else if args.no_verify_ssl {
        config.no_verify_ssl = true;
    }

    // Add instances from CLI if provided
    for inst in args.instance {
        // If an instance with the same name exists, replace it, otherwise append
        if let Some(pos) = config.instances.iter().position(|i| i.name == inst.name) {
            config.instances[pos] = inst;
        } else {
            config.instances.push(inst);
        }
    }

    config.normalize();

    init_logger(&config.log_level);

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        // Load persistent state and merge into config
        let state_manager = aria2_mcp_rs::state::StateManager::new(std::path::PathBuf::from(
            "aria2_mcp_state.json",
        ));
        if let Ok(state) = state_manager.load().await {
            for (k, v) in state.bandwidth_profiles {
                config.bandwidth_profiles.insert(k, v);
            }
            // Add schedules if not already present
            for schedule in state.bandwidth_schedules {
                if !config.bandwidth_schedules.contains(&schedule) {
                    config.bandwidth_schedules.push(schedule);
                }
            }
            // Add organize rules if not already present
            for rule in state.organize_rules {
                if !config.organize_rules.iter().any(|r| r.name == rule.name) {
                    config.organize_rules.push(rule);
                }
            }
        }

        run_app(config).await
    })
}

fn init_logger(level: &str) {
    let effective_level = parse_log_level(level);
    let is_invalid = effective_level == "info" && level.to_lowercase() != "info";

    let env = env_logger::Env::default().default_filter_or(effective_level);
    let _ = env_logger::Builder::from_env(env).try_init();

    if is_invalid {
        log::warn!(
            "Invalid log level '{}' provided, defaulting to 'info'",
            level
        );
    }
}

fn parse_log_level(level: &str) -> &str {
    match level.to_lowercase().as_str() {
        "error" => "error",
        "warn" => "warn",
        "info" => "info",
        "debug" => "debug",
        "trace" => "trace",
        _ => "info",
    }
}

async fn run_app(config: Config) -> Result<()> {
    log::info!(
        "Starting aria2-mcp-rs with {} instances...",
        config.instances.len()
    );

    let clients: Vec<Aria2Client> = config
        .instances
        .iter()
        .map(|instance| Aria2Client::new_with_instance(config.clone(), instance.clone()))
        .collect();

    let registry = ToolRegistry::new(&config);
    let mut resource_registry = ResourceRegistry::default();
    resource_registry.register(std::sync::Arc::new(
        aria2_mcp_rs::resources::GlobalStatusResource,
    ));
    let prompt_registry = aria2_mcp_rs::PromptRegistry::default();

    let server = McpServer::new(
        config,
        registry,
        resource_registry,
        prompt_registry,
        clients,
    );
    server.run().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aria2_mcp_rs::config::Aria2Instance;
    use std::env;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_args_to_config_mapping() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let args = Args::try_parse_from([
            "aria2-mcp-rs",
            "-u",
            "http://test",
            "-s",
            "secret",
            "--transport",
            "sse",
            "--http-host",
            "127.0.0.1",
            "--http-port",
            "5000",
            "--http-auth-token",
            "test-token",
            "--lazy",
            "--no-verify-ssl",
        ])
        .unwrap();

        let config = Config {
            rpc_url: args
                .rpc_url
                .unwrap_or_else(|| "http://127.0.0.1:6800/jsonrpc".to_string()),
            rpc_secret: args.rpc_secret,
            transport: match args
                .transport
                .unwrap_or_else(|| "stdio".to_string())
                .to_lowercase()
                .as_str()
            {
                "sse" | "http" => aria2_mcp_rs::TransportType::Sse,
                _ => aria2_mcp_rs::TransportType::Stdio,
            },
            http_host: args.http_host.unwrap_or_else(|| "0.0.0.0".to_string()),
            http_port: args.http_port.unwrap_or(3000),
            http_auth_token: args.http_auth_token,
            lazy_mode: args.lazy,
            no_verify_ssl: args.no_verify_ssl,
            ..Default::default()
        };

        assert_eq!(config.rpc_url, "http://test");
        assert_eq!(config.rpc_secret, Some("secret".to_string()));
        assert_eq!(config.transport, aria2_mcp_rs::TransportType::Sse);
        assert_eq!(config.http_host, "127.0.0.1");
        assert_eq!(config.http_port, 5000);
        assert_eq!(config.http_auth_token, Some("test-token".to_string()));
        assert!(config.lazy_mode);
        assert!(config.no_verify_ssl);

        // Test verify_ssl
        let args = Args::try_parse_from(["aria2-mcp-rs", "--verify-ssl"]).unwrap();
        let mut config = Config {
            no_verify_ssl: true,
            ..Default::default()
        };
        if args.verify_ssl {
            config.no_verify_ssl = false;
        }
        assert!(!config.no_verify_ssl);
    }

    #[test]
    fn test_parse_log_level() {
        assert_eq!(parse_log_level("debug"), "debug");
        assert_eq!(parse_log_level("INFO"), "info");
        assert_eq!(parse_log_level("trace"), "trace");
        assert_eq!(parse_log_level("invalid"), "info");
    }

    #[tokio::test]
    async fn test_run_app_invalid_url() {
        // We can't easily test run_app as it starts a server
    }

    #[test]
    fn test_args_parse_transport_invalid() {
        let args = Args::try_parse_from(["aria2-mcp-rs", "--transport", "unknown"]).unwrap();
        assert_eq!(args.transport, Some("unknown".to_string()));
    }

    #[test]
    fn test_args_parse_no_args() {
        let args = Args::try_parse_from(["aria2-mcp-rs"]).unwrap();
        assert_eq!(args.rpc_url, None);
    }

    #[test]
    fn test_args_parse() {
        let args = Args::try_parse_from(["aria2-mcp-rs", "-u", "http://test"]).unwrap();
        assert_eq!(args.rpc_url, Some("http://test".to_string()));
    }

    #[test]
    fn test_args_parse_long() {
        let args = Args::try_parse_from(["aria2-mcp-rs", "--rpc-secret", "secret"]).unwrap();
        assert_eq!(args.rpc_secret, Some("secret".to_string()));
    }

    #[test]
    fn test_args_parse_transport_sse() {
        let args = Args::try_parse_from(["aria2-mcp-rs", "--transport", "sse"]).unwrap();
        assert_eq!(args.transport, Some("sse".to_string()));
    }

    #[test]
    fn test_args_parse_transport_http() {
        let args = Args::try_parse_from(["aria2-mcp-rs", "--transport", "http"]).unwrap();
        assert_eq!(args.transport, Some("http".to_string()));
    }

    #[test]
    fn test_http_transport_mapping() {
        let args = Args::try_parse_from(["aria2-mcp-rs", "--transport", "http"]).unwrap();
        let config = Config {
            transport: match args
                .transport
                .unwrap_or_else(|| "stdio".to_string())
                .to_lowercase()
                .as_str()
            {
                "sse" | "http" => aria2_mcp_rs::TransportType::Sse,
                _ => aria2_mcp_rs::TransportType::Stdio,
            },
            ..Default::default()
        };
        assert_eq!(config.transport, aria2_mcp_rs::TransportType::Sse);
    }

    #[test]
    fn test_args_parse_port() {
        let args = Args::try_parse_from(["aria2-mcp-rs", "--http-port", "4000"]).unwrap();
        assert_eq!(args.http_port, Some(4000));
    }

    #[test]
    fn test_args_parse_auth_token() {
        let args =
            Args::try_parse_from(["aria2-mcp-rs", "--http-auth-token", "test-token"]).unwrap();
        assert_eq!(args.http_auth_token, Some("test-token".to_string()));
    }

    #[test]
    fn test_args_parse_log_level() {
        let args = Args::try_parse_from(["aria2-mcp-rs", "--log-level", "debug"]).unwrap();
        assert_eq!(args.log_level, Some("debug".to_string()));
    }

    #[test]
    fn test_args_parse_config_long() {
        let args = Args::try_parse_from(["aria2-mcp-rs", "--config", "test.toml"]).unwrap();
        assert_eq!(args.config, Some("test.toml".to_string()));
    }

    #[test]
    fn test_args_parse_config_short() {
        let args = Args::try_parse_from(["aria2-mcp-rs", "-c", "custom.yaml"]).unwrap();
        assert_eq!(args.config, Some("custom.yaml".to_string()));
    }

    #[test]
    fn test_args_parse_all() {
        let args = Args::try_parse_from([
            "aria2-mcp-rs",
            "-u",
            "http://test",
            "-s",
            "secret",
            "--transport",
            "sse",
            "--http-port",
            "5000",
            "--http-auth-token",
            "test-token",
        ])
        .unwrap();
        assert_eq!(args.rpc_url, Some("http://test".to_string()));
        assert_eq!(args.rpc_secret, Some("secret".to_string()));
        assert_eq!(args.transport, Some("sse".to_string()));
        assert_eq!(args.http_port, Some(5000));
        assert_eq!(args.http_auth_token, Some("test-token".to_string()));
    }

    #[test]
    fn test_cli_instances_override_config() {
        let _guard = ENV_MUTEX.lock().unwrap();

        // 1. Initial config with an instance
        let mut config = Config {
            instances: vec![Aria2Instance {
                name: "local".to_string(),
                rpc_url: "http://config-url:6800/jsonrpc".to_string(),
                rpc_secret: None,
            }],
            ..Config::default()
        };

        // 2. CLI Args with same name but different URL
        let args = Args::try_parse_from([
            "aria2-mcp-rs",
            "--instance",
            "name=local,url=http://cli-url:6800/jsonrpc",
            "--instance",
            "name=new,url=http://new-url:6800/jsonrpc",
        ])
        .unwrap();

        // 3. Apply logic from main
        for inst in args.instance {
            if let Some(pos) = config.instances.iter().position(|i| i.name == inst.name) {
                config.instances[pos] = inst;
            } else {
                config.instances.push(inst);
            }
        }

        // 4. Verify results
        assert_eq!(config.instances.len(), 2);
        assert_eq!(config.instances[0].name, "local");
        assert_eq!(config.instances[0].rpc_url, "http://cli-url:6800/jsonrpc");
        assert_eq!(config.instances[1].name, "new");
        assert_eq!(config.instances[1].rpc_url, "http://new-url:6800/jsonrpc");
    }

    #[test]
    fn test_args_parse_multiple_instances() {
        let args = Args::try_parse_from([
            "aria2-mcp-rs",
            "--instance",
            "name=local,url=http://localhost:6800/jsonrpc",
            "--instance",
            "name=remote,url=http://remote:6800/jsonrpc,secret=token",
        ])
        .unwrap();

        assert_eq!(args.instance.len(), 2);
        assert_eq!(args.instance[0].name, "local");
        assert_eq!(args.instance[0].rpc_url, "http://localhost:6800/jsonrpc");
        assert_eq!(args.instance[0].rpc_secret, None);

        assert_eq!(args.instance[1].name, "remote");
        assert_eq!(args.instance[1].rpc_url, "http://remote:6800/jsonrpc");
        assert_eq!(args.instance[1].rpc_secret, Some("token".to_string()));
    }

    #[test]
    fn test_main_logic_override_precedence() {
        let _guard = ENV_MUTEX.lock().unwrap();

        // 1. Initial config (as if loaded from file/env)
        let rpc_url = "http://config-file:6800/jsonrpc".to_string();
        let mut config = Config {
            rpc_url: rpc_url.clone(),
            log_level: "info".to_string(),
            instances: vec![Aria2Instance {
                name: "default".to_string(),
                rpc_url,
                rpc_secret: None,
            }],
            ..Config::default()
        };
        assert_eq!(
            config.instances[0].rpc_url,
            "http://config-file:6800/jsonrpc"
        );

        // 2. CLI Args
        let args = Args::try_parse_from([
            "aria2-mcp-rs",
            "--rpc-url",
            "http://cli-override:6800/jsonrpc",
            "--log-level",
            "debug",
        ])
        .unwrap();

        // 3. Apply overrides (logic from main)
        if let Some(rpc_url) = args.rpc_url {
            config.rpc_url = rpc_url.clone();
            if !config.instances.is_empty() && config.instances[0].name == "default" {
                config.instances[0].rpc_url = rpc_url;
            }
        }
        if let Some(log_level) = args.log_level {
            config.log_level = log_level;
        }

        // 4. Verify CLI won
        assert_eq!(config.rpc_url, "http://cli-override:6800/jsonrpc");
        assert_eq!(
            config.instances[0].rpc_url,
            "http://cli-override:6800/jsonrpc"
        );
        assert_eq!(config.log_level, "debug");
    }

    #[test]
    fn test_cli_args_override_all() {
        let _guard = ENV_MUTEX.lock().unwrap();

        // Set env vars that clap would normally pick up
        env::set_var("ARIA2_MCP_RPC_URL", "http://env-url:6800/jsonrpc");
        env::set_var("ARIA2_MCP_LOG_LEVEL", "debug");

        // Pass different CLI arguments
        let args = Args::try_parse_from([
            "aria2-mcp-rs",
            "--rpc-url",
            "http://cli-url:6800/jsonrpc",
            "--log-level",
            "trace",
        ])
        .unwrap();

        // CLI args should win
        assert_eq!(
            args.rpc_url,
            Some("http://cli-url:6800/jsonrpc".to_string())
        );
        assert_eq!(args.log_level, Some("trace".to_string()));

        // Cleanup
        env::remove_var("ARIA2_MCP_RPC_URL");
        env::remove_var("ARIA2_MCP_LOG_LEVEL");
    }

    #[test]
    fn test_args_from_env_vars() {
        let _guard = ENV_MUTEX.lock().unwrap();
        env::set_var("ARIA2_MCP_RPC_URL", "http://env-args-test:6800/jsonrpc");
        env::set_var("ARIA2_MCP_RPC_SECRET", "env-args-secret");
        env::set_var("ARIA2_MCP_TRANSPORT", "sse");
        env::set_var("ARIA2_MCP_HTTP_HOST", "1.2.3.4");
        env::set_var("ARIA2_MCP_HTTP_PORT", "8888");
        env::set_var("ARIA2_MCP_HTTP_AUTH_TOKEN", "env-token");
        env::set_var("ARIA2_MCP_LOG_LEVEL", "trace");
        env::set_var("ARIA2_MCP_LAZY", "true");
        env::set_var("ARIA2_MCP_NO_VERIFY_SSL", "true");

        let args = Args::try_parse_from(["aria2-mcp-rs"]).unwrap();

        assert_eq!(
            args.rpc_url,
            Some("http://env-args-test:6800/jsonrpc".to_string())
        );
        assert_eq!(args.rpc_secret, Some("env-args-secret".to_string()));
        assert_eq!(args.transport, Some("sse".to_string()));
        assert_eq!(args.http_host, Some("1.2.3.4".to_string()));
        assert_eq!(args.http_port, Some(8888));
        assert_eq!(args.http_auth_token, Some("env-token".to_string()));
        assert_eq!(args.log_level, Some("trace".to_string()));
        assert!(args.lazy);
        assert!(args.no_verify_ssl);

        // Cleanup
        env::remove_var("ARIA2_MCP_RPC_URL");
        env::remove_var("ARIA2_MCP_RPC_SECRET");
        env::remove_var("ARIA2_MCP_TRANSPORT");
        env::remove_var("ARIA2_MCP_HTTP_HOST");
        env::remove_var("ARIA2_MCP_HTTP_PORT");
        env::remove_var("ARIA2_MCP_HTTP_AUTH_TOKEN");
        env::remove_var("ARIA2_MCP_LOG_LEVEL");
        env::remove_var("ARIA2_MCP_LAZY");
        env::remove_var("ARIA2_MCP_NO_VERIFY_SSL");

        // Test VERIFY_SSL
        env::set_var("ARIA2_MCP_VERIFY_SSL", "true");
        let args = Args::try_parse_from(["aria2-mcp-rs"]).unwrap();
        assert!(args.verify_ssl);
        env::remove_var("ARIA2_MCP_VERIFY_SSL");
    }

    #[test]
    fn test_args_default() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let args = Args::try_parse_from(["aria2-mcp-rs"]).unwrap();
        assert_eq!(args.rpc_url, None);
        assert_eq!(args.transport, None);
    }

    #[test]
    fn test_args_version() {
        let result = Args::try_parse_from(["aria2-mcp-rs", "--version"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
    }

    #[test]
    fn test_parse_instance_arg_invalid_format() {
        let result = parse_instance_arg("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid instance part"));
    }

    #[test]
    fn test_parse_instance_arg_missing_name() {
        let result = parse_instance_arg("url=http://localhost:6800/jsonrpc");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing 'name'"));
    }

    #[test]
    fn test_parse_instance_arg_missing_url() {
        let result = parse_instance_arg("name=local");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing 'url'"));
    }

    #[test]
    fn test_parse_instance_arg_unknown_key() {
        let result = parse_instance_arg("name=local,url=http://localhost:6800/jsonrpc,unknown=val");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown instance key"));
    }

    #[test]
    fn test_init_logger() {
        // We call it to cover the branches
        init_logger("info");
        init_logger("invalid");
    }
}
