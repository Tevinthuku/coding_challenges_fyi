use actix_cors::Cors;
use actix_web::http;

pub fn setup_cors() -> Cors {
    Cors::default()
        .allow_any_method()
        .allow_any_header()
        .allow_any_origin()
        .max_age(3600)
}
