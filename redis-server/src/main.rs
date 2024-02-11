use anyhow::Context;
use log::error;
use redis_server::{cmd::Command, resp::Frame};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut connection = redis_server::connection::Connection::new(socket);

            let content = connection.read().await.unwrap().unwrap();
            let frame = redis_server::resp::Frame::deserialize(content.clone());
            let frame = match frame {
                Ok(frame) => frame,
                Err(err) => {
                    error!("Failed to deserialize {err:?}");
                    let frame = Frame::Error(err.to_string());
                    connection.write_frame(frame).await.unwrap();
                    return;
                }
            };
            let command = Command::from_frame(frame);
            let maybe_err = match command {
                Ok(command) => command
                    .execute(&mut connection)
                    .await
                    .context("Failed to execute command")
                    .err(),
                Err(err) => Some(err),
            };

            if let Some(err) = maybe_err {
                error!("Command error: {err:?}");
                let frame = Frame::Error(err.to_string());
                connection.write_frame(frame).await.unwrap();
            }
        });
    }
}
