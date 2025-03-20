use rocket::{http::Status, outcome::IntoOutcome, request::{self, FromRequest}, Request};
use serde::{Deserialize, Serialize};


pub mod login;
pub mod register;
pub mod profile;
pub mod db;
pub mod logout;

#[derive(FromForm, Serialize, Deserialize)]
struct Register<'r> {
    username: &'r str,
    password: &'r str,
    firstname: &'r str,
    lastname: &'r str,
    income: u32,
    accounttype: &'r str,
}

#[derive(FromForm, Serialize, Deserialize)]
struct Login<'r> {
    username: &'r str,
    password: &'r str
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

pub fn routes() -> Vec<rocket::Route> {
    routes![profile::index, profile::no_auth_index, login::login, login::login_page, login::post_login, logout::logout, register::register, register::register_page, register::post_register, db::db]
}
