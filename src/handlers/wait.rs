use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, mpsc};
use tokio::net::TcpStream;
use tokio::time::timeout;
use futures::future::join_all;

use crate::SharedState;
use crate::helpers::helpers::{get_next_element, send_message_to_server};

pub async fn handle_wait_request<'a>(
    stream: Arc<Mutex<TcpStream>>,
    lines: &'a mut std::str::Lines<'a>,
    state: &'a Arc<Mutex<SharedState>>
) -> () {
    println!("Handling WAIT request");

    let min_number_of_acks = get_next_element(lines).parse::<usize>().unwrap_or(0);
    let time_limit_millisecs = get_next_element(lines).parse::<u64>().unwrap_or(0);
    let timeout_duration = Duration::from_millis(time_limit_millisecs);

    let repl_ack_msg = "*3\r\n$8\r\nREPLCONF\r\n$6\r\nGETACK\r\n$1\r\n*\r\n".to_string();

    let (tx, mut rx) = mpsc::channel(32);

    let ack_futures = {
        let state_guard = state.lock().await;
        state_guard.replicas.iter().map(|(_id, replica)| {
            let replica_stream = replica.stream.clone();
            let msg = repl_ack_msg.clone();
            let tx = tx.clone();
            
            tokio::spawn(async move {
                if let Err(err) = process_acks(replica_stream, &msg, &tx).await {
                    eprintln!("Failed to process acks: {}", err);
                }
            })
        }).collect::<Vec<_>>()
    };

    let ack_task = tokio::spawn(async move {
        join_all(ack_futures).await;
    });

    let mut number_of_acks = 0;
    let _wait_result = timeout(timeout_duration, async {
        println!("About to loop for acks {}", min_number_of_acks);

        while number_of_acks < min_number_of_acks {
            println!("Waiting for acks: {}/{}", number_of_acks, min_number_of_acks);
            if rx.recv().await.is_some() {
                println!("Received ack");
                number_of_acks += 1;
            } else {
                println!("No ack received");
                break;
            }
        }
    }).await;

    // Cancel the ack task if it's still running
    ack_task.abort();

    let wait_response = format!(":{}\r\n", number_of_acks);

    println!("Finished waiting for acks: {}", wait_response);
    let mut stream_lock = stream.lock().await;
    send_message_to_server(&mut stream_lock, &wait_response, false).await.unwrap();

    println!("Sent WAIT response to client");
    // Ok(())
}

async fn process_acks(
    stream: Arc<Mutex<TcpStream>>,
    msg: &str,
    tx: &mpsc::Sender<()>
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Sending REPLCONF GETACK message to replica");
    let mut stream_lock = stream.lock().await;
    if stream_lock.peer_addr().is_ok() {
        send_message_to_server(&mut stream_lock, msg, true).await?;
        println!("Received ack from replica");
        tx.send(()).await?;
        Ok(())
    } else {
        eprintln!("Failed to send REPLCONF GETACK message to replica");
        Err("Failed to send REPLCONF GETACK message to replica")?
    }
}