mod archive;
mod config;
mod minecraft_mod;

use actix_cors::Cors;
use actix_web::middleware;
use actix_web::web::Data;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use log::{info, warn};
use service::initialize_database;
use service::sea_orm::{Database, DatabaseConnection};

use crate::config::ServerConfig;

pub struct AppState {
    db: DatabaseConnection,
    config: ServerConfig,
}

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();
    if let Err(e) = dotenvy::dotenv() {
        warn!("Failed to load .env file: {}", e);
    }
    let config = ServerConfig::load();

    info!("Connecting to database at {}", config.database_url);
    let db = Database::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");
    initialize_database(&db)
        .await
        .expect("Failed to initialize database");
    info!("Successfully connected to database");

    let app_state = web::Data::new(AppState {
        db,
        config: config.clone(),
    });

    info!("Starting server at http://localhost:{}", config.port);
    HttpServer::new(move || {
        App::new()
            .app_data(Data::clone(&app_state))
             .wrap(middleware::Logger::default())
            .wrap(Cors::permissive())
            .default_service(web::route().to(not_found))
            .configure(init)
    })
    .bind(("127.0.0.1", config.port))?
    .run()
    .await
}

fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/archives").configure(archive::init));
    cfg.service(web::scope("/mods").configure(minecraft_mod::init));
}

async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("Not found")
}
