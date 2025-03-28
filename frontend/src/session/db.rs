use std::ops::Deref;
use interface::database::CodeRef;
use rocket::State;
use serde_json::{json, Value};

use crate::dbconnector::DbConnection;

#[get("/db")]
pub(super) async fn db(queries: &State<CodeRef>) -> std::io::Result<()> {
    let mut conn = DbConnection::open("_").await?;
    let result: Vec<Value> = conn.execute_simple(queries.deref(), "test", json!("null")).await?;
    println!("{:?}", result);
    Ok(())
}
