use crate::dbconnector::{one, DbConnection};
use crate::Queries;
use interface::frontend::{Login, UserModel, UserSession};
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_dyn_templates::{context, Template};
use sha2::{Digest, Sha256};

#[get("/session/login")]
pub(super) fn login(_user: UserSession) -> Redirect {
    Redirect::to(uri!(crate::session::profile::index))
}

#[get("/session/login", rank = 2)]
pub(super) fn login_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render(
        "login",
        context! {
            flash: &flash,
        },
    )
}

#[post("/session/login", data = "<login>")]
pub(super) async fn post_login(
    queries: &State<Queries>,
    jar: &CookieJar<'_>,
    login: Form<Login<'_>>,
) -> Result<Redirect, Flash<Redirect>> {
    let mut db = DbConnection::open("_")
        .await
        .map_err(|_| Flash::error(Redirect::to(uri!(login_page)), "Database offline"))?;
    let existing_users: Vec<UserModel> = db
        .execute(&queries.user, "user_by_username", &login.username(), false)
        .await
        .map_err(|_| Flash::error(Redirect::to(uri!(login_page)), "Query error"))?;
    println!("Existing users: {:?}", existing_users);
    let user = one(&existing_users).map_err(|_| Flash::error(Redirect::to(uri!(login_page)), "Invalid username"))?;
    if hex::encode(Sha256::digest(login.password().as_bytes())) != user.password_hash {
        return Err(Flash::error(Redirect::to(uri!(login_page)), "Invalid password"));
    }
    let user = serde_json::to_string(user).map_err(|_| Flash::error(Redirect::to(uri!(login_page)), "Could not serialize user!"))?;
    jar.add_private(("username", user));
    Ok(Redirect::to(uri!(crate::session::profile::index)))
}
