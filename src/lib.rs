pub mod aria2;
pub mod config;
pub mod error;
pub mod server;
pub mod tools;

pub use aria2::Aria2Client;
pub use config::{Config, TransportType};
pub use error::{Error, Result};
pub use server::McpServer;
pub use tools::{
    ConfigureAria2Tool, InspectDownloadTool, ManageDownloadsTool, McpeTool, MonitorQueueTool,
    SearchDownloadsTool, ToolRegistry,
};
