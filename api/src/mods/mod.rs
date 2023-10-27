mod search;

use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(search::search_mods);
}
