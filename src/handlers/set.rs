
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tokio::net::TcpStream;

use crate::SharedState;
use crate::helpers::helpers::{
    get_next_element,
    send_message_to_server
};

pub async fn handle_set_request<'a>(stream: Arc<Mutex<TcpStream>>, lines: &'a mut std::str::Lines<'a>, state: &'a Arc<Mutex<SharedState>>, number_of_elements: i32) -> () {
    let mut state = state.lock().await;
    let key = get_next_element(lines);
    let value = get_next_element(lines);

    if number_of_elements == 2 {
        state.store.insert(key, value);
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

                state.store.insert(key, format!("{}\r\n{}", value, expiration_time));
                let message = "+OK\r\n".to_string();
                send_message_to_server(stream, &message, false).await.unwrap();
                return
        },
        _ => {
            state.store.insert(key, value);
            let message = "+OK\r\n".to_string();
            send_message_to_server(stream, &message, false).await.unwrap();
            return
        }
    }

}