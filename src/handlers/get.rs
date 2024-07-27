
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::SharedState;
use crate::helpers::helpers::get_next_element;

pub fn handle_get_request(lines: &mut std::str::Lines, state: &Arc<Mutex<SharedState>>) -> String {
    let key = get_next_element(lines);
    println!("Key: {}", key);
    let state = state.lock().unwrap();
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
                    return "$-1\r\n".to_string();
                }
            }
            
            let len_of_value = value.len();
            let response = format!("${}\r\n{}\r\n", len_of_value, value);
            return response;
        }
        None => return "$-1\r\n".to_string()
    }
}