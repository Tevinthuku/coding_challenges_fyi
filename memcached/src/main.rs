pub mod commands;
pub mod db;
pub mod response;

use anyhow::anyhow;
use bytes::{BufMut, BytesMut};
use crossbeam::channel::Receiver;
use db::Db;
use itertools::Itertools;
use std::{
    env,
    net::{TcpListener, TcpStream},
    num::NonZeroUsize,
    sync::Arc,
};

fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or("11211".to_string());
    let address = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(address)?;
    let (tx, rx) = crossbeam::channel::unbounded::<TcpStream>();

    let thread_count = std::thread::available_parallelism()
        .unwrap_or(NonZeroUsize::MIN)
        .get();

    let db = Arc::new(Db::new());
    let threads = (0..thread_count)
        .map(|_| {
            let rx = rx.clone();
            let db = db.clone();
            std::thread::spawn(move || {
                handle_streams(rx, db);
            })
        })
        .collect_vec();
    for stream in listener.incoming() {
        tx.send(stream?)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
    }
    drop(tx);
    for thread in threads {
        if let Err(err) = thread.join() {
            eprintln!("Error joining thread: {:?}", err)
        }
    }
    println!("Closing server");
    Ok(())
}

fn handle_streams(receiver: Receiver<TcpStream>, db: Arc<Db>) {
    for stream in receiver {
        handle_stream(stream, &db);
    }
}

fn handle_stream(mut stream: TcpStream, db: &Db) {
    use std::io::Read;
    use std::io::Write;
    let mut buffer = BytesMut::with_capacity(1024);

    loop {
        let response = loop {
            let mut buf = [0; 1024];
            let bytes_read = stream.read(&mut buf);
            let bytes_read = match bytes_read {
                Ok(bytes_read) => bytes_read,
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::BrokenPipe {
                        break Ok(None);
                    }
                    eprintln!("Failed to read from stream: {:?}", err);
                    break Err(anyhow!(err).context("Failed to read from stream"));
                }
            };
            let buf = &buf[..bytes_read];
            buffer.extend_from_slice(buf);
            buffer.put_u8(b' ');
            let response = commands::execute_command(&buffer, db).map(Some);
            if response.is_ok() {
                buffer.clear();
                break response;
            }
            if bytes_read == 0 {
                break response;
            }
        };
        let response = response.unwrap_or_else(|err| {
            eprintln!("Failed to execute command: {:?}", err);
            Some(response::Response::Error(format!("{err}")))
        });
        let response = match response {
            Some(response) => response,
            None => break,
        };
        let response = response.into_bytes();
        if let Err(err) = stream.write(&response) {
            eprintln!("Failed to write to stream: {:?}", err);
        }
        if let Err(err) = stream.flush() {
            eprintln!("Failed to flush stream: {:?}", err);
        }
    }
}
