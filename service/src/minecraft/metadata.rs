use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
};

use anyhow::Context;
use zip::{read::ZipFile, ZipArchive};

pub fn parse_namespace(archive: &mut ZipArchive<BufReader<&File>>) -> anyhow::Result<String> {
    let fabric_file = archive.by_name("fabric.mod.json");
    if let Ok(mut file) = fabric_file {
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let json_value = serde_json::from_str::<serde_json::Value>(&content)?;

        let namespace = json_value
            .get("id")
            .context("Failed to get namespace")?
            .as_str();
        if let Some(namespace) = namespace {
            return Ok(namespace.to_string());
        }
    }

    // TODO: Parse forge, quark, etc.

    Err(anyhow::anyhow!("Failed to parse namespace"))
}

pub fn parse_language_file(file: &mut ZipFile) -> anyhow::Result<HashMap<String, String>> {
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let json_value = serde_json::from_str::<serde_json::Value>(&content)?;

    let mut map: HashMap<String, String> = HashMap::new();

    for (key, value) in json_value.as_object().unwrap() {
        if let Some(value) = value.as_str() {
            map.insert(key.to_string(), value.to_string());
        }
    }

    Ok(map)
}
