use crate::renderer::util::{create_image_view, Dev};
use ash::extensions::khr::{Surface, Swapchain as SwapchainKhr};
use ash::vk;
use log::warn;
use winit::dpi::PhysicalSize;

pub struct Swapchain {
    pub handle: vk::SwapchainKHR,
    pub format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
    pub image_views: Vec<vk::ImageView>,
}

impl Swapchain {
    pub fn cleanup(&self, dev: &Dev) {
        let swapchain_ext = SwapchainKhr::new(&dev.instance, dev);
        for image_view in &self.image_views {
            unsafe { dev.destroy_image_view(*image_view, None) };
        }
        unsafe { swapchain_ext.destroy_swapchain(self.handle, None) };
    }
}

pub fn create_swapchain(
    surface: vk::SurfaceKHR,
    window_size: PhysicalSize<u32>,
    dev: &Dev,
    surface_ext: &Surface,
    swapchain_ext: &SwapchainKhr,
) -> Swapchain {
    let capabilities =
        unsafe { surface_ext.get_physical_device_surface_capabilities(dev.physical, surface) }
            .unwrap();
    let formats = {
        unsafe { surface_ext.get_physical_device_surface_formats(dev.physical, surface) }.unwrap()
    };
    let image_count = select_image_count(capabilities);
    let format = select_format(&formats);
    let extent = select_extent(capabilities, window_size);
    let handle = create_handle(
        surface,
        image_count,
        format,
        extent,
        capabilities,
        swapchain_ext,
    );
    let image_views = create_image_views(handle, format.format, swapchain_ext, dev);
    Swapchain {
        handle,
        format,
        extent,
        image_views,
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
    // Let's select something with a SRGB color space, so that at least colors look the way they are
    // intended to. The actual format is determined by whatever the driver presents first; the
    // tutorial looks for B8G8R8A8_SRGB specifically, but I don't see the point.
    for format in formats {
        if format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
            return *format;
        }
    }
    let format = formats[0];
    warn!(
        "surface doesn't support srgb color space, \x1B[1mfallback\x1B[0m: {:?} {:?}",
        format.format, format.color_space
    );
    format
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
    swapchain_ext: &SwapchainKhr,
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
    unsafe { swapchain_ext.create_swapchain(&create_info, None) }.unwrap()
}

fn create_image_views(
    swapchain: vk::SwapchainKHR,
    format: vk::Format,
    swapchain_ext: &SwapchainKhr,
    dev: &Dev,
) -> Vec<vk::ImageView> {
    let images = unsafe { swapchain_ext.get_swapchain_images(swapchain) }.unwrap();
    let mut image_views = Vec::new();
    for image in images {
        image_views.push(create_image_view(
            image,
            format,
            vk::ImageAspectFlags::COLOR,
            dev,
        ));
    }
    image_views
}
