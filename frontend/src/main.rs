#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;

mod dbconnector;
mod session;
mod api;
mod models;

use std::path::Path;
use rocket::fs::{FileServer, NamedFile};
use rocket_dyn_templates::Template;
use interface::database::CodeRef;
use crate::dbconnector::DbConnection;

#[get("/")]
async fn index() -> Option<NamedFile>{
    let path = Path::new("static/index.html");
    NamedFile::open(path).await.ok()
}

#[derive(Debug)]
pub struct Queries {
    pub user: CodeRef,
    pub api: CodeRef,
}

async fn prepare_queries() -> Queries {
    // TODO we might want to put this in a separate file
    let code_user = r#"
func test() { return [1+2, 2+3]; }

func count(data, acc) { return (acc ?: 0) + 1; }

func is_username(entry, username) { return entry["username"] == username; }

query user_by_username on users
filter is_username
limit 1;

query user_count on users reduce count;

    "#;

    // Write your API in database language!
    let code_api = r#"
// params: {"key": "...", "user": "...", "token": "..."|null}
func get_value(params) {
    result = value(params);
    if result["auth"] return result["value"];
    return null;
}

// params: {"key": "...", "user": "...", "token": "..."|null}
func is_authorized(params) {  // = can add token?
    return !!value(params)["auth"];
}

func by_key(entry, params) {
    return entry["key"] == params["key"];
}

func select_key(entry, acc, params) {
    if !defined acc acc = {};
    if entry["value"] acc["value"] = entry["value"];
    if entry["owner"] == params["user"] acc["auth"] = true;
    if entry["token"] == params["token"] acc["auth"] = true;
    return acc;
}

func has_value(entry) {
    return !!entry["value"];
}

func extract_key(entry) {
    return entry["key"];
}

query value on kv
filter by_key
reduce select_key;

query get_keys on kv
filter has_value
map extract_key;

    "#;

    let mut conn = DbConnection::open("_").await.expect("Connection to database failed");
    Queries {
        user: conn.prepare(code_user).await.expect("Could not prepare queries"),
        api: conn.prepare(code_api).await.expect("Could not prepare queries"),
    }
}

#[launch]
async fn rocket() -> _ {
    let queries = prepare_queries().await;
    println!("Query refs: {:?}", &queries);
    rocket::build()
        .manage(queries)
        .attach(Template::fairing())
        .mount("/static", FileServer::from("static/"))
        .mount("/", routes![index])
        .mount("/session", session::routes())
        .mount("/api", api::routes())
}
