use crate::databases::api::get_user;
use crate::dbconnector::{one, DbConnection};
use crate::Queries;
use interface::frontend::{QueryForm, UserSession};
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde_json::Value;
use std::io::ErrorKind;

#[get("/database/<database>")]
pub async fn database_page(
    queries: &State<Queries>,
    flash: Option<FlashMessage<'_>>,
    database: &str,
    user: Option<UserSession>,
) -> std::io::Result<Template> {
    let db = get_user(queries, database).await?;
    let db = db.ok_or_else(|| std::io::Error::new(ErrorKind::NotFound, "Database not found"))?;
    if !db.allow_public_queries && db.username != user.map(|u| u.username).unwrap_or("".into()) {
        return Err(std::io::Error::new(ErrorKind::NotFound, "Not authorized"));
    }

    // TODO does flash work here?
    Ok(Template::render(
        "database",
        context! {
            flash: flash,
            database: database,
            form: QueryForm{
                query: "query q on XXX;",
                query_name: "q",
                params: "{}",
            }
        },
    ))
}

#[post("/database/<database>/query", data = "<form>")]
pub async fn database_query(
    queries: &State<Queries>,
    flash: Option<FlashMessage<'_>>,
    database: &str,
    user: Option<UserSession>,
    form: Form<QueryForm<'_>>,
) -> std::io::Result<Template> {
    // TODO deduplicate
    let db = get_user(queries, database).await?;
    let db = db.ok_or_else(|| std::io::Error::new(ErrorKind::NotFound, "Database not found"))?;
    if !db.allow_public_queries && db.username != user.map(|u| u.username).unwrap_or("".into()) {
        return Err(std::io::Error::new(ErrorKind::NotFound, "Not authorized"));
    }

    let mut err: Option<String> = None;
    let mut msgs: Vec<String> = Vec::new();
    let mut conn = DbConnection::open(database).await.expect("db offline");
    match conn.prepare(&form.query).await {
        Ok(code_ref) => {
            let params: Value = serde_json::from_str(form.params)?;
            match conn.execute::<Value, Value>(&code_ref, form.query_name, &params).await {
                Ok(result) => {
                    for e in &result {
                        msgs.push(serde_json::to_string(&e)?)
                    }
                }
                Err(e) => {
                    err = Some(format!("{}", e));
                }
            }
        }
        Err(e) => {
            err = Some(format!("{}", e));
        }
    }

    // TODO does flash work here?
    Ok(Template::render(
        "database",
        context! {
            flash: flash,
            database: database,
            form: form.into_inner(),
            err: err,
            msgs: msgs,
        },
    ))
}
