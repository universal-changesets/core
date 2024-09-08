use super::config::get_config;
use extism::*;
use semver::Version;

pub fn get_version_via_plugin() -> anyhow::Result<Version> {
    let config = get_config()?;
    let cached_plugin_path = config.cache_plugin_from_url()?;

    let url = Wasm::file(cached_plugin_path);

    let path = config.plugin.versioned_file;
    let parent_dir = path.parent().unwrap();

    let manifest = extism::Manifest::new([url]).with_allowed_path(
        parent_dir.to_str().unwrap().to_string(),
        parent_dir.to_str().unwrap(),
    );

    let mut plugin = extism::Plugin::new(manifest, [], true).unwrap();
    let response = plugin.call::<&str, &str>("get_version", path.to_str().unwrap());
    let parsed_version = Version::parse(response.unwrap().to_string().as_str())?;
    Ok(parsed_version)
}
