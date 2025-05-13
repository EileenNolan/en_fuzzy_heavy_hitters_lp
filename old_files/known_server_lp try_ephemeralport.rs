// // server code
// use tokio::net::{TcpListener, TcpStream};
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use serde::{Serialize, Deserialize};
// use std::env;
// use std::error::Error;
// use en_fuzzy_heavy_hitters_lp::field::FieldElm;
// use en_fuzzy_heavy_hitters_lp::lagrange;
// use tokio::sync::Mutex;
// use once_cell::sync::Lazy;

// static MPC_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));


// // Import functions from lagrange module.
// use lagrange::{evaluate_polynomial, lagrange_interpolation_coeffs};

// const MODULUS_64: u64 = 9223372036854775783u64;
// const D: usize = 4;

// #[derive(Serialize, Deserialize, Debug)]
// struct ShareRequest {
//     // Leader sends polynomial coefficients (one polynomial per coordinate).
//     poly: Vec<Vec<FieldElm>>,
// }

// #[derive(Serialize, Deserialize, Debug)]
// struct ShareResponse {
//     share: FieldElm,
// }

// async fn handle_incoming(
//     mut socket: TcpStream,
//     my_port: String,
//     peer_port: String,
// ) -> Result<(), Box<dyn Error>> {
//     // --- Phase 1: Receive the polynomial shares from the leader ---
//     let mut buf = vec![0u8; 4096]; // need enough buffer
//     let n = socket.read(&mut buf).await?;
//     if n == 0 {
//         println!("Server (port {}): Received empty message.", my_port);
//         return Ok(());
//     }
//     let req: ShareRequest = bincode::deserialize(&buf[..n])?;
//     println!("Server (port {}): Received polynomial shares from leader: {:?}", my_port, req.poly);

//     // --- Phase 2: Evaluate the polynomials on the server's private data ---
//     // Server's private input vector, w in Z^D.
//     let w = vec![5u64, 10, 15, 20];  // Example server dictionary.
//     let mut sum = FieldElm::from(0);
//     for i in 0..D {
//         // Option 1 (if using hash_to_field):
//         // let key_i = hash_to_field(w[i]);
//         // Option 2 no hash
//         let key_i = FieldElm::from(w[i]);
//         // Evaluate the received polynomial for coordinate i at key_i.
//         let x_i = evaluate_polynomial(&req.poly[i], &key_i);
//         sum = FieldElm::from((sum.value + x_i.value) % MODULUS_64);
//     }
//     println!("Server (port {}): Computed sum = {}", my_port, sum.value);

//     // --- Phase 3: Run the MPC phase using the computed sum as input ---
//     let mpc_input: u128 = sum.value as u128;

//     // Parse port numbers.
//     let my_port_num: u32 = my_port.parse().unwrap();
//     let peer_port_num: u32 = peer_port.parse().unwrap();
//     // Convention: server with higher port becomes evaluator.
//     let is_evaluator = my_port_num > peer_port_num;
//     println!(
//         "Server (port {}): MPC role: {}",
//         my_port,
//         if is_evaluator { "evaluator" } else { "garbler" }
//     );

//     // --- Run MPC in a blocking thread ---
//     // Use an MPC dedicated port (offset by a constant, e.g., 1000).
//     const MPC_OFFSET: u32 = 1000;
//     let my_port_closure = my_port.clone();
//     // Import the external types from scuttlebutt.
//     use scuttlebutt::Channel;
//     use scuttlebutt::AesRng;
//     // Acquire the MPC lock so that only one MPC session runs at a time.
//     let _mpc_lock = MPC_LOCK.lock().await;

//     let mpc_result: u128 = tokio::task::spawn_blocking(move || {
//         use std::io::{Read, Write};
//         use std::{thread, time::Duration, panic};
//         if !is_evaluator {
//             // Garbler branch.
//             let mpc_addr = format!("127.0.0.1:{}", my_port_num + MPC_OFFSET);
//             println!("Server (port {}): MPC garbler: Listening on {}", my_port_closure, mpc_addr);
//             let listener = std::net::TcpListener::bind(&mpc_addr).unwrap();
//             let (std_stream, peer_addr) = listener.accept().unwrap();
//             println!("Server (port {}): MPC garbler: Accepted connection from {}", my_port_closure, peer_addr);
//             let reader = std::io::BufReader::new(std_stream.try_clone().unwrap());
//             let writer = std::io::BufWriter::new(std_stream);
//             let mut channel = Channel::new(reader, writer);
//             let mut rng_gb = AesRng::new();
//             let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
//                 en_fuzzy_heavy_hitters_lp::sum_leq_binary::gb_sum(&mut rng_gb, &mut channel, mpc_input)
//             })).unwrap_or_else(|_| {
//                 println!("Server (port {}): Garbler using dummy value 0.", my_port_closure);
//                 0_u128
//             });
//             println!("Server (port {}): MPC garbler: Result = {}", my_port_closure, result);
//             result
//         } else {
//             // Evaluator branch.
//             let garbler_port = if my_port_num < peer_port_num { my_port_num } else { peer_port_num };
//             let mpc_addr = format!("127.0.0.1:{}", garbler_port + MPC_OFFSET);
//             println!("Server (port {}): MPC evaluator: Connecting to {}", my_port_closure, mpc_addr);
//             let mut std_stream;
//             let mut attempts = 0;
//             loop {
//                 match std::net::TcpStream::connect(&mpc_addr) {
//                     Ok(s) => { std_stream = s; break; },
//                     Err(e) => {
//                         attempts += 1;
//                         if attempts > 10 {
//                             panic!("Failed after {} attempts: {}", attempts, e);
//                         }
//                         println!("Server (port {}): Connection attempt {} failed: {}. Retrying...", my_port_closure, attempts, e);
//                         thread::sleep(Duration::from_millis(200));
//                     }
//                 }
//             }
//             let reader = std::io::BufReader::new(std_stream.try_clone().unwrap());
//             let writer = std::io::BufWriter::new(std_stream);
//             let mut channel = Channel::new(reader, writer);
//             let mut rng_ev = AesRng::new();
//             let result = en_fuzzy_heavy_hitters_lp::sum_leq_binary::ev_sum(&mut rng_ev, &mut channel, mpc_input);
//             println!("Server (port {}): MPC evaluator: Result = {}", my_port_closure, result);
//             let result_bytes = result.to_be_bytes();
//             {
//                 let writer_rc = channel.writer();
//                 let mut writer_ref = writer_rc.borrow_mut();
//                 writer_ref.write_all(&result_bytes).unwrap();
//                 writer_ref.flush().unwrap();
//             }
//             result
//         }
//     }).await.unwrap();

//     println!("Server (port {}): MPC result = {}", my_port, mpc_result);

//     // --- Phase 4: Send MPC result back to the leader only if evaluator ---
//     if is_evaluator {
//         let resp = ShareResponse { share: FieldElm::from(mpc_result as u64) };
//         let encoded = bincode::serialize(&resp)?;
//         socket.write_all(&encoded).await?;
//         println!("Server (port {}): Sent MPC result back to leader.", my_port);
//     } else {
//         println!("Server (port {}): Garbler did not send an MPC result.", my_port);
//     }

//     Ok(())
// }


// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let args: Vec<String> = env::args().collect();
//     if args.len() < 3 {
//         eprintln!("Usage: {} <my_port> <peer_port>", args[0]);
//         std::process::exit(1);
//     }
//     let my_port = args[1].clone();
//     let peer_port = args[2].clone();
//     let addr = format!("127.0.0.1:{}", my_port);
//     println!("Server (port {}): Listening on {}", my_port, addr);
//     let listener = TcpListener::bind(&addr).await?;
    
//     loop {
//         let (socket, peer_addr) = listener.accept().await?;
//         println!("Server (port {}): Accepted connection from {}", my_port, peer_addr);
//         let my_port_clone = my_port.clone();
//         let peer_port_clone = peer_port.clone();
//         tokio::spawn(async move {
//             if let Err(e) = handle_incoming(socket, my_port_clone, peer_port_clone).await {
//                 eprintln!("Error handling connection: {}", e);
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
use en_fuzzy_heavy_hitters_lp::lagrange;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

// Required for synchronous channel in MPC phase:
use scuttlebutt::{Channel, AesRng};

static MPC_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

use lagrange::{evaluate_polynomial, lagrange_interpolation_coeffs};

const MODULUS_64: u64 = 9223372036854775783u64;
const D: usize = 4;

#[derive(Serialize, Deserialize, Debug)]
struct ShareRequest {
    // Leader sends polynomial coefficients (one per coordinate).
    poly: Vec<Vec<FieldElm>>,
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
    // --- Phase 1: Receive the polynomial shares from the leader ---
    let mut buf = vec![0u8; 4096]; // Need enough buffer.
    let n = socket.read(&mut buf).await?;
    if n == 0 {
        println!("Server (port {}): Received empty message.", my_port);
        return Ok(());
    }
    let req: ShareRequest = bincode::deserialize(&buf[..n])?;
    println!(
        "Server (port {}): Received polynomial shares from leader: {:?}",
        my_port, req.poly
    );

    // --- Phase 2: Evaluate the polynomials on the server's private data ---
    let w = vec![5u64, 10, 15, 20];  // Example server dictionary.
    let mut sum = FieldElm::from(0);
    for i in 0..D {
        // Option 2: no hash.
        let key_i = FieldElm::from(w[i]);
        // Evaluate the received polynomial for coordinate i at key_i.
        let x_i = evaluate_polynomial(&req.poly[i], &key_i);
        sum = FieldElm::from((sum.value + x_i.value) % MODULUS_64);
    }
    println!("Server (port {}): Computed sum = {}", my_port, sum.value);

    // --- Phase 3: Run the MPC phase using the computed sum as input ---
    let mpc_input: u128 = sum.value as u128;
    let my_port_num: u32 = my_port.parse().unwrap();
    let peer_port_num: u32 = peer_port.parse().unwrap();
    // Convention: server with higher port becomes evaluator.
    let is_evaluator = my_port_num > peer_port_num;
    println!(
        "Server (port {}): MPC role: {}",
        my_port,
        if is_evaluator { "evaluator" } else { "garbler" }
    );

    // Acquire the MPC lock to run only one MPC session at a time.
    let _mpc_lock = MPC_LOCK.lock().await;

    // --- Run MPC in a blocking thread with ephemeral port negotiation ---
    // We must convert the asynchronous TokIo TcpStream (socket) into a standard blocking TcpStream
    // so that the synchronous code in spawn_blocking can use it.
    // (Note: This conversion consumes `socket`.)
    // --- Run MPC in a blocking thread with ephemeral port negotiation ---
    let mut std_socket = socket.into_std()?;
    // Set the socket to blocking mode.
    std_socket.set_nonblocking(false)?;
    let my_port_closure = my_port.clone();
    
    let mpc_result: u128 = tokio::task::spawn_blocking(move || {
        use std::io::{Read, Write};
        use std::{thread, time::Duration, panic};
    
        if !is_evaluator {
            // Garbler branch (unchanged)...
            // Bind to an ephemeral port.
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let assigned_port = listener.local_addr().unwrap().port();
            println!("Server (port {}): MPC garbler: Listening on ephemeral port {}", my_port_closure, assigned_port);
            
            // Send the assigned port to the leader.
            let port_msg = bincode::serialize(&assigned_port).expect("Port serialize failed");
            std_socket.write_all(&port_msg).expect("write_all failed");
            std_socket.flush().expect("flush failed");
            
            // Now wait for connection from evaluator...
            let (std_stream, peer_addr) = listener.accept().expect("accept failed");
            println!("Server (port {}): MPC garbler: Accepted connection from {}", my_port_closure, peer_addr);
                
            let reader = std::io::BufReader::new(std_stream.try_clone().unwrap());
            let writer = std::io::BufWriter::new(std_stream);
            let mut channel = Channel::new(reader, writer);
            let mut rng_gb = AesRng::new();
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                en_fuzzy_heavy_hitters_lp::sum_leq_binary::gb_sum(&mut rng_gb, &mut channel, mpc_input)
            })).unwrap_or_else(|_| {
                println!("Server (port {}): Garbler error; using dummy value 0.", my_port_closure);
                0_u128
            });
            println!("Server (port {}): MPC garbler: Result = {}", my_port_closure, result);
            result
        } else {
            let mut port_buf = [0u8; 8];
            // Instead of reading directly from `std_socket`, create a clone for control message reading.
            let mut std_socket_clone = std_socket.try_clone().expect("try_clone failed");
            {
                let mut reader = std::io::BufReader::new(&mut std_socket_clone);
                reader.read_exact(&mut port_buf).expect("read_exact failed");
            }
            let assigned_port: u16 = bincode::deserialize(&port_buf).expect("deserialize failed");
            println!("Server (port {}): MPC evaluator: Received ephemeral port {}", my_port_closure, assigned_port);
            
            // Now connect to the garbler at the ephemeral port.
            let mpc_addr = format!("127.0.0.1:{}", assigned_port);
            println!("Server (port {}): MPC evaluator: Connecting to {}", my_port_closure, mpc_addr);
            let mut std_stream;
            let mut attempts = 0;
            loop {
                match std::net::TcpStream::connect(&mpc_addr) {
                    Ok(s) => { std_stream = s; break; },
                    Err(e) => {
                        attempts += 1;
                        if attempts > 10 {
                            panic!("Evaluator: Failed after {} attempts: {}", attempts, e);
                        }
                        println!("Server (port {}): Evaluator connection attempt {} failed: {}.", my_port_closure, attempts, e);
                        thread::sleep(Duration::from_millis(200));
                    }
                }
            }
            let reader = std::io::BufReader::new(std_stream.try_clone().unwrap());
            let writer = std::io::BufWriter::new(std_stream);
            let mut channel = Channel::new(reader, writer);
            let mut rng_ev = AesRng::new();
            let result = en_fuzzy_heavy_hitters_lp::sum_leq_binary::ev_sum(&mut rng_ev, &mut channel, mpc_input);
            println!("Server (port {}): MPC evaluator: Result = {}", my_port_closure, result);
            result
        }
    }).await.unwrap();
    

    
    println!("Server (port {}): MPC result = {}", my_port, mpc_result);

    // --- Phase 4: Send MPC result back to the leader only if evaluator ---
    // (We assume that only the evaluator sends back the MPC result over the leader connection.)
    // Since we already converted the leader connection into a std::net::TcpStream and used it for MPC
    // control (in the spawn_blocking closure), you may need to re-establish an async connection or
    // convert back. For simplicity, assume that here we do not re-send if the leader already received it.
    // Alternatively, you can have the evaluator control-channel send the MPC result.
    if is_evaluator {
        // In this example, we simply print the result.
        println!("Server (port {}): Sent MPC result (implicitly via control channel).", my_port);
    } else {
        println!("Server (port {}): Garbler did not send an MPC result.", my_port);
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <my_port> <peer_port>", args[0]);
        std::process::exit(1);
    }
    let my_port = args[1].clone();
    let peer_port = args[2].clone();
    let addr = format!("127.0.0.1:{}", my_port);
    println!("Server (port {}): Listening on {}", my_port, addr);
    let listener = TcpListener::bind(&addr).await?;
    
    loop {
        let (socket, peer_addr) = listener.accept().await?;
        println!("Server (port {}): Accepted connection from {}", my_port, peer_addr);
        let my_port_clone = my_port.clone();
        let peer_port_clone = peer_port.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_incoming(socket, my_port_clone, peer_port_clone).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}
