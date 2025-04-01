
use interface::frontend::UserSession;
use rocket::response::Redirect;

use rocket_dyn_templates::{Template, context};


#[get("/")]
pub(super) fn index(user: UserSession) -> Template {
    Template::render("session", context! {
        username: user.username,
    })
}

#[get("/", rank = 2)]
pub(super) fn no_auth_index() -> Redirect {
    Redirect::to(uri!("/session/login"))
}
