use std::collections::HashMap;

use entity::minecraft::{
    minecraft_mod::{self, ModStatus},
    mod_provider::{self, ModProviderType},
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DatabaseMod {
    pub id: i32,
    pub status: ModStatus,
    pub name: String,
    pub description: String,
    pub image_url: Option<String>,
    pub page_url: HashMap<ModProviderType, String>,
}

pub async fn search_mods_in_database(
    db: &DatabaseConnection,
    query: &str,
    page: u64,
) -> Result<Vec<DatabaseMod>, DbErr> {
    let pages = minecraft_mod::Entity::find()
        .filter(minecraft_mod::Column::Status.eq(ModStatus::Normal))
        .filter(minecraft_mod::Column::Name.contains(query))
        .order_by_asc(minecraft_mod::Column::Id)
        .paginate(db, 10);

    let raw_mods = pages.fetch_page(page).await?;
    let mut mods = Vec::with_capacity(raw_mods.len());

    for raw_mod in raw_mods {
        let providers = raw_mod
            .find_related(mod_provider::Entity)
            .order_by_desc(mod_provider::Column::UpdatedAt)
            .all(db)
            .await?;
        // Prefer the latest updated mod provider information.
        let preferred_provider = providers.first();

        if let Some(preferred) = preferred_provider {
            mods.push(DatabaseMod {
                id: raw_mod.id,
                status: raw_mod.status,
                name: raw_mod.name.unwrap_or(preferred.display_name.clone()),
                description: preferred.description.clone(),
                image_url: preferred.image_url.clone(),
                page_url: providers
                    .into_iter()
                    .map(|provider| (provider.provider_type, provider.page_url))
                    .collect(),
            });
        }
    }

    Ok(mods)
}
