use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use hex;
use std::fs;

use crate::helpers::helpers::send_message_to_server;

pub async fn handle_psync_request<'a>(
    stream: Arc<Mutex<TcpStream>>
) -> () {
    println!("Handling PSYNC request");

    // lock the stream for sending the initial response
    {
        let mut stream = stream.lock().await;
        let message: String = "+FULLRESYNC 8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb 0\r\n".to_string();
        send_message_to_server(&mut stream, &message, false).await.unwrap();
        println!("Finished handling PSYNC initial request");
    } 
    // release lock

    let file_path = "data/fake.rdb";
    if !std::path::Path::new(file_path).exists() {
        println!("RDB file not found at path: {}", file_path);
        return;
    }

    let file_contents = fs::read(file_path).unwrap();
    let rdb_file = hex::decode(&file_contents).unwrap();
    let rdb_length = rdb_file.len();
    
    // lock the stream for sending the RDB file
    {
        let mut stream = stream.lock().await;
        let message = format!("${}\r\n", rdb_length);
        send_message_to_server(&mut stream, &message, false).await.unwrap();
    
        stream.write_all(&rdb_file).await.unwrap();
        stream.flush().await.unwrap();
    } 
    // release lock

    println!("Finished sending PSYNC file");
}