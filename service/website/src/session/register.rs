use crate::dbconnector::DbConnection;
use crate::models::UserModel;
use crate::Queries;
use interface::frontend::{Register, UserSession};
use lazy_static::lazy_static;
use regex::Regex;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_dyn_templates::{Template};
use serde_json::{Value};
use sha2::{Digest, Sha256};

lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"^\w{3,}$").unwrap();
    static ref PASSWORD_REGEX: Regex = Regex::new(r"^.{8,}$").unwrap();
}

// if we are registered (that is, we have a user), redirect to index instead
#[get("/session/register")]
pub(super) fn register(_user: UserSession) -> Redirect {
    Redirect::to(uri!(crate::session::profile::index))
}

// we are not registered here
#[get("/session/register", rank = 2)]
pub fn register_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render("register", &flash)
}

// do the registration
// this means creating a user in the web interface that has all required
// attributes for creating a user in the actual db
#[post("/session/register", data = "<register>")]
pub(super) async fn post_register(
    queries: &State<Queries>,
    jar: &CookieJar<'_>,
    register: Form<Register<'_>>,
) -> Result<Redirect, Flash<Redirect>> {
    let user = UserModel {
        username: register.username.to_owned(),
        password_hash: hex::encode(Sha256::digest(register.password.as_bytes())),
        allow_public_queries: register.accounttype == "community",
        allow_public_kv: register.accounttype == "enterprise",
    };

    if !USERNAME_REGEX.is_match(&register.username) {
        return Err(Flash::error(Redirect::to(uri!(register_page)), "Invalid username."));
    }
    if !PASSWORD_REGEX.is_match(&register.password) {
        return Err(Flash::error(Redirect::to(uri!(register_page)), "Invalid password."));
    }

    let mut db = DbConnection::open("_")
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "Database not reachable"))?;
    let existing_users: Vec<Value> = db
        .execute(&queries.user, "user_by_username", &register.username, false)
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "User by username query error"))?;
    if !existing_users.is_empty() {
        return Err(Flash::error(Redirect::to(uri!(register_page)), "Username already taken."));
    }
    db.append("users", user)
        .await
        .map_err(|e| Flash::error(Redirect::to(uri!(register_page)), format!("Append failed: {}", e)))?;

    jar.add_private(("username", register.username.to_owned()));
    Ok(Redirect::to(uri!(crate::session::profile::index)))
}
