mod config;

use anyhow::{bail, Context, Result};
use config::{Config, ProxyConfig};
use log::{debug, error, info};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
    net::{TcpListener, TcpStream},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug");
    }

    pretty_env_logger::init();

    info!("welcome to cuzi :)");

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
    name: &str,
) -> Result<()> {
    // bail!("xd");
    debug!("{} {} opened", proxy_info.path, name);

    loop {
        let mut buffer = [0; 1024];
        let read = from.read(&mut buffer).await?;

        debug!(
            "{} {} read {} bytes: {:?}",
            proxy_info.path,
            name,
            read,
            String::from_utf8(buffer[..read].to_vec()).unwrap_or("<binary>".to_string())
        );

        if read == 0 {
            break;
        }

        to.write_all(&buffer[..read]).await?;

        debug!("{} {} wrote {} bytes", proxy_info.path, name, read);
    }

    info!("{} {} closed", proxy_info.path, name);
    Ok(())
}

fn find_proxy<'a>(path: &str, config: &'a Config) -> Option<&'a ProxyConfig> {
    for proxy in &config.proxies {
        if proxy.path == path {
            return Some(&proxy);
        }
    }

    return None;
}

type ReadHalfBuf = BufReader<ReadHalf<TcpStream>>;

struct Headers {
    method: String,
    uri: String,
    headers: HashMap<String, String>,
}

async fn read_headers(buf: &mut ReadHalfBuf) -> Result<Headers> {
    let mut line = String::new();

    debug!("Reading headers");

    buf.read_line(&mut line).await?;

    debug!("Header: {:?}", line);

    let (method, uri, _version) =
        if let [method, uri, version] = line.split(' ').collect::<Vec<_>>().as_slice() {
            (method.to_string(), uri.to_string(), version.to_string())
        } else {
            bail!("Wrong number of elements in first line");
        };

    let mut headers = HashMap::new();

    loop {
        line.clear();
        if buf.read_line(&mut line).await? == 0 {
            break;
        }

        debug!("Header: {:?}", line);

        if line == "\r\n" {
            break;
        }

        let index = line.find(':').context("No : found in header")?;

        let (key, value) = line.split_at(index);

        headers.insert(key.to_string(), value.to_string());
    }

    debug!("Headers OK");

    Ok(Headers {
        method,
        uri,
        headers,
    })
}

async fn write_headers_to_server(headers: &Headers, buf: &mut WriteHalf<TcpStream>) -> Result<()> {
    buf.write_all(format!("{} {} HTTP/1.1\r\n", headers.method, headers.uri).as_bytes())
        .await?;

    for (key, value) in &headers.headers {
        buf.write_all(format!("{}: {}\r\n", key, value).as_bytes())
            .await?;
    }

    buf.write_all("\r\n".as_bytes()).await?;

    Ok(())
}

async fn handle_client(stream: TcpStream, addr: SocketAddr, config: Arc<Config>) -> Result<()> {
    info!("Client connected from {}:{}", addr.ip(), addr.port());

    let (stream_read, stream_write) = tokio::io::split(stream);

    let mut stream_read_buf = BufReader::new(stream_read);

    // Read the headers
    let headers = read_headers(&mut stream_read_buf).await?;

    info!("Request is {} {}", headers.method, headers.uri);

    // Find which server to connect
    let proxy_config = find_proxy(&headers.uri, &config)
        .with_context(|| format!("No proxy matched {}", headers.uri))?;

    debug!("Proxy config: {:?}", proxy_config);

    let server_address = format!("{}:{}", proxy_config.target, proxy_config.target_port);

    debug!("Connecting to {server_address}");

    // Connect to server
    let server_stream = TcpStream::connect(server_address).await?;

    debug!("Connected to server");

    let (server_read, mut server_write) = tokio::io::split(server_stream);
    let server_read_buf = BufReader::new(server_read);

    // modify headers

    // send headers to server

    write_headers_to_server(&headers, &mut server_write).await?;

    tokio::try_join!(
        async move { handler(stream_read_buf, server_write, proxy_config, "<").await },
        async move { handler(server_read_buf, stream_write, proxy_config, ">").await }
    )?;

    Ok(())
}
