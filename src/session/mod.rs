pub mod storage;

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::ops::Deref;

use rocket::Request;
use rocket::request::{FromRequest, self};
use rocket::tokio::sync::RwLock;
use rocket::outcome::Outcome;
use rocket::http::Status;
use state::TypeMap;
use self::storage::SessionStorage;

static SESSION_COOKIE_NAME: &str = "SESSIONID";
static SESSION_EXPIRE_DURATION: Duration = Duration::from_secs(3600);

#[derive(Debug)]
pub struct SessionErr(&'static str);

#[macro_export]
macro_rules! some_session_err {
    ($expr:expr, $err_str:expr) => (match $expr {
        Some(val) => val,
        None => return rocket::outcome::Outcome::Error((rocket::http::Status::InternalServerError, $crate::session::SessionErr($err_str))) 
    });
    ($expr:expr, $status:expr) => (match $expr {
        Some(val) => val,
        None => return rocket::outcome::Outcome::Error(($status, $crate::session::SessionErr("")))
    });
    ($expr:expr, $status:expr, $err_str:expr) => (match $expr {
        Some(val) => val,
        None => return rocket::outcome::Outcome::Error(($status, $crate::session::SessionErr($err_str)))
    });
}

pub type ClonableSession = Arc<RwLock<SessionData>>;

#[derive(Debug)]
pub struct SessionData {
    pub (super) type_state: TypeMap![Send + Sync],
    pub (super) last_access: Instant
}

pub struct Session<T> {
    ref_val: Arc<T>,
}

pub struct SessionWriter {
    inner: ClonableSession
}

// Struct Impls
impl SessionData {
    pub(super) fn new() -> Self {
        SessionData {
            type_state: <TypeMap![Send + Sync]>::new(),
            last_access: Instant::now()
        }
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

// Non-Rocket Trait Impls
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

// Rocket Impls 
#[rocket::async_trait]
impl<'r, T: 'static> FromRequest<'r> for Session<T> {
    type Error = SessionErr;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, SessionErr> {
        let session = match SessionStorage::get_or_create_sess(request).await {
            Ok(val) => val,
            Err(e) => return Outcome::Error((Status::InternalServerError, e)) 
        };

        let sess_data = session.read().await;
        let val_opt = sess_data.type_state.try_get::<Arc<T>>();
        
        match val_opt {
            Some(val) => Outcome::Success(Session::new(val.clone())),
            None => Outcome::Error((Status::InternalServerError, SessionErr("Unable to find value in store")))
        }
    }
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for SessionWriter {
    type Error = SessionErr;

    async fn from_request(request: &'a Request<'_>) -> request::Outcome<Self, Self::Error> {
        let session = match SessionStorage::get_or_create_sess(request).await {
            Ok(val) => val,
            Err(e) => return Outcome::Error((Status::InternalServerError, e)) 
        };
        Outcome::Success(SessionWriter { inner: session })
    }
}