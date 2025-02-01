use bytes::{BufMut, BytesMut};
use std::sync::Arc;
use std::{borrow::Cow, env, future::Future, path::Path};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::Semaphore;
use tokio::{fs::File, signal};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
};

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
    let stream_capacity = env::var("STREAM_CAPACITY").unwrap_or("5000".to_owned());
    let stream_capacity = stream_capacity
        .parse::<usize>()
        .inspect_err(|e| {
            eprintln!("Invalid STREAM_CAPACITY: {e:?}");
        })
        .unwrap_or(5000);
    let (tx, rx) = mpsc::channel::<TcpStream>(stream_capacity);

    let stream_processor = tokio::spawn(process_streams(rx, Cow::Owned(file_directory)));

    select! {
        _ = accept_connections(address, tx.clone()) => {}
        _ = shutdown => {}
    }

    drop(tx);

    if let Err(e) = stream_processor.await {
        eprintln!("Stream processor shutdown error: {e:?}");
    }

    Ok(())
}

async fn accept_connections(address: &str, sender: Sender<TcpStream>) -> std::io::Result<()> {
    let listener = TcpListener::bind(address).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        sender.send(stream).await.map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to send stream to receiver: {err:?}"),
            )
        })?;
    }
}

async fn process_streams(mut receiver: Receiver<TcpStream>, file_directory: Cow<'static, str>) {
    let semaphore = Arc::new(Semaphore::new(512));
    while let Some(stream) = receiver.recv().await {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let file_directory = file_directory.clone();
        tokio::spawn(async move {
            // Permit is automatically released when dropped at the end of this scope
            let _permit = permit;

            if let Err(e) = handle_tcp_stream(stream, &file_directory).await {
                eprintln!("Error handling client: {e:?}");
            }
        });
    }
}

async fn handle_tcp_stream(mut stream: TcpStream, file_directory: &str) -> std::io::Result<()> {
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

        let mut buffer = Vec::with_capacity(1024 * 1024);

        let mut file = match open_file(file_directory, path).await {
            Some(file) => file,
            None => {
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

async fn open_file(file_directory: &str, path: &str) -> Option<File> {
    let path = Path::new(file_directory).join(path);

    let base = tokio::fs::canonicalize(file_directory).await.ok()?;
    let target = tokio::fs::canonicalize(&path).await.ok()?;

    if !target.starts_with(&base) {
        return None;
    }

    File::open(path).await.ok()
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
                    if backoff > 10 {
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
