use std::net::TcpListener;
use std::io::{Read, Write};

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
        match stream {
            Ok(_stream) => {
                let mut _processing_stream = _stream;
                let mut buffer = [0; 1024];
                let buffer_alt = [0; 1024];
                let buffer_str = String::from_utf8_lossy(&buffer_alt);
                for line in buffer_str.lines() {
                    if line.trim() == "PING" {
                        _processing_stream.read(&mut buffer).unwrap();
                        _processing_stream.write_all(b"+PONG\r\n").unwrap();
                        _processing_stream.flush().unwrap();
                        println!("PONG!")
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
