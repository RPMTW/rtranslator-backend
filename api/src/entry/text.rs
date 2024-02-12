use actix_web::{
    error::{ErrorInternalServerError, ErrorNotFound},
    post, web,
};
use log::warn;
use serde::Deserialize;

use crate::AppState;
use service::entry::translation;

#[derive(Debug, Deserialize)]
pub struct TranslateTextPayload {
    pub content: String,
}

// TODO: User authorization
#[post("/{entry_key}/translate")]
pub async fn translate_text(
    app_state: web::Data<AppState>,
    entry_key: web::Path<String>,
    payload: web::Json<TranslateTextPayload>,
) -> actix_web::Result<String> {
    let entry = translation::get_text_entry(&app_state.db, entry_key.to_string()).await;
    if entry.is_none() {
        return Err(ErrorNotFound("Text Entry not found"));
    }

    let result = translation::add_translation(
        &app_state.db,
        entry_key.to_string(),
        payload.content.clone(),
    )
    .await;

    match result {
        Ok(model) => Ok(model.id.to_string()),
        Err(e) => {
            warn!(
                "Failed to add translation for the text entry whose key is {} : {:?}",
                entry_key, e
            );
            Err(ErrorInternalServerError(
                "Failed to add translation for the text entry.",
            ))
        }
    }
}

// #[get("/{text_entry_key}/translations")]
// pub async fn get_translations(
//     app_state: web::Data<AppState>,
//     text_entry_key: web::Path<String>,
// ) -> actix_web::Result<web::Json<Vec<TextTranslation>>> {
// }
// }
// }
