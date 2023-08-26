use serde::Deserialize;
use std::{fmt, net::SocketAddr, sync::Arc};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
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
    file.shutdown().await.unwrap();

    serde_json::from_str(file_content.as_str()).unwrap()
}

async fn handler(
    mut from: ReadHalf<TcpStream>,
    mut to: WriteHalf<TcpStream>,
    mut buffer: Vec<u8>,
    proxy_info: ProxyConfig,
) {
    loop {
        let line_bytes = extract_line(&mut from, &mut buffer).await;
        let line = String::from_utf8(line_bytes.to_vec());

        if line.is_err() {
            to.write_all(&line_bytes).await.unwrap();
        } else {
            let line = line.unwrap();
            if line.starts_with("Host:") {
                to.write_all(format!("Host: {}\r\n", proxy_info.target).as_bytes())
                    .await
                    .unwrap();
            } else {
                to.write_all(line.as_bytes()).await.unwrap();
            }
        }
    }
}

fn find_proxy(path: String, config: &Config) -> Option<ProxyConfig> {
    for proxy in &config.proxies {
        if proxy.path == path {
            return Some(proxy.clone());
        }
    }

    return None;
}

fn get_line(data: &[u8]) -> Option<(Vec<u8>, usize)> {
    if let Some(index) = data.iter().position(|&byte| byte == b'\n') {
        return Some((data[0..=index].to_vec(), index + 1));
    }
    None
}

fn parse_path(line: &String) -> Option<String> {
    let path_info: Vec<&str> = line.split(' ').collect();

    if path_info.len() > 1 {
        return Some(path_info[1].to_string());
    }

    None
}

async fn extract_line(reader: &mut ReadHalf<TcpStream>, buffer: &mut Vec<u8>) -> Vec<u8> {
    let mut temp_buffer = vec![0; 1024];

    loop {
        if let Some((line, end_index)) = get_line(&buffer) {
            *buffer = buffer[end_index..].to_vec();
            return line;
        }
        reader.read(&mut temp_buffer).await.unwrap();
        buffer.extend(temp_buffer.iter());
    }
}

async fn find_path(
    client: &mut ReadHalf<TcpStream>,
    config: &Config,
) -> (TcpStream, ProxyConfig, Vec<u8>) {
    let mut total_buffer: Vec<u8> = Vec::new();
    let mut buffer: Vec<u8> = Vec::new();

    loop {
        let line_bytes = extract_line(client, &mut buffer).await;
        let line = String::from_utf8(line_bytes.to_vec()).unwrap();
        total_buffer.extend(line_bytes.iter());

        if line.starts_with("GET") {
            if let Some(path) = parse_path(&line) {
                let proxy = find_proxy(path, config).unwrap();
                let server_stream = TcpStream::connect(proxy.to_string()).await.unwrap();
                total_buffer.extend(buffer.iter());

                return (server_stream, proxy, total_buffer);
            }
        }
    }
}

async fn handle_client(stream: TcpStream, addr: SocketAddr, config_arc: ArcConfig) {
    println!("Client connected from {}:{}", addr.ip(), addr.port());

    let config = config_arc.lock().await;
    let (mut stream_read, stream_write) = tokio::io::split(stream);
    let (server_stream, proxy, total_buffer) = find_path(&mut stream_read, &config).await;
    let (server_read, server_write) = tokio::io::split(server_stream);

    let client_proxy_info = proxy.clone();

    tokio::join!(
        async move { handler(stream_read, server_write, total_buffer, client_proxy_info).await },
        async move { handler(server_read, stream_write, Vec::new(), proxy).await }
    );
}
