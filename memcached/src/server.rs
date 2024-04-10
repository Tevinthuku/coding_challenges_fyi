use std::{future::Future, sync::Arc};

use crate::{anyhow, commands, response};
use anyhow::Context;
use bytes::{BufMut, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use tokio::sync::broadcast::Receiver as BroadCastReceiver;
use tokio::sync::broadcast::Sender as BroadCastSender;

use tokio::sync::mpsc::Sender;

use crate::db::Db;

pub async fn run(tcp_listener: TcpListener, shut_down: impl Future) -> anyhow::Result<()> {
    let db = Arc::new(Db::new());
    let (shut_down_signal_sender, _) = tokio::sync::broadcast::channel(1);
    let (shut_down_complete_sender, mut shut_down_complete_receiver) =
        tokio::sync::mpsc::channel::<()>(1);

    let mut listener = Listener {
        tcp_listener,
        db,
        shut_down_signal: shut_down_signal_sender,
        _shut_down_complete: shut_down_complete_sender,
    };

    tokio::select! {
        _ = shut_down => {
            println!("Shutting down server");
        }
        _ = listener.accept() => {}
    }
    let Listener {
        shut_down_signal,
        _shut_down_complete,
        ..
    } = listener;
    // notify all connections that the server is shutting down
    drop(shut_down_signal);

    // Drop the final Listener sender.
    drop(_shut_down_complete);

    let _ = shut_down_complete_receiver.recv().await;

    Ok(())
}

struct Listener {
    tcp_listener: TcpListener,
    db: Arc<Db>,
    // notifies connections all of which subscribed to the broadcast sender that the server is shutting down.
    shut_down_signal: BroadCastSender<()>,
    // goes out of scope once the ConnectionHandler is dropped, thus signals to the server that it is finally safe to shut down once all senders are dropped.
    _shut_down_complete: Sender<()>,
}

impl Listener {
    pub async fn accept(&mut self) -> anyhow::Result<()> {
        loop {
            let (stream, _) = self
                .tcp_listener
                .accept()
                .await
                .context("Failed to accept a new connection")?;

            let mut handler = ConnectionHandler {
                connection: Connection {
                    stream,
                    buffer: BytesMut::with_capacity(1024),
                    db: self.db.clone(),
                },
                shut_down_signal: self.shut_down_signal.subscribe(),
                _shut_down_complete: self._shut_down_complete.clone(),
            };

            tokio::spawn(async move { handler.run().await });
        }
    }
}

struct ConnectionHandler {
    connection: Connection,
    shut_down_signal: BroadCastReceiver<()>,
    // once the ConnectionHandler is dropped, the sender is dropped as well, thus notifying the server that it is safe to shut down.
    _shut_down_complete: Sender<()>,
}

impl ConnectionHandler {
    async fn run(&mut self) {
        tokio::select! {
            _ = self.shut_down_signal.recv() => {
                println!("Shutting down connection")
            }
            _ = self.connection.execute() => {}
        }
    }
}

struct Connection {
    stream: TcpStream,
    db: Arc<Db>,
    buffer: BytesMut,
}

impl Connection {
    async fn execute(&mut self) {
        loop {
            let response = loop {
                let bytes_read = self.stream.read_buf(&mut self.buffer).await;
                let bytes_read = match bytes_read {
                    Ok(bytes_read) => bytes_read,
                    Err(err) => {
                        eprintln!("Failed to read from stream: {:?}", err);
                        break Err(anyhow!(err).context("Failed to read from stream"));
                    }
                };

                self.buffer.put_u8(b' ');
                let response = commands::execute_command(&self.buffer, &self.db).map(Some);
                if response.is_ok() {
                    self.buffer.clear();
                    break response;
                }
                if bytes_read == 0 {
                    // This means that the connection was closed (BrokenPipe)
                    break Ok(None);
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
            if let Err(err) = self.stream.write(&response).await {
                eprintln!("Failed to write to stream: {:?}", err);
            }
            if let Err(err) = self.stream.flush().await {
                eprintln!("Failed to flush stream: {:?}", err);
            }
        }
    }
}
