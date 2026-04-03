use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use reqwest::header::{HeaderMap, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use tar::Archive;

const REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/SimplePixelFont/spfc/main/registry.json";

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RegistryManifest {
    pub targets: HashMap<String, TargetInfo>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TargetInfo {
    pub description: String,
    pub repo: String,
    pub versions: Vec<String>,
    pub latest: String,
}

#[derive(Deserialize, Debug)]
struct GithubRelease {
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize, Debug)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

pub struct PluginManager {
    pub plugin_dir: PathBuf,
    pub registry_path: PathBuf,
    client: reqwest::Client,
}

/// Maps the current runtime platform to the Rust target triple used in artifact filenames.
/// Must match the targets produced by builder.yml.
fn current_rust_target() -> Result<&'static str> {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    match (arch, os) {
        ("x86_64",  "linux")   => Ok("x86_64-unknown-linux-gnu"),
        ("aarch64", "linux")   => Ok("aarch64-unknown-linux-gnu"),
        ("arm",     "linux")   => Ok("armv7-unknown-linux-gnueabihf"),
        ("x86",     "linux")   => Ok("i686-unknown-linux-gnu"),
        ("x86_64",  "macos")   => Ok("x86_64-apple-darwin"),
        ("aarch64", "macos")   => Ok("aarch64-apple-darwin"),
        ("x86_64",  "windows") => Ok("x86_64-pc-windows-gnu"),
        ("x86_64",  "freebsd") => Ok("x86_64-unknown-freebsd"),
        _ => anyhow::bail!("Unsupported platform: arch={} os={}", arch, os),
    }
}

/// Returns the expected dynamic library extension for the current platform.
fn lib_extension() -> &'static str {
    if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    }
}

impl PluginManager {
    pub fn new() -> Result<Self> {
        let proj_dirs = directories::ProjectDirs::from("org", "SimplePixelFont", "spfc")
            .context("Could not determine config directory")?;

        let plugin_dir = proj_dirs.data_dir().join("plugins");
        let registry_path = proj_dirs.data_dir().join("registry.json");

        fs::create_dir_all(&plugin_dir)?;

        let mut headers = HeaderMap::default();
        headers.insert(USER_AGENT, "spfc-cli".parse().unwrap());

        Ok(Self {
            plugin_dir,
            registry_path,
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()?,
        })
    }

    pub async fn update_registry(&self) -> Result<()> {
        log::info!("Updating local plugin registry...");
        let text = self.client.get(REGISTRY_URL).send().await?.text().await?;
        fs::write(&self.registry_path, text)?;
        log::info!("Registry updated successfully.");
        Ok(())
    }

    pub async fn get_registry(&self) -> Result<RegistryManifest> {
        if !self.registry_path.exists() {
            self.update_registry().await?;
        }
        let data = fs::read_to_string(&self.registry_path)?;
        serde_json::from_str(&data).context("Failed to parse registry.json.")
    }

    pub async fn install(&self, target: &str, version: Option<String>) -> Result<()> {
        let registry = self.get_registry().await?;
        let info = registry
            .targets
            .get(target)
            .context(format!("Target '{}' not found in registry.", target))?;

        let target_version = version.unwrap_or_else(|| info.latest.clone());

        if !info.versions.contains(&target_version) {
            anyhow::bail!(
                "Version {} is not available for target {}. Available: {}",
                target_version,
                target,
                info.versions.join(", ")
            );
        }

        // Resolve the Rust target triple for the current platform.
        // This must match the triple used in artifact filenames by the build workflow.
        let rust_target = current_rust_target()
            .context("Could not determine a supported platform for plugin download.")?;

        log::info!("Resolving download for {} v{} ({})...", target, target_version, rust_target);

        let url = format!(
            "https://api.github.com/repos/{}/releases/tags/target-{}-v{}",
            info.repo, target, target_version
        );
        let release: GithubRelease = self
            .client
            .get(&url)
            .send()
            .await?
            .json()
            .await
            .context("Failed to fetch GitHub release metadata.")?;

        // Artifacts follow the naming pattern built by .ci/build.sh:
        //   spfc-target-{name}.{tag}.{rust_target}.tar.gz
        // We match on the rust target triple and the .tar.gz suffix.
        let asset = release
            .assets
            .iter()
            .find(|a| a.name.contains(rust_target) && a.name.ends_with(".tar.gz"))
            .context(format!(
                "No pre-compiled plugin found for platform '{}'. \
                 The release may not include a build for your system.",
                rust_target
            ))?;

        log::info!("Downloading {}...", asset.browser_download_url);
        let bytes = self
            .client
            .get(&asset.browser_download_url)
            .send()
            .await?
            .bytes()
            .await?;

        // Extract the tarball. The archive structure from build.sh is:
        //   {rust_target}/
        //     {lib_prefix}spfc_target_{name}.{ext}
        //     LICENSE-APACHE
        let ext = lib_extension();
        let target_dir = self.plugin_dir.join(target).join(&target_version);
        fs::create_dir_all(&target_dir)?;

        let cursor = Cursor::new(bytes);
        let gz = GzDecoder::new(cursor);
        let mut archive = Archive::new(gz);

        let mut installed = false;
        for entry in archive.entries()? {
            let mut entry = entry?;
            let entry_path = entry.path()?.to_path_buf();

            let filename = match entry_path.file_name() {
                Some(f) => f.to_string_lossy().into_owned(),
                None => continue,
            };

            // Extract only the dynamic library; skip the license and directories.
            if filename.ends_with(ext) {
                let dest = target_dir.join(&filename);
                entry.unpack(&dest)
                    .context(format!("Failed to extract {} to {:?}", filename, dest))?;
                log::info!("Installed {} v{} → {:?}", target, target_version, dest);
                installed = true;
            }
        }

        if !installed {
            anyhow::bail!(
                "Downloaded archive for '{}' v{} contained no '{}' file.",
                target,
                target_version,
                ext
            );
        }

        Ok(())
    }

    /// Returns a formatted string listing all available targets from the registry,
    /// including their descriptions, version history, and local installation status.
    pub async fn list_targets(&self) -> Result<String> {
        let registry = self.get_registry().await?;

        if registry.targets.is_empty() {
            return Ok("No targets found in registry.".to_string());
        }

        // Sort targets alphabetically for a stable display order.
        let mut targets: Vec<(&String, &TargetInfo)> = registry.targets.iter().collect();
        targets.sort_by_key(|(name, _)| *name);

        let ext = lib_extension();
        let total = targets.len();
        let separator = "─".repeat(60);

        let mut out = format!("Available Targets ({total})\n{separator}\n");

        for (name, info) in targets {
            // Check whether any version is installed locally.
            let install_status = self.installed_version(name, ext);

            let installed_label = match &install_status {
                Some(v) => format!(" [installed: v{}]", v),
                None => String::new(),
            };

            // Header line: name + latest version + installed marker
            out.push_str(&format!(
                "\n  {:<20} latest: v{}{}\n",
                name, info.latest, installed_label
            ));

            // Description
            out.push_str(&format!("  {}\n", info.description));

            // Available versions (all of them, condensed)
            out.push_str(&format!(
                "  Versions : {}\n",
                info.versions.join(", ")
            ));

            // Source repo
            out.push_str(&format!("  Repo     : {}\n", info.repo));
        }

        out.push_str(&format!("\n{separator}\n"));
        out.push_str("  Run `spfc install <target>` to install the latest version.\n");
        out.push_str("  Run `spfc install <target>@<version>` to pin a specific version.\n");

        Ok(out)
    }

    /// Checks whether a target has any version installed locally,
    /// returning the highest installed version string if so.
    fn installed_version(&self, target: &str, ext: &str) -> Option<String> {
        let target_base = self.plugin_dir.join(target);
        if !target_base.exists() {
            return None;
        }

        let mut versions: Vec<String> = fs::read_dir(&target_base)
            .ok()?
            .flatten()
            .filter_map(|entry| {
                let version = entry.file_name().to_string_lossy().into_owned();
                // Confirm the library file actually exists inside this version folder.
                let lib_dir = entry.path();
                let has_lib = fs::read_dir(&lib_dir)
                    .map(|mut d| d.any(|e| {
                        e.map(|e| e.file_name().to_string_lossy().ends_with(ext))
                            .unwrap_or(false)
                    }))
                    .unwrap_or(false);
                if has_lib { Some(version) } else { None }
            })
            .collect();

        versions.sort();
        versions.pop()
    }
}