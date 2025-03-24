// clear && FLUTTER_RUST_EMBEDDER_LOG=trace cargo run --example simple
// clear && cargo build
// RUST_LOG=trace cargo run --example simple
// target based filtering (default target is the module name)
// RUST_LOG=flutter_rust_embedder=trace,wgpu=info cargo run --example simple
// RUST_LOG=error cargo run --example simple
// RUST_LOG=flutter_rust_embedder=trace,winit=error cargo run --example simple

use flutter_rust_embedder::application::{AppError, GPUContext};
use tracing::{info, info_span};
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
        .event_format(Format::default().with_thread_ids(true));
    // .with_span_events(fmt::format::FmtSpan::ACTIVE);

    let env_filter = EnvFilter::from_default_env();
    let subscriber = Registry::default()
        .with(fmt_layer)
        .with(perfetto_layer)
        .with(env_filter);

    tracing::subscriber::set_global_default(subscriber).unwrap();
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), AppError> {
    init_subscriber();
    let _span = info_span!("main").entered();
    info!("start app");
    // env_logger::init();

    let app_config = flutter_rust_embedder::application::AppConfig {
        asset_dir: std::path::PathBuf::from(
            "C:/workspace/rusty/build/windows/x64/runner/Debug/data",
        ),
        flutter_engine_path: std::path::PathBuf::from(
            "C:/libs/flutter/engine/src/out/host_debug/flutter_engine.dll",
        ),
    };

    let instance_desc = wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN | wgpu::Backends::METAL,
        flags: wgpu::InstanceFlags::default(),
        backend_options: wgpu::BackendOptions::default(),
    };
    let instance = wgpu::Instance::new(&instance_desc);

    info!("Surface created");

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .unwrap();

    let mut app = flutter_rust_embedder::application::App::new(
        app_config,
        GPUContext {
            instance: instance,
            adapter: adapter,
            device: device,
            queue: queue,
        },
    );
    app.run()
}
