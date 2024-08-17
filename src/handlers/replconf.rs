
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::helpers::helpers::send_message_to_server;
use crate::SharedState;

pub async fn handle_replconf_request<'a>(
    stream: Arc<Mutex<TcpStream>>, 
    _lines: &'a mut std::str::Lines<'a>,
    state: &'a Arc<Mutex<SharedState>>
) -> () {
    println!("Handling REPLCONF request");
    
    // Store the wrapped stream in the shared state
    {
        let mut state_guard = state.lock().await;
        println!("REPLCONF - Storing stream in shared state");
        println!("REPLCONF - Stream: {:?}", stream);
        state_guard.stream = Some(stream.clone());
    }

    let message: String = "+OK\r\n".to_string();
    // Lock the stream to send a message
    {
        println!("REPLCONF - Attempting to lock the stream...");
        let mut stream_lock = stream.lock().await;
        println!("REPLCONF - Successfully locked the stream");
        send_message_to_server(&mut *stream_lock, &message, true).await.unwrap();
    }

    return

}