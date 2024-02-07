mod search;

use actix_web::{get, web};
use log::warn;
use service::minecraft::metadata::{lookup_mod_metadata, ModMetadata};

use crate::AppState;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(search::search_mods);
    cfg.service(search::search_mod_entries);
    cfg.service(get_mod_metadata);
}

#[get("/{mod_id}/metadata")]
pub async fn get_mod_metadata(
    app_state: web::Data<AppState>,
    mod_id: web::Path<i32>,
) -> actix_web::Result<web::Json<ModMetadata>> {
    let result = lookup_mod_metadata(&app_state.db, *mod_id).await;

    match result {
        Ok(Some(metadata)) => Ok(web::Json(metadata)),
        Ok(None) => Err(actix_web::error::ErrorNotFound("Mod not found")),
        Err(err) => {
            warn!("Failed to get mod metadata: {}", err);
            Err(actix_web::error::ErrorInternalServerError(
                "Failed to get mod metadata",
            ))
        }
    }
}
