pub mod commands;
pub mod db;

use crossbeam::channel::Receiver;
use itertools::Itertools;
use std::{
    env,
    net::{TcpListener, TcpStream},
    num::NonZeroUsize,
};

fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or("11211".to_string());
    let address = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(address)?;
    let (tx, rx) = crossbeam::channel::unbounded::<TcpStream>();

    let thread_count = std::thread::available_parallelism()
        .unwrap_or(NonZeroUsize::MIN)
        .get();
    let threads = (0..thread_count)
        .map(|_| {
            let rx = rx.clone();
            std::thread::spawn(move || {
                handle_streams(rx);
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
    Ok(())
}

fn handle_streams(receiver: Receiver<TcpStream>) {
    for stream in receiver {
        handle_stream(stream);
    }
}

fn handle_stream(mut stream: TcpStream) {
    use std::io::Read;
    use std::io::Write;
    let mut buf = Vec::with_capacity(1024);
    let _ = stream.read_to_end(&mut buf);
    let _ = stream.write(&buf);
}
