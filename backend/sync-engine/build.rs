fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["proto/sync_service.proto"],
            &["proto/"],
        )?;
    println!("cargo:rerun-if-changed=proto/sync.proto");
    println!("cargo:rerun-if-changed=proto/sync_service.proto");
    Ok(())
}
