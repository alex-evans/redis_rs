use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpStream;

use crate::SharedState;
use crate::helpers::helpers::send_message_to_server;

pub async fn handle_info_request<'a>(
    stream: Arc<Mutex<TcpStream>>, 
    state_ref: &'a Arc<Mutex<SharedState>>
) -> () {
    println!("Handling INFO request");

    // Lock the state to determine the role
    let role = {
        let state = state_ref.lock().await;
        match state.store.get("replicaof") {
            Some(_full_value) => "slave".to_string(),
            None => "master".to_string(),
        }
    };

    println!("Role: {}", role);

    // Prepare the message outside of the lock
    let master_replid = "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb";
    let master_repl_offset = "0";
    let message = format!(
        "${}\r\nrole:{}\r\nmaster_replid:{}\r\nmaster_repl_offset:{}\r\n",
        role.len() + 5 + master_replid.len() + 13 + master_repl_offset.len() + 18 + 6,
        role,
        master_replid,
        master_repl_offset
    );

    // Lock the stream only to send the message
    {
        let mut stream = stream.lock().await;
        send_message_to_server(&mut stream, &message, false).await.unwrap();
    }
}