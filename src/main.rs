use anyhow::Result;
use aria2_mcp_rs::{Aria2Client, Config, McpServer, ResourceRegistry, ToolRegistry};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short = 'u',
        long,
        env = "ARIA2_MCP_RPC_URL",
        default_value = "http://localhost:6800/jsonrpc"
    )]
    rpc_url: String,
    #[arg(short = 's', long, env = "ARIA2_MCP_RPC_SECRET")]
    rpc_secret: Option<String>,
    #[arg(short, long, env = "ARIA2_MCP_TRANSPORT", default_value = "stdio")]
    transport: String,
    #[arg(long, env = "ARIA2_MCP_HTTP_PORT", default_value = "3000")]
    http_port: u16,
    #[arg(long, env = "ARIA2_MCP_HTTP_AUTH_TOKEN")]
    http_auth_token: Option<String>,
    #[arg(short = 'L', long, env = "ARIA2_MCP_LOG_LEVEL", default_value = "info")]
    log_level: String,
    #[arg(short, long, env = "ARIA2_MCP_LAZY")]
    lazy: bool,
    #[arg(long, env = "ARIA2_MCP_NO_VERIFY_SSL")]
    no_verify_ssl: bool,
    #[arg(long, env = "ARIA2_MCP_VERIFY_SSL")]
    verify_ssl: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut config = Config::load()?;

    // Override config with CLI arguments if provided
    config.rpc_url = args.rpc_url;
    if let Some(secret) = args.rpc_secret {
        config.rpc_secret = Some(secret);
    }
    config.transport = match args.transport.to_lowercase().as_str() {
        "sse" | "http" => aria2_mcp_rs::TransportType::Sse,
        _ => aria2_mcp_rs::TransportType::Stdio,
    };
    config.http_port = args.http_port;
    if let Some(token) = args.http_auth_token {
        config.http_auth_token = Some(token);
    }
    config.log_level = args.log_level;
    if args.lazy {
        config.lazy_mode = true;
    }
    if args.verify_ssl {
        config.no_verify_ssl = false;
    } else if args.no_verify_ssl {
        config.no_verify_ssl = true;
    }

    init_logger(&config.log_level);

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_app(config))
}

fn init_logger(level: &str) {
    let effective_level = parse_log_level(level);
    let is_invalid = effective_level == "info" && level.to_lowercase() != "info";

    let env = env_logger::Env::default().default_filter_or(effective_level);
    env_logger::Builder::from_env(env).init();

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

    let server = McpServer::new(config, registry, resource_registry, clients);
    server.run().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(args.transport, "unknown".to_string());
    }

    #[test]
    fn test_args_parse_no_args() {
        let args = Args::try_parse_from(["aria2-mcp-rs"]).unwrap();
        assert_eq!(args.rpc_url, "http://localhost:6800/jsonrpc".to_string());
    }

    #[test]
    fn test_args_parse() {
        let args = Args::try_parse_from(["aria2-mcp-rs", "-u", "http://test"]).unwrap();
        assert_eq!(args.rpc_url, "http://test".to_string());
    }

    #[test]
    fn test_args_parse_long() {
        let args = Args::try_parse_from(["aria2-mcp-rs", "--rpc-secret", "secret"]).unwrap();
        assert_eq!(args.rpc_secret, Some("secret".to_string()));
    }

    #[test]
    fn test_args_parse_transport_sse() {
        let args = Args::try_parse_from(["aria2-mcp-rs", "--transport", "sse"]).unwrap();
        assert_eq!(args.transport, "sse".to_string());
    }

    #[test]
    fn test_args_parse_port() {
        let args = Args::try_parse_from(["aria2-mcp-rs", "--http-port", "4000"]).unwrap();
        assert_eq!(args.http_port, 4000);
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
        assert_eq!(args.log_level, "debug".to_string());
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
        assert_eq!(args.rpc_url, "http://test".to_string());
        assert_eq!(args.rpc_secret, Some("secret".to_string()));
        assert_eq!(args.transport, "sse".to_string());
        assert_eq!(args.http_port, 5000);
        assert_eq!(args.http_auth_token, Some("test-token".to_string()));
    }

    #[test]
    fn test_args_default() {
        let args = Args::try_parse_from(["aria2-mcp-rs"]).unwrap();
        assert_eq!(args.rpc_url, "http://localhost:6800/jsonrpc".to_string());
        assert_eq!(args.transport, "stdio".to_string());
    }

    #[test]
    fn test_args_version() {
        let result = Args::try_parse_from(["aria2-mcp-rs", "--version"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
    }
}
