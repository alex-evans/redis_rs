
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::helpers::helpers::{
    // get_next_element,
    send_message_to_server
};
use crate::SharedState;

pub async fn handle_replconf_request<'a>(
    stream: Arc<Mutex<TcpStream>>, 
    _lines: &'a mut std::str::Lines<'a>,
    state: &'a Arc<Mutex<SharedState>>
) -> () {
    println!("Handling REPLCONF request");
    
    // Wrap the stream in an Arc<Mutex<>> for shared access
    // let stream_arc = Arc::new(Mutex::new(stream));

    // Store the wrapped stream in the shared state
    {
        let mut state_guard = state.lock().await;
        state_guard.stream = Some(stream.clone());
    }

    let message: String = "+OK\r\n".to_string();
    // Lock the stream to send a message
    {
        let mut stream_lock = stream.lock().await;
        send_message_to_server(&mut *stream_lock, &message, false).await.unwrap();
    }

    return

}