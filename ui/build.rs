fn main() {
    // Force-build net-manthan before building ui
    println!("cargo:rerun-if-changed=../net-manthan/src/main.rs");
    std::process::Command::new("cargo")
        .args(["build", "-p", "net-manthan"])
        .status()
        .expect("Failed to build net-manthan");
}
