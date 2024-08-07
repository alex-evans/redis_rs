
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

pub fn determine_number_of_elements(line: &str) -> i32 {
    let characters: String = line.chars().skip(1).collect();
    let characters_as_int: Result<i32, _> = characters.parse();
    if let Ok(num) = characters_as_int {
        println!("Characters as integer: {}", num);
        return num;
    } else {
        println!("Failed to convert characters to integer");
        return -1;
    }
}

pub fn get_next_element(lines: &mut std::str::Lines) -> String {
    let _skip_line = lines.next().unwrap_or("");
    let return_line = lines.next().unwrap_or("");
    return return_line.to_string();
}

pub async fn send_message_to_server(stream: &mut TcpStream, message: &str) -> Result<String, std::io::Error> {
    println!("Sending message to server: {}", message);
    AsyncWriteExt::write_all(stream, message.as_bytes()).await?;

    // let mut reader = BufReader::new(stream);
    // let mut response = String::new();
    // AsyncBufReadExt::read_line(&mut reader, &mut response).await?;
    // println!("Received response from server: {}", response);
    // Ok(response)
    Ok("".to_string())
}