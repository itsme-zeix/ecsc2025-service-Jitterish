#[macro_use] extern crate rocket;

use rocket::http::CookieJar;
use rocket::response::content;
//use rocket::fs::NamedFile;
use rocket::form::{Form, Strict};
use rocket_dyn_templates::Template;
use std::fs::File;
use std::io::prelude::*;
use rocket_dyn_templates::context;

/*
 * according to the docs, this automatically handles
 * duplicates and unexpected values. How exactly does it 
 * do that? (using strict disables this behavior)
 */
#[derive(FromForm)]
struct User {
    username: String,
    password: String,
}

#[get("/", rank = 2)]
fn no_auth_index() -> &'static str {
    "hello not authenticated"
}

// a template example with an actual
// template
#[get("/register")]
fn templates() -> Template {
    Template::render("index", context! {})
}


#[post("/register", data = "<user>")]
fn templates() -> Template {
    format!("Logging in user with username {} and password {}", user.name, user.password)
}


// this is better: this is a raw html file,
// but has a template file extension and is served
// with empty context
#[get("/static-templates")]
fn static_templates() -> Template {
    Template::render("static-index", context! {})
}


// return raw html
// this is a bit shitty, but works
#[get("/static")]
fn static_html() -> content::RawHtml<String> {
    let mut f = File::open("static/static-index.html").expect("File does not exist");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("Could not read file to string");
    content::RawHtml(contents)
}

#[get("/<name>/<age>", rank = 2)]
fn wave(name: &str, age: u8) -> String {
    format!("👋 Hello, {} year old named {}!", age, name)
}

// a post endpoint
// for actual user management, we want a cookie jar
#[post("/login", data = "<user>")]
fn login(user: Form<User>) -> String{
    format!("Logging in user with username {} and password {}", user.name, user.password)
}


#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, login, static_html, static_templates, templates])
                .mount("/wave", routes![wave])
                // required to do templates
                .attach(Template::fairing())
}

