
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::helpers::helpers::{get_next_element, send_message_to_server};
use crate::SharedState;

pub async fn handle_replconf_request<'a>(
    stream: Arc<Mutex<TcpStream>>, 
    lines: &'a mut std::str::Lines<'a>,
    state: &'a Arc<Mutex<SharedState>>
) -> () {
    println!("Handling REPLCONF request");
    
    let sub_command: String = get_next_element(lines);
    // let sub_value: String = get_next_element(lines);

    if sub_command.to_uppercase() == "LISTENING-PORT" {
        let replca_host = "127.0.0.1".to_string();
        let replca_port = "6380".to_string();

        // Build a new stream to the replica
        let replica_stream = TcpStream::connect(format!("{}:{}", replca_host, replca_port)).await.unwrap();
        let replica_stream_arc = Arc::new(Mutex::new(replica_stream));
        
        // Store the wrapped stream in the shared state
        {
            let mut state_guard = state.lock().await;
            println!("REPLCONF - Storing stream in shared state");
            state_guard.stream = Some(replica_stream_arc.clone());
        }
    }

    // Lock the stream to send a message
    {
        let mut stream_lock = stream.lock().await;
        let message: String = "+OK\r\n".to_string();
        send_message_to_server(&mut stream_lock, &message, false).await.unwrap();
    }

    println!("REPLCONF - Successfully sent response to client");

    return

}