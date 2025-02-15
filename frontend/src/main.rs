#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/<name>/<age>", rank = 2)]
fn wave(name: &str, age: u8) -> String {
    format!("👋 Hello, {} year old named {}!", age, name)
}


#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
                .mount("/wave", routes![wave])
}

