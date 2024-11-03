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
    /// The URL of the plugin to use. As a shorthand for github, you can use the following format: gh:{owner}/{repo}@{version}
    /// ->
    /// https://github.com/owner/repo/releases/download/version/plugin.wasm
    pub url: String,
    pub sha256: Option<String>,
}

impl Plugin {
    pub fn get_url(&self) -> anyhow::Result<String> {
        if self.url.starts_with("http") {
            return Ok(self.url.clone());
        }

        return parse_shorthand_github_url(&self.url);
    }
}

/// Parses a shorthand github url and returns the full url. Example:
/// gh:universal-changesets/rust-cargo-plugin@version
/// ->
/// https://github.com/owner/repo/releases/download/version/plugin.wasm
fn parse_shorthand_github_url(url: &str) -> anyhow::Result<String> {
    if !url.starts_with("gh:") {
        return Err(anyhow::anyhow!("invalid url"));
    }
    let url = url.replace("gh:", "");
    let version_parts: Vec<&str> = url.split("@").collect();
    if version_parts.len() != 2 {
        return Err(anyhow::anyhow!("invalid url"));
    }
    let version = version_parts[1];
    let remaining = version_parts[0];
    let parts: Vec<&str> = remaining.split("/").collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("invalid url"));
    }
    let owner = parts[0];
    let repo = parts[1];
    let url = format!("https://github.com/{owner}/{repo}/releases/download/{version}/plugin.wasm");
    Ok(url)
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

        let plugin_url = self.plugin.get_url()?;
        let response = reqwest::blocking::get(&plugin_url)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        "gh:owner/repo@version",
        "https://github.com/owner/repo/releases/download/version/versionfile.wasm"
    )]
    #[case(
        "https://github.com/owner/repo/releases/download/version/versionfile.wasm",
        "https://github.com/owner/repo/releases/download/version/versionfile.wasm"
    )]
    #[case(
        "gh:alex-way/changesets-go-versionfile-plugin@0.0.2",
        "https://github.com/alex-way/changesets-go-versionfile-plugin/releases/download/0.0.2/versionfile.wasm"
    )]
    fn test_get_url(#[case] input: &str, #[case] expected: &str) {
        let plugin = Plugin {
            url: input.to_string(),
            sha256: None,
            versioned_file: "".into(),
        };
        let result = plugin.get_url().unwrap();
        assert_eq!(result, expected);
    }
}
