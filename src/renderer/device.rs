use ash::extensions::khr::Surface;
use ash::{vk, Instance};
use log::{debug, warn};
use std::ffi::CStr;

#[derive(Clone)]
pub struct DeviceInfo {
    pub physical_device: vk::PhysicalDevice,
    pub queue_family: u32,
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
        let name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
            .to_str()
            .unwrap()
            .to_owned();

        // The GPU has to have a graphics queue. Otherwise there's no way to do any rendering
        // operations, so this must be some weird compute-only accelerator or something.
        let Some(queue_family) = find_suitable_queue(&queue_families, surface_extension, device, surface) else {
            warn!("physical device rejected, no suitable queue, \x1B[1mname\x1B[0m: {name}");
            continue;
        };

        // Let's just select the first GPU for now. Linux seems to sort them by itself, I should
        // think more about selection later.
        debug!("physical device selected, \x1B[1mname\x1B[0m: {name}");
        return DeviceInfo {
            physical_device: device,
            queue_family,
        };
    }

    panic!("gpu not found");
}

fn find_suitable_queue(
    queues: &[vk::QueueFamilyProperties],
    surface_extension: &Surface,
    device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Option<u32> {
    // Some devices have separate graphics and present queues, but let's ignore them for now.
    for (index, family) in queues.iter().enumerate() {
        let index = index as u32;
        let supports_graphics = family.queue_flags.contains(vk::QueueFlags::GRAPHICS);
        let supports_present = unsafe {
            surface_extension.get_physical_device_surface_support(device, index, surface)
        }
        .unwrap();
        if supports_graphics && supports_present {
            return Some(index);
        }
    }
    None
}
