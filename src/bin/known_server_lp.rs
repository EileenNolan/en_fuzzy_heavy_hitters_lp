// dummy code with no mpc

// use tokio::net::{TcpListener, TcpStream};
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
//     share: FieldElm,
// }

// // This function connects to the peer server and sends it a share.
// async fn connect_to_peer(peer_port: String, my_share: FieldElm) -> Result<FieldElm, Box<dyn Error>> {
//     let addr = format!("127.0.0.1:{}", peer_port);
//     let mut stream = TcpStream::connect(&addr).await?;
//     println!("Server (port {}): Connected to peer at {}", peer_port, addr);

//     let request = ShareRequest { share: my_share };
//     let encoded = bincode::serialize(&request)?;
//     stream.write_all(&encoded).await?;
//     println!("Server (port {}): Sent share to peer.", peer_port);

//     let mut buf = vec![0u8; 1024];
//     let n = stream.read(&mut buf).await?;
//     let response: ShareResponse = bincode::deserialize(&buf[..n])?;
//     println!("Server (port {}): Received response from peer: {:?}", peer_port, response.share);

//     Ok(response.share)
// }

// // This function handles an incoming connection from the leader.
// async fn handle_incoming(
//     mut socket: TcpStream,
//     my_port: String,
//     peer_port: String,
// ) -> Result<(), Box<dyn Error>> {
//     let mut buf = vec![0u8; 1024];
//     let n = socket.read(&mut buf).await?;
//     if n == 0 {
//         println!("Server (port {}): Received empty message.", my_port);
//         return Ok(());
//     }

//     let req: ShareRequest = bincode::deserialize(&buf[..n])?;
//     println!("Server (port {}): Received share from leader: {:?}", my_port, req.share);

//     // Connect to the peer server and obtain its share.
//     let peer_share = connect_to_peer(peer_port.clone(), req.share).await?;

//     // Convert share values to numeric form for a dummy comparison.
//     let my_val = req.share.value as u64;
//     let peer_val = peer_share.value as u64;
//     println!("Server (port {}): Comparison values: mine = {}, peer = {}", my_port, my_val, peer_val);

//     // Dummy computation: check if (my_val - peer_val) is <= 0.
//     let leq = (my_val as i64 - peer_val as i64) <= 0;
//     println!("Server (port {}): Computation: {} - {} <= 0 ? {}", my_port, my_val, peer_val, leq);

//     // Prepare the response share.
//     let result = if leq {
//         FieldElm::from(1u64)
//     } else {
//         FieldElm::from(0u64)
//     };

//     let response = ShareResponse { share: result };
//     let encoded = bincode::serialize(&response)?;
//     socket.write_all(&encoded).await?;
//     println!("Server (port {}): Sent response back to leader.", my_port);

//     Ok(())
// }

// #[tokio::main(flavor = "current_thread")]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let args: Vec<String> = env::args().collect();
//     if args.len() < 3 {
//         eprintln!("Usage: {} <my_port> <peer_port>", args[0]);
//         std::process::exit(1);
//     }

//     let my_port = args[1].clone();
//     let peer_port = args[2].clone();
//     let addr = format!("127.0.0.1:{}", my_port);
//     let listener = TcpListener::bind(&addr).await?;
//     println!("Server (port {}): Listening on {}", my_port, addr);

//     // Process exactly one connection.
//     if let Ok((socket, peer_addr)) = listener.accept().await {
//         println!("Server (port {}): Accepted connection from {}", my_port, peer_addr);
//         // Process the connection inline (without using spawn_local)
//         if let Err(e) = handle_incoming(socket, my_port.clone(), peer_port.clone()).await {
//             eprintln!("Server (port {}): Error handling connection: {}", my_port, e);
//         }
//     } else {
//         eprintln!("Server (port {}): Failed to accept connection.", my_port);
//     }
//     println!("Server (port {}): Exiting after processing one connection.", my_port);

//     Ok(())
// }

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use std::env;
use std::error::Error;
pub use scuttlebutt::{AesRng, Channel};
use en_fuzzy_heavy_hitters_lp::field::FieldElm;
use std::io::Read;
use std::io::Write;
use std::panic;


use std::thread; // for sleep
use std::time::Duration; // for Duration::from_millis


// Import the MPC functions.
use en_fuzzy_heavy_hitters_lp::sum_leq_binary::{gb_sum, ev_sum}; // adjust path as needed

#[derive(Serialize, Deserialize, Debug)]
struct ShareRequest {
    share: FieldElm,
}

#[derive(Serialize, Deserialize, Debug)]
struct ShareResponse {
    share: FieldElm,
}

async fn handle_incoming(
    mut socket: TcpStream,
    my_port: String,
    peer_port: String,
) -> Result<(), Box<dyn Error>> {
    // Read the leader’s share.
    let mut buf = vec![0u8; 1024];
    let n = socket.read(&mut buf).await?;
    if n == 0 {
        println!("Server (port {}): Received empty message.", my_port);
        return Ok(());
    }
    let req: ShareRequest = bincode::deserialize(&buf[..n])?;
    println!("Server (port {}): Received share from leader: {:?}", my_port, req.share);

    // Parse your port numbers.
    let my_port_num: u32 = my_port.parse().unwrap();
    let peer_port_num: u32 = peer_port.parse().unwrap();
    // We decide: the server with the higher port becomes the evaluator.
    let is_evaluator = my_port_num > peer_port_num;
    println!(
        "Server (port {}): MPC role: {}",
        my_port,
        if is_evaluator { "evaluator" } else { "garbler" }
    );


    // Our MPC input is the share received from the leader.
    let mpc_input: u128 = req.share.value as u128;

    // Use an offset (e.g., 1000) to derive the dedicated MPC port.
    const MPC_OFFSET: u32 = 1000;
    let my_port_closure = my_port.clone();

   // Define an offset for the MPC dedicated port

// Spawn a blocking task to run the MPC protocol.
    let mpc_result: u128 = tokio::task::spawn_blocking(move || {
        // We'll use our local copy of the server port in the closure.
        // (my_port_closure was defined earlier as my_port.clone())
        if !is_evaluator {
            // This is the garbler branch.
            let mpc_addr = format!("127.0.0.1:{}", my_port_num + MPC_OFFSET);
            println!(
                "Server (port {}): MPC garbler: Listening on {}",
                my_port_closure, mpc_addr
            );
            let listener = std::net::TcpListener::bind(&mpc_addr).unwrap();
            let (std_stream, peer_addr) = listener.accept().unwrap();
            println!(
                "Server (port {}): MPC garbler: Accepted connection from {}",
                my_port_closure, peer_addr
            );
            let reader = std::io::BufReader::new(std_stream.try_clone().unwrap());
            let writer = std::io::BufWriter::new(std_stream);
            let mut channel = Channel::new(reader, writer);
            let mut rng_gb = AesRng::new();
            // Call the garbler's function (which you should have modified to return a u128).
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                gb_sum(&mut rng_gb, &mut channel, mpc_input)
            })).unwrap_or_else(|_| {
                println!("Server (port {}): Garbler failed to produce outputs. Exiting gracefully with dummy value 0.", my_port_closure);
                0_u128
            });            
            println!(
                "Server (port {}): MPC garbler: Result (possibly dummy) = {}",
                my_port_closure, result
            );
            result            
        } else {
            // Evaluator branch.
            // For connection we assume the garbler is on the lower port.
            let garbler_port = if my_port_num < peer_port_num {
                my_port_num
            } else {
                peer_port_num
            };
            let mpc_addr = format!("127.0.0.1:{}", garbler_port + MPC_OFFSET);
            println!(
                "Server (port {}): MPC evaluator: Connecting to {}",
                my_port_closure, mpc_addr
            );
            
            // Retry loop so we wait until the garbler is ready.
            let mut std_stream;
            let mut attempts = 0;
            loop {
                match std::net::TcpStream::connect(&mpc_addr) {
                    Ok(s) => {
                        std_stream = s;
                        break;
                    },
                    Err(e) => {
                        attempts += 1;
                        if attempts > 10 {
                            panic!("Failed to connect after {} attempts: {}", attempts, e);
                        }
                        println!("Server (port {}): Connection attempt {} failed: {}. Retrying...", my_port_closure, attempts, e);
                        thread::sleep(Duration::from_millis(200));
                    }
                }
            }
            
            let reader = std::io::BufReader::new(std_stream.try_clone().unwrap());
            let writer = std::io::BufWriter::new(std_stream);
            let mut channel = Channel::new(reader, writer);
            let mut rng_ev = AesRng::new();
            let result = ev_sum(&mut rng_ev, &mut channel, mpc_input);
            println!(
                "Server (port {}): MPC evaluator: Result = {}",
                my_port_closure, result
            );
            // Here, the evaluator computes the MPC output and returns it.
            result
        }
    })
    .await
    .unwrap();

    println!("Server (port {}): MPC result = {}", my_port, mpc_result);

    if is_evaluator {
        // This branch is for the evaluator: send the result back.
        let response = ShareResponse {
            share: FieldElm::from(mpc_result as u64),
        };
        let encoded = bincode::serialize(&response)?;
        socket.write_all(&encoded).await?;
        println!("Server (port {}): Sent MPC result back to leader.", my_port);
    } else {
        println!("Server (port {}): No result to send (garbler).", my_port);
    }
    

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <my_port> <peer_port>", args[0]);
        std::process::exit(1);
    }
    let my_port = args[1].clone();
    let peer_port = args[2].clone();
    let addr = format!("127.0.0.1:{}", my_port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Server (port {}): Listening on {}", my_port, addr);

    // Process one connection for testing.
    if let Ok((socket, peer_addr)) = listener.accept().await {
        println!("Server (port {}): Accepted connection from {}", my_port, peer_addr);
        if let Err(e) = handle_incoming(socket, my_port.clone(), peer_port.clone()).await {
            eprintln!("Server (port {}): Error handling connection: {}", my_port, e);
        }
    } else {
        eprintln!("Server (port {}): Failed to accept connection.", my_port);
    }
    println!("Server (port {}): Exiting after processing one connection.", my_port);

    Ok(())
}
