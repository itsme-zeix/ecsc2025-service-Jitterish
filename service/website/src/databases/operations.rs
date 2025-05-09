use crate::databases::api::get_user;
use crate::dbconnector::{one, DbConnection};
use crate::Queries;
use interface::frontend::{AppendForm, DbForm, QueryForm, SimpleQueryForm, UserSession};
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde_json::Value;
use std::io::ErrorKind;

// the basic interaction page for databases
// TODO: we do not want to return an error here, as it appears as 500
#[get("/database/<database>")]
pub async fn database_page(
    queries: &State<Queries>,
    flash: Option<FlashMessage<'_>>,
    database: &str,
    user: UserSession,
) -> std::io::Result<Template> {
    let db = get_user(queries, database).await?;
    let db = db.ok_or_else(|| std::io::Error::new(ErrorKind::NotFound, "Database not found"))?;
    if !db.allow_public_queries && db.username != user.username {
        return Err(std::io::Error::new(ErrorKind::NotFound, "Not authorized"));
    }

    // TODO does flash work here?
    Ok(Template::render(
        "database",
        context! {
            flash: flash,
            username: user.username,
            page: "database",
            database: database,
            form: QueryForm{
                query: "query q on XXX;",
                query_name: "q",
                params: "{}",
            },
                queries: ["test", "test2", "a","b","c"],
        },
    ))
}


//endpoint to list all collections in a database
#[post("/database/<database>/list", data = "<db>")]
pub async fn database_list(
    queries: &State<Queries>,
    flash: Option<FlashMessage<'_>>,
    database: &str,
    user: UserSession,
    db: Form<DbForm<'_>>,
) -> std::io::Result<Template> {
    let mut err: Option<String> = None;
    let mut msgs: Vec<String> = Vec::new();
    let mut conn = DbConnection::open(database).await.expect("db offline");
    let collections = conn.list_collections().await;
    match collections {
        Ok(collections) => {
            for e in &collections {
                msgs.push(serde_json::to_string(&e)?)
            }
        },
        Err(e) => {
            err = Some(format!("{}", e));
        }
    }

    println!("Found collections: {:?}", msgs);
    Ok(Template::render(
        "database",
        context! {
            flash: flash,
            username: user.username,
            page: "database",
            database: database,
            queries: ["test", "a","b","c"],
            err: err,
            msgs: msgs,
        },
    ))
}
// endpoint for predefined queries
#[post("/database/<database>/query", data = "<form>")]
pub async fn database_query(
    queries: &State<Queries>,
    flash: Option<FlashMessage<'_>>,
    database: &str,
    user: UserSession,
    form: Form<SimpleQueryForm<'_>>,
) -> std::io::Result<Template> {
    println!("Queries: {:?}", queries);
    let cr = &queries.user;
    let params: Value = serde_json::from_str(form.params)?;
    let mut err: Option<String> = None;
    let mut msgs: Vec<String> = Vec::new();
    let mut conn = DbConnection::open(database).await.expect("db offline");
    match conn.execute::<Value, Value>(cr, form.query, &params).await {
        Ok(result) => {
            for e in &result {
                msgs.push(serde_json::to_string(&e)?)
            }
        }
        Err(e) => {
            err = Some(format!("{}", e));
        }
    }
    Ok(Template::render(
        "database",
        context! {
            flash: flash,
            username: user.username,
            page: "database",
            database: database,
            queries: ["test", "a","b","c"],
            form: form.into_inner(),
            err: err,
            msgs: msgs,
        },
    ))
}


#[post("/database/<database>/append", data = "<form>")]
pub async fn database_append(
    queries: &State<Queries>,
    flash: Option<FlashMessage<'_>>,
    database: &str,
    user: UserSession,
    form: Form<AppendForm<'_>>,
) -> std::io::Result<Template> {
    // TODO does flash work here?
    let mut err: Option<String> = None;
    let mut msgs: Vec<String> = Vec::new();
    let mut conn = DbConnection::open(database).await.expect("db offline");
    let db = get_user(queries, database).await?;
    let db = db.ok_or_else(|| std::io::Error::new(ErrorKind::NotFound, "Database not found"))?;
    if !db.allow_public_queries && db.username != user.username {
        return Err(std::io::Error::new(ErrorKind::NotFound, "Not authorized"));
    }

    let parsed = serde_json::from_str::<Value>(form.data);
    match parsed {
        Ok(parsed_data) => {
            match conn.append(form.collection, parsed_data).await {
                Ok(_) => {
                    msgs.push(format!("{} successfully appended.", form.data));
                },
                Err(e) => {
                    err = Some(format!("{}", e));
                }
            }
        },
        Err(e) => {
            err = Some(format!("Incorrect json supplied: {}", e));
        },
    }
    Ok(Template::render(
        "database",
        context! {
            flash: flash,
            username: user.username,
            page: "database",
            database: database,
            queries: ["test", "a","b","c"],
            form: form.into_inner(),
            err: err,
            msgs: msgs,
        },
    ))
}

// query is a read, everyone can do that on all public tables
// apparently, this function creates a new query
// how do I use existing ones?
#[post("/database/<database>/customquery", data = "<form>")]
pub async fn database_query_custom(
    queries: &State<Queries>,
    flash: Option<FlashMessage<'_>>,
    database: &str,
    user: UserSession,
    form: Form<QueryForm<'_>>,
) -> std::io::Result<Template> {
    // TODO deduplicate
    let db = get_user(queries, database).await?;
    let db = db.ok_or_else(|| std::io::Error::new(ErrorKind::NotFound, "Database not found"))?;
    if !db.allow_public_queries && db.username != user.username {
        return Err(std::io::Error::new(ErrorKind::NotFound, "Not authorized"));
    }

    let mut err: Option<String> = None;
    let mut msgs: Vec<String> = Vec::new();
    let mut conn = DbConnection::open(database).await.expect("db offline");
    println!("Existing queries: {:?}", queries);
    println!("Form.query: {:?}", form.query);
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
            page: "database",
            username: user.username,
            flash: flash,
            database: database,
            queries: ["test", "a","b","c"],
            form: form.into_inner(),
            err: err,
            msgs: msgs,
        },
    ))
}
