use lazy_static::lazy_static;
use rocket::tokio::sync::RwLock;
use std::collections::HashMap;

pub struct UserCache {
    cache: RwLock<HashMap<String, (bool, bool)>>,
}

lazy_static! {
    pub static ref USER_CACHE: UserCache = UserCache {
        cache: RwLock::new(HashMap::new()),
    };
}

impl UserCache {
    pub async fn get(&self, username: &str) -> Option<(bool, bool)> {
        self.cache.read().await.get(username).map(|x| x.clone())
    }

    pub async fn set(&self, username: &str, allow_public_queries: bool, allow_public_kv: bool) {
        self.cache
            .write()
            .await
            .insert(username.to_string(), (allow_public_queries, allow_public_kv));
    }
}
