
pub fn handle_psync_request() -> String {
    println!("Handling PSYNC request");
    let response: String = "+FULLRESYNC 8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb 0\r\n".to_string();
    println!("Response: {}", response);
    return response;
}
    