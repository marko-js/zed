use zed_extension_api::{self as zed, LanguageServerId, Result};

// @marko/language-server speaks LSP over stdio (`createConnection` auto-detects
// the `--stdio` transport) and bundles its own TypeScript, so it runs without
// any workspace setup. We install it from npm into the extension's working
// directory and launch it with Node.
const MARKO_SERVER: NpmPackage = NpmPackage {
    name: "@marko/language-server",
    entry: "node_modules/@marko/language-server/bin.js",
    fits: |version| major(version) == Some(3),
    fallback: "^3",
    wanted: "3.x",
};

// @marko/ts-plugin is a TypeScript Server plugin that resolves `.marko` imports
// with types inside `.ts`/`.tsx` files. We install it alongside the server and
// register it with whichever TypeScript language server Zed runs (vtsls or
// typescript-language-server).
const TS_PLUGIN: NpmPackage = NpmPackage {
    name: "@marko/ts-plugin",
    entry: "node_modules/@marko/ts-plugin/package.json",
    fits: |version| major(version) == Some(3),
    fallback: "^3",
    wanted: "3.x",
};

/// A package installed from npm into the extension's working directory.
struct NpmPackage {
    name: &'static str,
    /// A file the package contains, relative to the working directory: for a
    /// server, the script to run with Node.
    entry: &'static str,
    /// Whether the extension can use a version: the major it is written for.
    /// A newer major may change what the extension has to set up, so it is
    /// only picked up by a release of the extension.
    fits: fn(&str) -> bool,
    /// What to install when npm's `latest` does not fit: a range or dist-tag
    /// that npm resolves to the newest version that does.
    fallback: &'static str,
    /// The versions that fit, for errors.
    wanted: &'static str,
}

impl NpmPackage {
    /// Installs or updates the package, and returns the absolute path of its
    /// entry. `status` is the language server that shows the progress.
    ///
    /// Only the first call in a session (while `checked` is false) asks npm
    /// for an update, so later calls do not wait on the registry.
    /// Zed's API reports only npm's `latest` version: when it fits it is
    /// installed unless already there; otherwise `fallback` is installed,
    /// which npm resolves to the newest fitting release. Whether that differs
    /// from the installed copy is unknown until npm resolves it, so the
    /// fallback is reinstalled once a session; npm leaves an up-to-date copy
    /// alone. When the registry or the install fails (eg offline), a fitting
    /// copy already installed is used.
    fn install(&self, checked: &mut bool, status: Option<&LanguageServerId>) -> Result<String> {
        let report = |state| {
            if let Some(id) = status {
                zed::set_language_server_installation_status(id, &state);
            }
        };
        let path = std::env::current_dir()
            .unwrap()
            .join(self.entry)
            .to_string_lossy()
            .into_owned();
        let installed = self.installed_version()?;
        let installed_fits = installed.as_deref().is_some_and(self.fits);
        if *checked && installed_fits {
            return Ok(path);
        }

        report(zed::LanguageServerInstallationStatus::CheckingForUpdate);
        let wanted = match zed::npm_package_latest_version(self.name) {
            Ok(latest) if (self.fits)(&latest) => latest,
            Ok(_) => self.fallback.to_string(),
            Err(error) if !installed_fits => {
                return Err(match installed {
                    Some(version) => format!(
                        "'{}' {} is needed, but {version} is installed and checking \
                         npm failed: {error}",
                        self.name, self.wanted
                    ),
                    None => error,
                });
            }
            Err(_) => {
                *checked = true;
                return Ok(path);
            }
        };

        if installed.as_deref() != Some(wanted.as_str()) {
            report(zed::LanguageServerInstallationStatus::Downloading);
            if let Err(error) = zed::npm_install_package(self.name, &wanted) {
                if !installed_fits {
                    return Err(error);
                }
            }
        }

        match self.installed_version()? {
            Some(version) if (self.fits)(&version) => {
                *checked = true;
                Ok(path)
            }
            Some(version) => Err(format!(
                "'{}' {} is needed, but npm installed {version}",
                self.name, self.wanted
            )),
            None => Err(format!(
                "installed package '{}' did not contain expected path '{}'",
                self.name, self.entry
            )),
        }
    }

    /// The installed version, if the entry exists.
    fn installed_version(&self) -> Result<Option<String>> {
        if !std::fs::metadata(self.entry).is_ok_and(|stat| stat.is_file()) {
            return Ok(None);
        }
        zed::npm_package_installed_version(self.name)
    }
}

struct MarkoExtension {
    /// Whether each package was checked against npm since Zed loaded the
    /// extension, which it does once a session and again after an update.
    checked_marko_server: bool,
    checked_ts_plugin: bool,
}

impl MarkoExtension {
    // Ensure @marko/ts-plugin is installed into the extension's working
    // directory and return that directory, which the TypeScript server uses as
    // the plugin probe location (it loads `<location>/node_modules/<name>`).
    // Its progress is not reported: these hooks run for the TypeScript
    // server, so nothing would clear a status set on the Marko server.
    fn ts_plugin_location(&mut self) -> Result<String> {
        TS_PLUGIN.install(&mut self.checked_ts_plugin, None)?;
        Ok(std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string())
    }
}

impl zed::Extension for MarkoExtension {
    fn new() -> Self {
        Self {
            checked_marko_server: false,
            checked_ts_plugin: false,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let server_path =
            MARKO_SERVER.install(&mut self.checked_marko_server, Some(language_server_id))?;
        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args: vec![server_path, "--stdio".to_string()],
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
                    "name": TS_PLUGIN.name,
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
                            "name": TS_PLUGIN.name,
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

fn major(version: &str) -> Option<u64> {
    version.split('.').next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packages_fit_their_major_only() {
        for package in [MARKO_SERVER, TS_PLUGIN] {
            assert!((package.fits)("3.0.0"), "{}", package.name);
            assert!((package.fits)("3.4.1-next.0"), "{}", package.name);
            assert!(!(package.fits)("2.9.0"), "{}", package.name);
            assert!(!(package.fits)("4.0.0"), "{}", package.name);
            assert!(!(package.fits)("latest"), "{}", package.name);
        }
    }
}
