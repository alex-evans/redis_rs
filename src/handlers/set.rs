
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::SharedState;
use crate::helpers::helpers::get_next_element;

pub fn handle_set_request(lines: &mut std::str::Lines, state: &Arc<Mutex<SharedState>>, number_of_elements: i32) -> String {
    let mut state = state.lock().unwrap();
    let key = get_next_element(lines);
    let value = get_next_element(lines);

    if number_of_elements == 2 {
        state.store.insert(key, value);
        return "+OK\r\n".to_string();
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
                return "+OK\r\n".to_string();
        },
        _ => {
            state.store.insert(key, value);
            return "+OK\r\n".to_string();
        }
    }

}