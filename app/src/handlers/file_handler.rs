use actix_multipart::Multipart;
use actix_web::{HttpResponse, Responder, http::StatusCode};
use actix_session::Session;
use futures_util::TryStreamExt;
use std::{
    fs::{File},
    io::Write,
    path::{Path},
};

pub async fn upload_file(mut payload: Multipart, session: Session) -> impl Responder {
    let session_id = match session.get::<String>("session_id") {
        Ok(Some(session_id)) => session_id,
        _ => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    println!("Session ID: {}", session_id);

    // 파일을 업로드할 디렉토리
    let upload_dir = "./uploads";

    if !Path::new(upload_dir).exists() {
        return HttpResponse::build(StatusCode::SERVICE_UNAVAILABLE)
            .body("Upload directory does not exist");
    }
    
    // 파일 가져오기
    let mut field = match payload.try_next().await {
        Ok(Some(field)) => field,
        Ok(None) => return HttpResponse::BadRequest().body("No file found in the request"),
        Err(e) => return HttpResponse::InternalServerError().body(format!("Error reading payload: {}", e)),
    };
    // 파일 확장자 추출
    let original_filename = field
        .content_disposition()
        .and_then(|cd| cd.get_filename())
        .map(String::from)
        .unwrap_or_else(|| "file.bin".to_string());

    let extension = Path::new(&original_filename)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("bin");

    let new_filename = format!("{}.{}", session_id, extension);
    let filepath = format!("{}/{}", upload_dir, sanitize_filename::sanitize(&new_filename));

    println!("Uploading file to: {}", filepath);

    // 파일 생성
    let mut file = match File::create(&filepath) {
        Ok(f) => f,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to create file: {}", e)),
    };

    while let Some(chunk) = field.try_next().await.transpose() {
        match chunk {
            Ok(data) => {
                if let Err(e) = file.write_all(&data) {
                    return HttpResponse::InternalServerError().body(format!("Failed to write to file: {}", e));
                }
            }
            Err(e) => return HttpResponse::InternalServerError().body(format!("Error reading chunk: {}", e)),
        }
    }

    if let Err(e) = session.insert("file_name", new_filename) {
        eprintln!("Failed to insert is_uploaded: {:?}", e);
    }

    HttpResponse::SeeOther()
        .insert_header(("Location", "/"))
        .finish()
}


