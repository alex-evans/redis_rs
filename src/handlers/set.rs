
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tokio::net::TcpStream;

use crate::SharedState;
use crate::helpers::helpers::{
    get_next_element,
    send_message_to_server
    // send_data_to_replica
};

pub async fn handle_set_request<'a>(
    stream: &'a mut TcpStream, 
    lines: &'a mut std::str::Lines<'a>, 
    state: &'a Arc<Mutex<SharedState>>, 
    number_of_elements: i32, 
    request: &str, 
) -> () {
    let mut state_guard = state.lock().await;
    let key = get_next_element(lines);
    let value = get_next_element(lines);

    if number_of_elements == 2 {
        state_guard.store.insert(key, value);
        let message = "+OK\r\n".to_string();
        send_message_to_server(stream, &message, false).await.unwrap();
        return
    }
    
    let sub_command: String = get_next_element(lines);
    let sub_value: String = get_next_element(lines);
    match sub_command.to_uppercase().as_str() {
        "PX" => {
            let expiration_duration = Duration::from_millis(sub_value.parse::<u64>().unwrap());
            let expiration_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
                + expiration_duration.as_millis() as u64;

                state_guard.store.insert(key, format!("{}\r\n{}", value, expiration_time));
                let message = "+OK\r\n".to_string();
                // send_message_to_server(stream, &message, false).await.unwrap();
                // send_message_to_server(stream, &request, false).await.unwrap();
                if let Err(e) = send_message_to_server(stream, &message, true).await {
                    eprintln!("Error sending message: {}", e);
                }
                if let Err(e) = send_message_to_server(stream, &request, true).await {
                    eprintln!("Error sending request: {}", e);
                }
                return
        },
        _ => {
            state_guard.store.insert(key, value);
            let message = "+OK\r\n".to_string();
            if let Err(e) = send_message_to_server(stream, &message, true).await {
                eprintln!("Error sending message: {}", e);
            }
            println!("Would be Sending request: {}", request);
            // if let Err(e) = send_message_to_server(stream, &request, true).await {
            //     eprintln!("Error sending request: {}", e);
            // }
            return
        }
    }

}