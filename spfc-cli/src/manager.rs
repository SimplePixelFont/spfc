use anyhow::{Context, Result};
// use reqwest::Certificate;
use reqwest::header::{HeaderMap, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
// use rustls::crypto::ring::default_provider;

const REGISTRY_URL: &str = "https://raw.githubusercontent.com/SimplePixelFont/spfc/main/registry.json";

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

impl PluginManager {
    pub fn new() -> Result<Self> {
        // default_provider().install_default()
        // .expect("Failed to install rustls crypto provider");

        let proj_dirs = directories::ProjectDirs::from("org", "SimplePixelFont", "spfc")
            .context("Could not determine config directory")?;
        
        let plugin_dir = proj_dirs.data_dir().join("plugins");
        let registry_path = proj_dirs.data_dir().join("registry.json");
        
        fs::create_dir_all(&plugin_dir)?;

        let mut headers = HeaderMap::default();
        headers.insert(USER_AGENT, "spfc-cli".parse().unwrap());

        // let certs = webpki_root_certs::TLS_SERVER_ROOT_CERTS
        // .iter()
        // .map(|cert| Certificate::from_der(cert).unwrap());

        let client = reqwest::Client::builder()
            // .tls_certs_only(certs) // Use webpki roots exclusively
            // .tls_backend_rustls() // Use rustls backend
            .default_headers(headers)
            .build()?;
        
        Ok(Self {
            plugin_dir,
            registry_path,
            client,
        })
    }

    pub async fn update_registry(&self) -> Result<()> {
        log::info!("🔄 Updating local plugin registry...");
        let text = self.client.get(REGISTRY_URL).send().await?.text().await?;
        fs::write(&self.registry_path, text)?;
        log::info!("✅ Registry updated successfully!");
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
        let info = registry.targets.get(target)
            .context(format!("Target '{}' not found in registry.", target))?;
            
        let target_version = version.unwrap_or_else(|| info.latest.clone());
        
        if !info.versions.contains(&target_version) {
            anyhow::bail!("Version {} is not available for target {}.", target_version, target);
        }

        log::info!("🔍 Resolving download for {} v{}...", target, target_version);

        let url = format!("https://api.github.com/repos/{}/releases/tags/pkg-{}-v{}", info.repo, target, target_version);
        let release: GithubRelease = self.client.get(&url).send().await?.json().await?;
        
        let arch = std::env::consts::ARCH; 
        let extension = if cfg!(target_os = "windows") { "dll" } else if cfg!(target_os = "macos") { "dylib" } else { "so" };
        
        let asset = release.assets.iter()
            .find(|a| a.name.contains(arch) && a.name.ends_with(extension))
            .context(format!("Could not find a pre-compiled binary for {} on your system.", arch))?;

        log::info!("🚀 Downloading from {}...", asset.browser_download_url);
        let bytes = self.client.get(&asset.browser_download_url).send().await?.bytes().await?;
        
        let filename = format!("spfc_target_{}.{}", target, extension);
        
        // NEW: Save into a version-specific folder
        let target_dir = self.plugin_dir.join(target).join(&target_version);
        fs::create_dir_all(&target_dir)?;
        let dest_path = target_dir.join(&filename);
        
        let mut temp_file = tempfile::NamedTempFile::new_in(&target_dir)?;
        temp_file.write_all(&bytes)?;
        temp_file.persist(&dest_path)?;
        
        log::info!("✅ Successfully installed {} v{} to {:?}", target, target_version, dest_path);
        Ok(())
    }
}