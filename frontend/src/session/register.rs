use rocket::request::FlashMessage;
use rocket::response::{Redirect, Flash};
use rocket::http::CookieJar;
use rocket::form::Form;

use rocket_dyn_templates::Template;

use super::{Register, User};

// if we are registered (that is, we have a user), redirect to index instead
#[get("/register")]
pub(super) fn register(_user: User) -> Redirect {
    Redirect::to(uri!("/session/index"))
}

// we are not registered here
#[get("/register", rank = 2)]
pub fn register_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render("register", &flash)
}

#[post("/register", data = "<register>")]
pub(super) fn post_register(jar: &CookieJar<'_>, register: Form<Register<'_>>) -> Result<Redirect, Flash<Redirect>> {
    let s = serde_json::to_string(&register.into_inner()).expect("Could not serialize user!");
    // println!("Registered successfully as {} {} with account type {}!", register.firstname, register.lastname, register.accounttype);
    println!("Serialized user: {:?}", s);
    jar.add_private(("user_id", "1"));
    Ok(Redirect::to(uri!("/session/")))
    //Err(Flash::error(Redirect::to(uri!(login_page)), "Invalid username/password."))
}
