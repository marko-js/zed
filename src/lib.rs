use zed_extension_api::{self as zed, LanguageServerId, Result};

// @marko/language-server speaks LSP over stdio (`createConnection` auto-detects
// the `--stdio` transport) and bundles its own TypeScript, so it runs without
// any workspace setup. We install it from npm into the extension's working
// directory and launch it with Node.
const PACKAGE_NAME: &str = "@marko/language-server";
const SERVER_PATH: &str = "node_modules/@marko/language-server/bin.js";

// @marko/ts-plugin is a TypeScript Server plugin that resolves `.marko` imports
// with types inside `.ts`/`.tsx` files. We install it alongside the server and
// register it with whichever TypeScript language server Zed runs (vtsls or
// typescript-language-server).
const TS_PLUGIN_PACKAGE_NAME: &str = "@marko/ts-plugin";

struct MarkoExtension {
    did_find_server: bool,
    did_find_ts_plugin: bool,
}

impl MarkoExtension {
    fn server_exists(&self) -> bool {
        std::fs::metadata(SERVER_PATH).is_ok_and(|stat| stat.is_file())
    }

    fn server_script_path(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        let server_exists = self.server_exists();
        if self.did_find_server && server_exists {
            return Ok(SERVER_PATH.to_string());
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let version = zed::npm_package_latest_version(PACKAGE_NAME)?;

        if !server_exists
            || zed::npm_package_installed_version(PACKAGE_NAME)?.as_deref() != Some(version.as_str())
        {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            let result = zed::npm_install_package(PACKAGE_NAME, &version);
            match result {
                Ok(()) => {
                    if !self.server_exists() {
                        return Err(format!(
                            "installed package '{PACKAGE_NAME}' did not contain expected path '{SERVER_PATH}'"
                        ));
                    }
                }
                Err(error) => {
                    // Keep using a previously installed copy if the update failed
                    // (e.g. offline); only surface the error if nothing is there.
                    if !self.server_exists() {
                        return Err(error);
                    }
                }
            }
        }

        self.did_find_server = true;
        Ok(SERVER_PATH.to_string())
    }

    // Ensure @marko/ts-plugin is installed into the extension's working
    // directory and return that directory, which the TypeScript server uses as
    // the plugin probe location (it loads `<location>/node_modules/<name>`).
    fn ts_plugin_location(&mut self) -> Result<String> {
        let installed = zed::npm_package_installed_version(TS_PLUGIN_PACKAGE_NAME)?;
        if !self.did_find_ts_plugin || installed.is_none() {
            let latest = zed::npm_package_latest_version(TS_PLUGIN_PACKAGE_NAME)?;
            if installed.as_deref() != Some(latest.as_str()) {
                if let Err(error) = zed::npm_install_package(TS_PLUGIN_PACKAGE_NAME, &latest) {
                    // Keep a previously installed copy if the update failed
                    // (e.g. offline); only surface the error if nothing is there.
                    if zed::npm_package_installed_version(TS_PLUGIN_PACKAGE_NAME)?.is_none() {
                        return Err(error);
                    }
                }
            }
            self.did_find_ts_plugin = true;
        }
        Ok(std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string())
    }
}

impl zed::Extension for MarkoExtension {
    fn new() -> Self {
        Self {
            did_find_server: false,
            did_find_ts_plugin: false,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let server_path = self.server_script_path(language_server_id)?;
        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args: vec![
                std::env::current_dir()
                    .unwrap()
                    .join(&server_path)
                    .to_string_lossy()
                    .to_string(),
                "--stdio".to_string(),
            ],
            env: Default::default(),
        })
    }

    // typescript-language-server takes tsserver plugins via initialization
    // options; teach it about Marko so `.marko` imports type-check in TS files.
    fn language_server_additional_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        target_language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        if target_language_server_id.as_ref() == "typescript-language-server" {
            let location = self.ts_plugin_location()?;
            return Ok(Some(serde_json::json!({
                "plugins": [{
                    "name": TS_PLUGIN_PACKAGE_NAME,
                    "location": location,
                }],
            })));
        }
        Ok(None)
    }

    // vtsls (Zed's default TypeScript server) takes tsserver plugins via
    // workspace configuration.
    fn language_server_additional_workspace_configuration(
        &mut self,
        _language_server_id: &LanguageServerId,
        target_language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        if target_language_server_id.as_ref() == "vtsls" {
            let location = self.ts_plugin_location()?;
            return Ok(Some(serde_json::json!({
                "vtsls": {
                    "tsserver": {
                        "globalPlugins": [{
                            "name": TS_PLUGIN_PACKAGE_NAME,
                            "location": location,
                            "enableForWorkspaceTypeScriptVersions": true,
                        }],
                    },
                },
            })));
        }
        Ok(None)
    }
}

zed::register_extension!(MarkoExtension);
