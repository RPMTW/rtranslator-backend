use std::collections::HashSet;

use ferinth::{
    structures::{
        project::ProjectType,
        search::{Facet, Sort},
    },
    Ferinth,
};
use serde::{Deserialize, Serialize};

use crate::minecraft::version;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveProvider {
    CurseForge,
    Modrinth,
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

pub struct ModDownloadInfo {
    pub url: String,
    pub size: usize,
    pub loader: ModLoader,
    pub game_version: String,
}

pub enum ModLoader {
    Fabric,
    Forge,
    Quilt,
}

fn create_ferinth_client() -> ferinth::Result<Ferinth> {
    let client = Ferinth::new("RTranslator", None, None, None)?;
    Ok(client)
}

pub async fn search_modrinth_mods(
    query: Option<&String>,
    page: Option<usize>,
) -> ferinth::Result<Vec<ArchiveResourceInfo>> {
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
    let mut mods: Vec<ArchiveResourceInfo> = Vec::new();

    hits.iter().for_each(|hit| {
        mods.push(ArchiveResourceInfo {
            identifier: Some(hit.project_id.to_string()),
            name: hit.title.clone(),
            description: Some(hit.description.clone()),
            image_url: hit.icon_url.clone().map(|url| url.to_string()),
            page_url: format!("https://modrinth.com/mod/{}", hit.project_id),
            included_in_database: false,
        });
    });

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
) -> ferinth::Result<Vec<(String, usize)>> {
    let result = match provider {
        ArchiveProvider::CurseForge => unimplemented!(),
        ArchiveProvider::Modrinth => fetch_modrinth_downloads(identifier).await,
    };

    result.map(|urls| urls.into_iter().collect())
}

async fn fetch_modrinth_downloads(identifier: &str) -> ferinth::Result<HashSet<(String, usize)>> {
    let client = create_ferinth_client()?;
    let project = client.get_project(identifier).await?;

    let mut version_filters: Vec<(&String, &String)> = Vec::new();
    for loader in &project.loaders {
        for game_version in &project.game_versions {
            version_filters.push((loader, game_version));
        }
    }

    let version_list = client.list_versions(identifier).await?;
    let mut downloads: HashSet<(String, usize)> = HashSet::new();

    for (loader, game_ver) in version_filters {
        if !version::is_stable(game_ver) {
            continue;
        }

        let versions = version_list
            .iter()
            .filter(|ver| ver.loaders.contains(loader) && ver.game_versions.contains(game_ver));
        let latest_version = versions.max_by_key(|ver| ver.date_published);

        if let Some(ver) = latest_version {
            // TODO
            let file = ver.files.iter().find(|x| x.primary).or(ver.files.first());
            if let Some(file) = file {
                downloads.insert((file.url.to_string(), file.size));
            }
        }
    }

    Ok(downloads)
}
