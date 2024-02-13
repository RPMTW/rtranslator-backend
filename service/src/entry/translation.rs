use entity::{
    entry::text_entry,
    translation::{
        text_translation,
        translation_flag::{TranslationFlag, TranslationFlagVec},
    },
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, DatabaseConnection, DbErr, EntityTrait, ModelTrait, Set,
};
use serde::Serialize;

pub async fn get_text_entry(db: &DatabaseConnection, key: String) -> Option<text_entry::Model> {
    text_entry::Entity::find_by_id(key).one(db).await.ok()?
}

/// Add a translated text for the text entry with the given key.
///
/// # Errors
///
/// If the translation model cannot be inserted into the database, a [`DbErr`] will be thrown.
pub async fn add_translation(
    db: &DatabaseConnection,
    text_entry_key: String,
    content: String,
) -> Result<text_translation::Model, DbErr> {
    let model = text_translation::ActiveModel {
        id: NotSet,
        content: Set(content),
        flags: Set(TranslationFlagVec(Vec::new())),
        entry_key: Set(text_entry_key),
    };
    let result = model.insert(db).await?;

    Ok(result)
}

#[derive(Debug, Serialize)]
pub struct TextTranslation {
    pub id: i32,
    pub content: String,
    pub flags: Vec<TranslationFlag>,
}

/// Gets all translations for the given text entry.
///
/// # Errors
///
/// Returns a [`DbErr`] if there is an error querying the database.
pub async fn get_translations(
    db: &DatabaseConnection,
    text_entry: text_entry::Model,
) -> Result<Vec<TextTranslation>, DbErr> {
    let translations = text_entry
        .find_related(text_translation::Entity)
        .all(db)
        .await?
        .into_iter()
        .map(|model| TextTranslation {
            id: model.id,
            content: model.content,
            flags: vec![],
        })
        .collect();

    Ok(translations)
}
