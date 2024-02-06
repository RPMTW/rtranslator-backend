use entity::{
    entry::text_entry,
    minecraft::minecraft_mod::{self, ModStatus},
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QueryTrait,
};
use serde::Serialize;

use super::metadata::ModMetadata;

#[derive(Debug, Serialize)]
pub struct TextEntry {
    pub key: String,
    pub value: String,
    pub preview_translation: Option<String>,
}

pub async fn search_mods_in_database(
    db: &DatabaseConnection,
    query: Option<String>,
    page: u64,
) -> Result<(u64, Vec<ModMetadata>), DbErr> {
    let pages = minecraft_mod::Entity::find()
        .filter(minecraft_mod::Column::Status.eq(ModStatus::Normal))
        .apply_if(query, |select, val| {
            select.filter(minecraft_mod::Column::Name.contains(val))
        })
        .order_by_asc(minecraft_mod::Column::Id)
        .paginate(db, 10);
    let total_pages = pages.num_pages().await?;

    let raw_mods = pages.fetch_page(page).await?;
    let mut mods = Vec::with_capacity(raw_mods.len());

    for raw_mod in raw_mods {
        if let Some(result) = ModMetadata::from_model(raw_mod, db).await {
            mods.push(result);
        }
    }

    Ok((total_pages, mods))
}

pub async fn search_mod_entries_in_database(
    db: &DatabaseConnection,
    mod_id: i32,
    query: Option<String>,
    page: u64,
) -> Result<(u64, Vec<TextEntry>), DbErr> {
    let pages = text_entry::Entity::find()
        .find_also_related(minecraft_mod::Entity)
        .filter(minecraft_mod::Column::Id.eq(mod_id))
        .apply_if(query, |select, val| {
            select.filter(minecraft_mod::Column::Name.contains(val))
        })
        .order_by_asc(text_entry::Column::Key)
        .paginate(db, 15);
    let total_pages = pages.num_pages().await?;

    let raw_entries: Vec<text_entry::Model> = pages
        .fetch_page(page)
        .await?
        .into_iter()
        .map(|(entry, _)| entry)
        .collect();
    let mut entries = Vec::with_capacity(raw_entries.len());

    for entry in &raw_entries {
        entries.push(TextEntry {
            key: entry.key.clone(),
            value: entry.value.clone(),
            // TODO: fetch preview translation from database.
            preview_translation: None,
        });
    }

    Ok((total_pages, entries))
}
