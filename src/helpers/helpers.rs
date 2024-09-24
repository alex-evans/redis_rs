use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::timeout;
use std::time::Duration;

use crate::SharedState;

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

    if wait_for_response {
        println!("Waiting for response...");
        let mut buffer = vec![0; 1024];
        let timeout_duration = Duration::from_secs(5);

        match timeout(timeout_duration, stream.read(&mut buffer)).await {
            Ok(Ok(bytes_read)) => {
                if bytes_read > 0 {
                    let response = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                    println!("Received response from server: {}", response);
                    Ok(response)
                } else {
                    Err("Server closed the connection".into())
                }
            }
            Ok(Err(err)) => Err(format!("Error reading from stream: {}", err).into()),
            Err(_) => Err("Timeout while reading from stream".into()),
        }

    } else {
        Ok(String::new())
    }
}

pub async fn is_replica(state: &Arc<Mutex<SharedState>>) -> bool {
    let state_guard = state.lock().await;
    state_guard.store.get("replicaof").is_some()
}