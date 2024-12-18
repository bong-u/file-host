use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    future::ready,
    future::Future,
};
use actix_web::cookie::time::Duration;
use anyhow::anyhow;
use actix_session::storage::{SessionStore, SessionKey, UpdateError, LoadError, SaveError};

/// 메모리 기반 세션 저장소 구조체
#[derive(Clone, Default)]
pub struct MemorySessionStore {
    pub sessions: Arc<Mutex<HashMap<String, (HashMap<String, String>, Duration)>>>, // 세션 데이터와 TTL
}

impl MemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// SessionStore 트레이트 구현
impl SessionStore for MemorySessionStore {
    // 세션 로드
    fn load(
        &self,
        session_key: &SessionKey,
    ) -> impl Future<Output = Result<Option<HashMap<String, String>>, LoadError>> {
        let sessions = self.sessions.lock().unwrap();
        let key = session_key.as_ref();

        // 세션 데이터만 반환
        let result = sessions.get(key).map(|(state, _)| state.clone());
        ready(Ok(result))
    }


    // 세션 저장
    fn save(
        &self,
        session_state: HashMap<String, String>,
        ttl: &Duration,
    ) -> impl Future<Output = Result<SessionKey, SaveError>> {
        let mut sessions = self.sessions.lock().unwrap();

        let key = uuid::Uuid::new_v4().to_string();
        sessions.insert(key.clone(), (session_state, *ttl));

        match SessionKey::try_from(key.clone()) {
            Ok(session_key) => ready(Ok(session_key)),
            Err(e) => ready(Err(SaveError::Other(anyhow!(e)))),
        }
    }

        // 세션 업데이트
    fn update(
        &self,
        session_key: SessionKey,
        session_state: HashMap<String, String>,
        ttl: &Duration,
    ) -> impl Future<Output = Result<SessionKey, UpdateError>> {
        let mut sessions = self.sessions.lock().unwrap();
        let key = session_key.as_ref().to_owned();

        if sessions.contains_key(&key) {
            sessions.insert(key.clone(), (session_state, *ttl));
            ready(Ok(session_key))
        } else {
            ready(Err(UpdateError::Other(anyhow!("Session key not found"))))
        }
    }

    // TTL만 갱신
    fn update_ttl(
        &self,
        session_key: &SessionKey,
        ttl: &Duration,
    ) -> impl Future<Output = Result<(), anyhow::Error>> {
        let mut sessions = self.sessions.lock().unwrap();
        let key = session_key.as_ref();

        if let Some((_, existing_ttl)) = sessions.get_mut(key) {
            *existing_ttl = *ttl;
            ready(Ok(()))
        } else {
            ready(Err(anyhow!("Session key not found")))
        }
    }

    // 세션 삭제
    fn delete(&self, session_key: &SessionKey) -> impl Future<Output = Result<(), anyhow::Error>> {
        let mut sessions = self.sessions.lock().unwrap();
        let key = session_key.as_ref();

        sessions.remove(key);
        ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_session::storage::{SessionStore};
    use actix_web::cookie::time::Duration;
    use std::collections::HashMap;
    use futures::executor::block_on;

    #[test]
    fn test_save_and_load_session() {
        let store = MemorySessionStore::new();
        let mut session_state = HashMap::new();
        session_state.insert("username".to_string(), "test_user".to_string());
        let ttl = Duration::minutes(30);

        // Save a session
        let session_key = block_on(store.save(session_state.clone(), &ttl)).unwrap();
        assert!(!session_key.as_ref().is_empty());

        // Load the session
        let loaded_state = block_on(store.load(&session_key)).unwrap();
        assert_eq!(loaded_state, Some(session_state));
    }

    #[test]
    fn test_update_session() {
        let store = MemorySessionStore::new();
        let mut session_state = HashMap::new();
        session_state.insert("username".to_string(), "test_user".to_string());
        let ttl = Duration::minutes(30);

        // Save a session
        let session_key = block_on(store.save(session_state.clone(), &ttl)).unwrap();

        // Update the session
        let mut updated_state = HashMap::new();
        updated_state.insert("username".to_string(), "updated_user".to_string());
        let new_ttl = Duration::minutes(60);

        let updated_key = block_on(store.update(session_key, updated_state.clone(), &new_ttl)).unwrap();

        // Load the updated session
        let loaded_state = block_on(store.load(&updated_key)).unwrap();
        assert_eq!(loaded_state, Some(updated_state));
    }

    #[test]
    fn test_delete_session() {
        let store = MemorySessionStore::new();
        let mut session_state = HashMap::new();
        session_state.insert("username".to_string(), "test_user".to_string());
        let ttl = Duration::minutes(30);

        // Save a session
        let session_key = block_on(store.save(session_state, &ttl)).unwrap();

        // Delete the session
        let result = block_on(store.delete(&session_key));
        assert!(result.is_ok());

        // Verify the session is deleted
        let loaded_state = block_on(store.load(&session_key)).unwrap();
        assert!(loaded_state.is_none());
    }
}

