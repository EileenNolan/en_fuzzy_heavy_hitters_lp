// use tokio::net::TcpListener;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use serde::{Serialize, Deserialize};
// use std::env;
// use std::error::Error;
// use en_fuzzy_heavy_hitters_lp::field::FieldElm;

// #[derive(Serialize, Deserialize, Debug)]
// struct ShareRequest {
//     share: FieldElm,
// }

// #[derive(Serialize, Deserialize, Debug)]
// struct ShareResponse {
//     result_share: FieldElm,
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let args: Vec<String> = env::args().collect();
//     let port = args.get(1).expect("Port number required").clone();

//     let addr = format!("127.0.0.1:{}", port);
//     let listener = TcpListener::bind(&addr).await?;
//     println!("Server listening on {}", addr);

//     loop {
//         let port_clone = port.clone();
//         let (mut socket, _) = listener.accept().await?;
//         tokio::spawn(async move {
//             let mut buf = vec![0u8; 1024];
//             match socket.read(&mut buf).await {
//                 Ok(n) if n > 0 => {
//                     let req: ShareRequest = bincode::deserialize(&buf[..n]).unwrap();
//                     println!("Server on port {} received share: {:?}", port_clone, req.share);

//                     let squared = req.share * req.share;
//                     let response = ShareResponse { result_share: squared };
//                     let encoded = bincode::serialize(&response).unwrap();
//                     socket.write_all(&encoded).await.unwrap();
//                 }
//                 _ => println!("Connection closed or failed."),
//             }
//         });
//     }
// }
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use std::env;
use std::error::Error;
use en_fuzzy_heavy_hitters_lp::field::FieldElm;

#[derive(Serialize, Deserialize, Debug)]
struct ShareRequest {
    share: FieldElm,
}

#[derive(Serialize, Deserialize, Debug)]
struct ShareResponse {
    share: FieldElm,
}

async fn connect_to_peer(peer_port: String, my_share: FieldElm) -> Result<FieldElm, Box<dyn Error>> {
    let addr = format!("127.0.0.1:{}", peer_port);
    let mut stream = TcpStream::connect(&addr).await?;
    println!("Connected to peer at {}", addr);

    let request = ShareRequest { share: my_share };
    let encoded = bincode::serialize(&request)?;
    stream.write_all(&encoded).await?;

    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await?;
    let response: ShareResponse = bincode::deserialize(&buf[..n])?;

    Ok(response.share)
}

async fn handle_incoming(
    mut socket: TcpStream,
    my_port: String,
    peer_port: String,
) -> Result<(), Box<dyn Error>> {
    let mut buf = vec![0u8; 1024];
    let n = socket.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }

    let req: ShareRequest = bincode::deserialize(&buf[..n])?;
    println!("Server on port {} received share: {:?}", my_port, req.share);

    // Connect to peer and get their share
    let peer_share = connect_to_peer(peer_port.clone(), req.share).await?;

    // Convert both shares to u64
    let my_val = req.share.value as u64;
    let peer_val = peer_share.value as u64;

    println!(
        "Server {} values: mine = {}, peer = {}",
        my_port, my_val, peer_val
    );

    // Compute (my_val - peer_val) <= 0
    let leq = (my_val as i64 - peer_val as i64) <= 0;
    println!(
        "Comparison result on port {}: {} - {} <= 0 ? {}",
        my_port, my_val, peer_val, leq
    );

    // Send result back to original client (just dummy result for now)
    let result = if leq {
        FieldElm::from(1u64)
    } else {
        FieldElm::from(0u64)
    };

    let response = ShareResponse { share: result };
    let encoded = bincode::serialize(&response)?;
    socket.write_all(&encoded).await?;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <my_port> <peer_port>", args[0]);
        std::process::exit(1);
    }

    let my_port = args[1].clone();
    let peer_port = args[2].clone();
    let addr = format!("127.0.0.1:{}", my_port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Server listening on {}", addr);

    tokio::task::LocalSet::new()
    .run_until(async move {
        loop {
            let accept_result = listener.accept().await;
            match accept_result {
                Ok((socket, _)) => {
                    let my_port_clone = my_port.clone();
                    let peer_port_clone = peer_port.clone();
                    tokio::task::spawn_local(async move {
                        if let Err(e) = handle_incoming(socket, my_port_clone, peer_port_clone).await {
                            eprintln!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    })
    .await;

    Ok(())
}