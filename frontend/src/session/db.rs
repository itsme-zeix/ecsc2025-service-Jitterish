use interface::frontend::User;

use crate::dbconnector::connect_to_backend;

#[get("/db")]
pub(super) async fn db(_user: User) {
    connect_to_backend().await;
}
