use std::collections::HashMap;

use entity::minecraft::{
    minecraft_mod::{self, ModStatus},
    mod_provider::{self, ModProviderType},
};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryOrder};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ModMetadata {
    pub id: i32,
    pub status: ModStatus,
    pub name: String,
    pub description: String,
    pub image_url: Option<String>,
    pub page_url: HashMap<ModProviderType, String>,
}

impl ModMetadata {
    pub async fn from_model(model: minecraft_mod::Model, db: &DatabaseConnection) -> Option<Self> {
        let providers = model
            .find_related(mod_provider::Entity)
            .order_by_desc(mod_provider::Column::UpdatedAt)
            .all(db)
            .await
            .ok()?;
        // Prefer the latest updated mod provider information.
        let preferred_provider = providers.first()?;

        Some(ModMetadata {
            id: model.id,
            status: model.status,
            name: model
                .name
                .unwrap_or(preferred_provider.display_name.clone()),
            description: preferred_provider.description.clone(),
            image_url: preferred_provider.image_url.clone(),
            page_url: providers
                .into_iter()
                .map(|provider| (provider.provider_type, provider.page_url))
                .collect(),
        })
    }
}

pub async fn lookup_mod_metadata(
    db: &DatabaseConnection,
    mod_id: i32,
) -> Result<Option<ModMetadata>, DbErr> {
    let model = minecraft_mod::Entity::find_by_id(mod_id).one(db).await?;

    if let Some(model) = model {
        Ok(ModMetadata::from_model(model, db).await)
    } else {
        Ok(None)
    }
}
