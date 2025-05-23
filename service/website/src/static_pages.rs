use interface::frontend::UserSession;
use rocket::request::FlashMessage;
use rocket_dyn_templates::{context, Template};

#[get("/")]
fn index(flash: Option<FlashMessage<'_>>, user: Option<UserSession>) -> Template {
    Template::render(
        "index",
        context! {
            flash: flash,
            username: user.map(|u| u.username),
        },
    )
}

#[get("/docs")]
fn docs(flash: Option<FlashMessage<'_>>, user: Option<UserSession>) -> Template {
    Template::render(
        "docs",
        context! {
            flash: flash,
            username: user.map(|u| u.username),
        },
    )
}

pub fn routes() -> Vec<rocket::Route> {
    routes![index, docs]
}
