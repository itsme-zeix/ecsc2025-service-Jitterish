use chrono::Utc;
use interface::frontend::{Complaint, SupportForm, UserModel, UserSession};
use rocket::{
    form::Form, http::CookieJar, request::FlashMessage, response::{Flash, Redirect}, State
};
use rocket_dyn_templates::{context, Template};

use crate::{db_connector::DbConnection, Queries};

#[get("/support")]
pub async fn support<'a>(
    queries: &State<Queries>,
    flash: Option<FlashMessage<'_>>,
    session: UserSession,
    jar: &CookieJar<'a>
) -> Result<Template, Flash<Redirect>> {
    let mut db = DbConnection::open("_")
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "Database not reachable"))?;

    // get all reported users
    let all_target_complaints: Vec<UserModel> = db
        .execute(&queries.complaint, "get_reported_users", &session.user.username, false)
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "Get complaints query error 1"))?;

    // the previous query returns reported users, letting us deduce the number of reported users,
    // but we can report a single user multiple times, so the number of reports might differ.
    // we get it via a dedicated query
    let updated_user: UserModel = db
        .execute(&queries.complaint, "get_user_numreports", &session.user.username, false)
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "Get complaints query error 2"))?
        .pop()
        .ok_or(Flash::error(Redirect::to("/"), "Could not update my usermodel!"))?;
    // update user session
    jar.remove_private("user");
    let user_json = serde_json::to_string(&updated_user).map_err(|_| Flash::error(Redirect::to("/"), "User encode error"))?;
    jar.add_private(("user", user_json));
    Ok(Template::render(
        "complaints",
        context! {
            flash: flash,
            username: session.user.username,
            user: updated_user,
            reported_targets: all_target_complaints.len(),
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
    session: UserSession,
    form: Form<SupportForm<'_>>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    // open db connection
    let mut db = DbConnection::open("_")
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "Database not reachable"))?;
    // get the user to report
    let target_user: UserModel = db
        .execute(&queries.user, "user_by_username", &form.target_user, false)
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "User by username query error"))?
        // we know the query has limit 1, so there will be exactly one report
        .pop()
        .ok_or(Flash::error(Redirect::to(uri!(support)), "Database does not exist!"))?;

    // now get the actual usermodel of the current user
    let mut logged_in_user: Vec<UserModel> = db
        .execute(&queries.user, "user_by_username", &session.user.username, false)
        .await
        .map_err(|_| Flash::error(Redirect::to("/"), "User by username query error"))?;
        // we know the query has limit 1, so there will be exactly one report
    let logged_in_user = logged_in_user
        .pop()
        .ok_or(Flash::error(Redirect::to(uri!(support)), "Your account is missing!"))?;
    let complaint = Complaint {
        reporting_user: logged_in_user,
        target_user: target_user,
        timestamp: Utc::now().to_rfc3339(),
        description: form.description.to_string(),
    };
    db.append("complaints", complaint)
        .await
        .map_err(|e| Flash::error(Redirect::to(uri!(support)), format!("Append failed: {}", e)))?;
    Ok(Flash::success(
        Redirect::to(uri!(support)),
        "your complaint has been submitted, we'll get back to you soon.", // TODO add LLM to generate replies
    ))
}
