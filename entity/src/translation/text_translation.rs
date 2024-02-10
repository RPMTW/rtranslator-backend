use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::entry::text_entry;

use super::translation_flag::TranslationFlagVec;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "text_translation")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub flags: TranslationFlagVec,

    pub entry_key: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::entry::text_entry::Entity",
        from = "Column::EntryKey",
        to = "crate::entry::text_entry::Column::Key"
    )]
    TextEntry,
}

impl Related<text_entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TextEntry.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
