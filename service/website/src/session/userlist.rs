use crate::dbconnector::DbConnection;
use crate::Queries;
use interface::frontend::{UserModel, UserSession};
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde_json::json;

#[get("/customers")]
pub async fn userlist(user: Option<UserSession>, queries: &State<Queries>) -> Result<Template, Flash<Redirect>> {
    let decoded = if let Some(inner) = user {
        let model: UserModel  = serde_json::from_str(&inner.username)
            .map_err(|_| Flash::error(Redirect::to("/"), "User decode error"))?;
        Some(model)
    } else {
        None
    };
    let mut db = DbConnection::open("_")
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "Database not reachable"))?;
    let public_users: Vec<String> = db
        .execute(&queries.user, "public_users", &json!("null"), false)
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "Could not query public users"))?;
    let api_users: Vec<String> = db
        .execute(&queries.user, "api_users", &json!("null"), false)
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "Could not query public users"))?;
    let private_users: Vec<String> = db
        .execute(&queries.user, "private_users", &json!("null"), false)
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "Could not query public users"))?;

    Ok(Template::render(
        "customers",
        context! {
            username: decoded.map(|u| u.username),
            public_users: public_users,
            api_users: api_users,
            private_users: private_users,  // TODO this might not scale!
        },
    ))
}
