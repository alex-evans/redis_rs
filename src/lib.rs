
use std::collections::HashMap;
// use std::sync::{Arc, Mutex};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct Config {
    pub port: String,
    pub replicaof: String
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
}

pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting server on port {}", config.port);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).await?;
    let state = Arc::new(tokio::sync::Mutex::new(SharedState {
        store: HashMap::new(),
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
        let config_clone = config.clone();
        tokio::spawn(async move {
            if let Err(err) = process_request(&config_clone, stream, state_clone).await {
                eprintln!("Failed to process request: {}", err);
            }
        });
    }
}

async fn process_request(config_ref: &Config, mut stream: tokio::net::TcpStream, state: Arc<Mutex<SharedState>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Handling process request");
    let mut buf = [0; 1024];

    loop {
        match stream.read(&mut buf).await {
            Ok(0) => {
                println!("Connection closed");
                return Ok(());
            }
            Ok(_) => {
                let request = std::str::from_utf8(&buf).unwrap();

                if request.starts_with('+') {
                    // Means it's a Simple String
                    let response = &request[1..request.len()-2];
                    println!("Received Simple String: {}", response);
                    // Process the response here
                    return Ok(());
                } else if request.starts_with('-') {
                    // Means it's an Error
                    let error = &request[1..request.len()-2];
                    println!("Received Error: {}", error);
                    // Process the error here
                    return Ok(());
                } else if request.starts_with(':') {
                    // Means it's an Integer
                    let integer = &request[1..request.len()-2];
                    println!("Received Integer: {}", integer);
                    // Process the integer here
                    return Ok(());
                } else if request.starts_with('$') {
                    // Means it's a Bulk String
                    let bulk_string = &request[1..request.len()-2];
                    println!("Received Bulk String: {}", bulk_string);
                    // Process the bulk string here
                    return Ok(());
                } else if request.starts_with('*') {
                    // Means it's a List
                    println!("Received List: {}", request);
                    list_request(config_ref, &request, &state, &mut stream).await;
                    // let response_string = list_request(config_ref, &request, &state);
                    // let response_bytes = response_string.as_bytes().try_into().unwrap();
                    // if let Err(e) = socket.write_all(response_bytes).await {
                    //     println!("Failed to write to connection: {}", e);
                    //     return;
                    // }
                    return Ok(());
                } else {
                    println!("Unknown request format");
                    return Ok(());
                }
            }
            Err(e) => {
                println!("Failed to read from connection: {}", e);
                return Ok(());
            }
        }
    }
}