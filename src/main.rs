mod config;

use anyhow::{bail, Context, Result};
use config::{Config, ProxyConfig};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
    net::{TcpListener, TcpStream},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = config::read_config_file().await?;
    let server = TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;

    let config_arc = Arc::new(config);

    loop {
        let (client, addr) = server.accept().await?;
        let config_clone = config_arc.clone();
        tokio::spawn(async move { handle_client(client, addr, config_clone).await });
    }
}

async fn handler(
    mut from: ReadHalfBuf,
    mut to: WriteHalf<TcpStream>,
    proxy_info: &ProxyConfig,
    name: &str,
) -> Result<()> {
    loop {
        let mut line = String::new();

        // Returns newline too
        if from.read_line(&mut line).await? == 0 {
            break;
        }
        println!("{name}: {}", line[..line.len() - 2].to_string());

        if line == "\r\n" {
            to.write_all(b"\r\n").await?;
            break;
        }
        if line.starts_with("Host:") {
            to.write_all(format!("Host: {}\r\n", proxy_info.target).as_bytes())
                .await?;
        } else {
            to.write_all(line.as_bytes()).await?;
        }
    }

    loop {
        let mut buffer = [0; 1024];
        let read = from.read(&mut buffer).await?;

        if read == 0 {
            break;
        }

        to.write_all(&buffer[..read]).await?;
    }

    println!("{name}: EOF");
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

    buf.read_line(&mut line).await?;

    let (method, uri, _version) =
        if let [method, uri, version] = line.split(' ').collect::<Vec<_>>().as_slice() {
            (method.to_string(), uri.to_string(), version.to_string())
        } else {
            bail!("Wrong number of elements in first line");
        };

    let mut headers = HashMap::new();

    loop {
        if buf.read_line(&mut line).await? == 0 {
            break;
        }

        if line == "\r\n" {
            break;
        }

        let index = line.find(':').context("No : found in header")?;

        let (key, value) = line.split_at(index);

        headers.insert(key.to_string(), value.to_string());
    }

    Ok(Headers {
        method,
        uri,
        headers,
    })
}

async fn handle_client(stream: TcpStream, addr: SocketAddr, config: Arc<Config>) -> Result<()> {
    println!("Client connected from {}:{}", addr.ip(), addr.port());

    let (stream_read, stream_write) = tokio::io::split(stream);

    let mut stream_read_buf = BufReader::new(stream_read);

    // Read the headers
    let headers = read_headers(&mut stream_read_buf).await?;

    // Find which server to connect
    let proxy_config = find_proxy(&headers.uri, &config)
        .with_context(|| format!("No proxy matched {}", headers.uri))?;

    // Connect to server
    let server_stream = TcpStream::connect(format!(
        "{}:{}",
        proxy_config.target, proxy_config.target_port
    ))
    .await?;

    let (server_read, server_write) = tokio::io::split(server_stream);
    let server_read_buf = BufReader::new(server_read);

    tokio::try_join!(
        async move { handler(stream_read_buf, server_write, proxy_config, "<").await },
        async move { handler(server_read_buf, stream_write, proxy_config, ">").await }
    )?;

    Ok(())
}
