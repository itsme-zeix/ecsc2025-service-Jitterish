use rocket::{http::Status, outcome::IntoOutcome, request::{self, FromRequest}, FromForm, Request};
use serde::{Deserialize, Serialize};


#[derive(FromForm, Serialize, Deserialize, Debug)]
pub struct Query<'r> {
    pub target: &'r str,
    pub query_str: &'r str,
}

#[derive(FromForm, Serialize, Deserialize)]
pub struct Register<'r> {
    pub username: &'r str,
    pub password: &'r str,
    firstname: &'r str,
    lastname: &'r str,
    pub accounttype: &'r str,
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
pub struct UserSession {
    pub username: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserSession {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<UserSession, Self::Error> {
        request.cookies().get_private("username")
            .map(|cookie| UserSession { username: cookie.value().to_owned() })
            .or_forward(Status::Unauthorized)
    }
}

#[derive(FromForm, Serialize, Deserialize, Debug, Clone)]
pub struct DbForm<'r> {
    pub db: &'r str,
}

#[derive(FromForm, Serialize, Deserialize, Debug, Clone)]
pub struct AppendForm<'r> {
    pub collection: &'r str,
    pub data: &'r str,
}

#[derive(FromForm, Serialize, Deserialize, Debug, Clone)]
pub struct SimpleQueryForm<'r> {
    pub query: &'r str,
    pub params: &'r str,
}

#[derive(FromForm, Serialize, Deserialize, Debug, Clone)]
pub struct QueryForm<'r> {
    pub query: &'r str,
    pub query_name: &'r str,
    pub params: &'r str,
}
