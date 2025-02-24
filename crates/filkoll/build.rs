use std::fs::File;
use std::hash::Hasher;
use std::io::Write;

fn find_cargo_lock() -> File {
    // Deal with differences between workspace and package on crates.io
    match File::open("../../Cargo.lock") {
        Ok(file) => file,
        Err(_) => File::open("Cargo.lock").expect("Failed to open Cargo.lock"),
    }
}

fn main() {
    let cargo_toml_raw =
        std::io::read_to_string(find_cargo_lock()).expect("Failed to read Cargo.lock");
    let mut hasher = std::hash::DefaultHasher::new();
    hasher.write(cargo_toml_raw.as_bytes());
    let hash = hasher.finish();

    println!("cargo::rerun-if-changed=../../Cargo.lock");
    // Write to OUT_DIR
    let mut file =
        File::create(std::env::var("OUT_DIR").expect("No OUT_DIR found") + "/lock_hash.rs")
            .expect("Failed to generate code");
    writeln!(file, "const LOCK_HASH: u64 = {hash:#x};").expect("Failed to write generated code");
}
