mod data;

use rocket::{get, launch, routes};
use rocket::fs::FileServer;

#[get("/")]
fn index() -> &'static str {
    "Hello world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", FileServer::from("./gacha_front/dist/"))
}
