use crate::adapters::{managed_json_field_write, standard_mcp_servers, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct VSCodeAdapter;

impl AppAdapter for VSCodeAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::Vscode
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            (
                ctx.workspace_file(".vscode/mcp.json")
                    .to_string_lossy()
                    .to_string(),
                10,
            ),
            (
                ctx.user_app_config_path(SupportedApp::Vscode)
                    .to_string_lossy()
                    .to_string(),
                20,
            ),
        ]
    }

    fn parse_source(
        &self,
        _ctx: &PlatformContext,
        path: &str,
        priority: u32,
        content: &str,
    ) -> ParsedSources {
        let parsed = parse_mcp_json(content);
        ParsedSources {
            sources: vec![LocalConfigSource {
                app: "vscode".to_string(),
                path: path.to_string(),
                exists: true,
                format: "json".to_string(),
                priority,
                content: Some(content.to_string()),
            }],
            servers: enable_servers_for_app(parsed.servers, SupportedApp::Vscode)
                .into_iter()
                .map(|server| (server, priority))
                .collect(),
            warnings: parsed.warnings,
            errors: parsed.errors,
        }
    }

    fn plan_apply(
        &self,
        ctx: &PlatformContext,
        config: &MCPConfig,
        previous_config: Option<&MCPConfig>,
    ) -> WriteOperation {
        managed_json_field_write(
            ctx.user_app_config_path(SupportedApp::Vscode)
                .to_string_lossy()
                .to_string(),
            "servers",
            standard_mcp_servers(config, SupportedApp::Vscode),
            SupportedApp::Vscode,
            config,
            previous_config,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::VSCodeAdapter;
    use crate::adapters::AppAdapter;
    use crate::core::{empty_apps, MCPConfig, MCPServer, SupportedApp, TransportSpec};
    use crate::platform::{PlatformContext, PlatformOs};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn ctx() -> PlatformContext {
        PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: PathBuf::from("/Users/test"),
            workspace_root: PathBuf::from("/workspace/project"),
        }
    }

    #[test]
    fn detects_workspace_and_user_sources() {
        let sources = VSCodeAdapter.detect_sources(&ctx());
        assert_eq!(sources.len(), 2);
        assert!(sources[0].0.ends_with(".vscode/mcp.json"));
        assert!(sources[1].0.ends_with("Code/User/mcp.json"));
    }

    #[test]
    fn plans_vscode_servers_payload() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Vscode, true);
        let op = VSCodeAdapter.plan_apply(
            &ctx(),
            &MCPConfig {
                version: 1,
                servers: vec![MCPServer {
                    description: None,
                    homepage: None,
                    id: "github".to_string(),
                    name: "GitHub".to_string(),
                    enabled: true,
                    transport: TransportSpec {
                        kind: "stdio".to_string(),
                        url: None,
                    },
                    command: Some(crate::core::CommandSpec {
                        program: "uvx".to_string(),
                        args: vec!["mcp-server-github".to_string()],
                        env: HashMap::new(),
                    }),
                    apps,
                }],
            },
            None,
        );
        assert_eq!(op.mode, "merge_json_object_entries");
        assert!(op.content.contains("github"));
    }
}
