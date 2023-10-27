use ferinth::{
    structures::{
        project::ProjectType,
        search::{Facet, Sort},
    },
    Ferinth,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MinecraftModInfo {
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub page_url: String,
    pub included_in_database: bool,
}

pub async fn search_mods(
    query: Option<&String>,
    page: Option<usize>,
) -> ferinth::Result<Vec<MinecraftModInfo>> {
    let client = Ferinth::new("RTranslator", None, None, None)?;
    let page = page.unwrap_or(0);

    let hits = client
        .search_paged(
            query.unwrap_or(&String::from("")),
            &Sort::Downloads,
            10,
            page * 10,
            vec![vec![Facet::ProjectType(ProjectType::Mod)]],
        )
        .await?
        .hits;
    let mut mods: Vec<MinecraftModInfo> = Vec::new();

    hits.iter().for_each(|hit| {
        mods.push(MinecraftModInfo {
            name: hit.title.clone(),
            description: Some(hit.description.clone()),
            image_url: hit.icon_url.clone().map(|url| url.to_string()),
            page_url: format!("https://modrinth.com/mod/{}", hit.project_id),
            included_in_database: false,
        });
    });

    Ok(mods)
}
