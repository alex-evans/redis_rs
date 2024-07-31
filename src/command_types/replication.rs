
use std::sync::{Arc, Mutex};

use crate::Config;
use crate::SharedState;

use crate::handlers::info::handle_info_request;


pub fn store_replication(config_ref: &Config, _request: &str, ref_state: &Arc<Mutex<SharedState>>) -> String {
    let mut state = ref_state.lock().unwrap();
    state.store.insert("replicaof".to_string(), config_ref.replicaof.clone());
    return handle_info_request(&ref_state);
}