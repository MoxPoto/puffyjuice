// Simply copies and converts to a correct format for a binary module
const LUA_BIN_PATH: &str =
    "C:\\Program Files (x86)\\Steam\\steamapps\\common\\GarrysMod\\garrysmod\\lua\\bin";

use serde::Deserialize;
use std::{env, fs, path};

#[derive(Deserialize)]
struct CargoRoot {
    package: PackageInfo,
}

#[derive(Deserialize)]
struct PackageInfo {
    name: String,
}

// Get the project name by parsing the Cargo.toml
fn getProjectName(pathToTOML: &String) -> String {
    let tomlContents =
        fs::read_to_string(pathToTOML).expect("Could not read Cargo.toml to extract project name!");
    let parsedCargo: CargoRoot = toml::from_str(&tomlContents).unwrap();

    return parsedCargo.package.name;
}

fn main() -> std::io::Result<()> {
    let out_dir = env::var("CRATE_OUT_DIR").unwrap();
    let tomlPath = env::var("CRATE_MANIFEST_PATH").unwrap();
    let projectName = getProjectName(&tomlPath);

    // And finally, copy the file to the correct format
    fs::copy(
        format!("{}/{}.dll", out_dir, projectName),
        format!("{}/gmcl_{}_win64.dll", LUA_BIN_PATH, projectName),
    )?;

    return Ok(());
}
