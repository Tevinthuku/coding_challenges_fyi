use std::{
    borrow::Cow,
    env,
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    num::NonZeroUsize,
    path::Path,
};

use bytes::{BufMut, BytesMut};
use crossbeam::channel::unbounded;
use crossbeam::channel::Receiver;

fn main() -> std::io::Result<()> {
    let address = env::var("ADDRESS").unwrap_or("127.0.0.1:80".to_owned());
    let file_directory = env::var("FILE_DIRECTORY").unwrap_or("./www".to_owned());
    run_server(&address, file_directory)
}

fn run_server(address: &str, file_directory: String) -> std::io::Result<()> {
    let (sender, receiver) = unbounded::<TcpStream>();

    let listener = TcpListener::bind(address)?;

    let available_parallelism = std::thread::available_parallelism().map_or(2, NonZeroUsize::get);

    let mut threads = Vec::with_capacity(available_parallelism);

    for _ in 0..available_parallelism {
        let receiver = receiver.clone();
        let file_directory = file_directory.clone();
        let thread = std::thread::spawn(move || process_requests(receiver, file_directory));
        threads.push(thread);
    }

    for stream in listener.incoming() {
        sender.send(stream?).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to send stream to receiver: {err:?}"),
            )
        })?;
    }

    for thread in threads {
        thread.join().map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to join thread: {err:?}"),
            )
        })?;
    }

    Ok(())
}

fn process_requests(receiver: Receiver<TcpStream>, file_directory: String) {
    for stream in receiver {
        handle_client(stream, &file_directory).unwrap();
    }
}

fn handle_client(mut stream: TcpStream, file_directory: &str) -> std::io::Result<()> {
    let mut buffer = [0; 1024];

    let read_bytes = stream.read(&mut buffer)?;

    let buffer = &buffer[0..read_bytes];

    let first_line = buffer
        .split(|byte| *byte == b'\r')
        .next()
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))?;

    let request = parse_request(first_line)?;

    let file_to_send = {
        let path = match request.path.as_ref() {
            "/" => "index.html",
            path => &path[1..],
        };

        let path = Path::new(file_directory).join(path);
        let mut buffer = Vec::with_capacity(1024 * 1024);
        let file = File::open(path);

        let mut file = match file {
            Ok(file) => file,
            Err(_) => {
                let mut response = BytesMut::with_capacity(512);
                response.put_slice(request.http_version);
                response.put_slice(b" 404 Not Found\r\n\r\n");
                return stream.write_all(&response);
            }
        };

        file.read_to_end(buffer.as_mut())?;
        buffer
    };

    let mut response = BytesMut::with_capacity(1024);
    response.put_slice(request.http_version);
    response.put_slice(b" 200 OK\r\n\r\n");
    response.put_slice(&file_to_send);
    response.put_slice(b"\r\n");
    stream.write_all(&response)
}

fn parse_request(first_line: &[u8]) -> std::io::Result<Request<'_>> {
    let mut split_by_space = first_line.split(|byte| *byte == b' ');
    let _method = split_by_space
        .next()
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))?;
    let path = split_by_space
        .next()
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))?;
    let http_version = split_by_space
        .next()
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))?;

    Ok(Request {
        path: String::from_utf8_lossy(path),
        http_version,
    })
}

#[derive(Debug)]
struct Request<'a> {
    path: Cow<'a, str>,
    http_version: &'a [u8],
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpStream,
        thread,
    };

    use crate::run_server;

    #[test]
    fn test_connection() {
        let address = "127.0.0.1:8080";
        let start_thread = thread::spawn(|| run_server(address, "./www".to_owned()));

        thread::sleep(std::time::Duration::from_secs(1));

        let mut stream = TcpStream::connect(address).expect("Failed to connect to server");
        println!("Connected to server");
        stream
            .write_all(b"GET / HTTP/1.1\r\n\r\n")
            .expect("Failed to write to stream");

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .expect("Failed to read from stream");

        drop(stream);

        assert!(response.contains("HTTP/1.1 200 OK"));
        // let _ = start_thread.join().expect("Failed to join thread");
    }
}
