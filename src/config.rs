use home::home_dir;
use serde::{Deserialize, Serialize};
use sha256::Sha256Digest;
use std::default::Default;
use std::io::Read;
use std::path::PathBuf;

pub const CHANGESET_DIRECTORY: &str = ".changeset";
pub const CONFIG_FILENAME: &str = "config.json";

#[derive(Serialize, Deserialize)]
pub struct Plugin {
    #[serde(rename = "versionedFile")]
    pub versioned_file: PathBuf,
    pub url: String,
    pub sha256: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub plugin: Plugin,
}

impl Config {
    pub fn cache_plugin_from_url(&self) -> anyhow::Result<PathBuf> {
        let home_dir = home_dir().ok_or(anyhow::anyhow!("no home dir"))?;

        let cache_dir = home_dir.join(".cache").join("changesets");

        if let Some(sha256) = self.plugin.sha256.to_owned() {
            let sha_plugin_path = cache_dir.join(sha256).join("plugin.wasm");
            if sha_plugin_path.exists() {
                return Ok(sha_plugin_path);
            }
        }

        let temp_dir = home_dir.join(".cache").join("changesets");
        std::fs::create_dir_all(&temp_dir)?;

        let response = reqwest::blocking::get(&self.plugin.url)?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("invalid status code"));
        }
        let body = response.bytes()?;
        let sum = body.digest();

        if let Some(sha256) = self.plugin.sha256.to_owned() {
            if sha256 != sum {
                return Err(anyhow::anyhow!("invalid checksum"));
            }
        }

        let plugin_path = cache_dir.join(sum);

        std::fs::create_dir_all(&plugin_path)?;

        std::fs::write(plugin_path.join("plugin.wasm"), self.plugin.url.as_bytes())?;
        Ok(plugin_path)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            plugin: Plugin {
                versioned_file: "".into(),
                url: "".into(),
                sha256: None,
            },
        }
    }
}

pub fn get_config() -> anyhow::Result<Config> {
    let filepath = PathBuf::from(CHANGESET_DIRECTORY).join(CONFIG_FILENAME);
    if !PathBuf::from(CHANGESET_DIRECTORY).exists() {
        std::fs::create_dir(CHANGESET_DIRECTORY)?;
    }
    let file = std::fs::File::open(filepath)?;
    let mut reader = std::io::BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    let config: Config = serde_json::from_str(&contents)?;
    Ok(config)
}
