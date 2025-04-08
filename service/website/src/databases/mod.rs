pub mod api;

pub fn routes() -> Vec<rocket::Route> {
    routes![api::api_create, api::api_token, api::api_get]
}
