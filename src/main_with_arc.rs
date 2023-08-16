use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

fn handle_client(client_stream: Arc<Mutex<TcpStream>>, target_addr: String) {
    let target_stream = TcpStream::connect(target_addr).unwrap();
    let target_stream = Arc::new(Mutex::new(target_stream));

    let client_stream_clone = Arc::clone(&client_stream);
    let target_stream_clone = Arc::clone(&target_stream);

    thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            match client_stream_clone.lock() {
                Ok(mut stream) => {
                    match stream.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => {
                            // The thing going into the server
                            let mut target_stream = target_stream_clone.lock().unwrap();
                            target_stream
                                .write_all(&buffer[..n])
                                .expect("Failed to write to target");
                        }
                        Err(_) => break,
                    }
                }
                Err(_) => break,
            }
        }
    });

    let mut buffer = [0; 1024];
    loop {
        match target_stream.lock() {
            Ok(mut stream) => match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => match client_stream.lock() {
                    Ok(mut client) => {
                        client
                            .write_all(&buffer[..n])
                            .expect("Failed to write to client");
                    }
                    Err(_) => break,
                },
                Err(_) => break,
            },
            Err(_) => break,
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");

    for client_stream in listener.incoming() {
        match client_stream {
            Ok(stream) => {
                let target_addr = "example.com:80".to_string(); // Change this to your target address
                let client_stream = Arc::new(Mutex::new(stream));
                let target_addr_clone = target_addr.clone();

                thread::spawn(move || {
                    handle_client(client_stream, target_addr_clone);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept client connection: {}", e);
            }
        }
    }
}
