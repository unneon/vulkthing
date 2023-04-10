use ash::extensions::khr::{Surface, Swapchain};
use ash::{vk, Instance};
use log::{debug, warn};
use std::ffi::CStr;

#[derive(Clone)]
pub struct DeviceInfo {
    pub physical_device: vk::PhysicalDevice,
    pub queue_families: QueueFamilies,
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub surface_formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

#[derive(Clone)]
pub struct QueueFamilies {
    pub graphics: u32,
    pub present: u32,
}

pub fn select_device(
    instance: &Instance,
    surface_extension: &Surface,
    surface: vk::SurfaceKHR,
) -> DeviceInfo {
    // Select the GPU. For now, just select the first discrete GPU with graphics support. Later,
    // this should react better to iGPU, dGPU and iGPU+dGPU setups. In more complex setups, it would
    // be neat if you could start the game on any GPU, display a choice to the user and seamlessly
    // switch to a new physical device.
    for device in unsafe { instance.enumerate_physical_devices() }.unwrap() {
        let properties = unsafe { instance.get_physical_device_properties(device) };
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(device) };
        let features = unsafe { instance.get_physical_device_features(device) };
        let extensions = unsafe { instance.enumerate_device_extension_properties(device) }.unwrap();
        let name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
            .to_str()
            .unwrap()
            .to_owned();

        // The GPU has to have a graphics queue. Otherwise there's no way to do any rendering
        // operations, so this must be some weird compute-only accelerator or something. This also
        // checks whether there is a present queue. This could be worked around using two separate
        // GPUs (or just one for headless benchmarking), but the OS should take care of handling
        // this sort of stuff between devices, probably?
        let Some(graphics) = find_graphics_queue(&queue_families) else {
            warn!("physical device rejected, no graphics queue, \x1B[1mname\x1B[0m: {name}");
            continue;
        };

        let Some(present) = find_present_queue(&queue_families, surface_extension, device, surface) else {
            warn!("physical device rejected, no present queue, \x1B[1mname\x1B[0m: {name}");
            continue;
        };

        if features.sampler_anisotropy == 0 {
            warn!("physical device rejected, no sampler anisotropy feature, \x1B[1mname\x1B[0m: {name}");
            continue;
        }

        // Check whether the GPU supports the swapchain extension. This should be implied by the
        // presence of the present queue, but we can check this explicitly.
        if !has_swapchain_extension(&extensions) {
            warn!("physical device rejected, no swapchain extension, \x1B[1mname\x1B[0m: {name}");
            continue;
        }

        // This queries some more details about swapchain support, and apparently this requires the
        // earlier extension check in order to be correct (not crash?). Also there shouldn't be
        // devices that support swapchains but no formats or present modes, but let's check anyway
        // because the tutorial does.
        let surface_capabilities =
            unsafe { surface_extension.get_physical_device_surface_capabilities(device, surface) }
                .unwrap();
        let surface_formats =
            unsafe { surface_extension.get_physical_device_surface_formats(device, surface) }
                .unwrap();
        let present_modes =
            unsafe { surface_extension.get_physical_device_surface_present_modes(device, surface) }
                .unwrap();
        if surface_formats.is_empty() || present_modes.is_empty() {
            warn!("physical device rejected, unsuitable swapchain, \x1B[1mname\x1B[0m: {name}");
            continue;
        }

        // Let's just select the first GPU for now. Linux seems to sort them by itself, I should
        // think more about selection later.
        debug!("physical device selected, \x1B[1mname\x1B[0m: {name}");
        return DeviceInfo {
            physical_device: device,
            queue_families: QueueFamilies { graphics, present },
            surface_capabilities,
            surface_formats,
            present_modes,
        };
    }

    panic!("gpu not found");
}

fn find_graphics_queue(queues: &[vk::QueueFamilyProperties]) -> Option<u32> {
    find_queue(queues, |_, q| {
        q.queue_flags.contains(vk::QueueFlags::GRAPHICS)
    })
}

fn find_present_queue(
    queues: &[vk::QueueFamilyProperties],
    surface_extension: &Surface,
    device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Option<u32> {
    find_queue(queues, |i, _| {
        unsafe { surface_extension.get_physical_device_surface_support(device, i, surface) }
            .unwrap()
    })
}

fn find_queue(
    queues: &[vk::QueueFamilyProperties],
    p: impl Fn(u32, &vk::QueueFamilyProperties) -> bool,
) -> Option<u32> {
    // Find the first queue that supports a given operation and return it. Not sure what to do when
    // there are multiple queues that support an operation? Also, graphics queue being distinct from
    // present queue is supposed to be somewhat rare, so not sure where can I test it.
    for (index, queue) in queues.iter().enumerate() {
        let index = index as u32;
        if p(index, queue) {
            return Some(index);
        }
    }
    None
}

fn has_swapchain_extension(extensions: &[vk::ExtensionProperties]) -> bool {
    extensions.iter().any(|ext| {
        let ext_name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
        ext_name == Swapchain::name()
    })
}
