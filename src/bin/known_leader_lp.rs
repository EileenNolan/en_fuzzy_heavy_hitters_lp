use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
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
async fn main() -> Result<(), Box<dyn Error>> {
    let secret = 6;
    let (share1, share2) = (secret / 2, secret - (secret / 2));
    println!("Secret: {}, Shares: {}, {}", secret, share1, share2);

    let mut socket1 = TcpStream::connect("127.0.0.1:9000").await?;
    let mut socket2 = TcpStream::connect("127.0.0.1:9001").await?;

    let req1 = ShareRequest { share: share1 };
    let req2 = ShareRequest { share: share2 };

    socket1.write_all(&bincode::serialize(&req1).unwrap()).await?;
    socket2.write_all(&bincode::serialize(&req2).unwrap()).await?;

    let mut buf1 = vec![0u8; 1024];
    let mut buf2 = vec![0u8; 1024];
    let n1 = socket1.read(&mut buf1).await?;
    let n2 = socket2.read(&mut buf2).await?;

    let resp1: ShareResponse = bincode::deserialize(&buf1[..n1]).unwrap();
    let resp2: ShareResponse = bincode::deserialize(&buf2[..n2]).unwrap();

    println!("Server responses: {}, {}", resp1.result_share, resp2.result_share);
    let reconstructed = resp1.result_share + resp2.result_share;
    println!("Reconstructed result (dummy): {}", reconstructed);

    Ok(())
}