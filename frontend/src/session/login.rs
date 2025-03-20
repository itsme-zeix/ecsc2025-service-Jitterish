use rocket::request::FlashMessage;
use rocket::response::{Redirect, Flash};
use rocket::http::CookieJar;
use rocket::form::Form;

use rocket_dyn_templates::Template;

use super::{Login, User};


#[get("/login")]
pub(super) fn login(_user: User) -> Redirect {
    Redirect::to(uri!("/session/profile"))
}

#[get("/login", rank = 2)]
pub(super) fn login_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render("login", &flash)
}

#[post("/login", data = "<login>")]
pub(super) fn post_login(jar: &CookieJar<'_>, login: Form<Login<'_>>) -> Result<Redirect, Flash<Redirect>> {
    if login.username == "Sergio" && login.password == "password" {
        jar.add_private(("user_id", "1"));
        Ok(Redirect::to(uri!(crate::session::profile::index)))
    } else {
        Err(Flash::error(Redirect::to(uri!(login_page)), "Invalid username/password."))
    }
}


