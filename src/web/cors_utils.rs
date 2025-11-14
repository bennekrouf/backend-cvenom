use rocket::http::Status;

/// Generic CORS handler that returns Status::Ok for any OPTIONS request
#[rocket::options("/<_..>")]
pub async fn universal_options_handler() -> Status {
    Status::Ok
}
