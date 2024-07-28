
use std::sync::{Arc, Mutex};

use crate::Config;
use crate::SharedState;


pub fn store_replication(config_ref: &Config, _request: &str, state: &Arc<Mutex<SharedState>>) -> String {
    // Store Replication
    let mut state = state.lock().unwrap();
    state.store.insert("replicaof".to_string(), config_ref.replicaof.clone());
    return "+OK\r\n".to_string();
}