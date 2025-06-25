use chrono::Utc;
use interface::frontend::{Complaint, SupportForm, UserModel, UserSession};
use rocket::{
    form::Form,
    request::FlashMessage,
    response::{Flash, Redirect},
    State,
};
use rocket_dyn_templates::{context, Template};
use serde_json::Value;

use crate::{dbconnector::DbConnection, Queries};

#[get("/support")]
pub async fn support(
    queries: &State<Queries>,
    flash: Option<FlashMessage<'_>>,
    user: UserSession,
) -> Result<Template, Flash<Redirect>> {
    let mut db = DbConnection::open("_")
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "Database not reachable"))?;

    // get all reported users. Later only print the length of the list
    let all_target_complaints: Vec<UserModel> = db
        // .execute(&queries.complaint, "get_reported_users", &user.username, false)
        .execute(&queries.complaint, "get_reported_users", &user.username, false)
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "Get complaints query error 1"))?;
    // println!("All users that I reported");
    // for c in all_target_complaints {
    //     println!("User: {:?}", c);
    // }
    println!("I reported {:?} users!", all_target_complaints.len());

    // the previous query returns reported users, letting us deduce the number of reported users,
    // but we can report a single user multiple times, so the number of reports might differ.
    // we get it via a dedicated query
    let updated_user: UserModel = db
        .execute(&queries.complaint, "get_user_numreports", &user.username, false)
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "Get complaints query error 2"))?
        .pop().expect("Could not update my usermodel!");
    println!("Updated user: {:?}", updated_user);
    // for c in new_user {
    //     println!("User: {:?}", c);
    // }

    Ok(Template::render(
        "complaints",
        context! {
            flash: flash,
            username: user.username,
            user: updated_user,
            reported_targets:  all_target_complaints.len(),
        },
    ))
}

#[get("/support", rank = 2)]
pub async fn no_auth_support() -> Redirect {
    Redirect::to(uri!(crate::session::login::login_page))
}

#[post("/support", data = "<form>")]
pub async fn save_complaint(
    queries: &State<Queries>,
    flash: Option<FlashMessage<'_>>,
    user: UserSession,
    form: Form<SupportForm<'_>>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
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
    // we know the query has limit 1, so there will be exactly one report
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
    // we know the query has limit 1, so there will be exactly one report
    let logged_in_user: Value = logged_in_user.pop().expect("Logged in user does not exist!");
    let logged_in_user: UserModel = serde_json::from_str(&logged_in_user.to_string()).unwrap();
    println!("Complaint, reporter: {:?}, target: {:?}", logged_in_user, target_user);
    let time = Utc::now();
    let complaint = Complaint {
        reporting_user: logged_in_user,
        target_user: target_user,
        timestamp: time.to_rfc3339(),
        description: form.description.to_string(),
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
    Ok(Flash::success(
        Redirect::to(uri!(support)),
        "your complaint has been submitted, we'll get back to you soon.", // TODO add LLM to generate replies
    ))
}
