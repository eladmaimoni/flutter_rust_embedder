// use std::error;
use std::ffi::CString;
use std::path::PathBuf;
// use std::fmt::Error;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info, instrument, warn};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::composition::Compositor;
use crate::flutter_embedder;

pub struct AppConfig {
    /// The directory where the flutter assets are located.
    /// On Windows, this is typically a folder named 'data' with a 'flutter_assets' subfolder.
    pub asset_dir: std::path::PathBuf,
    /// The path to the Flutter engine shared library.
    /// On Windows, this is typically a file named 'flutter_engine.dll'.
    /// The engine version should match the flutter
    pub flutter_engine_path: std::path::PathBuf,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("Failed to create C string from path: {0}")]
    CStringCreation(#[from] std::ffi::NulError),

    #[error("Failed to load Flutter engine: {0}")]
    FlutterEngineLoad(#[from] std::io::Error),

    #[error("Failed to load Flutter engine: {0}")]
    FlutterEngineSymbol(#[from] libloading::Error),

    #[error("Failed to start event loop: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),
}

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

pub struct App {
    config: AppConfig,
    window_session: Option<AppWindowSession>,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: config,
            window_session: None,
        }
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        let mut project_args = flutter_embedder::FlutterProjectArgs::default();

        let assets_path = self.config.asset_dir.join("flutter_assets");
        let icu_data_path = self.config.asset_dir.join("icudtl.dat");

        if !assets_path.exists() {
            error!("Assets path does not exist: {:?}", assets_path);
            return Err(AppError::InvalidPath(assets_path));
        }

        if !icu_data_path.exists() {
            error!("ICU data path does not exist: {:?}", icu_data_path);
            return Err(AppError::InvalidPath(icu_data_path));
        }

        // let asset_path_c = CString::new(assets_path.to_str().ok_or_else(err)

        // project_args.struct_size =
        //     std::mem::size_of::<flutter_embedder::FlutterProjectArgs>() as usize;
        // project_args.assets_path = assets_path.as_ptr();
        // project_args.icu_data_path = icu_data_path.as_ptr();
        let event_loop = EventLoop::new()?;
        event_loop.run_app(self)?;
        Ok(())
    }
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
