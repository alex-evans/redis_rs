use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Logs from your program will appear here!");
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    
    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
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
                            let response_list = handle_list_request(request);
                            let response_string = response_list.join("\r\n");
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
        });
    }
}

fn handle_list_request(request: &str) -> Vec<&str> {
    println!("Received List Request: {}", request);
    let mut lines = request.lines();
    println!("Lines: {:?}", lines);
    // let num_elements = lines.next().unwrap().parse::<usize>().unwrap();

    // Attempt to parse the number of elements
    let num_elements = match lines.next().unwrap().parse::<usize>() {
        Ok(num) => num,
        Err(e) => {
            println!("Failed to parse number of elements: {}", e);
            return Vec::new();
        }
    };

    println!("Number of Elements: {}", num_elements);
    let mut elements = Vec::new();

    for _ in 0..num_elements {
        let element = lines.next().unwrap();
        elements.push(element);
    }

    println!("Received List: {:?}", elements);
    elements
}
 