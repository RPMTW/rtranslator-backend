use actix_web::{post, web};

use crate::AppState;

#[post("/{text_entry_key}/translate")]
pub async fn translate_text(
    app_state: web::Data<AppState>,
    text_entry_key: web::Path<String>,
) -> actix_web::Result<String> {
    Ok("translation_id".to_string())
}
