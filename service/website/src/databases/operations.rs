use crate::databases::api::get_user;
use crate::dbconnector::DbConnection;
use crate::models::UserModel;
use crate::Queries;
use interface::database::Collection;
use interface::frontend::{AppendForm, QueryForm, SimpleQueryForm, UserSession};
use rocket::form::Form;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde_json::Value;
use std::fs;

struct DatabasePageContext {
    db: UserModel,
    user_owns_db: bool,
    conn: DbConnection,
    collections: Vec<Collection>,
    err: Option<String>,
    msgs: Vec<String>,
}

async fn database_page_basics(
    queries: &State<Queries>,
    database: &str,
    user: &UserSession,
) -> Result<DatabasePageContext, Flash<Redirect>> {
    let db = get_user(queries, database)
        .await
        .map_err(|_| Flash::error(Redirect::to(format!("/database/{}", database)), "Cannot fetch user"))?;
    let db = db.ok_or_else(|| Flash::error(Redirect::to(format!("/database/{}", database)), "No such db"))?;
    let user_owns_db = db.username == user.username;
    if !db.allow_public_queries && !user_owns_db {
        return Err(Flash::error(Redirect::to("/"), "Not authorized"));
    }

    let mut conn = DbConnection::open(database)
        .await
        .map_err(|_| Flash::error(Redirect::to(format!("/database/{}", database)), "Database offline"))?;
    let collections = conn
        .list_collections()
        .await
        .map_err(|_| Flash::error(Redirect::to(format!("/database/{}", database)), "Cannot list collections"))?;

    Ok(DatabasePageContext {
        db,
        user_owns_db,
        conn,
        collections,
        err: None,
        msgs: Vec::new(),
    })
}

// the basic interaction page for databases
// TODO: we do not want to return an error here, as it appears as 500
#[get("/database/<database>")]
pub async fn database_page(
    queries: &State<Queries>,
    flash: Option<FlashMessage<'_>>,
    database: &str,
    user: UserSession,
) -> Result<Template, Flash<Redirect>> {
    let ctx = database_page_basics(queries, database, &user).await?;

    Ok(Template::render(
        "database",
        context! {
            flash: flash,
            username: user.username,
            database: database,
            collections: ctx.collections,
            db: ctx.db,
            user_owns_db: ctx.user_owns_db,
            form: QueryForm{
                query: "query q on XXX;",
                query_name: "q",
                params: "{}",
            },
            queries: ["test", "test2", "a","b","c"],  // TODO
        },
    ))
}

// TODO not used in UI for now
// endpoint for predefined queries
#[post("/database/<database>/query", data = "<form>")]
pub async fn database_query(
    queries: &State<Queries>,
    flash: Option<FlashMessage<'_>>,
    database: &str,
    user: UserSession,
    form: Form<SimpleQueryForm<'_>>,
) -> Result<Template, Flash<Redirect>> {
    // TODO use ctx or remove
    println!("Queries: {:?}", queries);
    let cr = &queries.user;
    let params: Value = serde_json::from_str(form.params)
        .map_err(|_| Flash::error(Redirect::to(format!("/database/{}", database)), "Invalid JSON"))?;
    let mut err: Option<String> = None;
    let mut msgs: Vec<String> = Vec::new();
    let mut conn = DbConnection::open(database)
        .await
        .map_err(|_| Flash::error(Redirect::to(format!("/database/{}", database)), "Database offline"))?;
    // TODO use privileged access if necessary
    match conn.execute::<Value, Value>(cr, form.query_name, &params, false).await {
        Ok(result) => {
            for e in &result {
                msgs.push(
                    serde_json::to_string(&e).map_err(|_| {
                        Flash::error(Redirect::to(format!("/database/{}", database)), "DB returned invalid JSON")
                    })?,
                )
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
            database: database,
            queries: ["test", "a","b","c"],  // TODO
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
) -> Result<Template, Flash<Redirect>> {
    let mut ctx = database_page_basics(queries, database, &user).await?;
    if !ctx.user_owns_db {
        return Err(Flash::error(
            Redirect::to(format!("/database/{}", database)),
            "No write access",
        ));
    }

    let parsed = serde_json::from_str::<Value>(form.data);
    match parsed {
        Ok(parsed_data) => match ctx.conn.append(form.collection, parsed_data).await {
            Ok(_) => {
                ctx.msgs.push(format!("{} successfully appended.", form.data));
                ctx.collections = ctx
                    .conn
                    .list_collections()
                    .await
                    .map_err(|_| Flash::error(Redirect::to("/"), "Error in connection listing"))?;
            }
            Err(e) => {
                ctx.err = Some(format!("cannot append: {}", e));
            }
        },
        Err(e) => {
            ctx.err = Some(format!("Incorrect json supplied: {}", e));
        }
    }
    Ok(Template::render(
        "database",
        context! {
            flash: flash,
            username: user.username,
            database: database,
            collections: ctx.collections,
            db: ctx.db,
            user_owns_db: ctx.user_owns_db,
            queries: ["test", "a","b","c"],
            form: form.into_inner(),
            err: ctx.err,
            msgs: ctx.msgs,
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
) -> Result<Template, Flash<Redirect>> {
    let mut ctx = database_page_basics(queries, database, &user).await?;

    match ctx.conn.prepare(&form.query).await {
        Ok(code_ref) => match serde_json::from_str::<Value>(form.params) {
            Ok(params) => match ctx
                .conn
                .execute::<Value, Value>(&code_ref, form.query_name, &params, ctx.user_owns_db)
                .await
            {
                Ok(result) => {
                    for r in &result {
                        let s = serde_json::to_string(&r)
                            .map_err(|_| Flash::error(Redirect::to(format!("/database/{}", database)), "Query error"))?;
                        ctx.msgs.push(s)
                    }
                }
                Err(e) => {
                    ctx.err = Some(format!("{}", e));
                }
            },
            Err(e) => {
                ctx.err = Some(format!("{}", e));
            }
        },
        Err(e) => {
            ctx.err = Some(format!("{}", e));
        }
    }

    Ok(Template::render(
        "database",
        context! {
            flash: flash,
            username: user.username,
            database: database,
            collections: ctx.collections,
            db: ctx.db,
            user_owns_db: ctx.user_owns_db,
            queries: ["test", "a","b","c"],  // TODO
            form: form.into_inner(),
            err: ctx.err,
            msgs: ctx.msgs,
        },
    ))
}

#[get("/database/<database>/script")]
pub async fn database_script(queries: &State<Queries>, database: &str, user: UserSession) -> Result<String, Flash<Redirect>> {
    let ctx = database_page_basics(queries, database, &user).await?;
    if !ctx.db.allow_public_kv {
        return Err(Flash::error(
            Redirect::to(format!("/database/{}", database)),
            "No script attached",
        ));
    }
    Ok(fs::read_to_string("queries/api.qry").expect("API queries file is missing!"))
}
