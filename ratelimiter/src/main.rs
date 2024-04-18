use std::sync::Mutex;

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use ratelimiter::rate_limiters::IpRateLimiter;

#[get("/limited")]
async fn limited(data: web::Data<AppStateWithIpRateLimiter>, req: HttpRequest) -> impl Responder {
    let consumed = req
        .connection_info()
        .peer_addr()
        .map(|ip| {
            let mut limiter = data.limiter.lock().unwrap();
            limiter.consume_token(ip.to_owned())
        })
        .unwrap_or_else(|| {
            eprintln!("Failed to get IP address");
            false
        });
    if consumed {
        return HttpResponse::Ok().body("Limited, but ok for now, don't over use me!");
    }
    HttpResponse::TooManyRequests().body("Rate limit exceeded, try again later")
}

#[get("/unlimited")]
async fn unlimited() -> impl Responder {
    HttpResponse::Ok().body("Unlimited! Let's Go!")
}

struct AppStateWithIpRateLimiter {
    limiter: Mutex<IpRateLimiter>,
}
#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppStateWithIpRateLimiter {
                limiter: Mutex::new(IpRateLimiter::default()),
            }))
            .service(limited)
            .service(unlimited)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
