/*

serv - proxy - client

client ->

*/

use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::channel;
use std::thread;

fn handle_client(mut client_stream: TcpStream, target_addr: String) {
    // Proxy -> server connection

    let mut target_stream = TcpStream::connect(target_addr).unwrap();

    let mut target_reader = BufReader::new(target_stream.try_clone().unwrap());
    let mut client_reader = BufReader::new(client_stream.try_clone().unwrap());

    thread::spawn(move || {
        let mut buffer = [0; 1024];

        loop {
            match client_reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    // The thing going into the server

                    target_stream
                        .write_all(&buffer[..n])
                        .expect("Failed to write to target");
                }
                Err(_) => break,
            }
        }
    });

    let mut buffer = [0; 1024];

    loop {
        match target_reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                client_stream
                    .write_all(&buffer[..n])
                    .expect("Failed to write to client");
            }
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
                thread::spawn(move || {
                    handle_client(stream, target_addr);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept client connection: {}", e);
            }
        }
    }
}
