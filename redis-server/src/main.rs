use anyhow::Context;
use log::error;
use redis_server::{cmd::Command, db::Db, frame::Frame};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    let db = Db::new();

    loop {
        let (socket, _) = listener.accept().await?;
        let db = db.clone();

        tokio::spawn(async move {
            let mut connection = redis_server::connection::Connection::new(socket);

            loop {
                let bytes = connection.read_bytes().await.unwrap();
                if let Some(bytes) = bytes {
                    let content = bytes.to_vec();
                    let content = String::from_utf8_lossy(&content).to_string();

                    let frame = Frame::deserialize(content.clone())
                        .context(format!("Failed to deserialize {content:?}"));

                    let frame = match frame {
                        Ok(frame) => frame,
                        Err(err) => {
                            error!("Failed to parse frame: {:?}", err);
                            connection
                                .send_error("Failed to parse frame")
                                .await
                                .unwrap();
                            continue;
                        }
                    };

                    let response = match Command::from_frame(frame) {
                        Ok(command) => command.execute(&mut connection, &db).await,
                        Err(err) => {
                            error!("Failed to parse command: {:?}", err);
                            connection
                                .send_error("ERR failed to parse command")
                                .await
                                .unwrap();
                            continue;
                        }
                    };

                    if let Err(err) = response {
                        error!("Failed to execute command: {:?}", err);
                        connection
                            .send_error("ERR failed to execute command")
                            .await
                            .unwrap();
                    }
                } else {
                    break;
                }
            }
        });
    }
}
