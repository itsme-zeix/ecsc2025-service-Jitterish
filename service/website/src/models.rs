use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserModel {
    pub username: String, // = database name (at least 3 chars, [A-Za-z0-9_] only, must be unique)
    pub password_hash: String,

    // TODO add all the necessary fields for the frontend

    // Permissions for the database
    pub allow_public_queries: bool, // = anybody can query public collections
    pub allow_public_kv: bool,      // = anybody can access this with the "kv api"
}
