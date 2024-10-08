
use std::str;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tokio::net::TcpStream;

use crate::SharedState;
use crate::helpers::helpers::{
    get_next_element, 
    is_replica, 
    send_message_to_server,
};

pub async fn handle_set_request<'a>(
    stream: Arc<Mutex<TcpStream>>, 
    lines: &'a mut std::str::Lines<'a>, 
    state: &'a Arc<Mutex<SharedState>>, 
    number_of_elements: i32, 
    _request: &str, 
) -> () {
    println!("Handling SET request");

    // reset ack count
    {
        let mut state_lock = state.lock().await;
        state_lock.store.insert("replica_ack_count".to_string(), "0".to_string());
    }

    let replica = is_replica(state).await;

    let key = get_next_element(lines);
    let value = get_next_element(lines);

    if number_of_elements == 2 {
        store_key_value(state, &key, &value).await;
        if !replica {
            send_replica_message(state, &key, &value).await;
            send_ok_response(stream).await;
        };
        return;
    }

    let sub_command: String = get_next_element(lines);
    let sub_value: String = get_next_element(lines);

    match sub_command.to_uppercase().as_str() {
        "PX" => {
            if let Ok(expiration_duration) = sub_value.parse::<u64>() {
                let expiration_time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
                    + expiration_duration;

                let value_with_expiration = format!("{}\r\n{}", value, expiration_time);
                store_key_value(state, &key, &value_with_expiration).await;
                if !replica {
                    send_replica_message(state, &key, &value).await;
                    send_ok_response(stream).await;
                };
            } else {
                eprintln!("Failed to parse expiration duration: {}", sub_value);
            }
        },
        _ => {
            println!("Storing key-value pair in state");
            store_key_value(state, &key, &value).await;
            if !replica {
                send_replica_message(state, &key, &value).await;
                send_ok_response(stream).await;
            };
            println!("Successfully stored key-value pair in state");
        }
    }
}

async fn store_key_value(state: &Arc<Mutex<SharedState>>, key: &str, value: &str) {
    let mut state_guard = state.lock().await;
    state_guard.store.insert(key.to_string(), value.to_string());
}

async fn send_ok_response(stream: Arc<Mutex<TcpStream>>) {
    let message = "+OK\r\n".to_string();
    let mut stream_lock = stream.lock().await;
    if let Err(e) = send_message_to_server(&mut stream_lock, &message, false).await {
        eprintln!("Failed to send response to client: {}", e);
    }
}

async fn send_replica_message(state: &Arc<Mutex<SharedState>>, key: &str, value: &str) {
    println!("Sending SET message to replicas");
    let repl_command = format!(
        "*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
        key.len(),
        key,
        value.len(),
        value
    );
    // send SET message to all replicas
    {
        let state_guard = state.lock().await;
        for (id, replica) in &state_guard.replicas {
            let mut stream = replica.stream.lock().await;
            println!("Sending SET message to replica {}", id);
            if let Err(e) = send_message_to_server(&mut stream, &repl_command, false).await {
                eprintln!("Failed to send message to replica {}: {}", id, e);
            };
            println!("Sent SET message to replica {}", id);
        }
    }
    
    println!("Finished sending SET message to replicas - Now ACKing");

}
