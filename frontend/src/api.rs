use crate::dbconnector::DbConnection;
use rocket::serde::json::{json, Json};
use rocket::serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use serde_json::Value;

#[derive(Serialize, Deserialize)]
struct KvEntry<'r> {
    key: &'r str,
    value: Box<RawValue>,
    owner: &'r str,
}

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

pub fn routes() -> Vec<rocket::Route> {
    routes![api_create]
}
