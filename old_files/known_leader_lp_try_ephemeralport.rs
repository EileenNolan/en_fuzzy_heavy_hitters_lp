// use tokio::net::TcpStream;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use serde::{Serialize, Deserialize};
// use std::error::Error;
// use en_fuzzy_heavy_hitters_lp::field::FieldElm;
// use en_fuzzy_heavy_hitters_lp::lagrange;
// use futures::future;
// use lagrange::{compute_polynomials, print_polynomial, evaluate_polynomial};
// use std::sync::Arc;

// // define constants
// const D: usize = 4;
// const DELTA: i64 = 3;
// const P: i64 = 2;

// #[derive(Serialize, Deserialize, Debug)]
// struct ShareRequest {
//     // This structure contains a polynomial for each coordinate.
//     poly: Vec<Vec<FieldElm>>,
// }

// #[derive(Serialize, Deserialize, Debug)]
// struct ShareResponse {
//     // The server returns its MPC output as a FieldElm.
//     share: FieldElm,
// }

// /// Processes one query vector:
// /// 1. Computes polynomial shares from the query.
// /// 2. Opens fresh connections to Server A (port 8001) and Server B (port 8002).
// /// 3. Sends the corresponding polynomial shares to each server.
// /// 4. Waits for a response (from at least one server) and returns the MPC output.
// async fn process_query(q: Vec<u64>) -> Result<u128, Box<dyn Error + Send + Sync>> {
//     // Compute the secret-shared polynomial coefficients.
//     let (poly_shares_A, poly_shares_B) = compute_polynomials(&q);
    
//     let req_A = ShareRequest { poly: poly_shares_A.clone() };
//     let req_B = ShareRequest { poly: poly_shares_B.clone() };
    
//     // Debug: Print polynomials.
//     println!("Polynomials for Server A:");
//     for (i, poly) in poly_shares_A.iter().enumerate() {
//         let poly_name = format!("E_A{}", i);
//         print_polynomial(poly, &poly_name);
//     }
//     println!("Polynomials for Server B:");
//     for (i, poly) in poly_shares_B.iter().enumerate() {
//         let poly_name = format!("E_B{}", i);
//         print_polynomial(poly, &poly_name);
//     }

//     // Create fresh connections for this query.
//     // (Each query gets its own connection, ensuring isolation.)
//     let mut socket_a = TcpStream::connect("127.0.0.1:8001").await?;
//     let mut socket_b = TcpStream::connect("127.0.0.1:8002").await?;

//     // Send requests.
//     let req_a_bytes = bincode::serialize(&req_A)?;
//     let req_b_bytes = bincode::serialize(&req_B)?;
//     socket_a.write_all(&req_a_bytes).await?;
//     socket_b.write_all(&req_b_bytes).await?;
//     println!("Leader: Sent polynomial shares for query {:?} to servers.", q);

//     // Read responses from both servers.
//     let mut buf_a = vec![0u8; 1024];
//     let mut buf_b = vec![0u8; 1024];
//     let n_a = socket_a.read(&mut buf_a).await?;
//     let n_b = socket_b.read(&mut buf_b).await?;
//     println!("Leader: Received {} bytes from 8001 and {} bytes from 8002.", n_a, n_b);

//     // For simplicity, use the first nonzero response.
//     let mut share_resp: Option<ShareResponse> = None;
//     if n_a > 0 {
//         share_resp = Some(bincode::deserialize(&buf_a[..n_a])?);
//     }
//     if n_b > 0 {
//         share_resp = Some(bincode::deserialize(&buf_b[..n_b])?);
//     }

//     if let Some(resp) = share_resp {
//         Ok(u128::from(resp.share.value))
//     } else {
//         Err("Leader: No valid MPC output received.".into())
//     }
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
//     // Total number of queries to simulate.
//     let n_clients = 100;
//     // Process at most this many queries concurrently.
//     let batch_size = 1;
//     let mut aggregate: u128 = 0;
//     let mut queries = Vec::new();

//     // For testing, assume the server's dictionary is known.
//     let server_w = vec![5u64, 10, 15, 20];

//     // Generate queries: alternate between exact match and random.
//     for i in 0..n_clients {
//         let q = if i % 2 == 0 {
//             // Use exactly the server dictionary for even indices.
//             server_w.clone()
//         } else {
//             // Generate a random query vector in range [100, 500] for odd indices.
//             use rand::Rng;
//             let mut rng = rand::thread_rng();
//             (0..D).map(|_| rng.gen_range(100..500)).collect()
//         };
//         queries.push(q);
//     }
    
//     // Process queries in batches.
//     for batch in queries.chunks(batch_size) {
//         // We use tokio::spawn to run each query in its own task,
//         // ensuring that each session creates fresh connections.
//         let mut futures = Vec::new();
//         for q in batch {
//             let q_clone = q.clone();
//             futures.push(tokio::spawn(async move {
//                 process_query(q_clone).await
//             }));
//         }
//         // Await all futures concurrently.
//         let results = future::join_all(futures).await;
//         for res in results {
//             match res {
//                 Ok(Ok(value)) => {
//                     aggregate += value;
//                     println!("Processed query with MPC result: {}", value);
//                 }
//                 Ok(Err(e)) => eprintln!("Error processing query: {}", e),
//                 Err(e) => eprintln!("Task join error: {}", e),
//             }
//         }
//     }

//     println!("Leader: Aggregate MPC result = {}", aggregate);
//     Ok(())
// }


// use tokio::net::TcpStream;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use serde::{Serialize, Deserialize};
// use std::error::Error;
// use en_fuzzy_heavy_hitters_lp::field::FieldElm;
// use en_fuzzy_heavy_hitters_lp::lagrange;
// use futures::future;
// use lagrange::{compute_polynomials, print_polynomial, evaluate_polynomial};

// const D: usize = 4;
// const DELTA: i64 = 3;
// const P: i64 = 2;

// #[derive(Serialize, Deserialize, Debug)]
// struct ShareRequest {
//     // This structure contains a polynomial for each coordinate.
//     poly: Vec<Vec<FieldElm>>,
// }

// #[derive(Serialize, Deserialize, Debug)]
// struct ShareResponse {
//     // The server returns its MPC output as a FieldElm.
//     share: FieldElm,
// }

// /// Processes one query vector:
// /// 1. Computes polynomial shares from the query.
// /// 2. Opens fresh connections to Server A (port 8001) and Server B (port 8002).
// /// 3. Sends the corresponding request (the polynomial shares) to each server.
// /// 4. Waits for a response from at least one server and returns the MPC output as u128.
// async fn process_query(q: Vec<u64>) -> Result<u128, Box<dyn Error + Send + Sync>> {
//     // Compute the secret-shared polynomial coefficients.
//     let (poly_shares_A, poly_shares_B) = compute_polynomials(&q);
//     let req_A = ShareRequest { poly: poly_shares_A.clone() };
//     let req_B = ShareRequest { poly: poly_shares_B.clone() };

//     // Print polynomials for debugging.
//     println!("Polynomials for Server A:");
//     for (i, poly) in poly_shares_A.iter().enumerate() {
//         let poly_name = format!("E_A{}", i);
//         print_polynomial(poly, &poly_name);
//     }
//     println!("Polynomials for Server B:");
//     for (i, poly) in poly_shares_B.iter().enumerate() {
//         let poly_name = format!("E_B{}", i);
//         print_polynomial(poly, &poly_name);
//     }

//     // Create fresh connections for this query.
//     let mut socket_a = TcpStream::connect("127.0.0.1:8001").await?;
//     let mut socket_b = TcpStream::connect("127.0.0.1:8002").await?;

//     // Send the serialized requests.
//     socket_a.write_all(&bincode::serialize(&req_A)?).await?;
//     socket_b.write_all(&bincode::serialize(&req_B)?).await?;
//     println!("Leader: Sent polynomial shares for query {:?} to servers.", q);

//     // Wait for responses.
//     let mut buf_a = vec![0u8; 1024];
//     let mut buf_b = vec![0u8; 1024];
//     let n_a = socket_a.read(&mut buf_a).await?;
//     let n_b = socket_b.read(&mut buf_b).await?;
//     println!("Leader: Received {} bytes from 8001 and {} bytes from 8002.", n_a, n_b);

//     // Use the first nonzero response.
//     let mut share_resp: Option<ShareResponse> = None;
//     if n_a > 0 {
//         share_resp = Some(bincode::deserialize(&buf_a[..n_a])?);
//     }
//     if n_b > 0 {
//         share_resp = Some(bincode::deserialize(&buf_b[..n_b])?);
//     }

//     if let Some(resp) = share_resp {
//         Ok(u128::from(resp.share.value))
//     } else {
//         Err("Leader: No valid MPC output received.".into())
//     }
// }

// // Helper function to sample a query near the known server dictionary.
// fn sample_query_near_server(server: &[u64]) -> Vec<u64> {
//     use rand::Rng;
//     let mut rng = rand::thread_rng();
//     server.iter().map(|&v| {
//         let offset: i64 = rng.gen_range(-DELTA..=DELTA);
//         ((v as i64) + offset) as u64
//     }).collect()
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
//     // Total number of queries to simulate.
//     let n_clients = 100;
//     // Process at most this many queries concurrently.
//     let batch_size = 1;
//     let mut aggregate: u128 = 0;
//     let mut queries = Vec::new();

//     // Assume the server's dictionary values are known.
//     let server_w = vec![5u64, 10, 15, 20];

//     // For testing: half the queries use the exact server dictionary, half random.
//     for i in 0..n_clients {
//         let q = if i % 2 == 0 {
//             // Use the server values for even indices.
//             server_w.clone()
//         } else {
//             // Use a random query vector for odd indices.
//             use rand::Rng;
//             let mut rng = rand::thread_rng();
//             (0..D).map(|_| rng.gen_range(100..500)).collect()
//         };
//         queries.push(q);
//     }
    
//     // Process queries in batches.
//     for batch in queries.chunks(batch_size) {
//         let mut futures = Vec::new();
//         for q in batch {
//             let q_clone = q.clone();
//             // Spawn a new asynchronous task for each query.
//             futures.push(tokio::spawn(async move {
//                 process_query(q_clone).await
//             }));
//         }
//         // Wait for all queries in this batch concurrently.
//         let results = future::join_all(futures).await;
//         for res in results {
//             match res {
//                 Ok(Ok(value)) => {
//                     aggregate += value;
//                     println!("Processed query with MPC result: {}", value);
//                 }
//                 Ok(Err(e)) => eprintln!("Error processing query: {}", e),
//                 Err(e) => eprintln!("Task join error: {}", e),
//             }
//         }
//     }

//     println!("Leader: Aggregate MPC result = {}", aggregate);
//     Ok(())
// }



use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use std::error::Error;
use en_fuzzy_heavy_hitters_lp::field::FieldElm;
use en_fuzzy_heavy_hitters_lp::lagrange;
use futures::future;
use lagrange::{compute_polynomials, print_polynomial, evaluate_polynomial};

const D: usize = 4;
const DELTA: i64 = 3;
const P: i64 = 2;

#[derive(Serialize, Deserialize, Debug)]
struct ShareRequest {
    // This structure contains a polynomial for each coordinate.
    poly: Vec<Vec<FieldElm>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ShareResponse {
    // The server returns its MPC output as a FieldElm.
    share: FieldElm,
}

/// Processes one query vector:
/// 1. Computes polynomial shares from the query.
/// 2. Opens fresh connections to Server A (port 8001) and Server B (port 8002).
/// 3. Sends the corresponding request (the polynomial shares) to each server.
/// 4. Waits for a response from at least one server and returns the MPC output as u128.
async fn process_query(q: Vec<u64>) -> Result<u128, Box<dyn Error + Send + Sync>> {
    // Compute the secret-shared polynomial coefficients.
    let (poly_shares_A, poly_shares_B) = compute_polynomials(&q);
    let req_A = ShareRequest { poly: poly_shares_A.clone() };
    let req_B = ShareRequest { poly: poly_shares_B.clone() };

    // Print polynomials for debugging.
    println!("Polynomials for Server A:");
    for (i, poly) in poly_shares_A.iter().enumerate() {
        let poly_name = format!("E_A{}", i);
        print_polynomial(poly, &poly_name);
    }
    println!("Polynomials for Server B:");
    for (i, poly) in poly_shares_B.iter().enumerate() {
        let poly_name = format!("E_B{}", i);
        print_polynomial(poly, &poly_name);
    }

    // Create fresh connections for this query.
    let mut socket_a = TcpStream::connect("127.0.0.1:8001").await?;
    let mut socket_b = TcpStream::connect("127.0.0.1:8002").await?;

    // Send the serialized requests to both servers.
    socket_a.write_all(&bincode::serialize(&req_A)?).await?;
    socket_b.write_all(&bincode::serialize(&req_B)?).await?;
    println!("Leader: Sent polynomial shares for query {:?} to servers.", q);

    // --- Ephemeral Port Negotiation ---
    // The garbler will, in its MPC phase, bind to an ephemeral port and send that port
    // over its leader connection. Here the leader reads that control message.
    let mut ephem_port_buf = [0u8; 8];  // enough space to hold serialized port (e.g. u16)
    socket_a.read_exact(&mut ephem_port_buf).await?;
    let assigned_port: u16 = bincode::deserialize(&ephem_port_buf)?;
    println!("Leader: Received ephemeral port {} from Server A (garbler).", assigned_port);

    // Forward the ephemeral port to the evaluator.
    socket_b.write_all(&ephem_port_buf).await?;
    socket_b.flush().await?;
    println!("Leader: Forwarded ephemeral port {} to Server B (evaluator).", assigned_port);

    // --- Waiting for Final MPC Result ---
    // The evaluator, after connecting to the garbler on the ephemeral port and doing MPC,
    // sends its result back over its leader connection.
    let mut result_buf = vec![0u8; 1024];
    let n = socket_b.read(&mut result_buf).await?;
    println!("Leader: Received {} bytes from Server B (evaluator).", n);
    if n == 0 {
        return Err("Leader: No valid MPC output received.".into());
    }
    let share_resp: ShareResponse = bincode::deserialize(&result_buf[..n])?;
    Ok(u128::from(share_resp.share.value))
}



// Helper function to sample a query near the known server dictionary.
fn sample_query_near_server(server: &[u64]) -> Vec<u64> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    server.iter().map(|&v| {
        let offset: i64 = rng.gen_range(-DELTA..=DELTA);
        ((v as i64) + offset) as u64
    }).collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Total number of queries to simulate.
    let n_clients = 100;
    // Process at most this many queries concurrently.
    let batch_size = 1;
    let mut aggregate: u128 = 0;
    let mut queries = Vec::new();

    // Assume the server's dictionary values are known.
    let server_w = vec![5u64, 10, 15, 20];

    // For testing: half the queries use the exact server dictionary, half random.
    for i in 0..n_clients {
        let q = if i % 2 == 0 {
            // Use the server values for even indices.
            server_w.clone()
        } else {
            // Use a random query vector for odd indices.
            use rand::Rng;
            let mut rng = rand::thread_rng();
            (0..D).map(|_| rng.gen_range(100..500)).collect()
        };
        queries.push(q);
    }
    
    // Process queries in batches.
    for batch in queries.chunks(batch_size) {
        let mut futures = Vec::new();
        for q in batch {
            let q_clone = q.clone();
            // Spawn a new asynchronous task for each query.
            futures.push(tokio::spawn(async move {
                process_query(q_clone).await
            }));
        }
        // Wait for all queries in this batch concurrently.
        let results = future::join_all(futures).await;
        for res in results {
            match res {
                Ok(Ok(value)) => {
                    aggregate += value;
                    println!("Processed query with MPC result: {}", value);
                }
                Ok(Err(e)) => eprintln!("Error processing query: {}", e),
                Err(e) => eprintln!("Task join error: {}", e),
            }
        }
    }

    println!("Leader: Aggregate MPC result = {}", aggregate);
    Ok(())
}