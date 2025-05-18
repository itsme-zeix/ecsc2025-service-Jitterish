use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserModel {
    // TODO validate: only lowercase + _ allowed, at least 3 chars
    pub username: String, // = database name, must be unique
    pub password_hash: String,

    // TODO add all the necessary fields for the frontend

    // TODO we might split user and database in two different things, but then we have to write much more code...
    // Permissions for the database
    pub allow_public_queries: bool, // = anybody can query public collections
    pub allow_public_kv: bool,      // = anybody can access this with the "kv api"
}
