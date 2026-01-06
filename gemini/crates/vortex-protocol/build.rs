// VORTEX Protocol Build Script
// Compiles .proto files to Rust using prost-build
// By The Engineer (10-Persona Collective)

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the workspace root (two levels up from this crate)
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let proto_dir = manifest_dir.parent().unwrap().parent().unwrap().join("proto");
    let out_dir = manifest_dir.join("src/generated");
    
    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&out_dir)?;
    
    // Compile all proto files
    prost_build::Config::new()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .out_dir(&out_dir)
        .compile_protos(
            &[
                proto_dir.join("control.proto"),
                proto_dir.join("graph.proto"),
                proto_dir.join("worker.proto"),
            ],
            &[&proto_dir],
        )?;
    
    // Rerun if proto files change
    println!("cargo:rerun-if-changed={}/control.proto", proto_dir.display());
    println!("cargo:rerun-if-changed={}/graph.proto", proto_dir.display());
    println!("cargo:rerun-if-changed={}/worker.proto", proto_dir.display());
    
    Ok(())
}
