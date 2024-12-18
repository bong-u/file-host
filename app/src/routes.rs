use actix_files::Files;
use actix_web::web;
use crate::handlers::{root_handler, file_handler, session_handler};

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        Files::new("/uploads", "./uploads")
    )
    .service(
        web::scope("")
            .route("/", web::get().to(root_handler::root))
            .route("/", web::post().to(file_handler::upload_file))
            .route("/sessions", web::get().to(session_handler::get_sessions))
    );
}

