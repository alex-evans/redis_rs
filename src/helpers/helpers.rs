use std::sync::Arc;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

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
    let _skip_line: &str = lines.next().unwrap_or("");
    let return_line: &str = lines.next().unwrap_or("");
    return return_line.to_string();
}

pub async fn send_message_to_server(
    stream: &mut TcpStream,
    message: &str,
    wait_for_response: bool
) -> Result<String, Box<dyn std::error::Error>> {
    println!("Sending message to server: {}", message);
    stream.write_all(message.as_bytes()).await?;
    stream.flush().await?;

    {
        println!("Waiting for response: {}", wait_for_response);
        if wait_for_response {
            println!("ADE - Waiting for response from server...");
            let mut reader = BufReader::new(&mut *stream);
            println!("ADE - Reader: {:?}", reader);
            let mut response: String = String::new();
            println!("ADE - Reading line from server...");
            reader.read_line(&mut response).await?;
            println!("Received response from server: {}", response);
        }
    }

    return Ok(String::new());
}

pub async fn send_message_to_server_arc(
    stream: Arc<Mutex<TcpStream>>,
    message: &str,
    wait_for_response: bool
) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = stream.lock().await;
    send_message_to_server(&mut *stream, message, wait_for_response).await
}

pub async fn send_data_to_replica<'a>(
    stream: &mut TcpStream,
    request: &str
) -> () {
    println!("Sending data to replica");
    send_message_to_server(stream, &request, false).await.unwrap();
}