#![deny(unsafe_code)]

pub mod client;
pub mod hooks;
pub mod proxy;
pub mod registry;

pub use client::McpHttpClient;
pub use hooks::{EnterpriseHookPipeline, McpExecutionHook};
pub use proxy::{sync_servers_to_registry, RemoteMcpToolProxy};
pub use registry::{DatabaseAuditTool, InMemoryMcpRegistry, SystemInfoTool};
