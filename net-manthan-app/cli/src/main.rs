use net_manthan_core::types::Message;
use utils::{Client, IPC_SOCKET_ADDRESS};

fn main() {
    println!("Hello, world!");

    let mut client = match Client::new(IPC_SOCKET_ADDRESS) {
        Ok(client) => client,
        Err(e) => {
            println!("Could not connect to the server. ERR: {}", e);
            return;
        }
    };

    let handler = |message: Message| {
        println!("Received update: {:?}", message);
        // You can do anything here, like updating a UI, logging, etc.
    };

    if let Err(e) = client.send_and_stream(Message::ProgressRequest(vec![1]), handler) {
        println!("Error in streaming: {}", e);
    }
}
