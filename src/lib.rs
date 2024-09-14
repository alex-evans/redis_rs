
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::net::TcpListener;

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
}

mod server_types {
    pub mod master;
    pub mod replica;
}

mod handlers {
    pub mod echo;
    pub mod get;
    pub mod info;
    pub mod set;
    pub mod ping;
    pub mod psync;
    pub mod replconf;
    pub mod wait;
}

mod helpers {
    pub mod helpers;
}

use server_types::master::handle_master_connections;
use server_types::replica::handle_replica_connections;

pub struct SharedState {
    pub store: HashMap<String, String>,
    pub sender: broadcast::Sender<String>,
}

pub async fn run(
    config: Config
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting server on port {}", config.port);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).await?;
    let state = Arc::new(tokio::sync::Mutex::new(SharedState {
        store: HashMap::new(),
        sender: broadcast::channel(10).0,
    }));

    if config.replicaof.is_empty() {
        // Master Instance
        return handle_master_connections(listener, state).await;
    }
    
    // Replica Instance
    return handle_replica_connections(listener, state, config).await;
}
