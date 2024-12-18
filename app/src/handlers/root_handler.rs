use actix_web::{HttpResponse, Responder};
use askama::Template;
use actix_session::Session;
use std::env;

// Define the Askama template
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    base_url: String,
    file_name: String,
}

pub async fn root(session: Session) -> impl Responder {
    let session_id = match session.get::<String>("session_id") {
        Ok(Some(session_id)) => session_id,
        _ => {
            let new_session_id = uuid::Uuid::new_v4().to_string();
            if let Err(e) = session.insert("session_id", &new_session_id) {
                eprintln!("Failed to insert session_id: {:?}", e);
            }
            new_session_id
        }
    };

    let file_name = session
    .get::<String>("file_name")
    .unwrap_or(None)
    .unwrap_or_else(|| "".to_string());

    let base_url = match env::var("BASE_URL") {
        Ok(base_url) => base_url,
        Err(_) => return HttpResponse::InternalServerError().body("BASE_URL is not set"),
    };

    println!("Session ID: {}", session_id);
    println!("File name: {:?}", file_name);

    let template = IndexTemplate {
        base_url,
        file_name,
    };

    match template.render() {
        Ok(rendered) => HttpResponse::Ok().content_type("text/html").body(rendered),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

