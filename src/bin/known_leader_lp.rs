// use tokio::net::TcpStream;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use serde::{Serialize, Deserialize};
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
//     let secret = FieldElm::from(6);
//     let (share1, share2) = (FieldElm::from(3), FieldElm::from(3));

//     println!("Secret: {:?}, Shares: {:?}, {:?}", secret, share1, share2);

//     // let mut socket1 = TcpStream::connect("127.0.0.1:9000").await?;
//     // let mut socket2 = TcpStream::connect("127.0.0.1:9001").await?;

//     let mut socket1 = TcpStream::connect("127.0.0.1:8001").await?;
//     let mut socket2 = TcpStream::connect("127.0.0.1:8002").await?;

//     let req1 = ShareRequest { share: share1 };
//     let req2 = ShareRequest { share: share2 };

//     socket1.write_all(&bincode::serialize(&req1)?).await?;
//     socket2.write_all(&bincode::serialize(&req2)?).await?;

//     let mut buf1 = vec![0u8; 1024];
//     let mut buf2 = vec![0u8; 1024];
//     let n1 = socket1.read(&mut buf1).await?;
//     let n2 = socket2.read(&mut buf2).await?;

//     let resp1: ShareResponse = bincode::deserialize(&buf1[..n1])?;
//     let resp2: ShareResponse = bincode::deserialize(&buf2[..n2])?;

//     println!("Server responses: {:?}, {:?}", resp1.result_share, resp2.result_share);
//     let reconstructed = resp1.result_share + resp2.result_share;
//     println!("Reconstructed result (dummy): {:?}", reconstructed);

//     Ok(())
// }


// use tokio::net::TcpStream;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use serde::{Serialize, Deserialize};
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

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     // For demonstration, we define a secret and split it into two shares.
//     // (In your actual MPC these shares will be randomized/secret-shared.)
//     let secret = FieldElm::from(6);
//     let (share1, share2) = (FieldElm::from(3), FieldElm::from(3));

//     println!("Leader: Secret = {:?}, Share1 = {:?}, Share2 = {:?}", secret, share1, share2);

//     // Connect to two servers. (Ensure that you have started the servers on these ports.)
//     let mut socket1 = TcpStream::connect("127.0.0.1:8001").await?;
//     let mut socket2 = TcpStream::connect("127.0.0.1:8002").await?;

//     // Create the share requests
//     let req1 = ShareRequest { share: share1 };
//     let req2 = ShareRequest { share: share2 };

//     // Serialize and send the requests.
//     let encoded1 = bincode::serialize(&req1)?;
//     let encoded2 = bincode::serialize(&req2)?;
//     socket1.write_all(&encoded1).await?;
//     socket2.write_all(&encoded2).await?;
//     println!("Leader: Sent requests to servers.");

//     // Prepare buffers to receive responses.
//     let mut buf1 = vec![0u8; 1024];
//     let mut buf2 = vec![0u8; 1024];

//     let n1 = socket1.read(&mut buf1).await?;
//     let n2 = socket2.read(&mut buf2).await?;
//     println!("Leader: Received {} bytes from socket1 and {} bytes from socket2.", n1, n2);

//     // Deserialize the server responses.
//     let resp1: ShareResponse = bincode::deserialize(&buf1[..n1])?;
//     let resp2: ShareResponse = bincode::deserialize(&buf2[..n2])?;
//     println!("Leader: Received responses: {:?} and {:?}", resp1.share, resp2.share);

//     // Reconstruct the (dummy) result. Later this might be replaced by your MPC reconstruction.
//     let reconstructed = resp1.share + resp2.share;
//     println!("Leader: Reconstructed result (dummy) = {:?}", reconstructed);

//     Ok(())
// }

// use tokio::net::TcpStream;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use serde::{Serialize, Deserialize};
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

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let secret = FieldElm::from(6);
//     let (share1, share2) = (FieldElm::from(3), FieldElm::from(3));
//     println!("Leader: Secret = {:?}, Share1 = {:?}, Share2 = {:?}", secret, share1, share2);

//     // Connect to both servers.
//     let mut socket1 = TcpStream::connect("127.0.0.1:8001").await?;
//     let mut socket2 = TcpStream::connect("127.0.0.1:8002").await?;

//     let req1 = ShareRequest { share: share1 };
//     let req2 = ShareRequest { share: share2 };

//     socket1.write_all(&bincode::serialize(&req1)?).await?;
//     socket2.write_all(&bincode::serialize(&req2)?).await?;
//     println!("Leader: Sent requests to servers.");

//     let mut buf1 = vec![0u8; 1024];
//     let mut buf2 = vec![0u8; 1024];
//     let n1 = socket1.read(&mut buf1).await?;
//     let n2 = socket2.read(&mut buf2).await?;
//     println!("Leader: Received {} bytes from server on 8001 and {} bytes from server on 8002.", n1, n2);

//     // We expect that the evaluator (say, on port 8002) sends its result (16 bytes),
//     // and that the garbler (port 8001) sends 0 bytes.
//     let mpc_result = if n1 == 16 {
//         u128::from_be_bytes(buf1[..16].try_into().unwrap())
//     } else if n2 == 16 {
//         u128::from_be_bytes(buf2[..16].try_into().unwrap())
//     } else {
//         panic!("Leader: No valid MPC output received from either server.");
//     };

//     println!("Leader: Received MPC output = {}", mpc_result);

//     // And then do the reconstruction as needed.
//     // (For example, if you intended to combine results, you can do so. Right now we simply use the evaluator’s output.)
//     println!("Leader: Final MPC result = {:?}", mpc_result);
//     Ok(())
// }


use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize secret and shares.
    let secret = FieldElm::from(6);
    let (share1, share2) = (FieldElm::from(3), FieldElm::from(3));
    println!("Leader: Secret = {:?}, Share1 = {:?}, Share2 = {:?}", secret, share1, share2);

    // Connect to both servers.
    let mut socket1 = TcpStream::connect("127.0.0.1:8001").await?;
    let mut socket2 = TcpStream::connect("127.0.0.1:8002").await?;

    let req1 = ShareRequest { share: share1 };
    let req2 = ShareRequest { share: share2 };

    // Send requests.
    socket1.write_all(&bincode::serialize(&req1)?).await?;
    socket2.write_all(&bincode::serialize(&req2)?).await?;
    println!("Leader: Sent requests to servers.");

    // Read responses.
    let mut buf1 = vec![0u8; 1024];
    let mut buf2 = vec![0u8; 1024];
    let n1 = socket1.read(&mut buf1).await?;
    let n2 = socket2.read(&mut buf2).await?;
    println!(
        "Leader: Received {} bytes from server on 8001 and {} bytes from server on 8002.",
        n1, n2
    );

    // Instead of expecting exactly 16 bytes, try to deserialize a ShareResponse
    let mut share_resp: Option<ShareResponse> = None;
    if n1 > 0 {
        share_resp = Some(bincode::deserialize(&buf1[..n1])?);
    }
    if n2 > 0 {
        // If both return data, we simply use the last one (or you might choose differently)
        share_resp = Some(bincode::deserialize(&buf2[..n2])?);
    }

    let mpc_result = if let Some(resp) = share_resp {
        // Assume FieldElm.value is a u64; convert to u128.
        u128::from(resp.share.value)
    } else {
        panic!("Leader: No valid MPC output received from either server.");
    };

    println!("Leader: Received MPC output = {}", mpc_result);
    println!("Leader: Final MPC result = {:?}", mpc_result);
    Ok(())
}
