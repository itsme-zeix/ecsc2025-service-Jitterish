#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;

mod dbconnector;
mod session; 


use std::path::Path;

use rocket::fs::{FileServer, NamedFile};
use rocket_dyn_templates::Template;

#[get("/")]
async fn index() -> Option<NamedFile>{
    let path = Path::new("static/index.html");
    NamedFile::open(path).await.ok()
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::fairing())
        .mount("/static", FileServer::from("static/"))
        .mount("/", routes![index])
        .mount("/session", session::routes())
}
