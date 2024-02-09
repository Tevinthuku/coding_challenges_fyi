use bytes::BytesMut;
use log::{error, info};
use redis_server::resp::Frame;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024 * 4];

            // In a loop, read data from the socket and write the data back.
            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(_) => {}
                    Err(e) => {
                        error!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                }

                let buf = buf.to_vec();
                let content = String::from_utf8(buf).unwrap();

                let frame = redis_server::resp::Frame::deserialize(content.clone())
                    .unwrap_or_else(|| {
                        Ok(Frame::new_error(format!(
                            "Could not parse frame from {}",
                            content
                        )))
                    })
                    .unwrap_or_else(|err| {
                        Frame::new_error(format!(
                            "Error encountered: {} while parsing frame {}",
                            err, content
                        ))
                    });

                let response = redis_server::cmd::execute_command(frame);
                let response = response.serialize();
                // Write the data back
                if let Err(e) = socket.write_all(response.as_bytes()).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
