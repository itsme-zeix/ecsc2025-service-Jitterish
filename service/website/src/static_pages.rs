use interface::frontend::UserSession;
use rocket::request::FlashMessage;
use rocket_dyn_templates::{context, Template};

#[get("/")]
fn index_logged_in(flash: Option<FlashMessage<'_>>, user: UserSession) -> Template {
    Template::render(
        "index",
        context! {
            flash: flash,
            username: user.username,
        },
    )
}

#[get("/", rank = 2)]
fn index(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render(
        "index",
        context! {
            flash: flash
        },
    )
}

#[get("/docs")]
fn docs_logged_in(flash: Option<FlashMessage<'_>>, user: UserSession) -> Template {
    Template::render(
        "docs",
        context! {
            flash: flash,
            username: user.username,
        },
    )
}

#[get("/docs", rank = 2)]
fn docs(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render(
        "docs",
        context! {
            flash: flash
        },
    )
}

pub fn routes() -> Vec<rocket::Route> {
    routes![index, index_logged_in, docs, docs_logged_in]
}
