fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["proto/blog.proto"], &["proto"])?;
    println!("cargo:rerun-if-changed=proto/blog.proto");
    Ok(())
}