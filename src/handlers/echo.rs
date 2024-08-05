use tokio::net::TcpStream;

use crate::helpers::helpers::{
    get_next_element,
    send_message_to_server
};

pub async fn handle_echo_request<'a>(stream: &'a mut TcpStream, lines: &'a mut std::str::Lines<'a>) -> () {
    let echo_line = get_next_element(lines); // Skip the first line
    let len_of_echo_line = echo_line.len();
    let echo_response = format!("${}\r\n{}\r\n", len_of_echo_line, echo_line);
    send_message_to_server(stream, &echo_response).await.unwrap();
    return
}