fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/rpc.proto");

    tonic_build::configure()
        .build_server(false) // Disable server code generation
        .compile_protos(&["proto/rpc.proto"], &["proto/"])?;

    Ok(())
}
