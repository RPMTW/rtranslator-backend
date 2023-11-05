use std::{
    collections::HashMap,
    env::temp_dir,
    fs::{self, create_dir_all},
    io::BufReader,
    path::PathBuf,
    sync::Mutex,
};

use lazy_static::lazy_static;
use serde::Serialize;
use uuid::Uuid;
use zip::{read::ZipFile, ZipArchive};

use crate::minecraft::metadata::{parse_language_file, parse_namespace};

lazy_static! {
    pub static ref ARCHIVE_TASKS: Mutex<HashMap<String, ArchiveTask>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Serialize, Clone)]
pub struct ArchiveTask {
    pub stage: ArchiveTaskStage,
    pub progress: f32,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveTaskStage {
    Preparing,
    Downloading,
    Extracting,
    Saving,
    Completed,
    Failed,
}

pub async fn download_files(
    task_id: &str,
    downloads: Vec<(String, usize)>,
) -> anyhow::Result<Vec<PathBuf>> {
    let total_size: usize = downloads.iter().map(|(_, size)| size).sum();
    let mut downloaded_size = 0;
    let mut file_locations: Vec<PathBuf> = Vec::new();

    let dir = temp_dir().join("rtranslator-backend").join("archives");
    create_dir_all(&dir)?;

    for chuck in downloads.chunks(10) {
        let mut handles = Vec::new();
        for (url, size) in chuck {
            let dir = dir.clone();
            let url = url.clone();
            let size = *size;

            let handle = tokio::spawn(async move {
                let bytes = reqwest::get(url).await?.bytes().await?;

                let path = dir.join(Uuid::new_v4().to_string());

                tokio::fs::write(&path, bytes).await?;
                Ok::<_, anyhow::Error>((path, size))
            });
            handles.push(handle);
        }

        for handle in handles {
            let (path, size) = handle.await??;
            file_locations.push(path);
            downloaded_size += size;

            let mut tasks = ARCHIVE_TASKS.lock().unwrap();
            let task = tasks.get_mut(task_id).unwrap();
            let download_progress = downloaded_size as f32 / total_size as f32;
            task.progress = 0.05 + download_progress * 0.5;
        }
    }

    Ok(file_locations)
}

pub async fn extract_files(locations: Vec<PathBuf>) -> anyhow::Result<()> {
    for location in locations {
        let file = fs::File::open(&location)?;
        let reader = BufReader::new(&file);
        let mut archive = ZipArchive::new(reader)?;
        let namespace = parse_namespace(&mut archive);

        if let Ok(namespace) = namespace {
            let file_name = format!("assets/{}/lang/en_us.json", namespace);
            if let Ok(mut file) = archive.by_name(&file_name) {
                let map = parse_language_file(&mut file)?;
                println!("Map: {:?}", map);
            }
        }

        fs::remove_file(location)?;
    }

    Ok(())
}
