use actix_web::{get, web};
use serde::Deserialize;
use service::archive::resource::{self, ArchiveProvider, ArchiveResourceInfo};

#[derive(Debug, Deserialize)]
pub struct SearchFilter {
    query: Option<String>,
    page: Option<usize>,
    provider: ArchiveProvider,
}

#[get("/search")]
pub async fn search_resources(
    filter: web::Query<SearchFilter>,
) -> actix_web::Result<web::Json<Vec<ArchiveResourceInfo>>> {
    let result = resource::search_modrinth_mods(filter.query.as_ref(), filter.page).await;
    match result {
        Ok(mods) => Ok(web::Json(mods)),
        Err(_) => Err(actix_web::error::ErrorInternalServerError(
            "Failed to search mods from archive source provider.",
        )),
    }
}
