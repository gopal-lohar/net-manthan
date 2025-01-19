use serde::{Deserialize, Serialize};
use std::{
    io::{self, Read, Write},
    os::unix::net::UnixStream,
};

#[derive(Debug, Serialize, Deserialize)]
struct DownloadRequest {
    url: String,
    filename: String,
    filesize: Option<i64>,
    mime: Option<String>,
    referrer: Option<String>,
    headers: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct Response {
    status: String,
}

fn read_message() -> io::Result<Vec<u8>> {
    let mut length_bytes = [0u8; 4];
    io::stdin().read_exact(&mut length_bytes)?;
    let length = u32::from_ne_bytes(length_bytes);
    let mut content = vec![0; length as usize];
    io::stdin().read_exact(&mut content)?;
    Ok(content)
}

fn write_message(content: &[u8]) -> io::Result<()> {
    let length = content.len() as u32;
    io::stdout().write_all(&length.to_ne_bytes())?;
    io::stdout().write_all(content)?;
    io::stdout().flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    while let Ok(msg) = read_message() {
        let request: DownloadRequest = serde_json::from_slice(&msg)?;

        let mut stream = UnixStream::connect("/tmp/net-manthan.sock")?;
        stream.write_all(&serde_json::to_vec(&request)?)?;

        // let mut response = String::new();
        // stream.read_to_string(&mut response)?;

        let response = Response {
            status: "received".into(),
        };

        let response_json = serde_json::to_vec(&response)?;
        write_message(&response_json)?;
    }
    Ok(())
}
