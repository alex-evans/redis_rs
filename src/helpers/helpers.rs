use std::sync::Arc;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
// use tokio::sync::MutexGuard;

// use crate::SharedState;

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
        let mut reader = BufReader::new(&mut *stream);
        let mut response: String = String::new();
        reader.read_line(&mut response).await?;
        println!("Received response from server: {}", response);
        return Ok(response);
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

// pub async fn send_data_to_replica<'a>(
//     state: &'a Arc<Mutex<SharedState>>,
//     request: &str
// ) -> () {
//     println!("Sending data to replica");
//     let state: MutexGuard<SharedState> = state.lock().await;
//     for (key, value) in state.store.iter() {
//         println!("ADE Key: {}, Value: {}", key, value);
//     }
//     let repl1_host = state.store.get("repl1-listening-host").cloned().unwrap_or_default();
//     let repl1_port = state.store.get("repl1-listening-port").cloned().unwrap_or_default();
//     let repl1_address: String = format!("{}:{}", repl1_host, repl1_port);
//     println!("Connecting to Replica: {}", repl1_address);
//     match TcpStream::connect(&repl1_address).await {
//         Ok(stream) => {
//             let stream = Arc::new(Mutex::new(stream));
            
//             // Send Request to Replica
//             let message = format!("*3\r\n{}", request);
//             send_message_to_server_arc(stream.clone(), &message, false).await.unwrap();

//         },
//         Err(e) => {
//             println!("Error connecting to Replica: {:?}", e);
//         }
//     }

// }