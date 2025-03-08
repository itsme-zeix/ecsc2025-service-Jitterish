#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;

mod session;
mod dbconnector;

use rocket::response::content::RawHtml;
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;

#[get("/")]
fn index() -> RawHtml<&'static str> {
    RawHtml(r#"<a href="session">Use Sessions</a>."#)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::fairing())
        .mount("/static", FileServer::from("static"))
        .mount("/", routes![index])
        .mount("/session", session::routes())
}
