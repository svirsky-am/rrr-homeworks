// blog-client/build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)   // Только клиент!
        .build_client(true)
        .protoc_arg("--experimental_allow_proto3_optional") //для поддержки `optional` в proto3
        .compile(&["proto/blog.proto"], &["proto"])?;
    
    println!("cargo:rerun-if-changed=proto/blog.proto");
    Ok(())
}