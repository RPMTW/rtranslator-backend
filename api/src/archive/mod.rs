mod search;
mod task;

use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(search::search_resources);
    cfg.service(task::create_archive_task);
    cfg.service(task::get_archive_task);
}
