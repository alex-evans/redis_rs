
use std::sync::{Arc, Mutex};

use crate::SharedState;

pub fn handle_info_request(state: &Arc<Mutex<SharedState>>) -> String {
    let state = state.lock().unwrap();
    let master_replid: String = "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string();
    let master_repl_offset: String = "0".to_string();

    let role: String;
    match state.store.get("replicaof") {
        Some(_full_value) => {
            // let parts: Vec<&str> = full_value.split("\r\n").collect();
            // let value = parts.get(0).unwrap_or(&"");
            role = "slave".to_string()
        }
        None => role = "master".to_string()
    }

    let response: String = format!(
        "${}\r\nrole:{}\r\nmaster_replid:{}\r\nmaster_repl_offset:{}\r\n",
        role.len() + 5 + master_replid.len() + 15 + master_repl_offset.len() + 21,
        role,
        master_replid,
        master_repl_offset
    );
    return response;
}
