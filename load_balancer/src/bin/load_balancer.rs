use actix_web::middleware::Logger;
use actix_web::{
    error, http::header::ContentType, http::StatusCode, web, App, HttpRequest, HttpResponse,
    HttpServer,
};

use derive_more::{Display, Error};
use env_logger::Env;
use load_balancer::setup_cors;
use log::info;
use reqwest::Client;

#[derive(Debug, Display, Error)]
enum LoadBalancerError {
    InternalError(#[error(source)] reqwest::Error),
}

impl error::ResponseError for LoadBalancerError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

async fn handler(req: HttpRequest, payload: web::Bytes) -> Result<HttpResponse, LoadBalancerError> {
    let main_backend_server = "http://localhost:8081";
    let full_url = format!("{}{}", main_backend_server, req.uri());
    info!("full_url: {}", full_url);

    let client = Client::new();
    let request_builder = client
        .request(req.method().clone(), full_url)
        .headers(req.headers().clone().into())
        .body(payload);

    let response = request_builder
        .send()
        .await
        .map_err(LoadBalancerError::InternalError)?;

    let mut response_builder = HttpResponse::build(response.status());

    let response_builder = response
        .headers()
        .iter()
        .fold(&mut response_builder, |response_builder, header| {
            response_builder.append_header(header)
        });

    let body = response
        .text()
        .await
        .map_err(LoadBalancerError::InternalError)?;
    Ok(response_builder.body(body))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(setup_cors())
            .wrap(Logger::default())
            .service(web::resource("/{tail:.*}").to(handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
