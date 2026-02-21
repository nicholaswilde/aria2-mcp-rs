#![allow(dead_code)]
use anyhow::Result;
use aria2_mcp_rs::aria2::Aria2Client;
use aria2_mcp_rs::config::Config;
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};
use tokio::io::AsyncBufReadExt;

pub fn should_run_docker_tests() -> bool {
    std::env::var("RUN_DOCKER_TESTS").map(|v| v == "true").unwrap_or(false)
}

pub struct Aria2Container {
    _container: testcontainers::ContainerAsync<GenericImage>,
    pub host: String,
    pub port: u16,
}

impl Aria2Container {
    pub async fn new() -> Result<Self> {
        println!("🐳 Starting aria2 container...");

        let image = GenericImage::new("p3terx/aria2-pro", "latest")
            .with_wait_for(WaitFor::message_on_stdout(
                "IPv4 RPC: listening on TCP port 6800",
            ))
            .with_exposed_port(ContainerPort::Tcp(6800))
            .with_env_var("RPC_SECRET", "test-secret");

        let container = image.start().await?;

        // Pipe stdout logs
        let stdout = container.stdout(true);
        let stderr = container.stderr(true);
        tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                println!("DOCKER STDOUT: {}", line);
            }
        });
        tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                println!("DOCKER STDERR: {}", line);
            }
        });

        let port_6800 = container.get_host_port_ipv4(6800).await?;
        let host = "localhost".to_string();

        println!(
            "✅ aria2 container started at http://{}:{}",
            host, port_6800
        );

        Ok(Self {
            _container: container,
            host,
            port: port_6800,
        })
    }

    pub fn client(&self) -> Aria2Client {
        Aria2Client::new(self.config())
    }

    pub fn config(&self) -> Config {
        Config {
            rpc_url: format!("http://{}:{}/jsonrpc", self.host, self.port),
            rpc_secret: Some("test-secret".to_string()),
            transport: aria2_mcp_rs::TransportType::Stdio,
            port: 3000,
        }
    }
}
