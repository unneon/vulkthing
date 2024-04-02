use crate::renderer::util::{create_image_view, Dev, ImageResources};
use ash::extensions::khr::Swapchain as SwapchainKhr;
use ash::vk;
use winit::dpi::PhysicalSize;

pub struct Swapchain {
    pub handle: vk::SwapchainKHR,
    pub format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
    pub images: Vec<ImageResources>,
}

impl Swapchain {
    pub fn cleanup(&self, dev: &Dev) {
        let swapchain_ext = SwapchainKhr::new(&dev.instance, dev);
        for image in &self.images {
            unsafe { dev.destroy_image_view(image.view, None) };
        }
        unsafe { swapchain_ext.destroy_swapchain(self.handle, None) };
    }
}

pub fn create_swapchain(
    surface: vk::SurfaceKHR,
    window_size: PhysicalSize<u32>,
    dev: &Dev,
) -> Swapchain {
    let capabilities = unsafe {
        dev.surface_ext
            .get_physical_device_surface_capabilities(dev.physical, surface)
    }
    .unwrap();
    let formats = {
        unsafe {
            dev.surface_ext
                .get_physical_device_surface_formats(dev.physical, surface)
        }
        .unwrap()
    };
    let image_count = select_image_count(capabilities);
    let format = select_format(&formats);
    let extent = select_extent(capabilities, window_size);
    let handle = create_handle(surface, image_count, format, extent, capabilities, dev);
    let images = create_pseudo_image_resources(handle, format.format, dev);
    Swapchain {
        handle,
        format,
        extent,
        images,
    }
}

fn select_image_count(capabilities: vk::SurfaceCapabilitiesKHR) -> usize {
    // Use triple buffering, even if the platform allows to only use double buffering. The Vulkan
    // tutorial recommends setting this to min_image_count + 1 to prevent waiting for the image due
    // to driver overhead, but I think that after triple buffering, adding more images shouldn't be
    // able to fix any internal driver problems. It's also not covered by the Khronos
    // recommendation.
    // https://github.com/KhronosGroup/Vulkan-Samples
    let no_image_limit = capabilities.max_image_count == 0;
    let preferred_image_count = capabilities.min_image_count.max(3) as usize;
    if no_image_limit {
        preferred_image_count
    } else {
        preferred_image_count.min(capabilities.max_image_count as usize)
    }
}

fn select_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    for format in formats {
        // There is no display HDR support yet, so let's select the normal SRGB color space.
        let good_color_space = format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR;
        // Picking a format that is SRGB rather than UNORM means the last shader has to work in
        // linear space and NOT do a gamma correction. The conversion from linear space to SRGB
        // (sometimes called gamma correction) done by the hardware is faster and better anyway, the
        // simple power formula does not actually follow the SRGB EOTF curve accurately. Also, I've
        // seen both BGRA and RGBA on common hardware.
        let good_format = format.format == vk::Format::R8G8B8A8_SRGB
            || format.format == vk::Format::B8G8R8A8_SRGB;
        if good_color_space && good_format {
            return *format;
        }
    }
    // Let's error out instead of (approach from the tutorial) just picking the first returned
    // format and inevitably displaying wrong colors.
    panic!("surface doesn't support SRGB color space with a desired format");
}

fn select_extent(
    capabilities: vk::SurfaceCapabilitiesKHR,
    window_size: PhysicalSize<u32>,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        return capabilities.current_extent;
    }
    vk::Extent2D {
        width: window_size.width.clamp(
            capabilities.min_image_extent.width,
            capabilities.max_image_extent.width,
        ),
        height: window_size.height.clamp(
            capabilities.min_image_extent.height,
            capabilities.max_image_extent.height,
        ),
    }
}

fn create_handle(
    surface: vk::SurfaceKHR,
    image_count: usize,
    format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    capabilities: vk::SurfaceCapabilitiesKHR,
    dev: &Dev,
) -> vk::SwapchainKHR {
    let create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(image_count as u32)
        .image_format(format.format)
        .image_color_space(format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .pre_transform(capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(vk::PresentModeKHR::MAILBOX)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null());
    unsafe { dev.swapchain_ext.create_swapchain(&create_info, None) }.unwrap()
}

fn create_pseudo_image_resources(
    swapchain: vk::SwapchainKHR,
    format: vk::Format,
    dev: &Dev,
) -> Vec<ImageResources> {
    let images = unsafe { dev.swapchain_ext.get_swapchain_images(swapchain) }.unwrap();
    let mut image_views = Vec::new();
    for image in images {
        let view = create_image_view(image, format, vk::ImageAspectFlags::COLOR, dev);
        image_views.push(ImageResources {
            image,
            memory: vk::DeviceMemory::null(),
            view,
        });
    }
    image_views
}
