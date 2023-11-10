use sea_orm::{entity::prelude::*, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "mod_loader")]
pub enum ModLoader {
    #[sea_orm(string_value = "Fabric")]
    Fabric,
    #[sea_orm(string_value = "Forge")]
    Forge,
    #[sea_orm(string_value = "Quilt")]
    Quilt,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct ModLoaderVec(pub Vec<ModLoader>);
