// clear && FLUTTER_RUST_EMBEDDER_LOG=trace cargo run --example simple
// clear && cargo build
// RUST_LOG=trace cargo run --example simple
// target based filtering (default target is the module name)
// RUST_LOG=flutter_rust_embedder=trace,wgpu=info cargo run --example simple
// RUST_LOG=flutter_rust_embedder=trace,winit=error cargo run --example simple
// RUST_LOG=error cargo run --example simple

use tracing::{info, span};
use tracing_perfetto::PerfettoLayer;
use tracing_subscriber::fmt::format::Format;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, Registry};

fn init_subscriber() {
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    let time_str = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let trace_path = exe_dir.join(format!("trace-{}.perfetto-trace", time_str));
    let trace_file = std::fs::File::create(&trace_path).unwrap();
    let perfetto_layer =
        PerfettoLayer::new(std::sync::Mutex::new(trace_file)).with_debug_annotations(true);

    let fmt_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .event_format(Format::default().with_thread_ids(true))
        .with_span_events(fmt::format::FmtSpan::FULL);

    let env_filter = EnvFilter::from_default_env();
    let subscriber = Registry::default()
        .with(fmt_layer)
        .with(perfetto_layer)
        .with(env_filter);

    tracing::subscriber::set_global_default(subscriber).unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_subscriber();
    // env_logger::init();
    flutter_rust_embedder::run()
}
