use std::env;

use redis_starter_rust::{
    Config,
    run
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Logs from your program will appear here!");
    let config = Config::from_args(env::args().collect()).unwrap();
    run(config).await
}


