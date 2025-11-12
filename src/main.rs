use std::net::SocketAddr;

use clap::{Parser, Subcommand};
use tungstenite::http;

mod client;
mod server;

#[derive(Parser, Debug)]
#[command(name = "wusp")]
#[command(author = "dragsbruh")]
#[command(version = "0.1.0")]
#[command(subcommand_required = true)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// connect to an existing wusp server and bind stdin/stdout with
    Client {
        #[arg(env = "WUSP_HOST")]
        /// url of a running wusp server (must start with ws:// or wss:// protocol)
        host: http::Uri,

        #[arg(long, env = "WUSP_AUTH")]
        /// authentication string to use
        auth: Option<String>,
    },

    /// starts tcp over websocket proxy server
    Server {
        #[arg(env = "WUSP_ADDRESS")]
        /// address to bind to and listen on (http)
        address: SocketAddr,

        #[arg(env = "WUSP_TARGET")]
        /// target tcp server to proxy requests to
        target: SocketAddr,

        #[arg(long, env = "WUSP_AUTH")]
        /// optional authentication string to check for (literal)
        auth: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    tokio::select! {
        _ = app() => (),
        _ = tokio::signal::ctrl_c() => {
            eprintln!("\rclosing (due to ctrl+c)")
        },
    }
}

async fn app() {
    let cmd = Cli::parse();

    let result = match cmd.command {
        Command::Client { host, auth } => client::start_client(host.clone(), auth).await,

        Command::Server {
            address,
            target,
            auth,
        } => server::start_server(address.clone(), target.clone(), auth.clone()).await,
    };
    if let Err(e) = result {
        eprintln!("fatal error: {}", e);
    }
}
