use std::{
    borrow::Cow,
    env,
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
};

use bytes::{BufMut, BytesMut};
use crossbeam::channel::bounded;
use crossbeam::channel::Receiver;

const THREAD_COUNT: usize = 8;

fn main() -> std::io::Result<()> {
    let address = env::var("ADDRESS").unwrap_or("127.0.0.1:80".to_owned());
    let (sender, receiver) = bounded::<TcpStream>(2000);

    let listener = TcpListener::bind(address)?;

    let mut threads = Vec::with_capacity(THREAD_COUNT);

    for _ in 0..THREAD_COUNT {
        let receiver = receiver.clone();
        let thread = std::thread::spawn(move || process_requests(receiver));
        threads.push(thread);
    }

    for stream in listener.incoming() {
        sender.send(stream?).unwrap();
    }

    for thread in threads {
        thread.join().unwrap();
    }

    Ok(())
}

fn process_requests(receiver: Receiver<TcpStream>) {
    for stream in receiver {
        handle_client(stream).unwrap();
    }
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
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
        let path = Path::new("./").join(path);
        let mut buffer = Vec::with_capacity(1024 * 1024);
        let mut file = File::open(path);
        let mut file = match file {
            Ok(file) => file,
            Err(err) => {
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
    let method = split_by_space
        .next()
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))?;
    let path = split_by_space
        .next()
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))?;
    let http_version = split_by_space
        .next()
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))?;

    Ok(Request {
        method,
        path: String::from_utf8_lossy(path),
        http_version,
    })
}

#[derive(Debug)]
struct Request<'a> {
    method: &'a [u8],
    path: Cow<'a, str>,
    http_version: &'a [u8],
}
