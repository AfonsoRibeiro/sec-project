fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("./src/location_storage.proto")?;
    tonic_build::compile_protos("./src/location_proof.proto")?;
    tonic_build::compile_protos("./src/location_master.proto")?;
    tonic_build::compile_protos("./src/double_echo_broadcast.proto")?;
    Ok(())
}