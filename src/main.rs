use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected");
                return;
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
                return;
            }
        };

        let message = String::from_utf8_lossy(&buffer[..bytes_read]);

        println!("Received: {}", message);

        // Hvis beskeden starter med /message, svarer vi med en smiley :)
        let response = if message.starts_with("/message") {
            ":)\n"
        } else {
            "Server received your message\n"
        };

        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Error writing to client: {}", e);
            return;
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Could not bind to port 8080");
    println!("Server listening on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New client connected!");
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}