use rocket::response::{Redirect, Flash};
use rocket::http::CookieJar;

#[post("/logout")]
pub fn logout(jar: &CookieJar<'_>) -> Flash<Redirect> {
    jar.remove_private("user_id");
    Flash::success(Redirect::to(uri!("/")), "Successfully logged out.")
}
