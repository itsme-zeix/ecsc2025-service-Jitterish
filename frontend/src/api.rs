use crate::dbconnector::DbConnection;
use crate::Queries;
use rocket::serde::json::{json, Json};
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use serde_json::value::RawValue;
use serde_json::Value;
use std::panic::resume_unwind;

#[derive(Serialize, Deserialize)]
struct KvEntry<'r> {
    key: &'r str,
    value: Box<RawValue>,
    owner: &'r str,
}

#[derive(Serialize, Deserialize)]
struct KvToken<'r> {
    key: &'r str,
    token: &'r str,
}

#[post("/create", format = "json", data = "<value>")]
async fn api_create(value: Json<Box<RawValue>>) -> Json<Value> {
    // TODO select database, check permissions

    let mut db = DbConnection::open("kv")
        .await
        .expect("no idea how error handling works in rocket");

    let key = uuid::Uuid::new_v4().to_string();
    db.append(
        "kv",
        KvEntry {
            key: key.as_str(),
            value: value.into_inner(),
            owner: "root",
        },
    )
    .await
    .expect("...");

    Json(json!({"key": key}))
}

#[post("/grant/<key>/<token>")]
async fn api_token(queries: &State<Queries>, key: &str, token: &str) -> std::io::Result<Json<Value>> {
    // TODO select database, check permissions

    let mut db = DbConnection::open("kv").await?;

    let result: bool = db
        .execute_func(
            &queries.api,
            "is_authorized",
            &json!({"key": key, "user": "notroot", "token": ""}),
        )
        .await?;

    if result {
        db.append("kv", KvToken { key, token }).await?;
    }

    Ok(Json(json!({"ok": result})))
}

#[get("/<what>/<key>?<token>")]
async fn api_get(queries: &State<Queries>, what: &str, key: Option<&str>, token: Option<&str>) -> std::io::Result<Json<Value>> {
    // TODO select database, check permissions

    let mut db = DbConnection::open("kv").await?;

    let result: Value = db
        .execute_func(
            &queries.api,
            &format!("get_{}", what),
            &json!({"key": key, "user": "root", "token": token}),
        )
        .await?;
    Ok(Json(result))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![api_create, api_token, api_get]
}
