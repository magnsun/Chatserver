use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

struct ChatServer {
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>>,
}

impl ChatServer {
    fn new() -> Self {
        ChatServer {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn handle_connection(&self, stream: TcpStream, addr: String) {
        let clients = self.clients.clone();
        let stream = Arc::new(Mutex::new(stream)); // Wrap stream in Arc<Mutex>

        {
            let mut clients_lock = clients.lock().unwrap();
            clients_lock.insert(addr.clone(), stream.clone()); // Indsæt den klonede Arc
        }

        let mut reader = BufReader::new(stream.lock().unwrap().try_clone().unwrap()); // Forsøg at klone her

        loop {
            let mut buffer = String::new();
            let bytes_read = reader.read_line(&mut buffer).await.unwrap();
            if bytes_read == 0 {
                break; // forbindelsen er lukket
            }

            let message = buffer.trim().to_string();
            if message.starts_with('/') {
                self.handle_command(&message, &stream).await; // send en Arc<Mutex>
            } else {
                self.broadcast_message(&message, &addr).await;
            }
        }

        clients.lock().unwrap().remove(&addr);
    }

    async fn broadcast_message(&self, message: &str, sender: &str) {
        let clients = self.clients.lock().unwrap();
        for (addr, stream) in clients.iter() {
            if addr != sender {
                let _ = stream.lock().unwrap().write_all(format!("{}: {}\n", sender, message).as_bytes()).await;
            }
        }
    }

    async fn handle_command(&self, command: &str, stream: &Arc<Mutex<TcpStream>>) {
        match command {
            "/status" => {
                let response = format!("Server is running. Active clients: {}\n", self.clients.lock().unwrap().len());
                let _ = stream.lock().unwrap().write_all(response.as_bytes()).await;
            }
            _ => {
                let response = "Unknown command\n";
                let _ = stream.lock().unwrap().write_all(response.as_bytes()).await;
            }
        }
    }

    async fn run(&self, addr: &str) {
        let listener = TcpListener::bind(addr).await.unwrap();
        println!("Server running on {}", addr);

        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let addr = stream.peer_addr().unwrap().to_string();
            let server = self.clone();
            tokio::spawn(async move {
                server.handle_connection(stream, addr).await;
            });
        }
    }
}

#[tokio::main]
async fn main() {
    let server = ChatServer::new();
    server.run("127.0.0.1:8080").await;
}