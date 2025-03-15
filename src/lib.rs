use tracing::{debug, error, info, warn};
use winit::application::ApplicationHandler;
use winit::error;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

mod logging;
mod windowing;

struct WindowGPUState<'window> {
    surface: wgpu::Surface<'window>,
}

#[derive(Default, Debug)]
struct App {
    window: Option<Window>,
}

impl App {
    fn new() -> Self {
        Self::default()
    }
}
impl App {
    fn on_new_window(&mut self, window: &Window) {
        info!("New window created");

        let instance_desc = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
        };
        let instance = wgpu::Instance::new(&instance_desc);

        let surface = instance.create_surface(window).unwrap();

        info!("Surface created");
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default();
        let window = event_loop.create_window(window_attributes);

        match window {
            Ok(window) => {
                self.on_new_window(&window);
                self.window = Some(window);
            }
            Err(error) => {
                error!("Failed to create window {:?}", error);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Early return if we don't have a window or if the event is for a different window
        if self
            .window
            .as_ref()
            .map_or(true, |window| window.id() != window_id)
        {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                info!("Window closed");
            }
            _ => {
                debug!("Window event: {:?}", event);
            }
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    logging::init_tracing();
    let event_loop = EventLoop::new()?;
    let mut app = App::default();

    // For alternative loop run options see `pump_events` and `run_on_demand` examples.
    event_loop.run_app(&mut app).map_err(Into::into)
}
