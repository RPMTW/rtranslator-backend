use actix_web::{get, web};
use log::warn;
use serde::Deserialize;
use service::archive::resource::{self, ArchiveProvider, ArchiveResourceInfo};

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchFilter {
    query: Option<String>,
    page: Option<usize>,
    provider: ArchiveProvider,
}

#[get("/search")]
pub async fn search_resources(
    app_state: web::Data<AppState>,
    filter: web::Query<SearchFilter>,
) -> actix_web::Result<web::Json<Vec<ArchiveResourceInfo>>> {
    let result =
        resource::search_modrinth_mods(&app_state.db, filter.query.as_ref(), filter.page).await;

    match result {
        Ok(mods) => Ok(web::Json(mods)),
        Err(e) => {
            warn!("Failed to search mods from archive source provider: {:?}", e);
            Err(actix_web::error::ErrorInternalServerError(
                "Failed to search mods from archive source provider.",
            ))
        }
    }
}
