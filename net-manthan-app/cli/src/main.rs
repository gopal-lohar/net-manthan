use std::collections::HashMap;
use std::io::{stdout, Write};

use net_manthan_core::types::{ChunkProgress, Message};
use utils::{Client, IPC_SOCKET_ADDRESS};

fn render_progress(progress: &HashMap<u32, ChunkProgress>) {
    let width = 50; // Progress bar width
    print!("\x1B[2J\x1B[H"); // Move cursor to the top without clearing the screen

    let mut chunk_ids: Vec<_> = progress.keys().copied().collect();
    chunk_ids.sort(); // Ensure fixed order

    for (index, chunk_id) in chunk_ids.iter().enumerate() {
        let chunk = &progress[chunk_id];
        let percentage = chunk.bytes_downloaded as f64 / chunk.total_bytes as f64;
        let filled = (percentage * width as f64) as usize;
        let bar = format!(
            "\x1B[32m{}\x1B[0m{}", // Green `#` and spaces
            "#".repeat(filled),
            " ".repeat(width - filled)
        );

        print!("\x1B[{};0H", index + 1); // Move cursor to line (index + 1)
        println!(
            "Chunk {} [{}] {:.2}% @ {:.2} KB/s",
            chunk.chunk_id,
            bar,
            percentage * 100.0,
            chunk.speed / 1024.0
        );
    }

    stdout().flush().unwrap();
}

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
        // println!("Received update: {:?}", message);
        match message {
            Message::ProgressResponse(progress) => render_progress(&progress),
            _ => println!("Received message: {:?}", message),
        }
        // You can do anything here, like updating a UI, logging, etc.
    };

    if let Err(e) = client.send_and_stream(Message::ProgressRequest(vec![1]), handler) {
        println!("Error in streaming: {}", e);
    }
}
