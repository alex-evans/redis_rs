
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;

use crate::Config;
use crate::SharedState;
use crate::handlers::echo::handle_echo_request;
use crate::handlers::get::handle_get_request;
use crate::handlers::info::handle_info_request;
use crate::handlers::psync::handle_psync_request;
use crate::handlers::replconf::handle_replconf_request;
use crate::handlers::set::handle_set_request;

use crate::helpers::helpers::{
    determine_number_of_elements,
    get_next_element
};

pub fn list_request(_config_ref: &Config, request: &str, state: &Arc<Mutex<SharedState>>, stream: &mut TcpStream) -> () {
    println!("Received List Request: {}", request);
    let initial_response = _initial_request(request, state);
    let message = initial_response[0].to_string();
    
    println!("Sending message to client: {}", message);
    let _ = stream.write_all(message.as_bytes());    

    if initial_response[1] == "PSYNC" {
        let psync_message = "$0\r\n".to_string();
        println!("Sending PSYNC message to client: {}", psync_message);
        let _ = stream.write_all(psync_message.as_bytes());
    }
}

fn _initial_request(request: &str, state: &Arc<Mutex<SharedState>>) -> Vec<String> {
    let mut lines = request.lines();
    let first_line = lines.next().unwrap();
    let number_of_elements = determine_number_of_elements(&first_line);
    if number_of_elements < 0 {
        return vec!["-ERR Invalid request".to_string(), "ERROR".to_string()];
    } else {
        let element_one = get_next_element(&mut lines);
        match element_one.to_uppercase().as_str() {
            "ECHO" => return vec![handle_echo_request(&mut lines), "ECHO".to_string()],
            "PING" => return vec!["+PONG\r\n".to_string(), "PING".to_string()],
            "SET" => return vec![handle_set_request(&mut lines, &state, number_of_elements), "SET".to_string()],
            "GET" => return vec![handle_get_request(&mut lines, &state), "GET".to_string()],
            "INFO" => return vec![handle_info_request(&state), "INFO".to_string()],
            "REPLCONF" => return vec![handle_replconf_request(), "REPLCONF".to_string()], 
            "PSYNC" => return vec![handle_psync_request(), "PSYNC".to_string()],
            _ => return vec!["-ERR Invalid request".to_string(), "ERROR".to_string()]
        }
    }
}
