fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    flutter_rust_embedder::run()
}
