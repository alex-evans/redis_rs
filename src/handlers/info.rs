
use std::sync::{Arc, Mutex};

use crate::SharedState;

pub fn handle_info_request(state_ref: &Arc<Mutex<SharedState>>) -> String {
    println!("Handling INFO request");
    let state = state_ref.lock().unwrap();
    let master_replid: String = "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string();
    let master_repl_offset: String = "0".to_string();

    let role: String;
    match state.store.get("replicaof") {
        Some(_full_value) => {
            role = "slave".to_string()
        }
        None => role = "master".to_string()
    }

    println!("Role: {}", role);

    let response: String = format!(
        "${}\r\nrole:{}\r\nmaster_replid:{}\r\nmaster_repl_offset:{}\r\n",
        role.len() + 5 + master_replid.len() + 13 + master_repl_offset.len() + 18 + 6,  // Added 6 at the end for the \r\n bytes
        role,
        master_replid,
        master_repl_offset
    );

    println!("Response: {}", response);
    return response;
}
