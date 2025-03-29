use crate::dbconnector::DbConnection;
use crate::models::UserModel;
use interface::frontend::{Register, User};
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
#[get("/register")]
pub(super) fn register(_user: User) -> Redirect {
    Redirect::to(uri!("/session/index"))
}

// we are not registered here
#[get("/register", rank = 2)]
pub fn register_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render("register", &flash)
}

#[post("/register", data = "<register>")]
pub(super) async fn post_register(
    queries: &State<Queries>,
    jar: &CookieJar<'_>,
    register: Form<Register<'_>>,
) -> Result<Redirect, Flash<Redirect>> {
    // TODO validate input

    let user = UserModel {
        username: register.username.to_owned(),
        password_hash: hex::encode(Sha256::digest(register.password.as_bytes())),
        allow_public_queries: false,
        allow_public_kv: false,
    };

    let mut db = DbConnection::open("_")
        .await
        .expect("no idea how error handling works in rocket");
    let existing_users: Vec<Value> = db
        .execute_query(&queries.user, "user_by_username", &register.username)
        .await
        .expect("...");
    if !existing_users.is_empty() {
        return Err(Flash::error(Redirect::to(uri!(register_page)), "Username already taken."));
    }
    db.append("users", user).await.expect("...");

    let s = serde_json::to_string(&register.into_inner()).expect("Could not serialize user!");
    // println!("Registered successfully as {} {} with account type {}!", register.firstname, register.lastname, register.accounttype);
    println!("Serialized user: {:?}", s);
    jar.add_private(("user_id", "1"));
    Ok(Redirect::to(uri!("/session/")))
    //Err(Flash::error(Redirect::to(uri!(login_page)), "Invalid username/password."))
}
