use std::io::{self, Read, Write};
use serde_json::{Value, json};

fn read_message() -> Option<Value> {
    let mut length_buffer = [0u8; 4];
    io::stdin().read_exact(&mut length_buffer).ok()?;
    let length = u32::from_le_bytes(length_buffer) as usize;

    let mut message_buffer = vec![0; length];
    io::stdin().read_exact(&mut message_buffer).ok()?;

    serde_json::from_slice(&message_buffer).ok()
}

fn write_message(response: &Value) -> io::Result<()> {
    let response_bytes = serde_json::to_vec(response).expect("Failed to serialize response");
    let length = response_bytes.len() as u32;

    io::stdout().write_all(&length.to_le_bytes())?;
    io::stdout().write_all(&response_bytes)?;
    io::stdout().flush()?;
    Ok(())
}

fn main() {
    loop {
        if let Some(message) = read_message() {
            eprintln!("Received: {:?}", message);
            let response = json!({ "response": "Hello from down-poc native host!" });
            write_message(&response).expect("Failed to send response");
        } else {
            break;
        }
    }
}

