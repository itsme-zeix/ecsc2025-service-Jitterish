#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

mod databases;
mod dbconnector;
mod models;
mod session;

use crate::dbconnector::DbConnection;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use interface::database::CodeRef;
use interface::frontend::UserSession;
use rocket::fs::FileServer;
use rocket::request::FlashMessage;
use rocket::response::Redirect;
use rocket_dyn_templates::{context, Template};
use std::error::Error;
use std::fs;
use std::fs::create_dir_all;
use std::path::Path;

#[get("/")]
fn index_logged_in(_user: UserSession) -> Redirect {
    Redirect::to(uri!(crate::session::profile::index))
}

#[get("/", rank = 2)]
fn index(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render("login", context! {
        page: "login",
        flash: flash
    })
}

#[derive(Debug)]
pub struct Queries {
    pub user: CodeRef,
    pub api: CodeRef,
}

async fn prepare_queries() -> Queries {
    let code_user = fs::read_to_string("queries/user.qry").expect("cannot read user.qry");
    let code_api = fs::read_to_string("queries/api.qry").expect("cannot read api.qry");

    let mut conn = DbConnection::open("_").await.expect("Connection to database failed");
    Queries {
        user: conn.prepare(&code_user).await.expect("Could not prepare queries"),
        api: conn.prepare(&code_api).await.expect("Could not prepare queries"),
    }
}

fn get_secret_key() -> Result<String, Box<dyn Error>> {
    let secret_file = Path::new("data/secret.txt");
    if secret_file.exists() {
        Ok(fs::read_to_string(secret_file)?)
    } else {
        let mut buf = [0u8; 32];
        getrandom::fill(&mut buf).expect("getrandom failed");
        let key = BASE64_STANDARD.encode(buf);
        create_dir_all("data")?;
        fs::write(secret_file, &key)?;
        Ok(key)
    }
}

#[launch]
async fn rocket() -> _ {
    let queries = prepare_queries().await;
    let secret_key = get_secret_key().expect("failed to get secret key");
    println!("Query refs: {:?}", &queries);
    rocket::build()
        .configure(rocket::Config::figment().merge(("secret_key", secret_key)))
        .manage(queries)
        .attach(Template::fairing())
        .mount("/static", FileServer::from("static/"))
        .mount("/", routes![index, index_logged_in])
        .mount("/", session::routes())
        .mount("/", databases::routes())
}
