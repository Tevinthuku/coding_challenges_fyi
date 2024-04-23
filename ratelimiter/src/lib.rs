use actix_web::{get, web, HttpResponse, Responder};
use rate_limiters::sliding_window_counter::distributed::DistributedSlidingWindowCounter;
use tokio::sync::Mutex;

pub mod rate_limiters;

#[get("/limited")]
async fn limited(data: web::Data<AppStateWithIpRateLimiter>) -> impl Responder {
    let limiter = &mut data.limiter.lock().await;
    let consumed_result = limiter.consume_token().await;
    let consumed = match consumed_result {
        Ok(consumed) => consumed,
        Err(err) => {
            return HttpResponse::InternalServerError()
                .body(format!("Internal Server Error: {err:?}"))
        }
    };
    if consumed {
        return HttpResponse::Ok().body("Limited, but ok for now, don't over use me!");
    }
    HttpResponse::TooManyRequests().body("Rate limit exceeded, try again later")
}

pub struct AppStateWithIpRateLimiter {
    limiter: Mutex<DistributedSlidingWindowCounter>,
}

impl AppStateWithIpRateLimiter {
    pub fn new(limiter: DistributedSlidingWindowCounter) -> Self {
        Self {
            limiter: Mutex::new(limiter),
        }
    }
}
