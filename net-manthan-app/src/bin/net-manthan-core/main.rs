use bincode;
use net_manthan_core::{Message, SOCKET_ADDR};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let listener = TcpListener::bind(SOCKET_ADDR).await?;
    println!("Service listening on {}", SOCKET_ADDR);

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            if let Ok(n) = socket.read(&mut buf).await {
                if n > 0 {
                    let message: Message = bincode::deserialize(&buf[..n]).unwrap();
                    println!("Received: {:?}", message);

                    // Respond to the client
                    let response =
                        bincode::serialize(&Message::Hello("Message received".to_string()))
                            .unwrap();
                    socket.write_all(&response).await.unwrap();
                }
            }
        });
    }
}
