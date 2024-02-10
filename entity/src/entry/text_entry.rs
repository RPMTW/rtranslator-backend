use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{minecraft::mod_loader::ModLoaderVec, misc::StringVec, translation::text_translation};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "text_entry")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
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
    #[sea_orm(has_many = "crate::translation::text_translation::Entity")]
    TextTranslation,
}

impl Related<crate::minecraft::minecraft_mod::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MinecraftMod.def()
    }
}

impl Related<text_translation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TextTranslation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
