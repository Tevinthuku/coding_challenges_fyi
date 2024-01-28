use actix_web::{middleware::Logger, web, App, HttpServer, Responder};
use env_logger::Env;
use load_balancer::setup_cors;

async fn index(port: web::Data<u16>) -> impl Responder {
    let html_body = format!(
        "<html><body><h1>Hello, running on port {}!</h1></body></html>",
        port.into_inner()
    );
    actix_web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let port = std::env::var("PORT")
        .map(|port| port.parse().expect("port must be a number"))
        .unwrap_or(8080);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(setup_cors())
            .app_data(web::Data::new(port))
            .route("/", web::get().to(index))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
