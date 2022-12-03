use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:2002").expect("Binding failed!");
    for stream in listener.incoming() {
        match stream {
            Err(e) => {
                eprint!("failed {}", e);
                println!("hello");
            }
            Ok(stream) => {
                println!("Connected!");
                stream.shutdown(std::net::Shutdown::Both);
            }
        }
    }
}
