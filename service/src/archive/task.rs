use std::{
    collections::{HashMap, HashSet},
    env::temp_dir,
    fs::{self, create_dir_all},
    io::BufReader,
    path::PathBuf,
    sync::Mutex,
};

use entity::{
    entry::text_entry,
    minecraft::{mod_loader::{ModLoader, ModLoaderVec}, minecraft_mod},
    misc::StringVec,
};
use lazy_static::lazy_static;
use sea_orm::{sea_query::OnConflict, DatabaseConnection, EntityTrait, Set};
use serde::Serialize;
use zip::ZipArchive;

use super::resource::ModDownloadInfo;
use crate::minecraft::metadata::{parse_language_file, parse_namespace};

lazy_static! {
    pub static ref ARCHIVE_TASKS: Mutex<HashMap<String, ArchiveTask>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Serialize, Clone)]
pub struct ArchiveTask {
    pub stage: ArchiveTaskStage,
    pub progress: f32,

    // This field is only set when the task is completed.
    pub mc_mod: Option<minecraft_mod::Model>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveTaskStage {
    Preparing,
    Downloading,
    Extracting,
    Saving,
    Completed,
    Failed,
    // Timeout,
}

impl ArchiveTaskStage {
    pub fn is_finished(&self) -> bool {
        matches!(self, ArchiveTaskStage::Completed | ArchiveTaskStage::Failed)
    }
}

pub fn get_archives_directory() -> PathBuf {
    temp_dir().join("rtranslator-backend").join("archives")
}

pub fn update_task_progress(task_id: &str, stage: Option<ArchiveTaskStage>, progress: f32) {
    let mut tasks = ARCHIVE_TASKS.lock().unwrap();
    let task = tasks.get_mut(task_id).unwrap();

    if let Some(stage) = stage {
        task.stage = stage;
    }
    task.progress = progress;
}

pub fn remove_task(task_id: &str) {
    let mut tasks = ARCHIVE_TASKS.lock().unwrap();
    tasks.remove(task_id);
}

pub async fn download_files(
    downloads: &[ModDownloadInfo],
    max_simultaneous_downloads: usize,
    progress_changed: impl Fn(f32),
) -> anyhow::Result<()> {
    let total_size: usize = downloads.iter().map(|x| x.size).sum();
    let mut downloaded_size = 0;

    create_dir_all(get_archives_directory())?;

    for chuck in downloads.chunks(max_simultaneous_downloads) {
        let mut handles = Vec::with_capacity(chuck.len());

        for (index, info) in chuck.iter().enumerate() {
            let url = info.url.clone();
            let path = info.path.clone();

            let handle = tokio::spawn(async move {
                let bytes = reqwest::get(url).await?.bytes().await?;
                tokio::fs::write(path, bytes).await?;

                Ok::<_, anyhow::Error>(index)
            });
            handles.push(handle);
        }

        for handle in handles {
            let index = handle.await??;
            let info = chuck.get(index).unwrap();

            downloaded_size += info.size;
            progress_changed(downloaded_size as f32 / total_size as f32);
        }
    }

    Ok(())
}

#[derive(Debug)]
pub struct TextEntryData {
    pub key: String,
    pub value: String,
    pub namespaces: HashSet<String>,
    pub game_versions: HashSet<semver::Version>,
    pub loaders: HashSet<ModLoader>,
}

pub async fn parse_language_files(
    downloads: &[ModDownloadInfo],
    progress_changed: impl Fn(f32),
) -> anyhow::Result<Vec<TextEntryData>> {
    let locations = downloads.iter().map(|x| x.path.clone());
    let mut maps = Vec::new();
    let mut namespaces = Vec::new();

    for location in locations {
        let file = fs::File::open(&location)?;
        let reader = BufReader::new(&file);
        let mut archive = ZipArchive::new(reader)?;
        let namespace = parse_namespace(&mut archive);

        if let Ok(namespace) = namespace {
            let file_name = format!("assets/{}/lang/en_us.json", namespace);
            if let Ok(mut file) = archive.by_name(&file_name) {
                let map = parse_language_file(&mut file)?;
                maps.push(map);
                namespaces.push(namespace);
            }
        }

        fs::remove_file(location)?;
    }

    let keys = maps
        .iter()
        .flat_map(|x| x.keys())
        .collect::<HashSet<_>>()
        .into_iter();
    let keys_len = keys.len();
    let mut entries = Vec::with_capacity(keys_len);

    for (index, key) in keys.enumerate() {
        let filtered_maps = maps
            .iter()
            .enumerate()
            .filter(|(_, map)| map.contains_key(key))
            .collect::<Vec<_>>();

        let latest_value = filtered_maps.last().unwrap().1.get(key).unwrap();
        let mut data = TextEntryData {
            key: key.clone(),
            value: latest_value.to_string(),
            namespaces: HashSet::new(),
            game_versions: HashSet::new(),
            loaders: HashSet::new(),
        };

        for (index, _) in filtered_maps {
            let namespace = namespaces.get(index).unwrap();
            let download_info = downloads.get(index).unwrap();

            data.namespaces.insert(namespace.clone());
            data.game_versions
                .insert(download_info.game_version.clone());
            data.loaders.insert(download_info.loader.clone());
        }

        entries.push(data);
        progress_changed(index as f32 / keys_len as f32);
    }

    Ok(entries)
}

pub async fn save_text_entries(
    db: &DatabaseConnection,
    entries: Vec<TextEntryData>,
    mod_id: i32,
) -> Result<(), sea_orm::DbErr> {
    let mut models = Vec::with_capacity(entries.len());

    for entry in entries {
        let model = text_entry::ActiveModel {
            key: Set(entry.key),
            value: Set(entry.value),
            namespaces: Set(StringVec(entry.namespaces.into_iter().collect())),
            game_versions: Set(StringVec(
                entry
                    .game_versions
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect(),
            )),
            loaders: Set(ModLoaderVec(entry.loaders.into_iter().collect())),
            mod_id: Set(mod_id),
        };

        models.push(model);
    }

    for chuck in models.chunks(1000).map(|chunk| chunk.to_vec()) {
        text_entry::Entity::insert_many(chuck)
            .on_conflict(
                OnConflict::column(text_entry::Column::Key)
                    .update_columns([
                        text_entry::Column::Value,
                        text_entry::Column::Namespaces,
                        text_entry::Column::GameVersions,
                        text_entry::Column::Loaders,
                    ])
                    .to_owned(),
            )
            .exec(db)
            .await?;
    }
    Ok(())
}
