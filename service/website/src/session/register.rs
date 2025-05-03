use crate::dbconnector::DbConnection;
use crate::models::UserModel;
use interface::frontend::{Query, Register, UserSession};
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_dyn_templates::Template;
use serde_json::Value;
use sha2::{Digest, Sha256};
use crate::Queries;

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


#[post("/session/query", data = "<query>")]
pub fn query_user(_user: UserSession, query: Form<Query<'_>>) -> Redirect {
    println!("{:?}", query);
    // TODO: query db and handle the errors accordingly
    Redirect::to(uri!(crate::session::profile::index))
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
    // TODO validate input
    let mut is_public = true;
    if register.accounttype == "business" {
        is_public = false;
    }
    let user = UserModel {
        username: register.username.to_owned(),
        password_hash: hex::encode(Sha256::digest(register.password.as_bytes())),
        allow_public_queries: is_public,
        allow_public_kv: true,
    };

    let mut db = DbConnection::open("_")
        .await
        .expect("db offline");
    let existing_users: Vec<Value> = db
        .execute_query(&queries.user, "user_by_username", &register.username)
        .await
        .expect("query error");
    if !existing_users.is_empty() {
        return Err(Flash::error(Redirect::to(uri!(register_page)), "Username already taken."));
    }
    db.append("users", user).await
        .map_err(|e| Flash::error(Redirect::to(uri!(register_page)), format!("Append failed: {}", e)))?;

    jar.add_private(("username", register.username.to_owned()));
    Ok(Redirect::to(uri!(crate::session::profile::index)))
}
