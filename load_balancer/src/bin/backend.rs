use actix_web::{middleware::Logger, web, App, HttpServer, Responder};
use env_logger::Env;
use load_balancer::setup_cors;

async fn index() -> impl Responder {
    "Hello world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(setup_cors())
            .route("/hey", web::get().to(index))
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
