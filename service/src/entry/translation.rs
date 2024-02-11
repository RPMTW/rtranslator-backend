use entity::{entry::text_entry, translation::text_translation};
use sea_orm::{DatabaseConnection, EntityTrait, Set};

use crate::entry::translation;

pub async fn add_translation(
    db: &DatabaseConnection,
    text_entry_key: String,
) -> Result<text_translation::Model> {
    let entry = text_entry::Entity::find_by_id(text_entry_key)
        .one(db)
        .await?;

    let model = text_translation::ActiveModel {
        entry_key: Set(text_entry_key)
    }
    todo!()
}
