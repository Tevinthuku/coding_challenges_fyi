use std::{borrow::Cow, env, future::Future, num::NonZeroUsize, path::Path};
use tokio::{fs::File, signal};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    runtime::Builder,
    select,
};

use bytes::{BufMut, BytesMut};
use crossbeam::channel::Receiver;
use crossbeam::channel::{unbounded, Sender};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let address = env::var("ADDRESS").unwrap_or("127.0.0.1:80".to_owned());
    let file_directory = env::var("FILE_DIRECTORY").unwrap_or("./www".to_owned());
    run_server(&address, file_directory, signal::ctrl_c()).await
}

async fn run_server(
    address: &str,
    file_directory: String,
    shutdown: impl Future,
) -> std::io::Result<()> {
    let (sender, receiver) = unbounded::<TcpStream>();

    let available_parallelism = std::thread::available_parallelism().map_or(2, NonZeroUsize::get);

    let mut threads = Vec::with_capacity(available_parallelism);

    for _ in 0..available_parallelism {
        let receiver = receiver.clone();
        let file_directory = file_directory.clone();
        let thread = std::thread::spawn(move || process_streams(receiver, file_directory));
        threads.push(thread);
    }

    select! {
        _ = run_server_inner(address, sender.clone()) => {}
        _ = shutdown => {}
    }

    drop(sender);

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

async fn run_server_inner(address: &str, sender: Sender<TcpStream>) -> std::io::Result<()> {
    let listener = TcpListener::bind(address).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        sender.send(stream).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to send stream to receiver: {err:?}"),
            )
        })?;
    }
}

fn process_streams(receiver: Receiver<TcpStream>, file_directory: String) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async move {
        for stream in receiver {
            tokio::spawn(async move {
                if let Err(e) = handle_tcp_stream(stream, file_directory.clone()).await {
                    eprintln!("Error handling client: {e:?}");
                }
            });
        }
    })
}

async fn handle_tcp_stream(mut stream: TcpStream, file_directory: String) -> std::io::Result<()> {
    let mut buffer = BytesMut::with_capacity(1024);

    let read_bytes = stream.read_buf(&mut buffer).await?;

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

        let path = Path::new(&file_directory).join(path);
        let mut buffer = Vec::with_capacity(1024 * 1024);
        let file = File::open(path).await;

        let mut file = match file {
            Ok(file) => file,
            Err(_) => {
                let mut response = BytesMut::with_capacity(512);
                response.put_slice(request.http_version);
                response.put_slice(b" 404 Not Found\r\n\r\n");
                return stream.write_all(&response).await;
            }
        };

        file.read_to_end(buffer.as_mut()).await?;
        buffer
    };

    let mut response = BytesMut::with_capacity(1024);
    response.put_slice(request.http_version);
    response.put_slice(b" 200 OK\r\n\r\n");
    response.put_slice(&file_to_send);
    response.put_slice(b"\r\n");
    stream.write_all(&response).await
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

    use std::{io, time::Duration};

    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
        sync::oneshot,
        time,
    };

    use crate::run_server;

    async fn connect(address: &str) -> io::Result<TcpStream> {
        let mut backoff = 1;
        loop {
            match TcpStream::connect(address).await {
                Ok(socket) => return Ok(socket),
                Err(err) => {
                    if backoff > 30 {
                        return Err(err);
                    }
                }
            }
            time::sleep(Duration::from_secs(backoff)).await;
            backoff *= 2;
        }
    }

    #[tokio::test]
    async fn test_200_response() {
        let (send, recv) = oneshot::channel::<()>();

        let address = "127.0.0.1:8080";
        let join_handle = tokio::spawn(async move {
            run_server(address, "./www".to_owned(), recv).await.unwrap();
        });
        let mut stream = connect(address).await.unwrap();
        stream
            .write_all(b"GET / HTTP/1.1\r\n\r\n")
            .await
            .expect("Failed to write to stream");

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .await
            .expect("Failed to read from stream");

        drop(stream);

        assert!(response.contains("HTTP/1.1 200 OK"));
        send.send(()).expect("Failed to send shutdown signal");
        let _ = join_handle.await;
    }

    #[tokio::test]
    async fn test_not_found() {
        let (send, recv) = oneshot::channel::<()>();

        let address = "127.0.0.1:8081";
        let join_handle = tokio::spawn(async move {
            run_server(address, "./www".to_owned(), recv).await.unwrap();
        });
        let mut stream = connect(address).await.unwrap();
        stream
            .write_all(b"GET /notfound.html HTTP/1.1\r\n\r\n")
            .await
            .expect("Failed to write to stream");

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .await
            .expect("Failed to read from stream");

        drop(stream);

        assert!(response.contains("HTTP/1.1 404 Not Found"));
        send.send(()).expect("Failed to send shutdown signal");
        let _ = join_handle.await;
    }
}
