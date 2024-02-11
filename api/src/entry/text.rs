use actix_web::{get, post, web};

use crate::AppState;

#[post("/{text_entry_key}/translate")]
pub async fn translate_text(
    app_state: web::Data<AppState>,
    text_entry_key: web::Path<String>,
) -> actix_web::Result<String> {
    let entry = text_entry::Entity::find_by_id(text_entry_key)
        .one(db)
        .await?;

    Ok("translation_id".to_string())
}

// #[get("/{text_entry_key}/translations")]
// pub async fn get_translations(
//     app_state: web::Data<AppState>,
//     text_entry_key: web::Path<String>,
// ) -> actix_web::Result<web::Json<Vec<TextTranslation>>> {
// }
