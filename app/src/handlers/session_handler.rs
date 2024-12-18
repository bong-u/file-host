use actix_web::{web, HttpResponse, Responder};
use crate::memory_store::MemorySessionStore;

pub async fn get_sessions(store: web::Data<MemorySessionStore>) -> impl Responder {
    // Mutex 잠금 처리
    let sessions = match store.sessions.lock() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to lock session store: {}", e);
            return HttpResponse::InternalServerError()
                .body("Failed to access session store");
        }
    };

    println!("sessions: {:?}", sessions);

    // 세션 데이터를 key, value 형태로 문자열 생성
    let mut result = String::new();
    for (session_key, (session_data, ttl)) in sessions.iter() {
        result.push_str(&format!("Session Key: {}\n", session_key));
        for (key, value) in session_data.iter() {
            result.push_str(&format!("{} -> {}\n", key, value));
        }
        result.push_str(&format!("TTL: {:?}\n", ttl));
        result.push('\n');
    }


    // 응답 반환
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(result)
}

