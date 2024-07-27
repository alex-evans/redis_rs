use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::time::{Duration, SystemTime};
use std::env;

struct SharedState {
    store: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Logs from your program will appear here!");
    // ...

    let args: Vec<String> = env::args().collect();
    let mut port = String::from("6379");

    for i in 1..args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            port = args[i + 1].clone();
        }
    }

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

    let state = Arc::new(Mutex::new(SharedState {
        store: HashMap::new(),
    }));

    loop {
        let (socket, _) = listener.accept().await?;
        let state_clone = state.clone();
        tokio::spawn(async move {
                handle_connection(socket, state_clone).await;
        });
    }
}

async fn handle_connection(mut socket: tokio::net::TcpStream, state: Arc<Mutex<SharedState>>) {
    let mut buf = [0; 1024];

    loop {
        match socket.read(&mut buf).await {
            Ok(0) => {
                println!("Connection closed");
                return;
            }
            Ok(_) => {
                let request = std::str::from_utf8(&buf).unwrap();

                if request.starts_with('+') {
                    // Means it's a Simple String
                    let response = &request[1..request.len()-2];
                    println!("Received Simple String: {}", response);
                    // Process the response here
                } else if request.starts_with('-') {
                    // Means it's an Error
                    let error = &request[1..request.len()-2];
                    println!("Received Error: {}", error);
                    // Process the error here
                } else if request.starts_with(':') {
                    // Means it's an Integer
                    let integer = &request[1..request.len()-2];
                    println!("Received Integer: {}", integer);
                    // Process the integer here
                } else if request.starts_with('$') {
                    // Means it's a Bulk String
                    let bulk_string = &request[1..request.len()-2];
                    println!("Received Bulk String: {}", bulk_string);
                    // Process the bulk string here
                } else if request.starts_with('*') {
                    // Means it's a List
                    let response_string = handle_list_request(&request, &state);
                    let response_bytes = response_string.as_bytes().try_into().unwrap();
                    if let Err(e) = socket.write_all(response_bytes).await {
                        println!("Failed to write to connection: {}", e);
                        return;
                    }
                } else {
                    println!("Unknown request format");
                }
                buf.fill(0);  // Clear the buffer
            }
            Err(e) => {
                println!("Failed to read from connection: {}", e);
                return;
            }
        }
    }
}

fn handle_list_request(request: &str, state: &Arc<Mutex<SharedState>>) -> String {
    println!("Received List Request: {}", request);
    let mut lines = request.lines();
    let first_line = lines.next().unwrap();
    let number_of_elements = determine_number_of_elements(&first_line);
    if number_of_elements < 0 {
        return "-ERR Invalid request".to_string();
    } else {
        let element_one = get_next_element(&mut lines);
        match element_one.to_uppercase().as_str() {
            "ECHO" => return build_echo_response(&mut lines),
            "PING" => return "+PONG\r\n".to_string(),
            "SET" => return handle_set_request(&mut lines, &state, number_of_elements),
            "GET" => return handle_get_request(&mut lines, &state),
            _ => return "-ERR Invalid request".to_string()
        }
    }
}

fn determine_number_of_elements(line: &str) -> i32 {
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

fn get_next_element(lines: &mut std::str::Lines) -> String {
    let _skip_line = lines.next().unwrap_or("");
    let return_line = lines.next().unwrap_or("");
    return return_line.to_string();
}

fn build_echo_response(lines: &mut std::str::Lines) -> String {
    let echo_line = get_next_element(lines); // Skip the first line
    println!("Echo Line: {}", echo_line);
    let len_of_echo_line = echo_line.len();
    println!("Length of Echo Line: {}", len_of_echo_line);
    let echo_response = format!("${}\r\n{}\r\n", len_of_echo_line, echo_line);
    println!("Echo Response: {}", echo_response);
    return echo_response;
}

fn handle_set_request(lines: &mut std::str::Lines, state: &Arc<Mutex<SharedState>>, number_of_elements: i32) -> String {
    let mut state = state.lock().unwrap();
    let key = get_next_element(lines);
    let value = get_next_element(lines);

    if number_of_elements == 2 {
        state.store.insert(key, value);
        return "+OK\r\n".to_string();
    }
    
    let sub_command: String = get_next_element(lines);
    let sub_value: String = get_next_element(lines);
    match sub_command.to_uppercase().as_str() {
        "PX" => {
            let expiration_duration = Duration::from_millis(sub_value.parse::<u64>().unwrap());
            let expiration_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
                + expiration_duration.as_millis() as u64;

                state.store.insert(key, format!("{}\r\n{}", value, expiration_time));
                return "+OK\r\n".to_string();
        },
        _ => {
            state.store.insert(key, value);
            return "+OK\r\n".to_string();
        }
    }

}

fn handle_get_request(lines: &mut std::str::Lines, state: &Arc<Mutex<SharedState>>) -> String {
    let key = get_next_element(lines);
    println!("Key: {}", key);
    let state = state.lock().unwrap();
    match state.store.get(&key) {
        Some(full_value) => {
            let parts: Vec<&str> = full_value.split("\r\n").collect();
            let value = parts.get(0).unwrap_or(&"");
            
            let expire_time = parts.get(1).unwrap_or(&"");
            if *expire_time != "" {
                let current_time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                println!("Current Time: {}", current_time);
                println!("Expire Time: {}", expire_time);

                let expire_time_as_u64 = expire_time.parse::<u64>().unwrap();
                if current_time > expire_time_as_u64 {
                    return "$-1\r\n".to_string();
                }
            }
            
            let len_of_value = value.len();
            let response = format!("${}\r\n{}\r\n", len_of_value, value);
            return response;
        }
        None => return "$-1\r\n".to_string()
    }
}