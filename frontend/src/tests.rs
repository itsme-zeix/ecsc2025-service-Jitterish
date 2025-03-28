use rocket::serde::{Deserialize, Serialize};
use crate::dbconnector::{one, DbConnection};
use rocket::tokio;
use serde_json::json;


#[derive( Serialize, Deserialize, Debug, Clone)]
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

    let coderef = conn.prepare(
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

    let tmp: Vec<UserModel> = conn.execute_simple(&coderef, "first_entry", json!("null")).await.expect("execute 2");
    let result3: &UserModel = one(&tmp).expect("");
    println!("Query with limit 1: {:?}", result3);
}
