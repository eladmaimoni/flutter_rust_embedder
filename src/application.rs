use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

use crate::composition::Compositor;

#[derive(Debug)]
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

        let compositor = crate::composition::Compositor::new(
            device,
            queue,
            surface,
            surface_format,
            initial_size,
        );

        window.request_redraw();

        Self {
            window: window,
            compositor: compositor,
        }
    }

    #[instrument(level = "trace", skip_all)]
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
                self.window.pre_present_notify();
                self.compositor.present();
            }
            _ => {
                debug!("Window event: {:?}", event);
            }
        }
        false
    }
}

#[derive(Default)]
pub struct App {
    window_session: Option<AppWindowSession>,
}

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
        let Some(window_session) = self
            .window_session
            .as_mut()
            .filter(|window_session| window_session.window.id() == window_id)
        else {
            return;
        };

        let should_close = window_session.handle_window_event(event);

        if should_close {
            event_loop.exit();
        }
    }
}
