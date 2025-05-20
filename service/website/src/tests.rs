use crate::dbconnector::{one, DbConnection};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio;
use serde_json::json;
use serde_json::value::RawValue;

// REMOVE IN PROD

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UserModel {
    username: String,
}

#[tokio::test]
async fn test_database_connection() {
    let mut conn = DbConnection::open("frontend_tests").await.expect("open");

    conn.append(
        "demo",
        UserModel {
            username: "testuser".to_owned(),
        },
    )
    .await
    .expect("append");

    let coderef = conn
        .prepare(
            r"
        query all_entries on demo;
        query first_entry on demo limit 1;
    ",
        )
        .await
        .expect("prepare");

    println!("Coderef: {:?}", coderef);

    // raw queries (all println + return value from code)
    let result: Vec<Vec<UserModel>> = conn.execute(&coderef, "all_entries", json!("null")).await.expect("execute 1");
    println!("Full Result: {:?}", result);

    let result2 = one(&result).expect("");
    println!("Query Result only: {:?}", result2);

    let tmp: Vec<UserModel> = conn
        .execute_query(&coderef, "first_entry", json!("null"))
        .await
        .expect("execute 2");
    let result3: &UserModel = one(&tmp).expect("");
    println!("Query with limit 1: {:?}", result3);
}

#[derive(Serialize, Deserialize, Debug)]
struct TestX<'r> {
    a: &'r str,
    data: Box<RawValue>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TestY<'r> {
    Y { a: &'r str, data: Box<RawValue> },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TestZ<'r> {
    Z { a: &'r str, data: Box<RawValue> },
}

#[tokio::test]
async fn test_serde() {
    println!("{:?}", serde_json::from_str::<TestX>(r#"{"a": "b", "data": {"x":"y"}}"#));
    println!(
        "{:?}",
        serde_json::from_str::<TestY>(r#"{"type": "Y", "a": "b", "data": {"x":"y"}}"#)
    );
    println!(
        "{:?}",
        serde_json::from_str::<TestZ>(r#"{"Z": {"a": "b", "data": {"x":"y"}}}"#)
    );
}
