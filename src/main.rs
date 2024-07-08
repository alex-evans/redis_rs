use std::net::TcpListener;
use std::io::{Read, Write};
use std::str;

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").expect("Failed to bind to port 6379");
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Connection successful!");
                let mut buffer = Vec::new();
                let mut connection_buffer = [0; 1024];
                
                loop {
                    match stream.read(&mut connection_buffer) {
                        Ok(0) => {
                            println!("Connection closed!");
                            break;
                        }
                        Ok(bytes_read) => {
                            buffer.extend_from_slice(&connection_buffer[..bytes_read]);
                            let request = str::from_utf8(&buffer).unwrap();
                            
                            if request.contains("PING") {
                                if let Err(e) = stream.write_all(b"+PONG\r\n") {
                                    println!("Failed to write to connection: {}", e);
                                    break;
                                }
                                if let Err(e) = stream.flush() {
                                    println!("Failed to flush the connection: {}", e);
                                    break;
                                }
                                println!("PONG sent!");
                            }
                            buffer.clear();   // Clear the buffer
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
