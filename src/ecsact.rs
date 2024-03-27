use std::fs;
use zed_extension_api::{self as zed, Result};

struct EcsactExtension {
    cached_binary_path: Option<String>,
}

impl EcsactExtension {
    fn language_server_binary_path(&mut self, config: zed::LanguageServerConfig) -> Result<String> {
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            &config.name,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let (platform, arch) = zed::current_platform();
        let release = zed::latest_github_release(
            "ecsact-dev/ecsact_lsp_server",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let exe_suffix = match platform {
            zed::Os::Windows => ".exe",
            _ => "",
        };
        let asset_name = format!(
            "ecsact_lsp_server_{os}_{arch}{exe_suffix}",
            arch = match arch {
                zed::Architecture::Aarch64 => "arm64", // not supported yet
                zed::Architecture::X86 => "x86",       // not supported yet
                zed::Architecture::X8664 => "x64",
            },
            os = match platform {
                zed::Os::Mac => "macos",
                zed::Os::Linux => "linux",
                zed::Os::Windows => "windows",
            },
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("ecsact_lsp_server_{}", release.version);
        let binary_path = format!("{version_dir}/ecsact_lsp_server{exe_suffix}");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                &config.name,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &binary_path,
                zed::DownloadedFileType::Uncompressed,
            )
            .map_err(|e| format!("failed to download file: {e}"))?;
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for EcsactExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        config: zed::LanguageServerConfig,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: self.language_server_binary_path(config)?,
            args: vec![],
            env: Default::default(),
        })
    }
}

zed::register_extension!(EcsactExtension);
