use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Logs from your program will appear here!");
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    
    loop {
        let (mut socker, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            loop {
                match socker.read(&mut buf).await {
                    Ok(0) => {
                        println!("Connection closed");
                        return;
                    }
                    Ok(_) => {
                        let request = std::str::from_utf8(&buf).unwrap();
                        if request.starts_with('[') {
                            let command: Vec<&str> = request[1..request.len()-1].split(',').collect();
                            println!("Received command: {:?}", command);
                            match command[0] {
                                "PING" => {
                                    if let Err(e) = socker.write_all(b"+PONG\r\n").await {
                                        println!("Failed to write to connection: {}", e);
                                        return;
                                    }
                                    println!("PONG sent!");
                                }
                                "ECHO" => {
                                    if let Err(e) = socker.write_all(format!("+{}\r\n", command[1]).as_bytes()).await {
                                        println!("Failed to write to connection: {}", e);
                                        return;
                                    }
                                    println!("ECHO sent!");
                                }
                                _ => {
                                    println!("Unknown command: {}", command[0]);
                                }
                            }
                        }
                        buf.fill(0);  // Clear the buffer
                    }
                    Err(e) => {
                        println!("Failed to read from connection: {}", e);
                        return;
                    }
                }
            }
        });
    }
}
