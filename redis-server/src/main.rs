use std::{net::TcpListener, sync::Arc, thread};

use anyhow::Context;
use log::error;
use redis_server::{cmd::Command, db::Db, frame::Frame};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let listener = TcpListener::bind("127.0.0.1:6379")?;
    let db = Arc::new(Db::new());

    for stream in listener.incoming() {
        let stream = stream?;
        let db = db.clone();
        thread::spawn(move || {
            let mut connection = redis_server::connection::Connection::new(stream);
            loop {
                let bytes = connection.read_bytes().unwrap();
                if let Some(bytes) = bytes {
                    let frame = Frame::deserialize(&bytes)
                        .context(format!("Failed to deserialize {bytes:?}"))
                        .unwrap();

                    let response = match Command::from_frame(frame) {
                        Ok(command) => command.execute(&mut connection, &db),
                        Err(err) => {
                            error!("Failed to parse command: {:?}", err);
                            connection.send_error("ERR failed to parse command")
                        }
                    };

                    if let Err(err) = response {
                        error!("Failed to execute command: {:?}", err);
                        connection
                            .send_error("ERR failed to execute command")
                            .unwrap();
                    }
                } else {
                    break;
                }
            }
        });
    }
    Ok(())
}
