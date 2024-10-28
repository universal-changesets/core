use std::path::PathBuf;

use super::config::get_config;
use extism::*;
use extism_convert::Json;
use semver::Version;

pub fn setup_plugin() -> anyhow::Result<(Plugin, PathBuf)> {
    let config = get_config()?;
    let cached_plugin_path = config.cache_plugin_from_url()?;

    let plugin_file = Wasm::file(cached_plugin_path);

    let versioned_file_path = config.plugin.versioned_file;

    if !versioned_file_path.is_file() {
        return Err(anyhow::anyhow!("versioned file path is not a file"));
    }

    let versioned_file_parent_path = versioned_file_path.parent().unwrap();

    let manifest = extism::Manifest::new([plugin_file]).with_allowed_path(
        versioned_file_parent_path.to_str().unwrap().to_string(),
        versioned_file_parent_path,
    );

    let plugin = extism::Plugin::new(manifest, [], true).unwrap();
    return Ok((plugin, versioned_file_path));
}

pub fn get_version_via_plugin() -> anyhow::Result<Version> {
    let (mut plugin, versioned_file_path) = setup_plugin()?;
    let response =
        plugin.call::<&str, &str>("get_version", versioned_file_path.to_str().unwrap())?;

    let parsed_version = Version::parse(response.to_string().as_str())?;

    Ok(parsed_version)
}

#[derive(Debug, serde::Serialize)]
struct SetVersionRequest {
    pub path: String,
    pub version: String,
}

pub fn set_version_via_plugin(version: &Version) -> anyhow::Result<()> {
    let (mut plugin, versioned_file_path) = setup_plugin()?;

    let request = SetVersionRequest {
        path: versioned_file_path.to_str().unwrap().to_string(),
        version: version.to_string(),
    };

    plugin.call::<Json<SetVersionRequest>, &str>("set_version", request.into())?;

    Ok(())
}
