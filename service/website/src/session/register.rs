use crate::dbconnector::DbConnection;
use crate::Queries;
use interface::frontend::{Register, UserModel, UserSession};
use lazy_static::lazy_static;
use regex::Regex;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_dyn_templates::{Template};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"^\w{3,32}$").unwrap();
    static ref PASSWORD_REGEX: Regex = Regex::new(r"^.{8,}$").unwrap();
}

// if we are registered (that is, we have a user), redirect to index instead
#[get("/session/register")]
pub(super) fn register(_session: UserSession) -> Redirect {
    Redirect::to(uri!(crate::session::profile::index))
}

// we are not registered here
#[get("/session/register", rank = 2)]
pub fn register_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render("register", &flash)
}


fn recover_str(status_in: &str) -> Result<Value, Flash<Redirect>> {
    // let status_parsed: Value = serde_json::from_str(status_in).map_err(|_| Flash::error(Redirect::to("/"), "Status field could not be parsed!"))?;
    let status_parsed: Result<Value, _> = serde_json::from_str(status_in);

    if status_parsed.is_err() {
        let mut chars: Vec<char> = status_in.chars().collect();
        let first = chars.first().cloned();
        let last = chars.pop();
        if first.is_none() || last.is_none() {
            return Err(Flash::error(Redirect::to("/"), "Status field could not be parsed!"));
        }
        // if we only have one non-alphanumeric char at start or end, we can recover with quotes
        if first.unwrap().is_alphanumeric() || last.unwrap().is_alphanumeric() {
            let mut quoted = String::from(status_in);
            quoted.insert(0,'"');
            quoted.push('"');
            let n = serde_json::from_str(&quoted);
            if n.is_err() {
                Err(Flash::error(Redirect::to("/"), "Status field could not be parsed!"))
            } else {
                Ok(n.unwrap())
            }
        } else {
            // neither start nor end is alphanumeric, we cannot recover
            Err(Flash::error(Redirect::to("/"), "Status field could not be parsed!"))
        }
    } else {
        Ok(status_parsed.unwrap())
    }
}
    

fn setup_status(status_in: &str) -> Result<Value, Flash<Redirect>> {
    // we allow two parsing cases: 
    // supply a string -> transformed to expected json
    // supply a json -> check and fill missing fields
    let status_parsed: Result<Value, _> = serde_json::from_str(status_in);
    let result = if status_parsed.is_err() {
        recover_str(status_in)?
    } else {
        status_parsed.unwrap()
    };

    let r = match result {
        Value::String(inner) => {
            json!({
                "num_reports": 0,
                "looking_for_job": false,
                "current_salary": 0,
                "custom": inner,
            })
        },
        Value::Object(mut map) => {
            // we allow exactly 3 properties:
            // num_reports, looking_for_job, current_salary
            // if they do not exist, we init them 
            if map.get("num_reports").is_none() {
                map.insert(String::from("num_reports"),json!(0));
            }
            if map.get("looking_for_job").is_none() {
                map.insert(String::from("looking_for_job"),json!(false));
            }
            if map.get("current_salary").is_none() {
                map.insert(String::from("current_salary"),json!(0));
            }
            if map.get("custom").is_none() {
                map.insert(String::from("custom"),json!(""));
            }
            json!(map)
        },
        _ => {
            json!({
                "num_reports": 0,
                "looking_for_job": false,
                "current_salary": 0,
                "custom": String::new(),
            })
        },
    };
    Ok(r)
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
    let status = setup_status(register.status)?;
    let user = UserModel {
        username: register.username.to_owned(),
        password_hash: hex::encode(Sha256::digest(register.password.as_bytes())),
        status: status,
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
    db.append("users", user.clone())
        .await
        .map_err(|e| Flash::error(Redirect::to(uri!(register_page)), format!("Append failed: {}", e)))?;
    // this has now exactly one match
    let mut existing_users_new: Vec<Value> = db
        .execute(&queries.user, "user_by_username", &register.username, false)
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "User by username query error"))?;
    if let Some(new_user) = existing_users_new.pop() {
        let user = serde_json::to_string(&new_user).map_err(|_| Flash::error(Redirect::to(uri!(register_page)), "Could not serialize user!"))?;
        jar.add_private(("user", user));
        Ok(Redirect::to(uri!(crate::session::profile::index)))
    } else {
        Err(Flash::error(Redirect::to(uri!(register_page)), "User not added to db."))
    }

}
