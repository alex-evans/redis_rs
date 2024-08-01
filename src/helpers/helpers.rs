
use std::net::TcpStream;
use std::io::{Read, Write};

pub fn determine_number_of_elements(line: &str) -> i32 {
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

pub fn get_next_element(lines: &mut std::str::Lines) -> String {
    let _skip_line = lines.next().unwrap_or("");
    let return_line = lines.next().unwrap_or("");
    return return_line.to_string();
}

pub fn send_message_to_server(stream: &mut TcpStream, message: &str) -> String {
    let mut response = String::new();
    println!("Sending message to server: {}", message);
    stream.write_all(message.as_bytes()).unwrap();
    stream.read_to_string(&mut response).unwrap();
    println!("Received response: {}", response);
    response
}