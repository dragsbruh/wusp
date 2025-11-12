use std::{net::SocketAddr, sync::Arc};

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use tokio_tungstenite::accept_hdr_async;

use tungstenite::{
    Message,
    handshake::server::{Request, Response},
    http::StatusCode,
};

pub async fn start_server(
    addy: SocketAddr,
    target_addy: SocketAddr,
    auth: Option<String>,
) -> Result<()> {
    let listener = TcpListener::bind(addy).await.context("bind to address")?;
    let auth = Arc::new(auth);

    eprintln!("[server] listening on {}", addy);

    loop {
        let (stream, client_addy) = listener.accept().await?;
        let auth = auth.clone();

        eprintln!("[{}] handling connection request", client_addy);

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, client_addy, target_addy, auth).await {
                eprintln!("[{}] error handling client: {}", client_addy, e,);
            }
        });
    }
}

pub async fn handle_client(
    stream: TcpStream,
    client_addy: SocketAddr,
    target_addy: SocketAddr,
    auth: Arc<Option<String>>,
) -> Result<()> {
    let auth_ref = auth.as_ref();

    let callback = |req: &Request, response: Response| {
        if let Some(expected) = auth_ref {
            let hdr = req.headers().get("Authorization");

            if let Some(hdr_value) = hdr {
                if let Ok(value) = hdr_value.to_str() {
                    if value == expected {
                        return Ok(response);
                    }
                }
            }

            return Err(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Some("invalid authorization string".to_string()))
                .unwrap());
        };

        Ok(response)
    };

    let downstream = accept_hdr_async(stream, callback)
        .await
        .context("during websocket handshake")?;

    let mut upstream = tokio::net::TcpStream::connect(target_addy)
        .await
        .context("while connecting to upstream")?;

    eprintln!("[{}] connection established successfully", client_addy);

    let (mut downstream_write, mut downstream_read) = downstream.split();
    let (mut upstream_read, mut upstream_write) = upstream.split();

    let to_upstream = async {
        while let Some(msg) = downstream_read.next().await {
            let msg = msg?;

            match msg {
                Message::Text(utf8_bytes) => {
                    upstream_write.write_all(utf8_bytes.as_bytes()).await?
                }
                Message::Binary(bytes) => upstream_write.write_all(&bytes).await?,
                Message::Close(_) => break,
                _ => continue,
            };
        }
        Ok::<(), anyhow::Error>(())
    };

    let to_downstream = async {
        let mut buf = [0; 4096];
        loop {
            let n = upstream_read.read(&mut buf).await?;
            if n == 0 {
                break;
            }

            downstream_write
                .send(Message::Binary(buf[..n].to_vec().into()))
                .await?;
        }
        Ok::<(), anyhow::Error>(())
    };

    tokio::select! {
        res = to_upstream => res.context("to upstream")?,
        res = to_downstream => res.context("to downstream")?,
    };

    eprintln!("[{}] connection closed", client_addy);

    Ok(())
}
