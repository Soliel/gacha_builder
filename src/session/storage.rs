use std::time::Instant;
use std::{sync::Arc, time::Duration};

use dashmap::DashMap;
use rand::random;
use rocket::outcome::Outcome;
use rocket::{Request, State};
use rocket::http::{Cookie, SameSite};
use rocket::tokio::sync::RwLock;
use rocket::tokio::{task::JoinHandle, spawn, time::sleep};

use super::{SESSION_EXPIRE_DURATION, SESSION_COOKIE_NAME, SessionErr};
use super::{SessionData, ClonableSession};


pub struct SessionStorage {
    inner: Arc<DashMap<String, ClonableSession>>,
    join_handle: JoinHandle<()>
}

impl SessionStorage {
    pub fn new() -> Self {
        let inner = Arc::new(DashMap::new());
        let expiring_map = inner.clone();
        let join_handle = spawn(async move {
            loop {
                sleep(Duration::from_secs(60)).await;
                expiring_map.retain(|_, v: &mut ClonableSession| {
                    // If the value is locked, that means it's in use 
                    // and we do not need to delete it. 
                    if let Ok(value) = v.try_read() {
                        if value.last_access + SESSION_EXPIRE_DURATION > Instant::now() {
                            return true
                        }
                    }

                    false 
                })
            }
        });

        Self {
            inner,
            join_handle
        }
    }

    pub fn shutdown(&self) {
        self.join_handle.abort();
    }

    pub(super) fn new_session(&self) -> (String, ClonableSession) {
        let data = Arc::new(RwLock::new(SessionData::new()));

        let sess_key = random::<u128>().to_string();
        let extra = sess_key.clone();
        let cloned = data.clone();
        self.inner.insert(sess_key, data);

        (extra, cloned)
    }

    pub(super) fn build_session_cookie(sess_id: String) -> Cookie<'static> {
        Cookie::build((SESSION_COOKIE_NAME, sess_id))
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(true)
            .build()
    }

    pub(super) fn make_new_session(&self, request: &Request<'_>) -> ClonableSession {
        let sess_data = self.new_session();
        let new_cookie = SessionStorage::build_session_cookie(sess_data.0);

        eprintln!("Made new sessionID: {}", new_cookie.value());
        request.cookies().add_private(new_cookie);

        sess_data.1
    }
    
    pub(super) fn get_session_token<'a>(request: &'a Request<'_>) -> Option<String> {
        if let Some(sess_cookie) = request.cookies().get_private(SESSION_COOKIE_NAME) {
            eprintln!("Found SessionID: {}", sess_cookie.value());
            return Some(sess_cookie.value().to_owned());
        }

        None
    }

    pub(super) fn get_session(&self, sess_key: &str) -> Option<Arc<RwLock<SessionData>>> {      
        match self.inner.get(sess_key) {
            Some(val) => Some((*val).clone()),
            None => None 
        }
    }

    pub(super) async fn get_or_create_sess(request: &Request<'_>) -> Result<ClonableSession, SessionErr> {
        let store = match request.guard::<&State<SessionStorage>>().await {
            Outcome::Success(val) => val,
            Outcome::Error(_) => return Err(SessionErr("Unable to get store")),
            Outcome::Forward(_) => return Err(SessionErr("Unable to get store"))
        };

        let session = if let Some(sess_token) = SessionStorage::get_session_token(request) {
            request.local_cache(|| {
                match store.get_session(&sess_token) {
                    Some(val) => {
                        eprintln!("Found storage with ID: {}", sess_token);
                        val
                    },
                    None => store.make_new_session(request)
                }
            })
        } else {
            request.local_cache(|| store.make_new_session(request))
        };

        Ok(session.clone())
    }
}