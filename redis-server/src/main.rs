use std::{
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use anyhow::Context;
use crossbeam::channel::{bounded, Receiver};
use redis_server::{cmd::Command, db::Db, frame::Frame};

/// Tokio is needed for the background tasks of purging expired keys. More on this can be seen in the `db` module.
/// Everything else is synchronous.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let listener = TcpListener::bind("127.0.0.1:6379")?;
    let (sender, receiver) = bounded::<TcpStream>(200000);
    let db = Arc::new(Db::new()?);

    let mut threads = Vec::with_capacity(100);

    for _ in 0..100 {
        let receiver = receiver.clone();
        let db = db.clone();
        let thread = std::thread::spawn(move || {
            receive_streams(receiver, db);
        });
        threads.push(thread);
    }

    for stream in listener.incoming() {
        let stream = stream?;
        sender.send(stream).unwrap();
    }

    for thread in threads {
        thread.join().unwrap();
    }
    Ok(())
}

fn receive_streams(receiver: Receiver<TcpStream>, db: Arc<Db>) {
    for stream in receiver {
        let db = db.clone();
        handle_stream(stream, db)
    }
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
