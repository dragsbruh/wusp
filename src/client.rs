use anyhow::{Context, Result, anyhow};
use futures_util::{SinkExt, StreamExt};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::connect_async;

use tungstenite::{
    ClientRequestBuilder,
    http::{self, StatusCode},
};

pub async fn start_client(uri: http::Uri, auth: Option<String>) -> Result<()> {
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
            let n = match tokio::io::stdin()
                .read(&mut buf)
                .await
                .context("read from stdin")?
            {
                0 => continue,
                e => e,
            };

            let msg = tungstenite::Bytes::copy_from_slice(&buf[0..n]);

            upstream_write
                .send(tungstenite::Message::Binary(msg))
                .await
                .context("write to upstream")?;
        }

        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    };

    let to_downstream = async {
        while let Some(msg) = upstream_read.next().await {
            match msg? {
                tungstenite::Message::Text(utf8_bytes) => tokio::io::stdout()
                    .write_all(utf8_bytes.as_bytes())
                    .await
                    .context("write text to stdout")?,
                tungstenite::Message::Binary(bytes) => tokio::io::stdout()
                    .write_all(&bytes)
                    .await
                    .context("write binary to stdout")?,
                tungstenite::Message::Close(_) => break,
                _ => (),
            }

            tokio::io::stdout().flush().await.context("flush stdout")?;
        }
        eprintln!("warning: upstream might have closed connection");
        Ok::<(), anyhow::Error>(())
    };

    tokio::select! {
        res = to_upstream => res.context("to upstream")?,
        res = to_downstream => res.context("to downstream")?,
    };

    Ok(())
}
