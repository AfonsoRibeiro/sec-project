fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("./src/location_storage.proto")?;
    Ok(())
}