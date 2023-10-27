use actix_web::{get, web};
use serde::Deserialize;
use service::modrinth::{self, MinecraftModInfo};

#[derive(Debug, Deserialize)]
pub struct SearchFilter {
    query: Option<String>,
    page: Option<usize>,
    source: ModSource,
}

#[derive(Debug, Deserialize)]
enum ModSource {
    Database,
    CurseForge,
    Modrinth,
}

#[get("/search")]
pub async fn search_mods(
    filter: web::Query<SearchFilter>,
) -> actix_web::Result<web::Json<Vec<MinecraftModInfo>>> {
    let result = match filter.source {
        ModSource::Database => search_database(filter.into_inner()).await,
        ModSource::CurseForge => search_curseforge(filter.into_inner()).await,
        ModSource::Modrinth => search_modrinth(filter.into_inner()).await,
    };

    match result {
        Ok(mods) => Ok(web::Json(mods)),
        Err(_) => Err(actix_web::error::ErrorInternalServerError(
            "Failed to search mods",
        )),
    }
}

async fn search_database(filter: SearchFilter) -> anyhow::Result<Vec<MinecraftModInfo>> {
    todo!("Search database")
}

async fn search_curseforge(filter: SearchFilter) -> anyhow::Result<Vec<MinecraftModInfo>> {
    todo!("Search CurseForge")
}

async fn search_modrinth(_filter: SearchFilter) -> anyhow::Result<Vec<MinecraftModInfo>> {
    let mods = modrinth::search_mods(_filter.query.as_ref(), _filter.page)
        .await
        .unwrap();
    Ok(mods)
}
