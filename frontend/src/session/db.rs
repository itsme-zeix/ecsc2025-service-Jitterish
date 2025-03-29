use rocket::State;
use serde_json::{json, Value};

use crate::dbconnector::DbConnection;
use crate::Queries;

#[get("/db")]
pub(super) async fn db(queries: &State<Queries>) -> std::io::Result<()> {
    let mut conn = DbConnection::open("_").await?;
    let result: Vec<Value> = conn.execute_simple(&queries.user, "test", &json!("null")).await?;
    println!("{:?}", result);
    Ok(())
}
