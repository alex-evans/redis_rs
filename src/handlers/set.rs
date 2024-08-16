
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
    println!("ADE - We here 1");

    if number_of_elements == 2 {
        println!("ADE - We be here 1");
        {
            let mut state_guard = state.lock().await;
            state_guard.store.insert(key, value);
        }

        println!("ADE - We be here 2");
        // Use the stored stream from the state
        if let Some(stored_stream) = &state.lock().await.stream {
            let mut stored_stream_lock = stored_stream.lock().await;
            send_data_to_replica(&mut stored_stream_lock, state, &repl_command).await;
        } else {
            println!("WARNING - No stored stream found in state");
        }
        println!("ADE - We be here 3");

        let mut stream_lock = stream.lock().await;
        let message = "+OK\r\n".to_string();
        send_message_to_server(&mut stream_lock, &message, true).await.unwrap();
        println!("ADE - We be here 4");

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
                    let mut state_guard = state.lock().await;
                    state_guard.store.insert(key, format!("{}\r\n{}", value, expiration_time));
                }

                // Use the stored stream from the state
                if let Some(stored_stream) = &state.lock().await.stream {
                    let mut stored_stream_lock = stored_stream.lock().await;
                    send_data_to_replica(&mut stored_stream_lock, state, &repl_command).await;
                } else {
                    println!("WARNING - No stored stream found in state");
                }

                let mut stream_lock = stream.lock().await;
                let message = "+OK\r\n".to_string();
                send_message_to_server(&mut stream_lock, &message, true).await.unwrap();

                return;
        },
        _ => {
            println!("ADE - Dude We be here 1");
            {
                let mut state_guard = state.lock().await;
                state_guard.store.insert(key, value);
            }
            println!("ADE - Dude We be here 2");
            
            // Use the stored stream from the state
            if let Some(stored_stream) = &state.lock().await.stream {
                let mut stored_stream_lock = stored_stream.lock().await;
                send_data_to_replica(&mut stored_stream_lock, state, &repl_command).await;
            } else {
                println!("WARNING - No stored stream found in state");
            }

            println!("ADE - Dude We be here 3");
            let mut stream_lock = stream.lock().await;
            let message = "+OK\r\n".to_string();
            send_message_to_server(&mut stream_lock, &message, true).await.unwrap();
            
            println!("ADE - Dude We be here 4");
            return;
        }
    }

}