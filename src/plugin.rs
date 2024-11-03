use super::config::get_config;
use extism::*;
use extism_convert::Json;
use semver::Version;

pub fn setup_plugin(read_only: bool) -> anyhow::Result<Plugin> {
    let config = get_config()?;
    let cached_plugin_path = config.cache_plugin_from_url()?;

    let plugin_file = Wasm::file(cached_plugin_path);

    let mut current_dir = std::env::current_dir()?.to_str().unwrap().to_string();

    if read_only {
        current_dir = format!("ro:{}", current_dir);
    }

    let manifest = extism::Manifest::new([plugin_file])
        // Mounting to the root as the plugin is expected to be in the root of the fs
        .with_allowed_path(current_dir, "/");

    let plugin = extism::Plugin::new(manifest, [], true).unwrap();
    return Ok(plugin);
}

pub fn get_version_via_plugin() -> anyhow::Result<Version> {
    let mut plugin = setup_plugin(true)?;
    let response = plugin.call::<&str, &str>("get_version", "")?;

    let parsed_version = Version::parse(response.to_string().as_str())?;

    Ok(parsed_version)
}

#[derive(Debug, serde::Serialize)]
struct SetVersionRequest {
    pub version: String,
}

pub fn set_version_via_plugin(version: &Version) -> anyhow::Result<()> {
    let mut plugin = setup_plugin(false)?;

    let request = SetVersionRequest {
        version: version.to_string(),
    };

    plugin.call::<Json<SetVersionRequest>, &str>("set_version", request.into())?;

    Ok(())
}
