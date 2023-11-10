use sea_orm::{entity::prelude::*, DatabaseBackend, Schema, Statement};
use serde::{Deserialize, Serialize};

use crate::{
    database_initializer::DatabaseInitializer, minecraft::mod_loader::ModLoaderVec, misc::StringVec,
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "text_entry")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub key: String,
    #[sea_orm(column_type = "Text")]
    pub value: String,
    pub namespaces: StringVec,
    pub game_versions: StringVec,
    pub loaders: ModLoaderVec,

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

impl DatabaseInitializer for Entity {
    fn initialize(builder: &DatabaseBackend) -> Statement {
        let schema = Schema::new(*builder);
        let mut statement = schema.create_table_from_entity(Self);

        builder.build(statement.if_not_exists())
    }
}
