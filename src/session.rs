use std::ops::Add;
use std::time::{Instant, Duration};
use std::sync::{Arc, RwLock};

use dashmap::DashMap;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Request, Data};
use rocket::tokio::task::{spawn, JoinHandle};
use rocket::tokio::time::sleep;
use rocket::http::{Cookie, SameSite};
use state::TypeMap;
use rand::random;

static SESSION_COOKIE_NAME: &str = "SESSIONID";
static SESSION_EXPIRE_DURATION: Duration = Duration::from_secs(3600);

pub type ClonableSession = Arc<RwLock<SessionData>>;

struct SessionData {
    type_state: TypeMap![Send + Sync],
    last_access: Instant
}

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
                sleep(Duration::from_secs(1)).await;
                expiring_map.retain(|_, v: &mut ClonableSession| {
                    // If the value is locked, that means it's in use 
                    // and we do not need to delete it. 
                    if let Ok(value) = v.try_read() {
                        if value.last_access.add(SESSION_EXPIRE_DURATION) < Instant::now() {
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

    fn new_session() -> (&str, ClonableSession) {
        let data = Arc::new(RwLock::new(SessionData {
            type_state: <TypeMap![Send + Sync]>::new(),
            last_access: Instant::now()
        }));

        
    }
}

#[rocket::async_trait]
impl Fairing for SessionStorage {
    fn info(&self) -> Info {
        Info {
            name: "Session Storage",
            kind: Kind::Request 
                | Kind::Shutdown 
                | Kind::Singleton
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        if let Some(session_cookie) = request.cookies().get(SESSION_COOKIE_NAME) {
            if let Some(session) = self.inner.try_get(session_cookie.value()).try_unwrap() {
                let local_session = session.clone();
                request.local_cache(|| local_session);
            }
            // make new session, this is probably expired
        } else {
            // make new session
        }
    }
}