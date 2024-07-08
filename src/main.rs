use std::net::TcpListener;
use std::io::{Read, Write};
use std::str;

fn main() {
    println!("Logs from your program will appear here!");

    let listener = match TcpListener::bind("127.0.0.1:6379") {
        Ok(listener) => listener,
        Err(e) => {
            println!("Failed to bind to port 6379: {}", e);
            return;
        }
    };
    
    for stream in listener.incoming() {
        println!("New connection!");
        match stream {
            Ok(mut stream) => {
                println!("Connection successful!");
                let mut buffer = [0; 1024];
                
                loop {
                    match stream.read(&mut buffer) {
                        Ok(0) => {
                            println!("Connection closed!");
                            break;
                        }
                        Ok(_) => {
                            let request = str::from_utf8(&buffer).unwrap();
                            if request.starts_with("PING") {
                                stream.write_all(b"+PONG\r\n").unwrap();
                                stream.flush().unwrap();
                                println!("PONG!");
                            }
                            buffer.fill(0);   // Clear the buffer
                        }
                        Err(e) => {
                            println!("Failed to read from connection: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to establish a connection: {}", e);
            }
        }
    }
}
