
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

    let key = get_next_element(lines);
    println!("Key: {}", key);

    // lock the state to get the value
    let value_info = {
        let state = state.lock().await;
        state.store.get(&key).cloned()
    };
    println!("Value info: {:?}", value_info);
    let message = if let Some(full_value) = value_info {
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
                "$-1\r\n".to_string()
            } else {
                let len_of_value = value.len();
                format!("${}\r\n{}\r\n", len_of_value, value)
            }
 
        } else {
            let len_of_value = value.len();
            format!("${}\r\n{}\r\n", len_of_value, value)
        }
 
    } else {
        "$-1\r\n".to_string()
    };

    // Lock the stream to send the message
    {
        let mut stream = stream.lock().await;
        send_message_to_server(&mut stream, &message, false).await.unwrap();
    }

}