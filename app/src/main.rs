mod routes;
mod handlers {
    pub mod root_handler;
    pub mod file_handler;
    pub mod session_handler;
}
mod memory_store;

use crate::routes::register_routes;
use actix_web::{web::Data, App, HttpServer};
use actix_web::cookie::Key;
use actix_session::{SessionMiddleware, config::PersistentSession};
use time::Duration;
use dotenv::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    const HOST: &str = "0.0.0.0";
    const PORT: u16 = 8081;

    let secret_key = Key::generate();
    let session_store = memory_store::MemorySessionStore::new();

    println!("Starting server at http://{}:{}", HOST, PORT);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(session_store.clone()))
            .configure(register_routes)
            .wrap(
                SessionMiddleware::builder(session_store.clone(), secret_key.clone())
                .cookie_secure(false)
                .session_lifecycle(PersistentSession::default().session_ttl(Duration::minutes(1)))
                .build(),
            )
    })
    .bind((HOST, PORT))?
    .run()
    .await
}

