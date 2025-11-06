use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::{fs as async_fs, sync::Mutex}; // Use tokio for async file I/O

const DEFAULT_FILE_PATH: &str = "./settings.yaml";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub security_token: String,
    pub host: String,
    pub services: Vec<Service>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Service {
    pub service_name: String,
    pub cmd: String,
}

#[derive(Debug, Deserialize)]
pub struct Root {
    pub settings: Settings,
}

lazy_static! {
    static ref CACHED_SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);
}

pub async fn read_yaml() -> Result<Settings, Box<dyn Error>> {
    // Await the lock acquisition.
    let mut cached_settings = CACHED_SETTINGS.lock().await;

    // If cached settings are available, return them
    if let Some(settings) = cached_settings.as_ref() {
        return Ok(settings.clone());
    }

    // If not cached, read from the file asynchronously
    let yaml_content = async_fs::read_to_string(DEFAULT_FILE_PATH).await?;
    let root: Root = serde_yaml::from_str(&yaml_content)?;

    // Cache the settings for future use
    let settings = root.settings.clone();
    *cached_settings = Some(settings.clone());

    Ok(settings)
}
