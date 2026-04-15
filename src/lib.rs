use zed_extension_api::{
    self as zed, settings::LspSettings, Architecture, Command, LanguageServerId, Os, Result,
    Worktree,
};

const REPO: &str = "Pollux12/gmod-glua-ls";
const BINARY_NAME: &str = "glua_ls";

struct GluaExtension {
    cached_binary_path: Option<String>,
}

impl GluaExtension {
    /// Map Zed's (Os, Architecture) to the release asset suffix used by
    /// the gmod-glua-ls GitHub Actions CI.
    ///
    /// Asset naming follows the standard cargo-dist / cargo-release pattern:
    ///   glua_ls-<triple>[.exe]
    /// Targets built by the upstream CI (7 assets per release):
    ///   x86_64-unknown-linux-gnu
    ///   aarch64-unknown-linux-gnu
    ///   x86_64-apple-darwin
    ///   aarch64-apple-darwin
    ///   x86_64-pc-windows-msvc  (.exe)
    fn binary_asset_name(os: Os, arch: Architecture) -> Result<String> {
        let triple = match (os, arch) {
            (Os::Linux, Architecture::X8664) => "x86_64-unknown-linux-gnu",
            (Os::Linux, Architecture::Aarch64) => "aarch64-unknown-linux-gnu",
            (Os::Mac, Architecture::X8664) => "x86_64-apple-darwin",
            (Os::Mac, Architecture::Aarch64) => "aarch64-apple-darwin",
            (Os::Windows, Architecture::X8664) => "x86_64-pc-windows-msvc",
            _ => return Err(format!("Unsupported platform: {os:?} / {arch:?}")),
        };

        let exe = if os == Os::Windows { ".exe" } else { "" };
        Ok(format!("{BINARY_NAME}-{triple}{exe}"))
    }

    fn language_server_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<String> {
        // 1. Check if user has glua_ls on their PATH already.
        if let Some(path) = worktree.which(BINARY_NAME) {
            return Ok(path);
        }

        // 2. Return cached path if the binary is already downloaded.
        if let Some(path) = &self.cached_binary_path {
            if std::path::Path::new(path).exists() {
                return Ok(path.clone());
            }
        }

        // 3. Fetch the latest release from GitHub and download the binary.
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let os = zed::current_platform().0;
        let arch = zed::current_platform().1;
        let asset_name = Self::binary_asset_name(os, arch)?;

        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "No asset '{asset_name}' found in release {}",
                    release.version
                )
            })?;

        let install_dir = format!("{BINARY_NAME}-{}", release.version);
        let binary_path = format!("{install_dir}/{BINARY_NAME}");

        // Skip download if we already have this version.
        if !std::path::Path::new(&binary_path).exists() {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &install_dir,
                // The release assets are bare binaries (not archives).
                zed::DownloadedFileType::Uncompressed,
            )
            .map_err(|e| format!("Failed to download {asset_name}: {e}"))?;

            // Make the binary executable on Unix.
            zed::make_file_executable(&binary_path)?;
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for GluaExtension {
    fn new() -> Self {
        GluaExtension {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let binary = self.language_server_binary(language_server_id, worktree)?;

        // Allow the user to pass extra args via Zed LSP settings:
        //   "lsp": { "gmod-glua-ls": { "initialization_options": { ... } } }
        let settings = LspSettings::for_worktree("gmod-glua-ls", worktree)
            .ok()
            .and_then(|s| s.binary)
            .map(|b| b.arguments.unwrap_or_default())
            .unwrap_or_default();

        Ok(Command {
            command: binary,
            // glua_ls speaks LSP over stdio with no extra flags needed.
            args: settings,
            env: vec![],
        })
    }
}

zed::register_extension!(GluaExtension);
