use std::str::FromStr;

use crate::error::ApiError;
use actix_web::{delete, get, post, web, HttpResponse, Responder, Result as ActixResponse};
use anyhow::Context;
use base64::{
    engine::{general_purpose::URL_SAFE, GeneralPurpose},
    Engine,
};
use log::warn;
use serde::{Deserialize, Serialize};
use sha256::digest;
use sqlx::PgPool;
use url::Url;

const ENGINE: GeneralPurpose = URL_SAFE;

#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub struct RequestData {
    url: Url,
}

#[derive(Serialize, Debug)]
#[cfg_attr(test, derive(Deserialize))]
pub struct Response {
    long_url: Url,
    short_url: Url,
    key: String,
}

#[post("/shorten")]
pub async fn shorten_url(
    data: web::Json<RequestData>,
    pool: web::Data<PgPool>,
    base_server_url: web::Data<Url>,
) -> Result<web::Json<Response>, ApiError> {
    let long_url = data.into_inner().url;
    let pool = pool.into_inner();
    let hash = generate_and_insert_hash(&pool, &long_url).await?;
    let short_url = {
        let server_url = base_server_url.into_inner();
        let server_url = server_url.as_str();
        let mut server_url = Url::from_str(server_url).unwrap();
        server_url.set_path(&hash);
        server_url
    };

    Ok(web::Json(Response {
        long_url,
        short_url,
        key: hash,
    }))
}

async fn generate_and_insert_hash(pool: &PgPool, url: &Url) -> Result<String, ApiError> {
    let maybe_url_exists = sqlx::query!(
        r#"
        SELECT hash
        FROM urls
        WHERE long_url = $1
        "#,
        url.as_str()
    )
    .fetch_optional(pool)
    .await
    .context("Failed to fetch long_url")
    .map_err(ApiError::InternalError)?;

    if let Some(record) = maybe_url_exists {
        return Ok(record.hash);
    }

    let hash = {
        let hash = digest(url.as_str());
        ENGINE.encode(hash)
    };
    // The default hash length is 7. If a hash collision is detected, the cursor is increased by 1
    // until a unique hash is found to avoid hash collisions.
    let mut cursor = 7;
    let original_hash_length = hash.len();
    loop {
        if cursor > original_hash_length {
            return Err(ApiError::InternalError(anyhow::anyhow!(
                "Failed to generate a unique hash for the URL: {url}"
            )));
        }
        let hash_insert = hash[..cursor].to_string();
        let maybe_record_exists = sqlx::query!(
            r#"
            SELECT hash, long_url
            FROM urls
            WHERE hash = $1
            "#,
            hash_insert
        )
        .fetch_optional(pool)
        .await
        .context("Failed to fetch URL data from the DB")
        .map_err(ApiError::InternalError)?;

        if let Some(record) = maybe_record_exists {
            warn!(
                "URL {} is stored for the hash {}. Increasing the cursor by 1.",
                record.long_url, record.hash,
            );
            cursor += 1;
            continue;
        }

        sqlx::query!(
            r#"
            INSERT INTO urls (hash, long_url)
            VALUES ($1, $2)
            "#,
            &hash_insert,
            url.as_str()
        )
        .execute(pool)
        .await
        .context("Failed to insert URL data to the DB")
        .map_err(ApiError::InternalError)?;
        break;
    }

    Ok(hash[..cursor].to_string())
}

#[get("/{hash}")]
pub async fn redirect_to_long_url(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> ActixResponse<impl Responder> {
    let hash = path.into_inner();
    let pool = pool.into_inner();
    let result = sqlx::query!(
        r#"
        SELECT long_url
        FROM urls
        WHERE hash = $1
        "#,
        hash
    )
    .fetch_optional(&*pool)
    .await
    .map_err(|err| {
        let err = anyhow::Error::new(err).context("Failed to fetch URL data from the DB");
        ApiError::InternalError(err)
    })?;
    let result = result.ok_or(ApiError::NotFound)?;
    let url = Url::parse(&result.long_url).map_err(|err| {
        let err = anyhow::Error::new(err).context("Failed to parse the long URL");
        ApiError::InternalError(err)
    })?;
    Ok(web::Redirect::to(url.to_string()).permanent())
}

#[delete("/{hash}")]
pub async fn delete_shortened_url(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let hash = path.into_inner();
    let pool = pool.into_inner();
    let result = sqlx::query!(
        r#"
        DELETE FROM urls
        WHERE hash = $1
        "#,
        hash
    )
    .execute(&*pool)
    .await
    .context("Failed to delete URL data from the DB")
    .map_err(ApiError::InternalError)?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use log::info;
    use sqlx::{PgConnection, PgPool};
    use url::Url;
    use uuid::Uuid;

    use crate::routes::{delete_shortened_url, shorten_url};

    use super::Response;

    #[sqlx::test]
    async fn test_hash_generation_and_insert_works(pool: PgPool) {
        let url = Url::parse("https://docs.rs/sqlx/latest/sqlx/attr.test.html").unwrap();
        let hash = super::generate_and_insert_hash(&pool, &url).await;
        assert!(hash.is_ok());
    }

    #[actix_rt::test]
    async fn test_creating_and_delete_url_works() {
        let pool = test_db_setup_and_migrate().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(web::Data::new(Url::parse("http://localhost:5000").unwrap()))
                .service(shorten_url)
                .service(delete_shortened_url),
        )
        .await;

        let create_req = test::TestRequest::post()
            .uri("/shorten")
            .set_json(super::RequestData {
                url: Url::parse("https://docs.rs/sqlx/latest/sqlx/attr.test.html").unwrap(),
            })
            .to_request();
        let create_resp: Response = test::call_and_read_body_json(&app, create_req).await;
        let key = create_resp.key;
        let req = test::TestRequest::delete()
            .uri(&format!("/{key}"))
            .to_request();
        let response = test::call_service(&app, req).await;
        assert!(response.status().is_success());
    }

    async fn test_db_setup_and_migrate() -> PgPool {
        use sqlx::Connection;
        let url = std::env::var("DATABASE_URL").unwrap();

        let mut conn = PgConnection::connect(&url).await.unwrap();

        let mut url = Url::parse(&url).unwrap();
        let new_db_name = Uuid::new_v4().to_string();
        info!("Generated new DB name: {}", new_db_name);

        sqlx::query(&format!(r#"CREATE DATABASE "{}""#, new_db_name))
            .execute(&mut conn)
            .await
            .unwrap();

        url.set_path(&new_db_name);

        let pool = PgPool::connect(url.as_str()).await.unwrap();

        sqlx::migrate!().run(&pool).await.unwrap();

        pool
    }
}
