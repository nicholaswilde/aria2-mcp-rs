pub mod aria2;
pub mod config;
pub mod error;
pub mod prompts;
pub mod resources;
pub mod server;
pub mod state;
pub mod tools;

pub use aria2::Aria2Client;
pub use config::{Config, TransportType};
pub use error::{Error, Result};
pub use prompts::PromptRegistry;
pub use resources::ResourceRegistry;
pub use server::McpServer;
pub use tools::{
    AddRssFeedTool, BulkManageDownloadsTool, CheckHealthTool, ConfigureAria2Tool,
    InspectDownloadTool, ListRssFeedsTool, ManageDownloadsTool, ManageTorrentTool, McpeTool,
    MonitorQueueTool, OrganizeCompletedTool, ScheduleLimitsTool, SearchDownloadsTool, ToolRegistry,
};
