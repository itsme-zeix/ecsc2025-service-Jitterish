#[macro_use] extern crate rocket;

use rocket::form::{Form, Strict};
/*
 * according to the docs, this automatically handles
 * duplicates and unexpected values. How exactly does it 
 * do that? (using strict disables this behavior)
 */

static STATIC_STR: &'static str= "
<head><link rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/bootstrap@4.0.0/dist/css/bootstrap.min.css\" integrity=\"sha384-Gn5384xqQ1aoWXA+058RXPxPg6fy4IWvTNh0E263XmFcJlSAwiGgFAW/dAiS6JXm\" crossorigin=\"anonymous\"></head>
<body>
<table class=\"table\">
  <thead>
    <tr>
      <th scope=\"col\">#</th>
      <th scope=\"col\">First</th>
      <th scope=\"col\">Last</th>
      <th scope=\"col\">Handle</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <th scope=\"row\">1</th>
      <td>Mark</td>
      <td>Otto</td>
      <td>@mdo</td>
    </tr>
    <tr>
      <th scope=\"row\">2</th>
      <td>Jacob</td>
      <td>Thornton</td>
      <td>@fat</td>
    </tr>
    <tr>
      <th scope=\"row\">3</th>
      <td>Larry</td>
      <td>the Bird</td>
      <td>@twitter</td>
    </tr>
  </tbody>
</table>
</body>
";

#[derive(FromForm)]
struct User {
    name: String,
    password: String,
}

#[get("/")]
fn index() -> &'static str {
    STATIC_STR
}

#[get("/<name>/<age>", rank = 2)]
fn wave(name: &str, age: u8) -> String {
    format!("👋 Hello, {} year old named {}!", age, name)
}

#[post("/login", data = "<user>")]
fn login(user: Form<User>) -> String{
    format!("Logging in user with username {} and password {}", user.name, user.password)
}


#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, login])
                .mount("/wave", routes![wave])
}

