
use std::sync::{Arc, Mutex};

use crate::SharedState;

pub fn handle_info_request(state: &Arc<Mutex<SharedState>>) -> String {
    let state = state.lock().unwrap();
    match state.store.get("replicaof") {
        Some(_full_value) => {
            // let parts: Vec<&str> = full_value.split("\r\n").collect();
            // let value = parts.get(0).unwrap_or(&"");
            return_slave_info()
        }
        None => return return_master_info()
    }
}

fn return_slave_info() -> String {
    let role: String = "slave".to_string();
    let response = format!("${}\r\nrole:{}\r\n", role.len() + 5, role);
    return response
}
    
fn return_master_info() -> String {
    let role: String = "master".to_string();
    let response = format!("${}\r\nrole:{}\r\n", role.len() + 5, role);
    return response;
}