use sea_orm::{entity::prelude::*, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "translation_flag")]
#[serde(rename_all = "snake_case")]
pub enum TranslationFlag {
    #[sea_orm(string_value = "Approved")]
    Approved,
    /// Translations migrated from [Crowdin](https://crowdin.com) platform.
    #[sea_orm(string_value = "Migrated")]
    Migrated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct TranslationFlagVec(pub Vec<TranslationFlag>);
