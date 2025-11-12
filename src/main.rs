mod client;
mod server;

use clap::{Arg, Command};
use tungstenite::http;

#[tokio::main]
async fn main() {
    let cmd = Command::new("wusp")
        .bin_name("wusp")
        .subcommand_required(true)
        .subcommands(&[
            Command::new("client")
                .args(&[
                    Arg::new("host")
                        .env("WUSP_HOST")
                        .value_parser(clap::value_parser!(http::Uri))
                        .help("websocket url to the tcp proxy")
                        .required(true),
                    Arg::new("auth")
                        .env("WUSP_AUTH")
                        .help("authentication string to pass to the proxy"),
                ])
                .about("links the tcp proxy running on `host` to stdin/stdout"),
            Command::new("server").args(&[
                Arg::new("address")
                    .env("WUSP_ADDR")
                    .value_parser(clap::value_parser!(std::net::SocketAddr))
                    .help("address to listen on (example: 0.0.0.0:8080)")
                    .required(true),
                Arg::new("target")
                    .env("WUSP_TARGET")
                    .value_parser(clap::value_parser!(std::net::SocketAddr))
                    .help("target to proxy tcp stream to (example: 10.10.10.10:8080)")
                    .required(true),
                Arg::new("auth")
                    .env("WUSP_AUTH")
                    .help("authentication string to check for (default: allow any)"),
            ]),
        ]);

    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("client", client_matches)) => {
            let host = client_matches
                .get_one::<http::Uri>("host")
                .expect("host is required");

            let auth = client_matches.get_one::<String>("auth");

            if let Err(e) = client::start_client(host.clone(), auth).await {
                eprintln!("fatal error in client: {}", e);
            }
        }
        Some(("server", server_matches)) => {
            let address = server_matches
                .get_one::<std::net::SocketAddr>("address")
                .expect("address is required");

            let target = server_matches
                .get_one::<std::net::SocketAddr>("target")
                .expect("target is required");

            let auth = match server_matches.get_one::<String>("auth") {
                Some(auth_str) => Some(auth_str.clone()),
                None => None,
            };

            println!("listening on {} proxying requests to {}", address, target);

            if let Err(e) =
                server::start_server(address.clone(), target.clone(), auth.clone()).await
            {
                eprintln!("fatal error in server: {}", e);
            }
        }
        _ => unreachable!(),
    }
}
