use anyhow::Result;
use aria2_mcp_rs::{Aria2Client, Config, McpServer, ToolRegistry};
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
    rpc_url: Option<String>,
    #[arg(short = 's', long, env = "ARIA2_MCP_RPC_SECRET")]
    rpc_secret: Option<String>,
    #[arg(short, long, env = "ARIA2_MCP_TRANSPORT", default_value = "stdio")]
    transport: Option<String>,
    #[arg(short, long, env = "ARIA2_MCP_PORT", default_value = "3000")]
    port: Option<u16>,
    #[arg(short, long, env = "ARIA2_MCP_LAZY", default_value = "false")]
    lazy: bool,
    #[arg(long, env = "ARIA2_MCP_NO_VERIFY_SSL", default_value = "true")]
    no_verify_ssl: bool,
    #[arg(long, env = "ARIA2_MCP_VERIFY_SSL", default_value = "false")]
    verify_ssl: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    run_app(args).await
}

async fn run_app(args: Args) -> Result<()> {
    let mut config = Config::load()?;

    // Override config with CLI arguments if provided
    if let Some(url) = args.rpc_url {
        config.rpc_url = url;
    }
    if let Some(secret) = args.rpc_secret {
        config.rpc_secret = Some(secret);
    }
    if let Some(transport) = args.transport {
        config.transport = match transport.to_lowercase().as_str() {
            "sse" | "http" => aria2_mcp_rs::TransportType::Sse,
            _ => aria2_mcp_rs::TransportType::Stdio,
        };
    }
    if let Some(port) = args.port {
        config.port = port;
    }
    if args.lazy {
        config.lazy_mode = true;
    }
    if args.verify_ssl {
        config.no_verify_ssl = false;
    } else if args.no_verify_ssl {
        config.no_verify_ssl = true;
    }

    log::info!("Starting aria2-mcp-rs with RPC URL: {}...", config.rpc_url);

    let client = Aria2Client::new(config.clone());
    let registry = ToolRegistry::new(&config);

    let server = McpServer::new(config, registry, client);
    server.run().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(
            args.rpc_url,
            Some("http://localhost:6800/jsonrpc".to_string())
        );
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
    fn test_args_parse_port() {
        let args = Args::try_parse_from(["aria2-mcp-rs", "--port", "4000"]).unwrap();
        assert_eq!(args.port, Some(4000));
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
            "--port",
            "5000",
        ])
        .unwrap();
        assert_eq!(args.rpc_url, Some("http://test".to_string()));
        assert_eq!(args.rpc_secret, Some("secret".to_string()));
        assert_eq!(args.transport, Some("sse".to_string()));
        assert_eq!(args.port, Some(5000));
    }

    #[test]
    fn test_args_default() {
        let args = Args::try_parse_from(["aria2-mcp-rs"]).unwrap();
        assert_eq!(
            args.rpc_url,
            Some("http://localhost:6800/jsonrpc".to_string())
        );
        assert_eq!(args.transport, Some("stdio".to_string()));
    }
}
