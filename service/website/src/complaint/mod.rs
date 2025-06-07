mod complaint;

pub fn routes() -> Vec<rocket::Route> {
    routes![complaint::user_down,]
}
