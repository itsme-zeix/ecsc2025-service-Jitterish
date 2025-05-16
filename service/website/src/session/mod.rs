pub mod login;
pub mod register;
pub mod profile;
pub mod logout;

pub fn routes() -> Vec<rocket::Route> {
    routes![profile::index, profile::no_auth_index, login::login, login::login_page, login::post_login, logout::logout, register::register, register::register_page, register::post_register]
}
