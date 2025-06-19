use chrono::Utc;
use interface::frontend::{Complaint, SupportForm, UserModel, UserSession};
use rocket::{
    form::Form,
    request::FlashMessage,
    response::{Flash, Redirect},
    State,
};
use serde_json::Value;

use crate::{dbconnector::DbConnection, Queries};

#[post("/support", data = "<form>")]
pub async fn user_down(
    queries: &State<Queries>,
    flash: Option<FlashMessage<'_>>,
    user: UserSession,
    form: Form<SupportForm<'_>>,
) -> Result<Redirect, Flash<Redirect>> {
    // open db connection
    let mut db = DbConnection::open("_")
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "Database not reachable"))?;
    // get the user to report
    let mut target_user: Vec<Value> = db
        .execute(&queries.user, "user_by_username", &form.target_user, false)
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "User by username query error"))?;
    if target_user.is_empty() {
        return Err(Flash::error(
            Redirect::to(uri!(crate::session::profile::index)),
            "User that you want to report does not exist!",
        ));
    }
    // we know the query has a limit 1, to there will be exactly one report
    let target_user: Value = target_user.pop().expect("Target user does not exist!");
    let target_user: UserModel = serde_json::from_str(&target_user.to_string()).unwrap();
    
    // now get the actual usermodel of the current user
    let mut logged_in_user: Vec<Value> = db
        .execute(&queries.user, "user_by_username", &user.username, false)
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "User by username query error"))?;
    if logged_in_user.is_empty() {
        return Err(Flash::error(
            Redirect::to(uri!(crate::session::profile::index)),
            "User that you want to report does not exist!",
        ));
    }
    // we know the query has a limit 1, to there will be exactly one report
    let logged_in_user: Value = logged_in_user.pop().expect("Logged in user does not exist!");
    let logged_in_user: UserModel = serde_json::from_str(&logged_in_user.to_string()).unwrap();
    println!("Complaint, reporter: {:?}, target: {:?}", logged_in_user, target_user);
    let time = Utc::now();
    let complaint = Complaint {
        reporting_user: logged_in_user,
        target_user: target_user,
        timestamp: time.to_rfc3339(),
    };
    db.append("complaints", complaint).await.map_err(|e| {
        Flash::error(
            Redirect::to(uri!(crate::session::profile::index)),
            format!("Append failed: {}", e),
        )
    })?;
    println!("##################################");
    println!("Got a report for {:?}", form.target_user);
    println!("##################################");
    // get all reported users. Later only print the length of the list
    let all_target_complaints: Vec<UserModel> = db
        // .execute(&queries.complaint, "get_reported_users", &user.username, false)
        .execute(&queries.complaint, "get_all_users", &user.username, false)
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "Get complaints query error"))?;
    // println!("All users that I reported");
    // for c in all_target_complaints {
    //     println!("User: {:?}", c);
    // }
    println!("I reported {:?} users!", all_target_complaints.len());

    // the previous query returns reported users, letting us deduce the number of reported users 
    // but we can report a single user multiple times, so the number of reports might differ
    // we get it via a dedicated query
    let mut new_user: Vec<UserModel> = db
        .execute(&queries.complaint, "get_user_numreports", &user.username, false)
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "Get complaints query error"))?;
    let updated = new_user.pop().expect("Could not update my usermodel!");
    println!("Updated user: {:?}", updated);
    // for c in new_user {
    //     println!("User: {:?}", c);
    // }
    Ok(Redirect::to(uri!(crate::session::profile::index)))
}
