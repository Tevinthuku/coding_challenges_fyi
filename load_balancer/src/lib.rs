use actix_cors::Cors;

// For the sake of this example, allow everything
pub fn setup_cors() -> Cors {
    Cors::default()
        .allow_any_method()
        .allow_any_header()
        .allow_any_origin()
        .max_age(3600)
}
