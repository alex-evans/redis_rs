use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpStream;

use crate::helpers::helpers::{
    get_next_element,
    send_message_to_server
};

pub async fn handle_echo_request<'a>(
    stream: Arc<Mutex<TcpStream>>, 
    lines: &'a mut std::str::Lines<'a>
) -> () {
    println!("Handling ECHO request");

    let mut stream = stream.lock().await;
    
    let echo_line = get_next_element(lines); // Skip the first line
    let len_of_echo_line = echo_line.len();
    let echo_response = format!("${}\r\n{}\r\n", len_of_echo_line, echo_line);
    
    send_message_to_server(&mut stream, &echo_response, true).await.unwrap();
    
    return
}