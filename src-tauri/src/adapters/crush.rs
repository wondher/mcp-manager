use crate::adapters::{managed_json_field_write, standard_mcp_servers, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app, extract_json_field_mcp_json, parse_mcp_json};
use crate::platform::PlatformContext;

/// Adapter for Charmbracelet Crush.
///
/// Crush stores MCP servers in `~/.config/crush/crush.json` (global) or
/// `.crush.json` (project-local), nested under a top-level `mcp` key. Each
/// server uses the standard shape: `{ type, command, args, env }` for stdio and
/// `{ type, url }` for http/sse — identical to [`standard_mcp_servers`].
pub struct CrushAdapter;

impl AppAdapter for CrushAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::Crush
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            (
                ctx.workspace_file(".crush.json")
                    .to_string_lossy()
                    .to_string(),
                40,
            ),
            (
                ctx.user_app_config_path(SupportedApp::Crush)
                    .to_string_lossy()
                    .to_string(),
                30,
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
        // Crush nests servers under `mcp`; flatten it into `{"mcpServers": ...}`
        // so the shared JSON parser can read the standard per-server fields.
        let normalized = match extract_json_field_mcp_json(content, "mcp") {
            Ok(value) => value,
            Err(error) => {
                return ParsedSources {
                    sources: vec![LocalConfigSource {
                        app: "crush".to_string(),
                        path: path.to_string(),
                        exists: true,
                        format: "json".to_string(),
                        priority,
                        content: Some(content.to_string()),
                    }],
                    servers: vec![],
                    warnings: vec![],
                    errors: vec![error],
                }
            }
        };
        let parsed = parse_mcp_json(&normalized);
        ParsedSources {
            sources: vec![LocalConfigSource {
                app: "crush".to_string(),
                path: path.to_string(),
                exists: true,
                format: "json".to_string(),
                priority,
                content: Some(normalized),
            }],
            servers: enable_servers_for_app(parsed.servers, SupportedApp::Crush)
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
            ctx.user_app_config_path(SupportedApp::Crush)
                .to_string_lossy()
                .to_string(),
            "mcp",
            standard_mcp_servers(config, SupportedApp::Crush),
            SupportedApp::Crush,
            config,
            previous_config,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::CrushAdapter;
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
    fn parses_crush_json_source() {
        let parsed = CrushAdapter.parse_source(
            &ctx(),
            "/Users/test/.config/crush/crush.json",
            30,
            r#"{"$schema":"https://charm.land/crush.json","mcp":{"playwright":{"type":"stdio","command":"npx","args":["@playwright/mcp@latest"]}}}"#,
        );
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.servers.len(), 1);
        assert_eq!(parsed.servers[0].0.id, "playwright");
    }

    #[test]
    fn parses_empty_crush_json_source() {
        let parsed = CrushAdapter.parse_source(
            &ctx(),
            "/Users/test/.config/crush/crush.json",
            30,
            "",
        );
        assert!(parsed.errors.is_empty());
        assert!(parsed.servers.is_empty());
    }

    #[test]
    fn plans_crush_json_merge() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Crush, true);
        let op = CrushAdapter.plan_apply(
            &ctx(),
            &MCPConfig {
                version: 1,
                servers: vec![MCPServer {
                    description: None,
                    homepage: None,
                    id: "playwright".to_string(),
                    name: "Playwright".to_string(),
                    enabled: true,
                    transport: TransportSpec {
                        kind: "stdio".to_string(),
                        url: None,
                    },
                    command: Some(crate::core::CommandSpec {
                        program: "npx".to_string(),
                        args: vec!["@playwright/mcp@latest".to_string()],
                        env: HashMap::new(),
                    }),
                    apps,
                }],
            },
            None,
        );
        assert_eq!(op.mode, "merge_json_object_entries");
        assert_eq!(op.field.as_deref(), Some("mcp"));
    }

    #[test]
    fn plans_crush_remote_server_merge() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Crush, true);
        let op = CrushAdapter.plan_apply(
            &ctx(),
            &MCPConfig {
                version: 1,
                servers: vec![MCPServer {
                    description: None,
                    homepage: None,
                    id: "linear".to_string(),
                    name: "Linear".to_string(),
                    enabled: true,
                    transport: TransportSpec {
                        kind: "http".to_string(),
                        url: Some("https://mcp.linear.app/mcp".to_string()),
                    },
                    command: None,
                    apps,
                }],
            },
            None,
        );
        assert_eq!(op.mode, "merge_json_object_entries");
        assert_eq!(op.field.as_deref(), Some("mcp"));
    }
}
