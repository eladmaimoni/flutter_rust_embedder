use std::sync::Arc;
use tracing::{debug, error, info, warn};
use winit::application::ApplicationHandler;
use winit::error;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

mod composition;
mod tracing_integration;
mod windowing;

use composition::Compositor;

struct AppWindowSession {
    window: Arc<Window>,
    compositor: Compositor,
}

impl AppWindowSession {
    async fn new(window: Arc<Window>) -> Self {
        let instance_desc = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
        };
        let instance = wgpu::Instance::new(&instance_desc);

        let surface = instance.create_surface(window.clone()).unwrap();

        info!("Surface created");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let initial_size = window.inner_size();

        let compositor = Compositor::new(device, queue, surface, surface_format, initial_size);

        // let compositor = Compositor {
        //     surface: surface,
        //     surface_format: surface_format,
        //     surface_size: initial_size,
        //     device: device,
        //     queue: queue,
        // };

        window.request_redraw();

        Self {
            window: window,
            compositor: compositor,
        }
    }

    fn handle_window_event(&mut self, event: WindowEvent) -> bool {
        match event {
            WindowEvent::CloseRequested => {
                info!("Window closed");
                return true;
            }
            WindowEvent::Resized(new_size) => {
                self.compositor.resize(new_size);
            }
            WindowEvent::RedrawRequested => {
                self.compositor.render();
            }
            _ => {
                debug!("Window event: {:?}", event);
            }
        }
        false
    }
}

#[derive(Default)]
struct App {
    window_session: Option<AppWindowSession>,
}

impl App {
    fn new() -> Self {
        Self::default()
    }
}
impl App {}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default();
        let window = event_loop.create_window(window_attributes);

        match window {
            Ok(window) => {
                info!("New window created");
                let window_session = pollster::block_on(AppWindowSession::new(Arc::new(window)));
                self.window_session = Some(window_session);
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
        // let Some(window_session) = self.window_session.as_ref().and_then(|window_session| {
        //     if window_session.window.id() == window_id {
        //         Some(window_session)
        //     } else {
        //         None
        //     }
        // }) else {
        //     return;
        // };

        let Some(window_session) = self
            .window_session
            .as_mut()
            .filter(|window_session| window_session.window.id() == window_id)
        else {
            return;
        };

        // let Some(window_session) = self.window_session.as_ref() else {
        //     return;
        // };
        // if self.window_session.as_ref().map_or(true, |window_session| {
        //     window_session.window.id() != window_id
        // }) {
        //     return;
        // }

        // match event {
        //     WindowEvent::CloseRequested => {
        //         event_loop.exit();
        //         info!("Window closed");
        //     }
        //     _ => {
        //         debug!("Window event: {:?}", event);
        //     }
        // }
        let should_close = window_session.handle_window_event(event);

        if should_close {
            event_loop.exit();
        }
        // Early return if we don't have a window or if the event is for a different window
        // if self
        //     .window
        //     .as_ref()
        //     .map_or(true, |window| window.id() != window_id)
        // {
        //     return;
        // }

        // match event {
        //     WindowEvent::CloseRequested => {
        //         event_loop.exit();
        //         info!("Window closed");
        //     }
        //     _ => {
        //         debug!("Window event: {:?}", event);
        //     }
        // }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_integration::init_tracing();
    let event_loop = EventLoop::new()?;
    let mut app = App::default();

    // For alternative loop run options see `pump_events` and `run_on_demand` examples.
    event_loop.run_app(&mut app).map_err(Into::into)
}
