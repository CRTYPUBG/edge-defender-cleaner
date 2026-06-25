use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use reqwest::header::USER_AGENT;

const GITHUB_API_URL: &str = "https://api.github.com/repos/CRTYPUBG/edge-defender-cleaner/releases/latest";
const VERSION_FILE: &str = "ver_edc.crty";

#[derive(Debug, Deserialize)]
pub struct GithubAsset {
    pub browser_download_url: String,
}

#[derive(Debug, Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub name: String,
    pub published_at: String,
    pub html_url: String,
    #[serde(default)]
    pub assets: Vec<GithubAsset>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LocalVersion {
    #[serde(default = "default_app")]
    pub app: String,
    #[serde(default = "default_repo")]
    pub repo: String,
    pub version: String,
    #[serde(default)]
    pub latest_version: String,
    #[serde(default)]
    pub update_available: bool,
    #[serde(default)]
    pub release_name: String,
    #[serde(default)]
    pub download_url: String,
    #[serde(default)]
    pub published_at: String,
}

fn default_app() -> String {
    "edge-defender-cleaner".to_string()
}

fn default_repo() -> String {
    "CRTYPUBG/edge-defender-cleaner".to_string()
}

/// Locate the ver_edc.crty file next to the executable
fn get_version_file_path() -> PathBuf {
    crate::ops::find_file(VERSION_FILE)
}

/// Async function to check for updates from GitHub API
pub async fn check_for_updates_async() -> Result<bool> {
    let version_file = get_version_file_path();
    
    // 1. Load local version file
    let mut local_version: LocalVersion = if version_file.exists() {
        let content = fs::read_to_string(&version_file)
            .context("Sürüm dosyası okunamadı (ver_edc.crty)")?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        LocalVersion {
            version: "v13.0.0".to_string(), // Fallback
            ..Default::default()
        }
    };

    // 2. Fetch latest release from GitHub API
    let client = reqwest::Client::new();
    let response = client
        .get(GITHUB_API_URL)
        .header(USER_AGENT, "edge-defender-cleaner-updater")
        .send()
        .await
        .context("GitHub API'ye bağlanılamadı. İnternet bağlantınızı kontrol edin.")?;

    if !response.status().is_success() {
        anyhow::bail!("GitHub API Hatası: {}", response.status());
    }

    let release: GithubRelease = response.json()
        .await
        .context("GitHub API yanıtı JSON formatında ayrıştırılamadı.")?;

    // 3. Compare versions
    let mut is_update_available = false;
    
    // Simple string inequality check. Ideally semantic versioning (semver) is better, 
    // but tag_name != local version matches the user requirement.
    if release.tag_name != local_version.version {
        is_update_available = true;
    }

    // 4. Generate updated local JSON file
    local_version.latest_version = release.tag_name.clone();
    local_version.update_available = is_update_available;
    local_version.release_name = release.name.clone();
    local_version.published_at = release.published_at.clone();
    
    if let Some(asset) = release.assets.first() {
        local_version.download_url = asset.browser_download_url.clone();
    } else {
        local_version.download_url = release.html_url.clone();
    }

    // 5. Save updated JSON back to disk
    let updated_json = serde_json::to_string_pretty(&local_version)?;
    fs::write(&version_file, updated_json)
        .context("Sürüm dosyası güncellenirken hata oluştu (ver_edc.crty)")?;

    Ok(is_update_available)
}

/// Synchronous wrapper for the update checker
pub fn check_for_updates() {
    let rt = tokio::runtime::Runtime::new().expect("Tokio runtime başlatılamadı");
    
    match rt.block_on(check_for_updates_async()) {
        Ok(has_update) => {
            if has_update {
                println!();
                println!("  [!] YENİ BİR GÜNCELLEME BULUNDU!");
                println!("  [!] Lütfen 'ver_edc.crty' dosyasını veya GitHub sayfasını kontrol edin.");
                println!();
            }
        }
        Err(e) => {
            eprintln!("  [!] Güncelleme kontrolü başarısız oldu: {}", e);
        }
    }
}
