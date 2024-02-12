use entity::{
    entry::text_entry,
    translation::{text_translation, translation_flag::TranslationFlagVec},
};
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, DatabaseConnection, DbErr, EntityTrait, Set};

pub async fn get_text_entry(db: &DatabaseConnection, key: String) -> Option<text_entry::Model> {
    text_entry::Entity::find_by_id(key).one(db).await.ok()?
}

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
