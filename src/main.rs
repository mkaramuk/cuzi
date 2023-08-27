use serde::Deserialize;
use std::{fmt, net::SocketAddr, sync::Arc};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

#[derive(Deserialize, Debug, Clone)]
struct ProxyConfig {
    path: String,
    target: String,
    target_port: i32,
}

#[derive(Deserialize, Debug, Clone)]
struct Config {
    port: i32,
    proxies: Vec<ProxyConfig>,
}

impl fmt::Display for ProxyConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}:{}", self.target, self.target_port);
    }
}

type ArcConfig = Arc<Mutex<Config>>;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let config = read_config().await;
    let server = TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();

    // Zaten sadece read yapılacak, acaba bunun bi yöntemi yok mu
    let config_arc = Arc::new(Mutex::new(config));

    loop {
        let (client, addr) = server.accept().await.unwrap();
        let config_clone = config_arc.clone();
        tokio::spawn(async move { handle_client(client, addr, config_clone).await });
    }
}

async fn read_config() -> Config {
    let mut file = File::open("./config.json").await.unwrap();
    let mut file_content = String::new();
    file.read_to_string(&mut file_content).await.unwrap();
    serde_json::from_str(file_content.as_str()).unwrap()
}

async fn handler(
    mut from: ReadHalfBuf,
    mut to: WriteHalf<TcpStream>,
    proxy_info: ProxyConfig,
    name: &str,
) {
    loop {
        let mut line = String::new();

        // Returns newline too
        if from.read_line(&mut line).await.unwrap() == 0 {
            break;
        }
        println!("{name}: {}", line[..line.len() - 2].to_string());

        if line == "\r\n" {
            to.write_all(b"\r\n").await.unwrap();
            break;
        }
        if line.starts_with("Host:") {
            to.write_all(format!("Host: {}\r\n", proxy_info.target).as_bytes())
                .await
                .unwrap();
        } else {
            to.write_all(line.as_bytes()).await.unwrap();
        }
    }

    loop {
        let mut buffer = [0; 1024];
        let read = from.read(&mut buffer).await.unwrap();

        if read == 0 {
            break;
        }

        to.write_all(&buffer[..read]).await.unwrap();
    }

    println!("{name}: EOF");
}

fn find_proxy(path: &str, config: &Config) -> Option<ProxyConfig> {
    for proxy in &config.proxies {
        if proxy.path == path {
            return Some(proxy.clone());
        }
    }

    return None;
}

type ReadHalfBuf = BufReader<ReadHalf<TcpStream>>;

async fn find_path(buf: &mut ReadHalfBuf, config: &Config) -> (TcpStream, ProxyConfig) {
    let mut line = String::new();

    buf.read_line(&mut line).await.unwrap();

    let splitted: Vec<_> = line.split(' ').collect();
    let (_protocol, uri, _versionn) = if let [protocol, uri, version] = splitted[..] {
        (protocol, uri, version)
    } else {
        panic!("Invalid request");
    };

    let proxy = find_proxy(uri, config).unwrap();
    let mut server_stream = TcpStream::connect(proxy.to_string()).await.unwrap();

    server_stream.write_all(line.as_bytes()).await.unwrap();

    return (server_stream, proxy);
}

async fn handle_client(stream: TcpStream, addr: SocketAddr, config_arc: ArcConfig) {
    println!("Client connected from {}:{}", addr.ip(), addr.port());

    let config = config_arc.lock().await;
    let (stream_read, stream_write) = tokio::io::split(stream);

    let mut stream_read_buf = BufReader::new(stream_read);

    let (server_stream, proxy) = find_path(&mut stream_read_buf, &config).await;
    let (server_read, server_write) = tokio::io::split(server_stream);

    let server_read_buf = BufReader::new(server_read);

    let client_proxy_info = proxy.clone();

    tokio::join!(
        async move { handler(stream_read_buf, server_write, client_proxy_info, "<").await },
        async move { handler(server_read_buf, stream_write, proxy, ">").await }
    );
}
