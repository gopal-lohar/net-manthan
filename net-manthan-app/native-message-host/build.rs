use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

fn main() {
    let workspace_root = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf();

    // Create .dev directory in workspace root if it doesn't exist
    let dev_dir = workspace_root.join(".dev");

    match fs::create_dir_all(&dev_dir) {
        Ok(_) => println!("Created .dev directory at: {}", dev_dir.display()),
        Err(e) => eprintln!("Error creating .dev directory: {}", e),
    }

    let binary_path = workspace_root
        .join("target")
        .join(env::var("PROFILE").unwrap())
        .join("native-message-host")
        .to_str()
        .unwrap()
        .to_string();

    // Native messaging host for net-manthan download manager extension.
    let manifest = serde_json::json!({
        "name": "com.net.manthan",
        "description": "Native messaging host for net-manthan download manager extension.",
        "path": binary_path,
        "type": "stdio",
        "allowed_extensions": ["net-manthan@example.com"]
    });

    let manifest_path = dev_dir.join("com.net.manthan.json");
    let mut file = File::create(&manifest_path).unwrap();
    file.write_all(serde_json::to_string_pretty(&manifest).unwrap().as_bytes()).unwrap();

    println!("Manifest written to: {}", manifest_path.display());
}