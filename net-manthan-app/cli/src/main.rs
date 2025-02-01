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

    match client.send_and_receive(Message::ProgressRequest(vec![1])) {
        Ok(response) => {
            println!("Response: {:?}", response);
        }
        Err(e) => {
            println!("Could not send the request. ERR: {}", e);
            return;
        }
    }
}
