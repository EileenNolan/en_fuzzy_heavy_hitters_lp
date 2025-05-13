// leader file
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use std::error::Error;
use en_fuzzy_heavy_hitters_lp::field::FieldElm;
use en_fuzzy_heavy_hitters_lp::lagrange;
use futures::future;
use lagrange::{compute_polynomials, print_polynomial};


//define constants
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
/// 2. Opens connections to Server A (port 8001) and Server B (port 8002).
/// 3. Sends the corresponding request (the polynomial shares) to each server.
/// 4. Waits for a response—the evaluator is expected to return a valid MPC output.
/// 5. Returns that output as u128.
async fn process_query(q: Vec<u64>) -> Result<u128, Box<dyn Error + Send + Sync>> {
    // Compute the secret-shared polynomial coefficients.
    let (poly_shares_A, poly_shares_B) = compute_polynomials(&q);
    // let req_A = ShareRequest { poly: poly_shares_A };
    // let req_B = ShareRequest { poly: poly_shares_B };
    let req_A = ShareRequest { poly: poly_shares_A.clone() };
    let req_B = ShareRequest { poly: poly_shares_B.clone() };
    // Print polynomials for debugging
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

    // Connect to Server A (garbler) and Server B (evaluator).
    let mut socket_a = TcpStream::connect("127.0.0.1:8001").await?;
    let mut socket_b = TcpStream::connect("127.0.0.1:8002").await?;

    // Send the requests.
    socket_a.write_all(&bincode::serialize(&req_A)?).await?;
    socket_b.write_all(&bincode::serialize(&req_B)?).await?;
    println!("Leader: Sent polynomial shares for query {:?} to servers.", q);

    // Wait for responses.
    let mut buf_a = vec![0u8; 1024];
    let mut buf_b = vec![0u8; 1024];
    let n_a = socket_a.read(&mut buf_a).await?;
    let n_b = socket_b.read(&mut buf_b).await?;
    println!("Leader: Received {} bytes from 8001 and {} bytes from 8002.", n_a, n_b);

    // Expect a valid MPC output from one of the servers (usually the evaluator).
    let mut share_resp: Option<ShareResponse> = None;
    if n_a > 0 {
        share_resp = Some(bincode::deserialize(&buf_a[..n_a])?);
    }
    if n_b > 0 {
        share_resp = Some(bincode::deserialize(&buf_b[..n_b])?);
    }

    if let Some(resp) = share_resp {
        Ok(u128::from(resp.share.value))
    } else {
        Err("Leader: No valid MPC output received.".into())
    }
}

// helper function for distribution sampling
fn sample_query_near_server(server: &[u64]) -> Vec<u64> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    server.iter().map(|&v| {
        // Generate an offset in the range [-DELTA, DELTA]
        let offset: i64 = rng.gen_range(-DELTA..=DELTA);
        // Convert v to i64, add the offset and cast back to u64.
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

    // // GENERATE RANDOM STRINGS FOR TESTING
    // let mut rng = rand::thread_rng();
    // for _ in 0..n_clients {
    //     let q: Vec<u64> = (0..D).map(|_| rng.gen_range(100..500)).collect();
    //     queries.push(q);
    // }

    //FOR TESTING ONLY. LEADER/CLIENT SHOULDN"T HAVE THE SERVER DICTIONARY VALUE
    // Assume the server's dictionary values are known.
    let server_w = vec![5u64, 10, 15, 20];

    // For testing, generate half the queries close to the server values,
    // and half completely random.
    for i in 0..n_clients {
        let q = if i % 2 == 0 {
            // For every 5th query, use exactly the server’s dictionary values.
            server_w.clone()
        // } else if i % 2 == 0 {
        //     //sample_query_near_server(&server_w)
        //     server_w.clone()
        } else {
            // Generate a random query vector (for example, in the range [100,500]).
            use rand::Rng;
            let mut rng = rand::thread_rng();
            (0..D).map(|_| rng.gen_range(100..500)).collect()
        };
        queries.push(q);
    }
    

    // Process in batches.
    for batch in queries.chunks(batch_size) {
        let mut futures = Vec::new();
        for q in batch {
            futures.push(process_query(q.clone()));
        }
        // Await all queries in this batch concurrently.
        let results = future::join_all(futures).await;
        for res in results {
            match res {
                Ok(value) => {
                    aggregate += value;
                    println!("Processed query with MPC result: {}", value);
                }
                Err(e) => eprintln!("Error processing query: {}", e),
            }
        }
    }

    println!("Leader: Aggregate MPC result = {}", aggregate);
    Ok(())
}
