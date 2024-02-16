use std::{
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use anyhow::Context;
use redis_server::{cmd::Command, db::Db, frame::Frame};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let listener = TcpListener::bind("127.0.0.1:6379")?;
    let db = Arc::new(Db::new());

    for stream in listener.incoming() {
        let stream = stream?;
        let db = db.clone();
        thread::spawn(move || handle_stream(stream, db));
    }
    Ok(())
}

fn handle_stream(stream: TcpStream, db: Arc<Db>) {
    let mut connection = redis_server::connection::Connection::new(stream);

    loop {
        let bytes = connection.read_bytes().unwrap();
        if let Some(bytes) = bytes {
            let frame = match Frame::deserialize(&bytes)
                .context(format!("Failed to deserialize {bytes:?}"))
            {
                Ok(frame) => frame,
                Err(err) => {
                    connection.send_error(err.to_string().as_str()).unwrap();
                    continue;
                }
            };

            let command = match Command::from_frame(frame) {
                Ok(command) => command,
                Err(err) => {
                    connection.send_error(err.to_string().as_str()).unwrap();
                    continue;
                }
            };

            if let Err(err) = command
                .execute(&mut connection, &db)
                .context("Failed to execute command")
            {
                connection.send_error(err.to_string().as_str()).unwrap();
                continue;
            }
        } else {
            break;
        }
    }
}
