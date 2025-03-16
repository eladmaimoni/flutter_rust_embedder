use winit::event_loop::EventLoop;

mod application;
mod composition;
mod flutter_embedder;
mod tracing_integration;
mod windowing;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _lib = flutter_embedder::load_flutter_engine(flutter_embedder::FlutterEngineMode::JIT)?;

    let event_loop = EventLoop::new()?;
    let mut app = application::App::default();

    // For alternative loop run options see `pump_events` and `run_on_demand` examples.
    event_loop.run_app(&mut app).map_err(Into::into)
}
