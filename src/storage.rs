use std::path::PathBuf;
use crate::data::Ledger;
use crate::settings::Settings;

pub fn data_dir() -> PathBuf {
    let base = dirs::data_dir()
        .expect("Could not find a data directory on this system");
    let nota_dir = base.join("Nota");
    std::fs::create_dir_all(&nota_dir)
        .expect("Could not create Nota data directory");
    nota_dir
}

pub fn attachments_dir() -> PathBuf {
    let dir = data_dir().join("attachments");
    std::fs::create_dir_all(&dir)
        .expect("Could not create attachments directory");
    dir
}

pub fn csv_path() -> PathBuf {
    data_dir().join("ledger.csv")
}

pub fn save(ledger: &Ledger) -> Result<(), Box<dyn std::error::Error>> {
    let path = csv_path();
    let mut writer = csv::Writer::from_path(path)?;
    for entry in &ledger.entries {
        writer.serialize(entry)?;
    }
    writer.flush()?;
    Ok(())
}

pub fn load() -> Result<Ledger, Box<dyn std::error::Error>> {
    let path = csv_path();
    if !path.exists() {
        return Ok(Ledger::default());
    }
    let mut reader = csv::Reader::from_path(path)?;
    let mut entries = Vec::new();
    for result in reader.deserialize() {
        let entry = result?;
        entries.push(entry);
    }
    Ok(Ledger { entries })
}

// Copies a file into the attachments folder, returns just the filename
pub fn save_attachment(
    source: &std::path::Path
) -> Result<String, Box<dyn std::error::Error>> {
    let filename = source
        .file_name()
        .ok_or("Invalid filename")?
        .to_string_lossy()
        .to_string();
    let dest = attachments_dir().join(&filename);
    std::fs::copy(source, &dest)?;
    Ok(filename)
}

pub fn attachment_path(filename: &str) -> PathBuf {
    attachments_dir().join(filename)
}

// ← NEW: export to a user-chosen path (used by Settings > Export)
pub fn export_csv(
    ledger: &Ledger,
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = csv::Writer::from_path(path)?;
    for entry in &ledger.entries {
        writer.serialize(entry)?;
    }
    writer.flush()?;
    Ok(())
}

pub fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

pub fn save_settings(s: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(s)?;
    std::fs::write(settings_path(), json)?;
    Ok(())
}

// Returns default settings if the file doesn't exist or is corrupted
pub fn load_settings() -> Settings {
    let path = settings_path();
    if !path.exists() { return Settings::default(); }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}