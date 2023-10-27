mod mods;

use std::env;

use actix_web::middleware;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use service::sea_orm::{Database, DatabaseConnection};

struct AppState {
    db: DatabaseConnection,
}

#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().unwrap();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let port = if let Ok(raw_port) = env::var("PORT") {
        raw_port.parse::<u16>().expect("PORT must be a number")
    } else {
        8080
    };

    let db = Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");
    let app_state = web::Data::new(AppState { db });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::Logger::default())
            .default_service(web::route().to(not_found))
            .configure(init)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/mods").configure(mods::init));
}

async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("Not found")
}
