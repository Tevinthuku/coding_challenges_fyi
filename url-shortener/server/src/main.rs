pub mod error;
pub mod routes;

use std::{env, io};

use actix_web::{web, App, HttpServer};

use routes::{delete_shortened_url, redirect_to_long_url, shorten_url};
use sqlx::{postgres::PgPoolOptions, PgPool};
use url::Url;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let pool = db_setup_and_migrate().await?;
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| 5000.to_string());
    let binding_address = format!("{host}:{port}");

    let base_server_url: Url = env::var("BASE_SERVER_URL")
        .map(|url| url.parse().unwrap())
        .unwrap_or_else(|_| format!("http://{host}:{port}").parse().unwrap());

    HttpServer::new(move || {
        let pool = pool.clone();
        let base_server_url = base_server_url.clone();
        App::new()
            .app_data(web::Data::new(pool))
            .app_data(web::Data::new(base_server_url))
            .service(shorten_url)
            .service(redirect_to_long_url)
            .service(delete_shortened_url)
    })
    .bind(binding_address)?
    .run()
    .await
}

async fn db_setup_and_migrate() -> io::Result<PgPool> {
    let url = env::var("DATABASE_URL").map_err(|err| {
        io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to get DATABASE_URL: {}. Set the URL as an env variable.",
                err
            ),
        )
    })?;
    let max_connections = env::var("MAX_CONNECTIONS")
        .map(|val| val.parse().unwrap())
        .unwrap_or(20);
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&url)
        .await
        .map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to connect to Postgres. Ensure that the Database server is running or if you are connected to the correct URL: Err = {:?}. ", err),
            )
        })?;

    migrate(&pool).await?;

    Ok(pool)
}

async fn migrate(pool: &PgPool) -> Result<(), io::Error> {
    sqlx::migrate!().run(pool).await.map_err(|err| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to run migrations: Check to see if there is an issue with the .sql files. Err =  {:?}", err),
        )
    })
}
