use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let proto_dir = manifest_dir.join("../../proto").canonicalize()?;

    // Use protox to parse proto files (no protoc binary needed).
    // Signature: compile(files, includes)
    let file_descriptors = protox::compile(
        ["transform.proto", "inspection.proto"],
        [proto_dir.to_str().unwrap()],
    )?;

    // Use tonic_prost_build to generate Rust code from file descriptors
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true) // client needed for integration tests
        .compile_fds(file_descriptors)?;

    Ok(())
}
