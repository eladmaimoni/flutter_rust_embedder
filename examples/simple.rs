// clear && FLUTTER_RUST_EMBEDDER_LOG=trace cargo run --example simple
// clear && cargo build
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    flutter_rust_embedder::run()
}
