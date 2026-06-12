use serde::Deserialize;

/// JSON format served at your manifest URL:
/// { "version": "0.2.0", "download_url": "https://...", "notes": "..." }
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateInfo {
    pub version:      String,
    pub download_url: String,
    pub notes:        String,
}

// During development, host this locally (e.g. with `npx serve`).
// In production, point to your release server or GitHub releases API.
const MANIFEST_URL: &str = "https://your-server.com/nota/version.json";

/// Blocking check — always call this from a background thread, never the UI thread.
pub fn check() -> Option<UpdateInfo> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent(concat!("Nota/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?;

    let info: UpdateInfo = client.get(MANIFEST_URL).send().ok()?.json().ok()?;

    let current = semver::Version::parse(env!("CARGO_PKG_VERSION")).ok()?;
    let latest  = semver::Version::parse(&info.version).ok()?;

    if latest > current { Some(info) } else { None }
}

/// Download the update zip to the OS temp folder.
/// Returns the path to the downloaded file.
pub fn download(info: &UpdateInfo) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let dest = std::env::temp_dir().join("nota_update.zip");

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let bytes = client.get(&info.download_url).send()?.bytes()?;
    std::fs::write(&dest, &bytes)?;
    Ok(dest)
}