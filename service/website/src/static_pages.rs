use interface::frontend::{UserModel, UserSession};
use rocket::{request::FlashMessage, response::{Flash, Redirect}};
use rocket_dyn_templates::{context, Template};

#[get("/")]
fn index(flash: Option<FlashMessage<'_>>, user: Option<UserSession>) -> Result<Template, Flash<Redirect>> {
    let decoded = if let Some(inner) = user {
        let model: UserModel  = serde_json::from_str(&inner.username)
            .map_err(|_| Flash::error(Redirect::to("/"), "User decode error"))?;
        Some(model)
    } else {
        None
    };
    Ok(Template::render(
        "index",
        context! {
            flash: flash,
            username: decoded.map(|u| u.username),
        },
    ))
}

#[get("/docs")]
fn docs(flash: Option<FlashMessage<'_>>, user: Option<UserSession>) -> Result<Template, Flash<Redirect>> {
    let decoded = if let Some(inner) = user {
        let model: UserModel  = serde_json::from_str(&inner.username)
            .map_err(|_| Flash::error(Redirect::to("/"), "User decode error"))?;
        Some(model)
    } else {
        None
    };
    Ok(Template::render(
        "docs",
        context! {
            flash: flash,
            username: decoded.map(|u| u.username),
        },
    ))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![index, docs]
}
