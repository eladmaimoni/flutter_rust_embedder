use core::error;
// use std::error;
use std::ffi::CString;
use std::path::PathBuf;
use std::pin::Pin;
// use std::fmt::Error;
use crate::composition::Compositor;
use crate::flutter_embedder;
use crate::utils::as_void_ptr;
use ash::vk::Handle;
use flutter_embedder::*;
use libloading::Library;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info, instrument, warn};
// use wgpu::{Adapter, Instance};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

pub type PinBox<T> = Pin<Box<T>>;

#[derive(Clone, Debug)]
pub struct GPUContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

#[derive(Clone, Debug)]
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
    #[error("Path not found path: {0}")]
    PathNoFound(PathBuf),

    #[error("Failed to create C string from path: {0}")]
    CStringCreation(#[from] std::ffi::NulError),

    #[error("Failed to load Flutter engine: {0}")]
    FlutterEngineLoad(#[from] std::io::Error),

    #[error("Failed to load Flutter engine: {0}")]
    FlutterEngineSymbol(#[from] libloading::Error),

    #[error("Failed to load Flutter engine proc table function: {0}")]
    FlutterEngineProcTable(String),

    #[error("Flutter engine API error {0}")]
    FlutterEngineError(FlutterEngineResult),

    #[error("Failed to start event loop: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),
}

#[derive(Debug)]
struct AppWindowSession {
    config: AppConfig,
    _flutter_engine_lib: Library,
    engine: flutter_embedder::FlutterEngineProcTable,
    engine_handle: FlutterEngine,
    window: Arc<Window>,
    compositor: Compositor,
}

impl AppWindowSession {
    fn new(
        config: AppConfig,
        window: Arc<Window>,
        gpu_context: GPUContext,
    ) -> Result<Self, AppError> {
        let assets_path = config.asset_dir.join("flutter_assets");
        let icu_data_path = config.asset_dir.join("icudtl.dat");

        if !assets_path.exists() {
            error!("Assets path does not exist: {:?}", assets_path);
            return Err(AppError::PathNoFound(assets_path));
        }

        if !icu_data_path.exists() {
            error!("ICU data path does not exist: {:?}", icu_data_path);
            return Err(AppError::PathNoFound(icu_data_path));
        }
        if !config.flutter_engine_path.exists() {
            error!(
                "Engine not found at path: {}",
                config.flutter_engine_path.display()
            );
            return Err(AppError::PathNoFound(config.flutter_engine_path.clone()));
        }
        let engine_lib = unsafe { Library::new(&config.flutter_engine_path)? };
        let flutter_engine_get_proc_addresses = unsafe {
            engine_lib.get::<fn(*mut FlutterEngineProcTable) -> FlutterEngineResult>(
                b"FlutterEngineGetProcAddresses\0",
            )?
        };

        let mut engine = flutter_embedder::FlutterEngineProcTable::default();
        engine.struct_size = std::mem::size_of::<flutter_embedder::FlutterEngineProcTable>();
        let res = flutter_engine_get_proc_addresses(&mut engine as *mut _ as _);
        if res != FlutterEngineResult_kSuccess {
            error!("Failed to get Flutter engine proc addresses: {:?}", res);
            return Err(AppError::FlutterEngineError(res));
        }

        let instance = gpu_context.instance;
        let device = gpu_context.device;
        let queue = gpu_context.queue;
        let surface = instance.create_surface(window.clone()).unwrap();

        let cap = surface.get_capabilities(&gpu_context.adapter);
        let surface_format = cap.formats[0];

        let initial_size = window.inner_size();

        window.request_redraw();

        Ok(Self {
            config: config,
            window: window,
            _flutter_engine_lib: engine_lib,
            engine: engine,
            engine_handle: std::ptr::null_mut(),
            compositor: crate::composition::Compositor::new(
                instance,
                device,
                queue,
                surface,
                surface_format,
                initial_size,
            ),
        })
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

    pub fn initialize(&mut self) -> Result<(), AppError> {
        // let flutter_renderer_config = create_flutter_renderer_config(&instance, &device);
        let asset_path_str = CString::new(self.config.asset_dir.to_str().unwrap())?;
        let icu_data_path_str = CString::new(self.config.asset_dir.to_str().unwrap())?;
        let mut project_args = flutter_embedder::FlutterProjectArgs::default();
        project_args.struct_size = std::mem::size_of::<flutter_embedder::FlutterProjectArgs>();
        project_args.assets_path = asset_path_str.as_ptr();
        project_args.icu_data_path = icu_data_path_str.as_ptr();
        project_args.platform_message_callback = None;
        project_args.vm_snapshot_data = std::ptr::null_mut();
        project_args.vm_snapshot_data_size = 0;
        project_args.vm_snapshot_instructions = std::ptr::null_mut();
        project_args.vm_snapshot_instructions_size = 0;
        project_args.isolate_snapshot_data = std::ptr::null_mut();
        project_args.isolate_snapshot_data_size = 0;
        project_args.isolate_snapshot_instructions = std::ptr::null_mut();
        project_args.isolate_snapshot_instructions_size = 0;
        project_args.root_isolate_create_callback = None;
        project_args.update_semantics_callback = None;
        project_args.log_message_callback = None;
        let mut render_config = flutter_embedder::FlutterRendererConfig::default();
        // self.compositor.c
        let mut engine_handle: FlutterEngine = std::ptr::null_mut();

        if let Some(initialize) = self.engine.Initialize {
            let res = unsafe {
                initialize(
                    FLUTTER_ENGINE_VERSION as usize,
                    &mut render_config as _,
                    &mut project_args as _,
                    as_void_ptr(self),
                    &mut engine_handle as _,
                )
            };
            info!("FlutterEngineInitialize returned: {}", res);
        } else {
            error!("FlutterEngineInitialize not found");
            return Err(AppError::FlutterEngineProcTable(
                "FlutterEngineInitialize".to_string(),
            ));
        }
        Ok(())
    }

    extern "C" fn platform_message_callback(
        message: *const FlutterPlatformMessage,
        user_data: *mut std::ffi::c_void,
    ) {
        let app = user_data as *mut AppWindowSession;
        let message = unsafe { &*message };
        unsafe {
            if let Some(app) = app.as_mut() {
                app.handle_platform_message(message);
            }
        }
    }

    fn handle_platform_message(&mut self, message: &FlutterPlatformMessage) {
        info!("Platform message received: {:?}", message);
    }
}

pub struct App {
    config: AppConfig,
    gpu_context: GPUContext,
    window_session: Option<Box<AppWindowSession>>,
}

impl App {
    pub fn new(config: AppConfig, gpu_context: GPUContext) -> Self {
        Self {
            config: config,
            gpu_context: gpu_context,
            window_session: None,
        }
    }

    pub fn run(&mut self) -> Result<(), AppError> {
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
                let window_session = AppWindowSession::new(
                    self.config.clone(),
                    Arc::new(window),
                    self.gpu_context.clone(),
                )
                .unwrap();

                let pinned_session = Box::new(window_session);

                self.window_session = Some(pinned_session);
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
