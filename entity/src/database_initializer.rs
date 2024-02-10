use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait, Schema};

use crate::{
    entry::text_entry,
    minecraft::{minecraft_mod, mod_provider},
    translation::text_translation,
};

pub async fn initialize_database(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    create_table(db, minecraft_mod::Entity).await?;
    create_table(db, mod_provider::Entity).await?;
    create_table(db, text_entry::Entity).await?;
    create_table(db, text_translation::Entity).await?;

    Ok(())
}

async fn create_table<E>(db: &DatabaseConnection, entity: E) -> Result<(), sea_orm::DbErr>
where
    E: EntityTrait,
{
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let mut statement = schema.create_table_from_entity(entity);
    db.execute(builder.build(statement.if_not_exists())).await?;

    Ok(())
}
