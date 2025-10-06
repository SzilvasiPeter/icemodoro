use directories_next::ProjectDirs;
use rfd::FileDialog;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Gets the application's data directory using ProjectDirs.
fn get_app_data_dir() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("org", "", "icemodoro").ok_or("Error at ProjectDirs!")?;
    let data_dir = proj_dirs.data_dir();
    fs::create_dir_all(data_dir)?;
    Ok(data_dir.to_path_buf())
}

/// Saves serializable data to a JSON file in the app's data directory.
pub fn save<T: Serialize>(filename: &str, data: &T) -> Result<()> {
    let mut path = get_app_data_dir()?;
    path.push(filename);

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, data)?;
    Ok(())
}

/// Loads data from a JSON file in the app's data directory.
/// Returns an error if file not found or corrupt.
pub fn load<T: DeserializeOwned + Default>(filename: &str) -> Result<T> {
    let mut path = get_app_data_dir()?;
    path.push(filename);

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}

/// Exports serializable data to a user-chosen file anywhere via save file dialog.
pub fn export<T: Serialize>(data: &T) -> Result<()> {
    let path = FileDialog::new()
        .set_title("Select location to export JSON file")
        .set_file_name("export.json")
        .save_file()
        .ok_or("Export cancelled")?;

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, data)?;
    Ok(())
}

/// Imports data from a user-selected JSON file anywhere,
/// then saves the imported data inside the app data directory under `filename`.
pub fn import<T: DeserializeOwned + Serialize>(filename: &str) -> Result<T> {
    let path = FileDialog::new()
        .set_title("Select JSON file to import")
        .pick_file()
        .ok_or("Import cancelled")?;

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data: T = serde_json::from_reader(reader)?;

    // Save the imported data inside app data dir
    save(filename, &data)?;
    Ok(data)
}
