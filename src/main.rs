mod config;
mod http;
mod types;

use anyhow::{Context, Result};
use config::{Config, ProxyConfig};
use http::{HTTPObject, HTTPReadState};
use log::{error, info};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncWriteExt, BufReader, WriteHalf},
    net::{TcpListener, TcpStream},
};
use types::ReadHalfBuf;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug");
    }

    pretty_env_logger::init();

    info!("Welcome to cuzi :)");

    let config = config::read_config_file().await?;
    let config = Arc::new(config);
    let server = TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;

    info!("Listening on port {}", config.port);

    loop {
        let (client, addr) = server.accept().await?;
        let config_clone = config.clone();
        tokio::spawn(async move {
            match handle_client(client, addr, config_clone).await {
                Err(e) => {
                    error!("{}", e);
                }
                Ok(_) => {}
            }
        });
    }
}

async fn handler(
    mut from: ReadHalfBuf,
    mut to: WriteHalf<TcpStream>,
    proxy_info: &ProxyConfig,
    mut obj: HTTPObject,
    name: &str,
) -> Result<()> {
    loop {
        if obj.state == HTTPReadState::Info {
            obj.read_info(&mut from).await?;
        }
        if name == "client" {
            to.write_all(format!("{} {} HTTP/1.1\r\n", obj.method, obj.uri).as_bytes())
                .await?;
        } else {
            to.write_all(format!("HTTP/1.1 {} {}\r\n", obj.status_code, obj.status).as_bytes())
                .await?;
        }

        obj.read_headers(&mut from).await?;

        if name == "client" {
            if let Some(x) = obj.get_header_mul("host") {
                *x = proxy_info.target.clone();
            }
        }

        for (header, value) in &obj.headers {
            to.write_all(format!("{}: {}\r\n", header, value).as_bytes())
                .await?;
        }
        to.write_all(b"\r\n").await?;
        obj.read_body(&mut from).await?;
        to.write_all(&obj.body).await?;
    }
}

fn find_proxy<'a>(path: &str, config: &'a Config) -> Option<&'a ProxyConfig> {
    for proxy in &config.proxies {
        if proxy.path == path {
            return Some(&proxy);
        }
    }

    return None;
}

async fn handle_client(stream: TcpStream, addr: SocketAddr, config: Arc<Config>) -> Result<()> {
    info!("Client connected from {}:{}", addr.ip(), addr.port());

    let (client_read, client_write) = tokio::io::split(stream);
    let mut client_read_buf = BufReader::new(client_read);
    let mut request = HTTPObject::new();

    request.read_info(&mut client_read_buf).await?;

    // Find which server to connect
    let proxy_config = find_proxy(&request.uri, &config)
        .with_context(|| format!("No proxy matched {}", request.uri))?;
    let server_address = format!("{}:{}", proxy_config.target, proxy_config.target_port);

    info!(
        "Proxy {} {} {}",
        request.method, request.uri, server_address
    );

    let server_stream = TcpStream::connect(server_address).await?;
    let (server_read, server_write) = tokio::io::split(server_stream);
    let server_read_buf = BufReader::new(server_read);
    // modify headers

    tokio::try_join!(
        async move {
            handler(
                client_read_buf,
                server_write,
                proxy_config,
                request,
                "client",
            )
            .await
        },
        async move {
            handler(
                server_read_buf,
                client_write,
                proxy_config,
                HTTPObject::new(),
                "server",
            )
            .await
        }
    )?;

    Ok(())
}
