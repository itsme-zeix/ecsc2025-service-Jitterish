use crate::dbconnector::DbConnection;
use crate::models::UserModel;
use crate::Queries;
use interface::frontend::UserSession;
use rocket::serde::json::{json, Json};
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use serde_json::value::RawValue;
use serde_json::Value;
use std::io::{Error, ErrorKind};

pub async fn get_user(queries: &Queries, name: &str) -> std::io::Result<Option<UserModel>> {
    // TODO we might need a cache here
    let mut db = DbConnection::open("_").await?;
    let users: Vec<UserModel> = db.execute_query(&queries.user, "user_by_username", &name).await?;
    Ok(users.into_iter().nth(0))
}

async fn check_api_user(queries: &Queries, name: &str) -> std::io::Result<()> {
    let allowed = get_user(queries, name)
        .await?
        .map(|user| user.allow_public_kv)
        .unwrap_or(false);
    if !allowed {
        Err(Error::new(ErrorKind::Other, "no such database"))
    } else {
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct KvEntry<'r> {
    key: &'r str,
    value: Box<RawValue>,
    owner: Option<&'r str>,
}

#[derive(Serialize, Deserialize)]
struct KvToken<'r> {
    key: &'r str,
    token: &'r str,
}

#[post("/<user>/create", format = "json", data = "<value>")]
async fn api_create(
    queries: &State<Queries>,
    session: Option<UserSession>,
    user: &str,
    value: Json<Box<RawValue>>,
) -> std::io::Result<Json<Value>> {
    check_api_user(queries, user).await?;

    let mut db = DbConnection::open(user).await?;

    let key = uuid::Uuid::new_v4().to_string();
    db.append(
        "kv",
        KvEntry {
            key: key.as_str(),
            value: value.into_inner(),
            owner: session.as_ref().map(|s| s.username.as_str()),
        },
    )
    .await?;

    Ok(Json(json!({"key": key})))
}

#[post("/<user>/grant/<key>/<token>")]
async fn api_token(
    queries: &State<Queries>,
    session: Option<UserSession>,
    user: &str,
    key: &str,
    token: &str,
) -> std::io::Result<Json<Value>> {
    check_api_user(queries, user).await?;

    let mut db = DbConnection::open(user).await?;

    let result: bool = db
        .execute_func(
            &queries.api,
            "is_authorized",
            &json!({"key": key, "user": session.map(|s| s.username), "token": ""}),
        )
        .await?;

    if result {
        db.append("kv", KvToken { key, token }).await?;
    }

    Ok(Json(json!({"ok": result})))
}

#[get("/<user>/<what>/<key>?<token>")]
async fn api_get(
    queries: &State<Queries>,
    session: Option<UserSession>,
    user: &str,
    what: &str,
    key: Option<&str>,
    token: Option<&str>,
) -> std::io::Result<Json<Value>> {
    check_api_user(queries, user).await?;

    let mut db = DbConnection::open(user).await?;

    let result: Value = db
        .execute_func(
            &queries.api,
            &format!("get_{}", what),
            &json!({"key": key, "user": session.map(|s| s.username), "token": token}),
        )
        .await?;
    Ok(Json(result))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![api_create, api_token, api_get]
}
