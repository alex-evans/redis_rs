
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
// use tokio::net::TcpStream;

#[derive(Clone)]
pub struct Config {
    pub port: String,
    pub replicaof: String,
}

impl Config {
    pub fn from_args(args: Vec<String>) -> Result<Config, &'static str> {
        let mut port = String::from("6379");
        let mut replicaof = String::from("");

        for i in 1..args.len() {
            if args[i] == "--port" || args[i] == "-p" {
                if i + 1 < args.len() {
                    port = args[i + 1].clone();
                } else {
                    return Err("No port provided");
                }
            } else if args[i] == "--replicaof" {
                if i + 1 < args.len() {
                    replicaof = args[i + 1].clone();
                } else {
                    return Err("No replicaof provided");
                }
            }
        }

        Ok(Config {
            port: port,
            replicaof: replicaof
        })

    }
}

mod command_types {
    pub mod list;
    pub mod replication;
}

mod handlers {
    pub mod echo;
    pub mod get;
    pub mod info;
    pub mod set;
    pub mod ping;
    pub mod psync;
    pub mod replconf;
}

mod helpers {
    pub mod helpers;
}

use command_types::list::list_request;
use command_types::replication::{
    store_replication,
    send_replication_data
};

pub struct SharedState {
    pub store: HashMap<String, String>,
    pub sender: broadcast::Sender<String>,
}

pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting server on port {}", config.port);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).await?;
    let state = Arc::new(tokio::sync::Mutex::new(SharedState {
        store: HashMap::new(),
        sender: broadcast::channel(10).0,
    }));

    let config_clone = config.clone();
    if config_clone.replicaof != "" {
        println!("Replicaof is set to: {}", config_clone.replicaof);
        store_replication(&config_clone, &state).await;
        send_replication_data(&config_clone, &state).await;
    }

    loop {
        let (stream, _) = listener.accept().await?;
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(err) = process_request(stream, state_clone).await {
                eprintln!("Failed to process request: {}", err);
            }
        });
    }
}

async fn process_request(
    stream: tokio::net::TcpStream, 
    state: Arc<Mutex<SharedState>>
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Handling process request");
    
    let stream = Arc::new(Mutex::new(stream));
    let mut buf = [0; 1024];

    loop {
        println!("Waiting for data...");
        let mut stream_lock = stream.lock().await;
        match stream_lock.read(&mut buf).await {
            Ok(0) => {
                println!("Connection closed");
                return Ok(());
            }
            Ok(n) => {
                let request = match std::str::from_utf8(&buf[..n]) {
                    Ok(req) => req,
                    Err(e) => {
                        eprintln!("Invalid UTF-8 sequence: {}", e);
                        continue;
                    }
                };
                println!("Received request: {}", request);

                if request.starts_with('*') {
                    drop(stream_lock); // Release the lock before processing
                    list_request(&request, &state, stream.clone()).await;
                    println!("Finished processing list request");
                } else {
                    println!("Unhandled request format");
                }
            }
            Err(e) => {
                eprintln!("Failed to read from connection: {}", e);
                return Ok(());
            }
        }
    }
}