mod antigravity;
mod claude_code;
mod claude_desktop;
mod cline;
mod codex;
mod cursor;
mod gemini_cli;
mod github_copilot;
mod iflow;
mod kiro;
mod opencode;
mod qwen_code;
mod vscode;
mod windsurf;

use crate::core::{LocalConfigSource, MCPConfig, MCPServer, SupportedApp, WriteOperation};
use crate::platform::PlatformContext;
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub use antigravity::AntigravityAdapter;
pub use claude_code::ClaudeCodeAdapter;
pub use claude_desktop::ClaudeDesktopAdapter;
pub use cline::ClineAdapter;
pub use codex::CodexAdapter;
pub use cursor::CursorAdapter;
pub use gemini_cli::GeminiCliAdapter;
pub use github_copilot::GithubCopilotAdapter;
pub use iflow::IFlowAdapter;
pub use kiro::KiroAdapter;
pub use opencode::OpenCodeAdapter;
pub use qwen_code::QwenCodeAdapter;
pub use vscode::VSCodeAdapter;
pub use windsurf::WindsurfAdapter;

pub struct ParsedSources {
    pub sources: Vec<LocalConfigSource>,
    pub servers: Vec<(MCPServer, u32)>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub trait AppAdapter {
    fn app(&self) -> SupportedApp;
    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)>;
    fn parse_source(
        &self,
        ctx: &PlatformContext,
        path: &str,
        priority: u32,
        content: &str,
    ) -> ParsedSources;
    fn plan_apply(
        &self,
        ctx: &PlatformContext,
        config: &MCPConfig,
        previous_config: Option<&MCPConfig>,
    ) -> WriteOperation;
}

fn collect_managed_server_ids(config: &MCPConfig, app: SupportedApp, ids: &mut BTreeSet<String>) {
    for server in &config.servers {
        if server.apps.get(&app).copied().unwrap_or(false) {
            ids.insert(server.id.clone());
        }
    }
}

fn managed_server_ids(
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
    app: SupportedApp,
) -> Vec<String> {
    let mut ids = BTreeSet::new();
    collect_managed_server_ids(config, app, &mut ids);
    if let Some(previous_config) = previous_config {
        collect_managed_server_ids(previous_config, app, &mut ids);
    }
    ids.into_iter().collect()
}

pub fn managed_json_field_write(
    path: String,
    field: &str,
    content: Value,
    app: SupportedApp,
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
) -> WriteOperation {
    WriteOperation {
        path,
        mode: "merge_json_object_entries".to_string(),
        field: Some(field.to_string()),
        remove_keys: Some(managed_server_ids(config, previous_config, app)),
        content: serde_json::to_string(&content).expect("serialize managed json field"),
    }
}

pub fn managed_toml_field_write(
    path: String,
    field: &str,
    content: Value,
    app: SupportedApp,
    config: &MCPConfig,
    previous_config: Option<&MCPConfig>,
) -> WriteOperation {
    WriteOperation {
        path,
        mode: "merge_toml_table_entries".to_string(),
        field: Some(field.to_string()),
        remove_keys: Some(managed_server_ids(config, previous_config, app)),
        content: serde_json::to_string(&content).expect("serialize managed toml field"),
    }
}

#[derive(Clone, Copy)]
struct StandardJsonAppProfile {
    include_transport_type: bool,
    tools: Option<&'static [&'static str]>,
}

fn standard_json_app_profile(app: SupportedApp) -> StandardJsonAppProfile {
    match app {
        SupportedApp::GithubCopilot => StandardJsonAppProfile {
            include_transport_type: true,
            tools: Some(&["*"]),
        },
        _ => StandardJsonAppProfile {
            include_transport_type: true,
            tools: None,
        },
    }
}

fn standard_json_transport_type(kind: &str) -> &'static str {
    match kind {
        "http" => "http",
        "sse" => "sse",
        "streamable-http" => "streamable-http",
        _ => "stdio",
    }
}

pub fn standard_mcp_servers(config: &MCPConfig, app: SupportedApp) -> Value {
    let mut servers = Map::new();
    let profile = standard_json_app_profile(app);

    for server in &config.servers {
        if server.enabled && server.apps.get(&app).copied().unwrap_or(false) {
            let mut value = if server.transport.kind == "stdio" {
                Map::from_iter([
                    (
                        "command".to_string(),
                        Value::String(
                            server
                                .command
                                .as_ref()
                                .map(|c| c.program.clone())
                                .unwrap_or_default(),
                        ),
                    ),
                    (
                        "args".to_string(),
                        Value::Array(
                            server
                                .command
                                .as_ref()
                                .map(|c| {
                                    c.args
                                        .iter()
                                        .cloned()
                                        .map(Value::String)
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default(),
                        ),
                    ),
                    (
                        "env".to_string(),
                        serde_json::to_value(
                            server
                                .command
                                .as_ref()
                                .map(|c| c.env.clone())
                                .unwrap_or_default(),
                        )
                        .expect("serialize command env"),
                    ),
                ])
            } else {
                Map::from_iter([(
                    "url".to_string(),
                    Value::String(server.transport.url.clone().unwrap_or_default()),
                )])
            };

            if profile.include_transport_type {
                value.insert(
                    "type".to_string(),
                    Value::String(standard_json_transport_type(&server.transport.kind).to_string()),
                );
            }

            if let Some(tools) = profile.tools {
                value.insert(
                    "tools".to_string(),
                    Value::Array(
                        tools
                            .iter()
                            .map(|tool| Value::String((*tool).to_string()))
                            .collect(),
                    ),
                );
            }

            servers.insert(server.id.clone(), Value::Object(value));
        }
    }
    Value::Object(servers)
}

pub fn opencode_mcp_servers(config: &MCPConfig, app: SupportedApp) -> Value {
    let mut servers = Map::new();
    for server in &config.servers {
        if server.enabled && server.apps.get(&app).copied().unwrap_or(false) {
            let value = if server.transport.kind == "stdio" {
                let mut command = vec![Value::String(
                    server
                        .command
                        .as_ref()
                        .map(|c| c.program.clone())
                        .unwrap_or_default(),
                )];
                command.extend(
                    server
                        .command
                        .as_ref()
                        .map(|c| {
                            c.args
                                .iter()
                                .cloned()
                                .map(Value::String)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default(),
                );
                serde_json::json!({
                    "type": "local",
                    "enabled": true,
                    "command": command,
                    "environment": server.command.as_ref().map(|c| c.env.clone()).unwrap_or_default(),
                })
            } else {
                serde_json::json!({
                    "type": "remote",
                    "enabled": true,
                    "url": server.transport.url.clone().unwrap_or_default(),
                })
            };
            servers.insert(server.id.clone(), value);
        }
    }
    Value::Object(servers)
}

pub fn adapters() -> Vec<Box<dyn AppAdapter>> {
    vec![
        Box::new(VSCodeAdapter),
        Box::new(CursorAdapter),
        Box::new(ClaudeCodeAdapter),
        Box::new(CodexAdapter),
        Box::new(ClaudeDesktopAdapter),
        Box::new(OpenCodeAdapter),
        Box::new(GithubCopilotAdapter),
        Box::new(GeminiCliAdapter),
        Box::new(AntigravityAdapter),
        Box::new(IFlowAdapter),
        Box::new(QwenCodeAdapter),
        Box::new(ClineAdapter),
        Box::new(WindsurfAdapter),
        Box::new(KiroAdapter),
    ]
}
