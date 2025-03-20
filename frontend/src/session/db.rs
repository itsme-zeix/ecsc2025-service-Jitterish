use crate::dbconnector::connect_to_backend;

use super::User;

#[get("/db")]
pub(super) async fn db(_user: User) {
    connect_to_backend().await;
}
