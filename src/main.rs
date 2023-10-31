mod config;
mod connection;
mod http;
mod types;

use anyhow::{Context, Result};
use config::{Config, ProxyConfig};
use http::{HTTPObject, HTTPReadState};
use log::{error, info};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, WriteHalf},
    net::{TcpListener, TcpStream},
};
use tokio_rustls::rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore, ServerName};
use tokio_rustls::TlsConnector;
use types::ReadHalfBuf;

use crate::connection::Connection;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug");
    }

    pretty_env_logger::init();

    info!("Welcome to cuzi :)");

    let config = config::read_config_file()
        .await
        .with_context(|| format!("Config file couldn't read"))?;
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

async fn handler<T, C>(
    mut from: ReadHalfBuf<T>,
    mut to: WriteHalf<C>,
    proxy_info: &ProxyConfig,
    mut obj: HTTPObject,
    name: &str,
) -> Result<()>
where
    T: AsyncRead + AsyncWrite,
    C: AsyncRead + AsyncWrite,
{
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

fn configure_proxy<'a>(
    request: &HTTPObject,
    config: &'a Config,
) -> Result<(&'a ProxyConfig, String)> {
    let proxy_config = find_proxy(&request.uri, &config)
        .with_context(|| format!("No proxy matched {}", request.uri))?;
    let server_address = format!("{}:{}", proxy_config.target, proxy_config.target_port);

    info!(
        "Proxy {} {} {}",
        request.method, request.uri, server_address
    );

    Ok((proxy_config, server_address))
}

async fn handle_connection<T, C>(
    server_conn: Connection<T>,
    client_conn: Connection<C>,
    request: HTTPObject,
    proxy_config: &ProxyConfig,
) -> Result<()>
where
    T: AsyncRead + AsyncWrite,
    C: AsyncRead + AsyncWrite,
{
    // Separate server and client connections read/writer streams.
    let server_read = server_conn.read_buffer;
    let server_write = server_conn.writer;

    let client_read = client_conn.read_buffer;
    let client_write = client_conn.writer;

    tokio::try_join!(
        async move { handler(client_read, server_write, proxy_config, request, "client",).await },
        async move {
            handler(
                server_read,
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

async fn handle_client(stream: TcpStream, addr: SocketAddr, config: Arc<Config>) -> Result<()> {
    info!("Client connected from {}:{}", addr.ip(), addr.port());

    let mut client_conn = Connection::new(stream, connection::ConnectionType::HTTP);
    let mut request = HTTPObject::new();

    // Read information about the request (first line)
    request.read_info(&mut client_conn.read_buffer).await?;

    // Find which server to connect.
    let (proxy_config, server_address) = configure_proxy(&request, &config)?;

    // Create stream to the server.
    let stream = TcpStream::connect(server_address).await?;

    if proxy_config.use_tls.unwrap_or(false) {
        let mut root_cert_store = RootCertStore::empty();
        root_cert_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(config));
        let dnsname = ServerName::try_from(proxy_config.target.as_str()).unwrap();

        let tls_stream = connector.connect(dnsname, stream).await?;
        let server_conn = Connection::new(tls_stream, connection::ConnectionType::HTTPS);
        return handle_connection(server_conn, client_conn, request, proxy_config).await;
    } else {
        let server_conn = Connection::new(stream, connection::ConnectionType::HTTP);
        return handle_connection(server_conn, client_conn, request, proxy_config).await;
    }
}
