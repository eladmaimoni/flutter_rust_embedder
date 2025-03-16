use std::env;
use std::path::PathBuf;

fn main() {
    // Tell Cargo to re-run this script if the wrapper header changes
    let embedder_header_path = std::path::Path::new("flutter_embedder.h");

    if !embedder_header_path.exists() {
        panic!("flutter_embedder.h not found. Please run `flutter build aot` first.");
    }

    println!("cargo:rerun-if-changed={}", embedder_header_path.display());

    let bindings = bindgen::Builder::default()
        // The input header we would like to generate bindings for
        .header(embedder_header_path.to_str().unwrap())
        // Tell bindgen to generate a block-list of types that
        // are defined in these headers. This helps avoid conflicts.
        .blocklist_file(".*stddef.h")
        .blocklist_file(".*stdint.h")
        // Make the bindings pretty and more Rust-like
        .derive_debug(true)
        .derive_default(true)
        .generate_comments(true)
        // Use Core Foundation types on macOS
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("flutter_embedder_bindings.rs"))
        .expect("Couldn't write bindings!");
}
