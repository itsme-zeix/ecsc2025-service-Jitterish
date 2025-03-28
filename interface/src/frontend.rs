use rocket::{http::Status, outcome::IntoOutcome, request::{self, FromRequest}, FromForm, Request};
use serde::{Deserialize, Serialize};

#[derive(FromForm, Serialize, Deserialize)]
pub struct Register<'r> {
    username: &'r str,
    password: &'r str,
    firstname: &'r str,
    lastname: &'r str,
    income: u32,
    accounttype: &'r str,
}

#[derive(FromForm, Serialize, Deserialize, Debug, Clone)]
pub struct Login<'r> {
    username: &'r str,
    password: &'r str
}

impl Login<'_> {
    pub fn username(&self) -> &str {
        self.username
    }
    pub fn password(&self) -> &str {
        self.password
    }
}

#[derive(Debug)]
pub struct User(pub usize);

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
