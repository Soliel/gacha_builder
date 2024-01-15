use std::ops::{Add, Deref};
use std::time::{Instant, Duration};
use std::sync::Arc;

use dashmap::DashMap;
use oauth2::http::request;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, Data, Rocket, Orbit, local};
use rocket::tokio::task::{spawn, JoinHandle};
use rocket::tokio::time::sleep;
use rocket::tokio::sync::RwLock;
use rocket::http::{Cookie, SameSite};
use rocket::http::Status;
use state::TypeMap;
use rand::random;

static SESSION_COOKIE_NAME: &str = "SESSIONID";
static SESSION_EXPIRE_DURATION: Duration = Duration::from_secs(3600);

pub type ClonableSession = Arc<RwLock<SessionData>>;

#[derive(Debug)]
pub struct SessionData {
    type_state: TypeMap![Send + Sync],
    last_access: Instant
}

pub struct Session<T> {
    ref_val: Arc<T>,
}

pub struct SessionWriter {
    inner: ClonableSession
}

#[derive(Debug)]
pub struct SessionErr(String);

pub struct SessionStorage {
    inner: Arc<DashMap<String, ClonableSession>>,
    join_handle: JoinHandle<()>
}

impl SessionData {
    fn new() -> Self {
        SessionData {
            type_state: <TypeMap![Send + Sync]>::new(),
            last_access: Instant::now()
        }
    }
}

impl SessionStorage {
    pub fn new() -> Self {
        let inner = Arc::new(DashMap::new());
        let expiring_map = inner.clone();
        let join_handle = spawn(async move {
            loop {
                sleep(Duration::from_secs(60)).await;
                expiring_map.retain(|k, v: &mut ClonableSession| {
                    // If the value is locked, that means it's in use 
                    // and we do not need to delete it. 
                    if let Ok(value) = v.try_read() {
                        if value.last_access.add(SESSION_EXPIRE_DURATION) > Instant::now() {
                            return true
                        }
                        eprintln!("Deleting session {}", k);
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

    fn new_session(&self) -> (String, ClonableSession) {
        let data = Arc::new(RwLock::new(SessionData::new()));

        let sess_key = random::<u128>().to_string();
        let extra = sess_key.clone();
        let cloned = data.clone();
        eprintln!("Inserting session: {}", &sess_key);
        self.inner.insert(sess_key, data);

        (extra, cloned)
    }

    fn build_session_cookie(sess_id: String) -> Cookie<'static> {
        Cookie::build((SESSION_COOKIE_NAME, sess_id))
            .http_only(true)
            .same_site(SameSite::Strict)
            .secure(true)
            .build()
    }

    fn make_new_session(&self, request: &mut Request<'_>) -> ClonableSession {
        let sess_data = self.new_session();
        let new_cookie = SessionStorage::build_session_cookie(sess_data.0);

        request.cookies().add_private(new_cookie);
        sess_data.1
    }
}

// This shouldn't be a fairing. It causes so many race conditions.
// It also makes more sense to only check sessions on requests that need it.
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
            eprintln!("Reading map");
            for val in self.inner.iter() {
                let map_len = val.read().await.type_state.len();
                eprintln!("Session: {} in map with {} values", val.key(), map_len);
            }

            if let Some(session) = self.inner.try_get(session_cookie.value()).try_unwrap() {
                let local_session = session.clone();
                local_session.write().await.last_access = Instant::now();
                request.local_cache(|| local_session);
            }
            
            // This session is invalid, make a new one!
            let sess_data = self.make_new_session(request);
            request.local_cache(|| sess_data);
        } else {
            // make new session
            let sess_data = self.make_new_session(request);
            request.local_cache(|| sess_data);
        }
    }

    async fn on_shutdown(&self, _rocket: &Rocket<Orbit>) {
        self.join_handle.abort();
    }
}

impl<T> Session<T> {
    fn new(ref_val: Arc<T>) -> Self{
        Session {
            ref_val
        }
    }
}

impl<T> Deref for Session<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.ref_val
    }
}

impl SessionWriter {
    pub async fn insert_session_data<'b, U: Send + Sync + 'static>(&'b self, val: U) {
        let guard = self.inner.write().await;
        guard.type_state.set(Arc::new(val));
    }

    pub async fn get_session_data<U: 'static>(&self) -> Option<Arc<U>> {
        let guard = self.inner.read().await;
        let val = guard.type_state.try_get::<Arc<U>>();
        match val {
            Some(arc_val) => Some(arc_val.clone()),
            None => None
        }
    }
}

#[rocket::async_trait]
impl<'r, T: 'static> FromRequest<'r> for Session<T> {
    type Error = SessionErr;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Why can't I just request from local_cache?
        // If we can't get from cache, that means a session has failed.
        // But this should never happen as the fairing should run first.
        let sess = request.local_cache(|| Arc::new(RwLock::new(SessionData::new())));
        let sess_data = sess.read().await;
        
        let val_opt = sess_data.type_state.try_get::<Arc<T>>();
        
        match val_opt {
            Some(val) => Outcome::Success(Session::new(val.clone())),
            None => Outcome::Error((Status::Ok, SessionErr("Unable to find in store".to_owned())))
        }
    }
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for SessionWriter {
    type Error = ();

    async fn from_request(request: &'a Request<'_>) -> Outcome<Self, Self::Error> {
        let sess = request.local_cache(|| Arc::new(RwLock::new(SessionData::new())));
        Outcome::Success(SessionWriter { inner: sess.clone() })
    }
}