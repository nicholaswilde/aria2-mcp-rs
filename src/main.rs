use anyhow::Result;
use aria2_mcp_rs::{Aria2Client, Config, McpServer, ToolRegistry};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        env = "ARIA2_MCP_RPC_URL",
        default_value = "http://localhost:6800/jsonrpc"
    )]
    rpc_url: Option<String>,
    #[arg(short, long, env = "ARIA2_MCP_RPC_SECRET")]
    rpc_secret: Option<String>,
    #[arg(short, long, env = "ARIA2_MCP_TRANSPORT", default_value = "stdio")]
    transport: Option<String>,
    #[arg(short, long, env = "ARIA2_MCP_PORT", default_value = "3000")]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

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
            "sse" => aria2_mcp_rs::TransportType::Sse,
            _ => aria2_mcp_rs::TransportType::Stdio,
        };
    }
    if let Some(port) = args.port {
        config.port = port;
    }

    log::info!("Starting aria2-mcp-rs with RPC URL: {}...", config.rpc_url);

    let client = Aria2Client::new(config.clone());
    let registry = ToolRegistry::new();

    let server = McpServer::new(config, registry, client);
    server.run().await?;

    Ok(())
}
