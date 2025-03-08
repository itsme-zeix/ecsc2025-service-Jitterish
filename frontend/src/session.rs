use rocket::outcome::IntoOutcome;
use rocket::request::{self, FlashMessage, FromRequest, Request};
use rocket::response::{Redirect, Flash};
use rocket::http::{CookieJar, Status};
use rocket::form::Form;

use crate::dbconnector::connect_to_backend;

use std::error::Error;

use rocket_dyn_templates::{Template, context};

#[derive(FromForm)]
struct Login<'r> {
    username: &'r str,
    password: &'r str
}

#[derive(FromForm)]
struct Register<'r> {
    username: &'r str,
    password: &'r str,
    firstname: &'r str,
    lastname: &'r str,
    income: u32,
}

#[derive(Debug)]
struct User(usize);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<User, Self::Error> {
        request.cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(User)
            .or_forward(Status::Unauthorized)
    }
}

#[macro_export]
macro_rules! session_uri {
    ($($t:tt)*) => (rocket::uri!("/session", $crate::session:: $($t)*))
}

pub use session_uri as uri;

#[get("/")]
fn index(user: User) -> Template {
    Template::render("session", context! {
        user_id: user.0,
    })
}

#[get("/", rank = 2)]
fn no_auth_index() -> Redirect {
    Redirect::to(uri!(login_page))
}

// if we are registered (that is, we have a user), redirect to index instead
#[get("/db")]
async fn db(_user: User) {
    connect_to_backend().await;
}

// if we are registered (that is, we have a user), redirect to index instead
#[get("/register")]
fn register(_user: User) -> Redirect {
    Redirect::to(uri!(index))
}

// we are not registered here
#[get("/register", rank = 2)]
fn register_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render("register", &flash)
}

#[post("/register", data = "<register>")]
fn post_register(jar: &CookieJar<'_>, register: Form<Register<'_>>) -> Result<Redirect, Flash<Redirect>> {
    println!("Registered successfully as {} {}!", register.firstname, register.lastname);
    jar.add_private(("user_id", "1"));
    Ok(Redirect::to(uri!(index)))
    //Err(Flash::error(Redirect::to(uri!(login_page)), "Invalid username/password."))
}

#[get("/login")]
fn login(_user: User) -> Redirect {
    Redirect::to(uri!(index))
}

#[get("/login", rank = 2)]
fn login_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render("login", &flash)
}
#[post("/login", data = "<login>")]
fn post_login(jar: &CookieJar<'_>, login: Form<Login<'_>>) -> Result<Redirect, Flash<Redirect>> {
    if login.username == "Sergio" && login.password == "password" {
        jar.add_private(("user_id", "1"));
        Ok(Redirect::to(uri!(index)))
    } else {
        Err(Flash::error(Redirect::to(uri!(login_page)), "Invalid username/password."))
    }
}

#[post("/logout")]
fn logout(jar: &CookieJar<'_>) -> Flash<Redirect> {
    jar.remove_private("user_id");
    Flash::success(Redirect::to(uri!(login_page)), "Successfully logged out.")
}

pub fn routes() -> Vec<rocket::Route> {
    routes![index, no_auth_index, login, login_page, post_login, logout, register, register_page, post_register, db]
}
