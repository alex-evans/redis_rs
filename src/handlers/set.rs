
use std::str;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tokio::net::TcpStream;

use crate::SharedState;
use crate::helpers::helpers::{
    get_next_element,
    send_message_to_server,
    send_data_to_replica
};

pub async fn handle_set_request<'a>(
    stream: Arc<Mutex<TcpStream>>, 
    lines: &'a mut std::str::Lines<'a>, 
    state: &'a Arc<Mutex<SharedState>>, 
    number_of_elements: i32, 
    _request: &str, 
) -> () {
    println!("Handling SET request");

    let key = get_next_element(lines);
    let value = get_next_element(lines);
    let repl_command = format!(
        "*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
        key.len(),
        key,
        value.len(),
        value
    );

    if number_of_elements == 2 {
        {
            let mut stream_lock = stream.lock().await;
            let message = "+OK\r\n".to_string();
            send_message_to_server(&mut stream_lock, &message, true).await.unwrap();
        }

        let stored_stream_option = {
            let mut state_guard = state.lock().await;
            state_guard.store.insert(key.clone(), value.clone());
            state_guard.stream.clone() // Clone the Arc to release the lock
        };

        // Use the stored stream from the state
        if let Some(stored_stream) = stored_stream_option {
            println!("Attempting to lock the stored stream...");
            let mut stored_stream_lock = stored_stream.lock().await;
            println!("Successfully locked the stored stream");
            send_data_to_replica(&mut stored_stream_lock, &repl_command).await;
        } else {
            println!("WARNING - No stored stream found in state");
        }

        return;
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
            
            {
                let mut stream_lock = stream.lock().await;
                let message = "+OK\r\n".to_string();
                send_message_to_server(&mut stream_lock, &message, true).await.unwrap();
            }

            let stored_stream_option = {
                let mut state_guard = state.lock().await;
                state_guard.store.insert(key.clone(), format!("{}\r\n{}", value, expiration_time));
                state_guard.stream.clone() // Clone the Arc to release the lock
            };
            
            // Use the stored stream from the state
            if let Some(stored_stream) = stored_stream_option {
                println!("Attempting to lock the stored stream...");
                let mut stored_stream_lock = stored_stream.lock().await;
                println!("Successfully locked the stored stream");
                send_data_to_replica(&mut stored_stream_lock, &repl_command).await;
            } else {
                println!("WARNING - No stored stream found in state");
            }

            return;
        },
        _ => {
            {
                let mut stream_lock = stream.lock().await;
                let message = "+OK\r\n".to_string();
                send_message_to_server(&mut stream_lock, &message, true).await.unwrap();
            } // Release the lock on the stream

            let stored_stream_option = {
                let mut state_guard = state.lock().await;
                state_guard.store.insert(key.clone(), value.clone());
                state_guard.stream.clone() // Clone the Arc to release the lock
            };

            // Use the stored stream from the state
            if let Some(stored_stream) = stored_stream_option {
                println!("Attempting to lock the stored stream...");
                let mut stored_stream_lock = stored_stream.lock().await;
                println!("Successfully locked the stored stream");
                send_data_to_replica(&mut stored_stream_lock, &repl_command).await;
            } else {
                println!("WARNING - No stored stream found in state");
            }
            
            return;
        }
    }

}