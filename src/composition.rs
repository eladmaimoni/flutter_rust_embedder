use std::pin::Pin;

use crate::{
    flutter_embedder::{
        FlutterBackingStore, FlutterBackingStoreConfig, FlutterCompositor, FlutterLayer,
        FlutterRendererConfig,
    },
    utils::as_void_ptr,
};
use tracing::instrument;

#[derive(Debug)]
struct CompositorInner {}

impl CompositorInner {}

#[derive(Debug)]
pub struct Compositor {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    surface_size: winit::dpi::PhysicalSize<u32>,
    present_surface_texture: Option<wgpu::SurfaceTexture>,
}

impl Compositor {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: wgpu::Surface<'static>,
        surface_format: wgpu::TextureFormat,
        surface_size: winit::dpi::PhysicalSize<u32>,
    ) -> Self {
        let mut instance = Compositor {
            device: device,
            queue: queue,
            surface: surface,
            surface_format: surface_format,
            surface_size: surface_size,
            present_surface_texture: None,
        };

        instance.resize(surface_size);
        instance
    }

    #[instrument(level = "info", skip(self))]
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.surface_size = new_size;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.surface_size.width,
            height: self.surface_size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };

        self.surface.configure(&self.device, &surface_config);
    }
    #[instrument(level = "debug", skip(self))]
    pub fn render(&mut self) {
        let Ok(surface_texture) = self.surface.get_current_texture() else {
            return;
        };

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // If you wanted to call any drawing commands, they would go here.

        // End the renderpass.
        drop(renderpass);

        self.queue.submit([encoder.finish()]);

        self.present_surface_texture = Some(surface_texture);
    }

    pub fn present(&mut self) {
        if let Some(surface_texture) = self.present_surface_texture.take() {
            surface_texture.present();
        }
    }

    pub fn get_flutter_compositor(self: Pin<&mut Self>) -> FlutterCompositor {
        FlutterCompositor {
            struct_size: size_of::<FlutterCompositor>(),
            user_data: as_void_ptr(unsafe { self.get_unchecked_mut() }),
            create_backing_store_callback: Some(Self::create_backing_store_callback),
            collect_backing_store_callback: Some(Self::collect_backing_store_callback),
            present_layers_callback: Some(Self::present_layers_callback),
            present_view_callback: None,
            avoid_backing_store_cache: false,
        }
    }

    extern "C" fn present_layers_callback(
        layers: *mut *const FlutterLayer,
        layers_count: usize,
        user_data: *mut ::core::ffi::c_void,
    ) -> bool {
        true
    }

    extern "C" fn create_backing_store_callback(
        config: *const FlutterBackingStoreConfig,
        backing_store_out: *mut FlutterBackingStore,
        user_data: *mut ::core::ffi::c_void,
    ) -> bool {
        let compositor = unsafe { &*(user_data as *const Compositor) };

        true
    }

    extern "C" fn collect_backing_store_callback(
        renderer: *const FlutterBackingStore,
        user_data: *mut ::core::ffi::c_void,
    ) -> bool {
        true
    }
}
