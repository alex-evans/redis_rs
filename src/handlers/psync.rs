
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
    
    let mut stream = stream.lock().await;

    let message: String = "+FULLRESYNC 8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb 0\r\n".to_string();
    send_message_to_server(&mut stream, &message, true).await.unwrap();
    println!("Finished handling PSYNC initial request");

    let file_path = "data/fake.rdb";
    let file_contents = fs::read(file_path).unwrap();
    let empty_rdb = hex::decode(&file_contents).unwrap();
    let empty_rdb_length = empty_rdb.len();
    let message = format!("${}\r\n", empty_rdb_length);
    send_message_to_server(&mut stream, &message, true).await.unwrap();
    
    {
        stream.write_all(&empty_rdb).await.unwrap();
    }

    println!("Finished sending PSYNC file");
    return
}
    