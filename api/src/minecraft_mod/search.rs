use actix_web::{get, web};
use serde::Deserialize;
use service::minecraft::search::{search_mods_in_database, DatabaseMod};

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchFilter {
    query: Option<String>,
    page: Option<u64>,
}

#[get("/search")]
pub async fn search_mods(
    app_state: web::Data<AppState>,
    filter: web::Query<SearchFilter>,
) -> actix_web::Result<web::Json<Vec<DatabaseMod>>> {
    let query = filter.query.clone().unwrap_or_default();
    let page = filter.page.unwrap_or(0);

    match search_mods_in_database(&app_state.db, &query, page).await {
        Ok(mods) => Ok(web::Json(mods)),
        Err(err) => {
            println!("Failed to search mods: {}", err);
            Err(actix_web::error::ErrorInternalServerError("Failed to search mods"))
        },
    }
}
