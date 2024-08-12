
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpStream;

use crate::SharedState;
use crate::handlers::echo::handle_echo_request;
use crate::handlers::get::handle_get_request;
use crate::handlers::info::handle_info_request;
use crate::handlers::ping::handle_ping_request;
use crate::handlers::psync::handle_psync_request;
use crate::handlers::replconf::handle_replconf_request;
use crate::handlers::set::handle_set_request;
 
use crate::helpers::helpers::{
    determine_number_of_elements,
    get_next_element
};

pub async fn list_request(request: &str, state: &Arc<Mutex<SharedState>>, stream: &mut TcpStream) -> () {
    println!("Received List Request: {}", request);
    let mut lines = request.lines();
    let first_line = lines.next().unwrap();
    let number_of_elements = determine_number_of_elements(&first_line);
    if number_of_elements < 0 {
        println!("-ERR Invalid request");
        return
    } 
    let element_one = get_next_element(&mut lines);
    match element_one.to_uppercase().as_str() {
        "ECHO" => handle_echo_request(stream, &mut lines).await,
        "PING" => handle_ping_request(stream).await,
        "SET" => handle_set_request(stream, &mut lines, state, number_of_elements, request).await,
        "GET" => handle_get_request(stream, &mut lines, state).await,
        "INFO" => handle_info_request(stream, state).await,
        "REPLCONF" => handle_replconf_request(stream).await, 
        "PSYNC" => handle_psync_request(stream).await,
        _ => println!("-ERR Invalid request")
    };

    return
}

