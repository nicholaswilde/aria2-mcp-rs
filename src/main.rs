use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        env = "ARIA2_RPC_URL",
        default_value = "http://localhost:6800/jsonrpc"
    )]
    rpc_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    println!("Starting aria2-mcp-rs with RPC URL: {}...", args.rpc_url);

    // TODO: Initialize config and start MCP server

    Ok(())
}
