use std::io::{self, Write};

use anyhow::{Context, Result, anyhow};
use futures_util::{SinkExt, StreamExt};
use tokio::io::AsyncReadExt;
use tokio_tungstenite::connect_async;
use tungstenite::{
    ClientRequestBuilder,
    http::{self, StatusCode},
};

pub async fn start_client(uri: http::Uri, auth: Option<&String>) -> Result<()> {
    let mut req = ClientRequestBuilder::new(uri);
    if let Some(auth_string) = auth {
        req = req.with_header("Authorization", auth_string);
    }

    let (upstream, response) = connect_async(req)
        .await
        .context("connect to upstream host")?;

    if response.status() != StatusCode::SWITCHING_PROTOCOLS {
        return Err(anyhow!(
            "unexpected status code: {} (expected 101)",
            response.status()
        ));
    }

    let (mut upstream_write, mut upstream_read) = upstream.split();

    let to_upstream = async {
        let mut buf: [u8; 4096] = [0; 4096];

        loop {
            let n = tokio::io::stdin()
                .read(&mut buf)
                .await
                .context("read from stdin")?;

            if n == 0 {
                break;
            }

            let msg = tungstenite::Bytes::from_owner(buf[0..n].to_vec());

            upstream_write
                .send(tungstenite::Message::Binary(msg))
                .await
                .context("write to upstream")?;
        }

        Ok::<(), anyhow::Error>(())
    };

    let to_downstream = async {
        while let Some(msg) = upstream_read.next().await {
            match msg? {
                tungstenite::Message::Text(utf8_bytes) => {
                    io::stdout().write_all(utf8_bytes.as_bytes())?
                }
                tungstenite::Message::Binary(bytes) => io::stdout().write_all(&bytes)?,
                tungstenite::Message::Close(_) => break,
                _ => (),
            }

            io::stdout().flush().context("flush stdout")?;
        }
        Ok::<(), anyhow::Error>(())
    };

    tokio::select! {
        res = to_upstream => res.context("to upstream")?,
        res = to_downstream => res.context("to downstream")?,
    };

    Ok(())
}
