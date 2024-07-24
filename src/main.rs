use std::vec;

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Logs from your program will appear here!");
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    
    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        println!("Connection closed");
                        return;
                    }
                    Ok(_) => {
                        let request = std::str::from_utf8(&buf).unwrap();

                        if request.starts_with('+') {
                            // Means it's a Simple String
                            let response = &request[1..request.len()-2];
                            println!("Received Simple String: {}", response);
                            // Process the response here
                        } else if request.starts_with('-') {
                            // Means it's an Error
                            let error = &request[1..request.len()-2];
                            println!("Received Error: {}", error);
                            // Process the error here
                        } else if request.starts_with(':') {
                            // Means it's an Integer
                            let integer = &request[1..request.len()-2];
                            println!("Received Integer: {}", integer);
                            // Process the integer here
                        } else if request.starts_with('$') {
                            // Means it's a Bulk String
                            let bulk_string = &request[1..request.len()-2];
                            println!("Received Bulk String: {}", bulk_string);
                            // Process the bulk string here
                        } else if request.starts_with('*') {
                            // Means it's a List
                            let response_list = handle_list_request(&request);
                            let response_string = response_list.join("\r\n");
                            let response_bytes = response_string.as_bytes().try_into().unwrap();
                            if let Err(e) = socket.write_all(response_bytes).await {
                                println!("Failed to write to connection: {}", e);
                                return;
                            }
                        } else {
                            println!("Unknown request format");
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

fn handle_list_request(request: &str) -> Vec<String> {
    println!("Received List Request: {}", request);
    let mut lines = request.lines();
    let first_line = lines.next().unwrap();
    let number_of_elements = determine_number_of_elements(&first_line);
    if number_of_elements < 0 {
        return vec!["-ERR Invalid request".to_string()];
    } else {
        let mut response_list = vec![];
        let element_one = get_next_element(&mut lines);
        for _ in 0..number_of_elements {
            match element_one.as_str() {
                "ECHO" => response_list.push(get_next_element(&mut lines)),
                _ => response_list.push("-ERR Invalid request".to_string())
            }
        }
        return response_list;
    }
}

fn determine_number_of_elements(line: &str) -> i32 {
    let characters: String = line.chars().skip(1).collect();
    let characters_as_int: Result<i32, _> = characters.parse();
    if let Ok(num) = characters_as_int {
        println!("Characters as integer: {}", num);
        return num;
    } else {
        println!("Failed to convert characters to integer");
        return -1;
    }
}

fn get_next_element(lines: &mut std::str::Lines) -> String {
    println!("Getting next element");
    let _skip_line = lines.next().unwrap();
    println!("_skip_line: {:?}", _skip_line.to_string());
    let return_line = lines.next().unwrap();
    println!("return_line: {:?}", return_line.to_string());
    return return_line.to_string();
}
