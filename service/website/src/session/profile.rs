
use interface::frontend::UserSession;
use rocket::response::Redirect;

use rocket_dyn_templates::{Template, context};

// TODO: we need to be careful here with special chars in usernames, as it is passed to the url
#[get("/session/profile")]
pub(super) fn index(user: UserSession) -> Template {
    Template::render("session", context! {
        username: user.username,
        page: "profile",
    })
}

#[get("/session/profile", rank = 2)]
pub(super) fn no_auth_index() -> Redirect {
    Redirect::to(uri!(crate::session::login::login_page))
}

