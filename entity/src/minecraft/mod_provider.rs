use sea_orm::{entity::prelude::*, DatabaseBackend, Schema, Statement};
use serde::{Deserialize, Serialize};

use crate::database_initializer::DatabaseInitializer;

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

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mod_provider_type")]
pub enum ModProviderType {
    #[sea_orm(string_value = "CurseForge")]
    CurseForge,
    #[sea_orm(string_value = "Modrinth")]
    Modrinth,
    #[sea_orm(string_value = "Custom")]
    Custom,
}

impl DatabaseInitializer for Entity {
    fn initialize(builder: &DatabaseBackend) -> Statement {
        let schema = Schema::new(*builder);
        let mut statement = schema.create_table_from_entity(Self);

        builder.build(statement.if_not_exists())
    }
}
