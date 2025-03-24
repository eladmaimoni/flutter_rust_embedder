use std::ffi::CString;

use crate::{
    flutter_embedder::{
        FlutterBackingStore, FlutterBackingStoreConfig, FlutterCompositor, FlutterLayer,
        FlutterRendererConfig, FlutterRendererType_kVulkan, FlutterVulkanRendererConfig,
    },
    utils::as_void_ptr,
};
use ash::vk::Handle;
use tracing::instrument;
struct RawVkInstance {
    instance: *mut ::core::ffi::c_void,
    version: u32,
    extensions: Vec<CString>,
}

struct RawVkDeviceAndQueue {
    device: *mut ::core::ffi::c_void,
    physical_device: *mut ::core::ffi::c_void,
    queue: *mut ::core::ffi::c_void,
    queue_family_index: u32,
    extensions: Vec<CString>,
}
//-> (*const std::ffi::c_void, u32, Vec<CString>)
fn extract_raw_vk_instance(instance: &wgpu::Instance) -> Option<RawVkInstance> {
    let res = unsafe {
        instance
            .as_hal::<wgpu_hal::api::Vulkan>()
            .map(|instance| -> RawVkInstance {
                let raw_instance = instance.shared_instance().raw_instance();
                let raw_handle = raw_instance.handle().as_raw();
                let raw_void = raw_handle as *mut ::core::ffi::c_void;

                let extensions = instance
                    .shared_instance()
                    .extensions()
                    .into_iter()
                    .map(|&s| s.to_owned())
                    .collect::<Vec<CString>>();

                RawVkInstance {
                    instance: raw_void,
                    version: 0,
                    extensions: extensions,
                }
            })
    };
    res
}

fn extract_raw_vk_device(device: &wgpu::Device) -> Option<RawVkDeviceAndQueue> {
    unsafe {
        device.as_hal::<wgpu_hal::api::Vulkan, _, Option<RawVkDeviceAndQueue>>(|device| {
            device.map(|device| {
                let extensions = device
                    .enabled_device_extensions()
                    .into_iter()
                    .map(|&s| s.to_owned())
                    .collect::<Vec<CString>>();
                // (raw_void, 0, extensions)
                RawVkDeviceAndQueue {
                    device: device.raw_device().handle().as_raw() as *mut ::core::ffi::c_void,
                    physical_device: device.raw_physical_device().as_raw()
                        as *mut ::core::ffi::c_void,
                    queue: device.raw_queue().as_raw() as *mut ::core::ffi::c_void,
                    queue_family_index: device.queue_family_index(),
                    extensions: extensions,
                }
            })
        })
    }
}

fn create_flutter_renderer_config(
    instance: &wgpu::Instance,
    device: &wgpu::Device,
) -> FlutterRendererConfig {
    let raw_instance = extract_raw_vk_instance(&instance).unwrap();
    let raw_device = extract_raw_vk_device(&device).unwrap();

    let mut enabled_device_extensions: Vec<*const std::ffi::c_char> = raw_device
        .extensions
        .iter()
        .map(|ext| ext.as_ptr())
        .collect();
    let mut enabled_instance_extensions: Vec<*const std::ffi::c_char> = raw_instance
        .extensions
        .iter()
        .map(|ext| ext.as_ptr())
        .collect();

    // let vulkan_config = FlutterVulkanRendererConfig {
    //     struct_size: size_of::<FlutterVulkanRendererConfig>(),
    //     version: raw_instance.version,
    //     instance: raw_instance.instance,
    //     physical_device: raw_device.physical_device,
    //     device: raw_device.device,
    //     queue_family_index: raw_device.queue_family_index,
    //     queue: raw_device.queue,
    //     enabled_instance_extension_count: raw_instance.extensions.len(),
    //     enabled_instance_extensions: enabled_instance_extensions.as_mut_ptr(),
    //     enabled_device_extension_count: raw_device.extensions.len(),
    //     enabled_device_extensions: enabled_device_extensions.as_mut_ptr(),
    //     get_instance_proc_address_callback: None,
    //     get_next_image_callback: None,
    //     present_image_callback: None,
    // };

    let mut config = FlutterRendererConfig::default();
    config.type_ = FlutterRendererType_kVulkan;
    let vk = unsafe { config.__bindgen_anon_1.vulkan.as_mut() };
    vk.struct_size = size_of::<FlutterVulkanRendererConfig>();
    vk.version = raw_instance.version;
    vk.instance = raw_instance.instance;
    vk.physical_device = raw_device.physical_device;
    vk.device = raw_device.device;
    vk.queue_family_index = raw_device.queue_family_index;
    vk.queue = raw_device.queue;
    vk.enabled_instance_extension_count = raw_instance.extensions.len();
    vk.enabled_instance_extensions = enabled_instance_extensions.as_mut_ptr();
    vk.enabled_device_extension_count = raw_device.extensions.len();
    vk.enabled_device_extensions = enabled_device_extensions.as_mut_ptr();
    vk.get_instance_proc_address_callback = None;
    vk.get_next_image_callback = None;
    vk.present_image_callback = None;

    config
}

#[derive(Debug)]
pub struct Compositor {
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    surface_size: winit::dpi::PhysicalSize<u32>,
    present_surface_texture: Option<wgpu::SurfaceTexture>,
}

impl Compositor {
    pub fn new(
        instance: wgpu::Instance,
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: wgpu::Surface<'static>,
        surface_format: wgpu::TextureFormat,
        surface_size: winit::dpi::PhysicalSize<u32>,
    ) -> Self {
        let mut instance = Compositor {
            instance: instance,
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

    pub fn get_flutter_compositor(&mut self) -> FlutterCompositor {
        FlutterCompositor {
            struct_size: size_of::<FlutterCompositor>(),
            user_data: as_void_ptr(self),
            create_backing_store_callback: Some(Self::create_backing_store_callback),
            collect_backing_store_callback: Some(Self::collect_backing_store_callback),
            present_layers_callback: Some(Self::present_layers_callback),
            present_view_callback: None,
            avoid_backing_store_cache: false,
        }
    }

    pub fn get_flutter_renderer_config(&mut self) -> FlutterRendererConfig {
        return create_flutter_renderer_config(&self.instance, &self.device);
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
