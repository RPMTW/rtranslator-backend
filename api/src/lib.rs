mod archive;
mod config;

use actix_cors::Cors;
use actix_web::middleware;
use actix_web::web::Data;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
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

    dotenvy::dotenv().unwrap();
    let config = ServerConfig::load();

    let db = Database::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");
    initialize_database(&db)
        .await
        .expect("Failed to initialize database");
    println!("Successfully connected to database");

    let app_state = web::Data::new(AppState {
        db,
        config: config.clone(),
    });

    println!("Starting server at http://localhost:{}", config.port);
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
}

async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("Not found")
}
