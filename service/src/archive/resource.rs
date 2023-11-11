use std::{collections::HashSet, path::PathBuf};

use entity::minecraft::{
    minecraft_mod::{self, ModStatus},
    mod_loader::ModLoader,
    mod_provider::{self, ModProviderType},
};
use ferinth::{
    structures::{
        project::ProjectType,
        search::{Facet, Sort},
    },
    Ferinth,
};
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, DatabaseConnection, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::minecraft::version;

use super::task::get_archives_directory;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveProvider {
    CurseForge,
    Modrinth,
}

impl ArchiveProvider {
    pub fn to_mod_provider_type(&self) -> ModProviderType {
        match self {
            ArchiveProvider::CurseForge => ModProviderType::CurseForge,
            ArchiveProvider::Modrinth => ModProviderType::Modrinth,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ArchiveResourceInfo {
    pub identifier: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub page_url: String,
    pub included_in_database: bool,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ModDownloadInfo {
    pub url: String,
    pub size: usize,
    pub loader: ModLoader,
    pub game_version: semver::Version,
    pub path: PathBuf,
}

fn create_ferinth_client() -> ferinth::Result<Ferinth> {
    let client = Ferinth::new("RTranslator", None, None, None)?;
    Ok(client)
}

pub async fn search_modrinth_mods(
    db: &DatabaseConnection,
    query: Option<&String>,
    page: Option<usize>,
) -> anyhow::Result<Vec<ArchiveResourceInfo>> {
    let client = create_ferinth_client()?;
    let page = page.unwrap_or(0);

    let hits = client
        .search_paged(
            query.unwrap_or(&String::from("")),
            &Sort::Relevance,
            10,
            page * 10,
            vec![vec![Facet::ProjectType(ProjectType::Mod)]],
        )
        .await?
        .hits;
    let mut mods: Vec<ArchiveResourceInfo> = Vec::with_capacity(hits.len());

    for hit in hits {
        let identifier = hit.project_id.to_string();

        let included_in_database =
            mod_provider::Entity::find_by_id((ModProviderType::Modrinth, identifier.clone()))
                .one(db)
                .await?
                .is_some();

        mods.push(ArchiveResourceInfo {
            identifier: Some(identifier),
            name: hit.title.clone(),
            description: Some(hit.description.clone()),
            image_url: hit.icon_url.clone().map(|url| url.to_string()),
            page_url: format!("https://modrinth.com/mod/{}", hit.project_id),
            included_in_database,
        });
    }

    Ok(mods)
}

pub async fn search_curseforge_mods() {
    unimplemented!()
}

pub async fn validate_resource_identifier(
    provider: &ArchiveProvider,
    identifier: &str,
) -> anyhow::Result<bool> {
    match provider {
        ArchiveProvider::CurseForge => unimplemented!(),
        ArchiveProvider::Modrinth => {
            let client = create_ferinth_client()?;
            let project = client.get_project(identifier).await?;

            Ok(project.project_type == ProjectType::Mod)
        }
    }
}

pub async fn fetch_downloads(
    provider: &ArchiveProvider,
    identifier: &str,
) -> anyhow::Result<Vec<ModDownloadInfo>> {
    let result = match provider {
        ArchiveProvider::CurseForge => unimplemented!(),
        ArchiveProvider::Modrinth => fetch_modrinth_downloads(identifier).await,
    };

    Ok(result?.into_iter().collect())
}

async fn fetch_modrinth_downloads(identifier: &str) -> anyhow::Result<HashSet<ModDownloadInfo>> {
    let client = create_ferinth_client()?;
    let project = client.get_project(identifier).await?;

    let mut filters: Vec<(&String, &String)> = Vec::new();
    for loader in &project.loaders {
        for game_ver in &project.game_versions {
            if version::is_stable(game_ver) {
                filters.push((loader, game_ver));
            }
        }
    }

    let version_list = client.list_versions(identifier).await?;
    let mut downloads: HashSet<ModDownloadInfo> = HashSet::new();
    let dir = get_archives_directory();

    for (loader, game_ver) in filters {
        let versions = version_list
            .iter()
            .filter(|ver| ver.loaders.contains(loader) && ver.game_versions.contains(game_ver));
        let latest_version = versions.max_by_key(|ver| ver.date_published);

        if let Some(meta) = latest_version {
            let file = meta.files.iter().find(|x| x.primary).or(meta.files.first());
            if let Some(file) = file {
                downloads.insert(ModDownloadInfo {
                    url: file.url.to_string(),
                    size: file.size,
                    loader: match loader.as_str() {
                        "fabric" => ModLoader::Fabric,
                        "forge" => ModLoader::Forge,
                        "neoforge" => ModLoader::Forge,
                        "quilt" => ModLoader::Quilt,
                        _ => continue,
                    },
                    game_version: version::to_semver(game_ver)?,
                    path: dir.join(Uuid::new_v4().to_string()),
                });
            }
        }
    }

    Ok(downloads)
}

pub async fn create_mod_model(
    db: &DatabaseConnection,
    provider: &ArchiveProvider,
    identifier: String,
    missing_entries: bool,
) -> anyhow::Result<minecraft_mod::Model> {
    let status = if missing_entries {
        ModStatus::MissingEntries
    } else {
        ModStatus::Normal
    };

    let existing_provider =
        mod_provider::Entity::find_by_id((provider.to_mod_provider_type(), identifier))
            .one(db)
            .await?;
    if let Some(provider) = existing_provider {
        let model = minecraft_mod::ActiveModel {
            id: Set(provider.mod_id),
            status: Set(status),
            ..Default::default()
        };
        return Ok(model.update(db).await?);
    }

    let model = minecraft_mod::ActiveModel {
        id: NotSet,
        status: Set(status),
        ..Default::default()
    };
    Ok(model.insert(db).await?)
}

pub async fn create_provider_model(
    db: &DatabaseConnection,
    provider: &ArchiveProvider,
    identifier: String,
    mod_id: i32,
) -> anyhow::Result<mod_provider::Model> {
    let is_existing =
        mod_provider::Entity::find_by_id((provider.to_mod_provider_type(), identifier.clone()))
            .one(db)
            .await?
            .is_some();

    let model = match provider {
        ArchiveProvider::CurseForge => unimplemented!(),
        ArchiveProvider::Modrinth => {
            let client = create_ferinth_client()?;
            let project = client.get_project(&identifier).await?;

            mod_provider::ActiveModel {
                identifier: Set(project.id.to_string()),
                provider_type: Set(mod_provider::ModProviderType::Modrinth),
                display_name: Set(project.title),
                description: Set(project.description),
                image_url: Set(project.icon_url.map(|url| url.to_string())),
                page_url: Set(format!("https://modrinth.com/mod/{}", project.id)),
                mod_id: Set(mod_id),
                ..Default::default()
            }
        }
    };

    let model = if is_existing {
        model.update(db).await?
    } else {
        model.insert(db).await?
    };
    Ok(model)
}
