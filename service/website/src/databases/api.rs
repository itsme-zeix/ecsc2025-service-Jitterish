use crate::dbconnector::DbConnection;
use crate::Queries;
use interface::frontend::UserModel;
use interface::frontend::UserSession;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use serde_json::{json, Value};
use std::io::{Error, ErrorKind};

pub async fn get_user(queries: &Queries, name: &str) -> std::io::Result<Option<UserModel>> {
    // TODO we might need a cache here
    let mut db = DbConnection::open("_").await?;
    let users: Vec<UserModel> = db.execute(&queries.user, "user_by_username", &name, false).await?;
    Ok(users.into_iter().nth(0))
}

async fn check_kv_user(queries: &Queries, name: &str) -> std::io::Result<()> {
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

#[post("/api/<user>/create", format = "json", data = "<value>")]
pub async fn api_create(
    queries: &State<Queries>,
    session: Option<UserSession>,
    user: &str,
    value: Json<Box<RawValue>>,
) -> std::io::Result<Json<Value>> {
    check_kv_user(queries, user).await?;

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

#[post("/api/<user>/grant/<key>/<token>")]
pub async fn api_token(
    queries: &State<Queries>,
    session: Option<UserSession>,
    user: &str,
    key: &str,
    token: &str,
) -> std::io::Result<Json<Value>> {
    check_kv_user(queries, user).await?;

    let mut db = DbConnection::open(user).await?;

    let result: bool = db
        .execute_one(
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

#[get("/api/<user>/<what>/<key>?<token>")]
pub async fn api_get(
    queries: &State<Queries>,
    session: Option<UserSession>,
    user: &str,
    what: &str,
    key: Option<&str>,
    token: Option<&str>,
) -> std::io::Result<Json<Value>> {
    check_kv_user(queries, user).await?;

    let mut db = DbConnection::open(user).await?;

    let result: Vec<Value> = db
        .execute(
            &queries.api,
            &format!("get_{}", what),
            &json!({"key": key, "user": session.map(|s| s.username), "token": token}),
            false,
        )
        .await?;
    Ok(Json(Value::from(result)))
}
