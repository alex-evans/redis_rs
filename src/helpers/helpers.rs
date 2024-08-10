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
    stream: Arc<Mutex<TcpStream>>, 
    message: &str,
    wait_for_response: bool
) -> Result<String, Box<dyn std::error::Error>> {
    println!("Sending message to server: {}", message);
    let mut stream = stream.lock().await;
    stream.write_all(message.as_bytes()).await?;
    stream.flush().await?;
    if wait_for_response {
        let mut reader = BufReader::new(&mut *stream);
        let mut response: String = String::new();
        reader.read_line(&mut response).await?;
        println!("Received response from server: {}", response);
        return Ok(response);
    }
    return Ok("".to_string());
}