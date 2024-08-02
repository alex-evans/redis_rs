
pub fn handle_replconf_request() -> String {
    println!("Handling REPLCONF request");
    let response: String = "+OK\r\n".to_string();
    println!("Response: {}", response);
    return response;
}