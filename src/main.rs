use std::env;
use redis_starter_rust::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Logs from your program will appear here!");

    let args: Vec<String> = env::args().collect();
    let mut port = String::from("6379");

    for i in 1..args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            port = args[i + 1].clone();
        }
    }

    run(port).await
}
