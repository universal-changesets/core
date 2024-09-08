use extism::*;
use semver::Version;

pub fn get_version_via_plugin(path: &str) -> anyhow::Result<Version> {
    // todo: add hash from config, wasm module from config, caching etc
    let url = Wasm::file("./rust_pdk_template.wasm");
    let manifest =
        extism::Manifest::new([url]).with_allowed_path(".changeset/".to_string(), ".changeset/");

    let mut plugin = extism::Plugin::new(manifest, [], true).unwrap();
    let response = plugin.call::<&str, &str>("get_version", path);
    let parsed_version = Version::parse(response.unwrap().to_string().as_str())?;
    Ok(parsed_version)
}
