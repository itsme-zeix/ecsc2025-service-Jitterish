#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;

mod dbconnector;
mod session;
mod models;

use std::path::Path;
use rocket::form::validate::contains;
use rocket::fs::{FileServer, NamedFile};
use rocket_dyn_templates::Template;
use interface::database::CodeRef;
use crate::dbconnector::DbConnection;

#[get("/")]
async fn index() -> Option<NamedFile>{
    let path = Path::new("static/index.html");
    NamedFile::open(path).await.ok()
}

async fn prepare_queries() -> CodeRef {
    // TODO we might want to put this in a separate file
    let code = "\
func test() { return [1+2, 2+3]; }

func count(data, acc) { return (acc ?: 0) + 1; }

func is_username(entry, username) { return entry[\"username\"] == username; }

query user_by_username on users
filter is_username
limit 1;

query user_count on users reduce count;

    ";
    let mut conn = DbConnection::open("_").await.expect("Connection to database failed");
    conn.prepare(code).await.expect("Could not prepare queries")
}

#[launch]
async fn rocket() -> _ {
    let code_ref = prepare_queries().await;
    println!("Query ref: {:?}", &code_ref);
    rocket::build()
        .manage(code_ref)
        .attach(Template::fairing())
        .mount("/static", FileServer::from("static/"))
        .mount("/", routes![index])
        .mount("/session", session::routes())
}
