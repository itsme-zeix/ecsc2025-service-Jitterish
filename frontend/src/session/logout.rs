use rocket::response::{Redirect, Flash};
use rocket::http::CookieJar;

#[post("/session/logout")]
pub fn logout(jar: &CookieJar<'_>) -> Flash<Redirect> {
    jar.remove_private("username");
    Flash::success(Redirect::to(uri!("/")), "Successfully logged out.")
}
