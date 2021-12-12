extern crate vergen;

use vergen::{ConstantsFlags, generate_cargo_keys};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/service.proto")?;

    let flags = ConstantsFlags::SEMVER | ConstantsFlags::REBUILD_ON_HEAD_CHANGE;
    generate_cargo_keys(flags).expect("Unable to generate the cargo keys!");

    Ok(())
}