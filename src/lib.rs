use schemars::JsonSchema;
use serde::Deserialize;
use std::env;
use zed_extension_api::{
    self as zed, Command, ContextServerConfiguration, ContextServerId, Project, Result, serde_json,
    settings::ContextServerSettings,
};

const PROJECT_NAME: &str = "mcp-server-master-go";
const PACKAGE_NAME: &str = "@mastergo/magic-mcp";
const PACKAGE_VERSION: &str = "0.0.1";
const SERVER_PATH: &str = "node_modules/@mastergo/magic-mcp/dist/index.js";

struct MasterGoModelContextExtension;

impl zed::Extension for MasterGoModelContextExtension {
    fn new() -> Self {
        Self
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Command> {
        let version = zed::npm_package_installed_version(PACKAGE_NAME)?;
        if version.as_deref() != Some(PACKAGE_VERSION) {
            zed::npm_install_package(PACKAGE_NAME, PACKAGE_VERSION)?;
        }

        let settings = ContextServerSettings::for_project(PROJECT_NAME, project)?;
        let Some(settings) = settings.settings else {
            return Err("missing `MASTER_GO_TOKEN` setting".into());
        };

        let settings: MasterGoContextServerSettings =
            serde_json::from_value(settings).map_err(|e| e.to_string())?;

        Ok(Command {
            command: zed::node_binary_path()?,
            args: vec![
                env::current_dir()
                    .unwrap()
                    .join(SERVER_PATH)
                    .to_string_lossy()
                    .to_string(),
            ],
            env: vec![("MASTER_GO_TOKEN".to_string(), settings.master_go_token)],
        })
    }
    fn context_server_configuration(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Option<ContextServerConfiguration>> {
        let installation_instructions =
            include_str!("../configuration/installation_instructions.md").to_string();
        let default_settings = include_str!("../configuration/default_settings.jsonc").to_string();
        let settings_schema =
            serde_json::to_string(&schemars::schema_for!(MasterGoContextServerSettings))
                .map_err(|e| e.to_string())?;

        Ok(Some(ContextServerConfiguration {
            installation_instructions,
            default_settings,
            settings_schema,
        }))
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct MasterGoContextServerSettings {
    MASTER_GO_TOKEN: String,
}

zed::register_extension!(MasterGoModelContextExtension);
