
use rocket::response::Redirect;

use rocket_dyn_templates::{Template, context};

use super::User;


#[get("/")]
pub(super) fn index(user: User) -> Template {
    Template::render("session", context! {
        user_id: user.0,
    })
}

#[get("/", rank = 2)]
pub(super) fn no_auth_index() -> Redirect {
    Redirect::to(uri!("/session/login"))
}
