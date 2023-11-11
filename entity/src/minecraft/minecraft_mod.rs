use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{entity::prelude::*, DatabaseBackend, Schema, Set, Statement};
use serde::{Deserialize, Serialize};

use crate::database_initializer::DatabaseInitializer;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "minecraft_mod")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub status: ModStatus,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::minecraft::mod_provider::Entity")]
    ModProvider,
    #[sea_orm(has_many = "crate::entry::text_entry::Entity")]
    TextEntry,
}

impl Related<crate::minecraft::mod_provider::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ModProvider.def()
    }
}

impl Related<crate::entry::text_entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TextEntry.def()
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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mod_status")]
#[serde(rename_all = "snake_case")]
pub enum ModStatus {
    #[sea_orm(string_value = "Normal")]
    Normal,
    /// The mod is missing translatable entries.
    #[sea_orm(string_value = "MissingEntries")]
    MissingEntries,
}

impl DatabaseInitializer for Entity {
    fn initialize(builder: &DatabaseBackend) -> Statement {
        let schema = Schema::new(*builder);
        let mut statement = schema.create_table_from_entity(Self);

        builder.build(statement.if_not_exists())
    }
}
