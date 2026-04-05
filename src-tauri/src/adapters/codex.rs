use crate::adapters::{managed_toml_field_write, standard_mcp_servers, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app, extract_codex_mcp_json, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct CodexAdapter;

impl AppAdapter for CodexAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::Codex
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            (
                ctx.home_dir
                    .join(".codex/mcp.json")
                    .to_string_lossy()
                    .to_string(),
                30,
            ),
            (
                ctx.user_app_config_path(SupportedApp::Codex)
                    .to_string_lossy()
                    .to_string(),
                35,
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
        let normalized = if path.ends_with("config.toml") {
            match extract_codex_mcp_json(content) {
                Ok(value) => value,
                Err(error) => {
                    return ParsedSources {
                        sources: vec![LocalConfigSource {
                            app: "codex".to_string(),
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
            }
        } else {
            content.to_string()
        };
        let parsed = parse_mcp_json(&normalized);
        ParsedSources {
            sources: vec![LocalConfigSource {
                app: "codex".to_string(),
                path: path.to_string(),
                exists: true,
                format: "json".to_string(),
                priority,
                content: Some(normalized),
            }],
            servers: enable_servers_for_app(parsed.servers, SupportedApp::Codex)
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
        managed_toml_field_write(
            ctx.user_app_config_path(SupportedApp::Codex)
                .to_string_lossy()
                .to_string(),
            "mcp_servers",
            standard_mcp_servers(config, SupportedApp::Codex),
            SupportedApp::Codex,
            config,
            previous_config,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::CodexAdapter;
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
    fn parses_codex_toml_source() {
        let parsed = CodexAdapter.parse_source(
            &ctx(),
            "/Users/test/.codex/config.toml",
            35,
            r#"[mcp_servers.playwright]
command = "npx"
args = ["@playwright/mcp@latest"]
"#,
        );
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.servers.len(), 1);
    }

    #[test]
    fn plans_codex_toml_merge() {
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Codex, true);
        let op = CodexAdapter.plan_apply(
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
        assert_eq!(op.mode, "merge_toml_table_entries");
        assert_eq!(op.field.as_deref(), Some("mcp_servers"));
    }
}
