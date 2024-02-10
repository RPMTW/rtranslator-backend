use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{entity::prelude::*, Set};
use serde::{Deserialize, Serialize};

use super::minecraft_mod;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mod_provider")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub provider_type: ModProviderType,
    #[sea_orm(primary_key, auto_increment = false)]
    pub identifier: String,
    pub display_name: String,
    pub description: String,
    pub image_url: Option<String>,
    pub page_url: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub mod_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::minecraft::minecraft_mod::Entity",
        from = "Column::ModId",
        to = "crate::minecraft::minecraft_mod::Column::Id"
    )]
    MinecraftMod,
}

impl Related<crate::minecraft::minecraft_mod::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MinecraftMod.def()
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let now = Utc::now();
        if insert {
            self.created_at = Set(now);
        }
        self.updated_at = Set(now);

        Ok(self)
    }

    async fn after_save<C>(model: Model, db: &C, _: bool) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        let mc_mod = model.find_related(minecraft_mod::Entity).one(db).await?;
        if let Some(mc_mod) = mc_mod {
            let mut mc_mod: minecraft_mod::ActiveModel = mc_mod.into();
            mc_mod.name = Set(Some(model.display_name.clone()));
            mc_mod.update(db).await?;
        }

        Ok(model)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mod_provider_type")]
#[serde(rename_all = "lowercase")]
pub enum ModProviderType {
    #[sea_orm(string_value = "CurseForge")]
    CurseForge,
    #[sea_orm(string_value = "Modrinth")]
    Modrinth,
    #[sea_orm(string_value = "Custom")]
    Custom,
}
