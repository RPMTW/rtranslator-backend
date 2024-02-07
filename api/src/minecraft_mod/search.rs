use actix_web::{get, web};
use log::warn;
use serde::{Deserialize, Serialize};
use service::minecraft::{
    metadata::ModMetadata,
    search::{search_mod_entries_in_database, search_mods_in_database, TextEntry},
};

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchFilter {
    query: Option<String>,
    page: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct SearchModResponse {
    total_pages: u64,
    mods: Vec<ModMetadata>,
}

#[get("/search")]
pub async fn search_mods(
    app_state: web::Data<AppState>,
    filter: web::Query<SearchFilter>,
) -> actix_web::Result<web::Json<SearchModResponse>> {
    let page = filter.page.unwrap_or(0);
    let result = search_mods_in_database(&app_state.db, filter.query.clone(), page).await;

    match result {
        Ok((total_pages, mods)) => Ok(web::Json(SearchModResponse { total_pages, mods })),
        Err(err) => {
            warn!("Failed to search mods: {}", err);
            Err(actix_web::error::ErrorInternalServerError(
                "Failed to search mods",
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EntriesFilter {
    pub query: Option<String>,
    pub page: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct SearchEntriesResponse {
    pub total_pages: u64,
    pub entries: Vec<TextEntry>,
}

#[get("/{mod_id}/entries")]
pub async fn search_mod_entries(
    app_state: web::Data<AppState>,
    mod_id: web::Path<i32>,
    filter: web::Query<EntriesFilter>,
) -> actix_web::Result<web::Json<SearchEntriesResponse>> {
    let page = filter.page.unwrap_or(0);
    let result =
        search_mod_entries_in_database(&app_state.db, *mod_id, filter.query.clone(), page).await;

    match result {
        Ok((total_pages, entries)) => Ok(web::Json(SearchEntriesResponse {
            total_pages,
            entries,
        })),
        Err(err) => {
            warn!("Failed to search entries for specific mod: {}", err);
            Err(actix_web::error::ErrorInternalServerError(
                "Failed to search entries for specific mod",
            ))
        }
    }
}
