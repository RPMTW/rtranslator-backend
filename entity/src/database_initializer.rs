use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};

use crate::{
    entry::text_entry,
    minecraft::{minecraft_mod, mod_provider},
};

pub trait DatabaseInitializer {
    fn initialize(builder: &DatabaseBackend) -> Statement;
}

pub async fn initialize_database(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let builder = db.get_database_backend();
    let statements = vec![
        minecraft_mod::Entity::initialize(&builder),
        mod_provider::Entity::initialize(&builder),
        text_entry::Entity::initialize(&builder),
    ];

    for statement in statements {
        db.execute(statement).await?;
    }

    Ok(())
}
