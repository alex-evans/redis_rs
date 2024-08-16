
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpStream;
use std::time::SystemTime;

use crate::SharedState;
use crate::helpers::helpers::{
    get_next_element,
    send_message_to_server
};

pub async fn handle_get_request<'a>(
    stream: Arc<Mutex<TcpStream>>, 
    lines: &'a mut std::str::Lines<'a>, 
    state: &'a Arc<Mutex<SharedState>>
) -> () {
    println!("Handling GET request");

    let mut stream = stream.lock().await;

    let key = get_next_element(lines);
    println!("Key: {}", key);
    let state = state.lock().await;
    match state.store.get(&key) {
        Some(full_value) => {
            let parts: Vec<&str> = full_value.split("\r\n").collect();
            let value = parts.get(0).unwrap_or(&"");
            
            let expire_time = parts.get(1).unwrap_or(&"");
            if *expire_time != "" {
                let current_time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                let expire_time_as_u64 = expire_time.parse::<u64>().unwrap();
                if current_time > expire_time_as_u64 {
                    let message = "$-1\r\n".to_string();
                    send_message_to_server(&mut stream, &message, false).await.unwrap();
                    return
                }
            }
            
            let len_of_value = value.len();
            let message = format!("${}\r\n{}\r\n", len_of_value, value);
            send_message_to_server(&mut stream, &message, false).await.unwrap();
            return
        }
        None => {
            let message = "$-1\r\n".to_string();
            send_message_to_server(&mut stream, &message, false).await.unwrap();
            return
        }
    }
}
