use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use std::env;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
struct ShareRequest {
    share: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ShareResponse {
    result_share: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let port = args.get(1).expect("Port number required").clone(); // <- FIXED

    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Server listening on {}", addr);

    loop {
        let port_clone = port.clone(); // For logging inside the task
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];
            match socket.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    let req: ShareRequest = bincode::deserialize(&buf[..n]).unwrap();
                    println!("Server on port {} received share: {}", port_clone, req.share);

                    let squared = req.share * req.share;
                    let response = ShareResponse { result_share: squared };
                    let encoded = bincode::serialize(&response).unwrap();
                    socket.write_all(&encoded).await.unwrap();
                }
                _ => println!("Connection closed or failed."),
            }
        });
    }
}