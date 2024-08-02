
use std::sync::{Arc, Mutex};

use crate::Config;
use crate::SharedState;
use crate::handlers::echo::handle_echo_request;
use crate::handlers::get::handle_get_request;
use crate::handlers::info::handle_info_request;
use crate::handlers::set::handle_set_request;

use crate::helpers::helpers::{
    determine_number_of_elements,
    get_next_element
};


pub fn list_request(_config_ref: &Config, request: &str, state: &Arc<Mutex<SharedState>>) -> String {
    println!("Received List Request: {}", request);
    let mut lines = request.lines();
    let first_line = lines.next().unwrap();
    let number_of_elements = determine_number_of_elements(&first_line);
    if number_of_elements < 0 {
        return "-ERR Invalid request".to_string();
    } else {
        let element_one = get_next_element(&mut lines);
        match element_one.to_uppercase().as_str() {
            "ECHO" => return handle_echo_request(&mut lines),
            "PING" => return "+PONG\r\n".to_string(),
            "SET" => return handle_set_request(&mut lines, &state, number_of_elements),
            "GET" => return handle_get_request(&mut lines, &state),
            "INFO" => return handle_info_request(&state),
            "REPLCONF" => return "+OK\r\n".to_string(), 
            _ => return "-ERR Invalid request".to_string()
        }
    }
}

