pub mod api;
mod operations;

pub fn routes() -> Vec<rocket::Route> {
    routes![api::api_create, api::api_token, api::api_get, operations::database_page, operations::database_query]
}
