use std::ffi::CString;

use ash::vk::Handle;
use tracing::{debug, instrument};

use crate::{
    application::get_instance_proc_address_callback,
    flutter_embedder::{
        FlutterFrameInfo, FlutterRendererConfig, FlutterRendererType_kVulkan, FlutterVulkanImage,
        FlutterVulkanInstanceHandle, FlutterVulkanRendererConfig,
    },
    utils::as_void_ptr,
};
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

pub struct FlutterRendererConfigWrapper {
    pub config: FlutterRendererConfig,
    _owned_instance_extensions: Vec<*const std::ffi::c_char>,
    _owned_device_extensions: Vec<*const std::ffi::c_char>,
}

pub fn create_flutter_renderer_config(
    instance: &wgpu::Instance,
    device: &wgpu::Device,
) -> FlutterRendererConfigWrapper {
    let raw_instance = extract_raw_vk_instance(&instance).unwrap();
    let raw_device = extract_raw_vk_device(&device).unwrap();

    debug!("instance extensions {:?}", raw_instance.extensions);
    debug!("device extensions {:?}", raw_device.extensions);
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
    vk.get_instance_proc_address_callback = Some(get_instance_proc_address_callback);
    vk.get_next_image_callback = Some(get_next_image_callback);
    vk.present_image_callback = Some(present_image_callback);

    FlutterRendererConfigWrapper {
        config: config,
        _owned_instance_extensions: enabled_instance_extensions,
        _owned_device_extensions: enabled_device_extensions,
    }
}

extern "C" fn present_image_callback(
    arg1: *mut ::core::ffi::c_void,
    arg2: *const FlutterVulkanImage,
) -> bool {
    false
}

extern "C" fn get_next_image_callback(
    arg1: *mut ::core::ffi::c_void,
    arg2: *const FlutterFrameInfo,
) -> FlutterVulkanImage {
    FlutterVulkanImage::default()
}
