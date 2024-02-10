mod text;

use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(text::translate_text);
}
