use crate::model::Model;
use crate::renderer::debug::create_debug_messenger;
use crate::renderer::device::{select_device, DeviceInfo, QueueFamilies};
use crate::renderer::pipeline::{build_simple_pipeline, SimplePipeline, SimpleVertexLayout};
use crate::renderer::traits::VertexOps;
use crate::renderer::uniform::{Light, Material, ModelViewProjection};
use crate::renderer::util::{ImageResources, Queues, VulkanExtensions};
use crate::renderer::vertex::Vertex;
use crate::renderer::{util, Object, Renderer, Synchronization, UniformBuffer, FRAMES_IN_FLIGHT};
use crate::window::Window;
use crate::{VULKAN_APP_NAME, VULKAN_APP_VERSION, VULKAN_ENGINE_NAME, VULKAN_ENGINE_VERSION};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::{vk, Device, Entry, Instance};
use nalgebra_glm as glm;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::collections::HashSet;
use std::f32::consts::FRAC_PI_4;
use std::ffi::CString;
use std::mem::MaybeUninit;
use winit::dpi::PhysicalSize;

// Format used for passing HDR data between render passes to enable realistic differences in
// lighting parameters and improve postprocessing effect quality, not related to monitor HDR.
// Support for this format is required by the Vulkan specification.
const INTERNAL_HDR_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;

impl Renderer {
    pub fn new(window: &Window, models: &[Model]) -> Renderer {
        let entry = unsafe { Entry::load() }.unwrap();
        let instance = create_instance(window, &entry);
        let extensions = VulkanExtensions {
            debug: DebugUtils::new(&entry, &instance),
            surface: Surface::new(&entry, &instance),
        };
        let debug_messenger = create_debug_messenger(&extensions.debug);
        let surface = create_surface(window, &entry, &instance);
        let DeviceInfo {
            physical_device,
            queue_families,
            surface_capabilities,
            surface_formats,
            present_modes,
        } = select_device(&instance, &extensions.surface, surface);
        let logical_device = create_logical_device(&queue_families, &instance, physical_device);
        let graphics_queue = unsafe { logical_device.get_device_queue(queue_families.graphics, 0) };
        let present_queue = unsafe { logical_device.get_device_queue(queue_families.present, 0) };
        let queues = Queues {
            graphics: graphics_queue,
            present: present_queue,
        };
        let swapchain_extension = Swapchain::new(&instance, &logical_device);
        let swapchain_image_count = select_swapchain_image_count(surface_capabilities);
        let swapchain_format = select_swapchain_format(&surface_formats);
        let swapchain_extent =
            select_swapchain_extent(surface_capabilities, window.window.inner_size());
        let swapchain_present_mode = select_swapchain_present_mode(&present_modes);
        let swapchain = create_swapchain(
            swapchain_format,
            swapchain_extent,
            swapchain_present_mode,
            swapchain_image_count,
            &swapchain_extension,
            surface,
            surface_capabilities,
            &queue_families,
        );
        let swapchain_image_views = create_swapchain_image_views(
            swapchain,
            swapchain_format,
            &logical_device,
            &swapchain_extension,
        );
        let descriptor_set_layout = create_descriptor_set_layout(&logical_device);
        let offscreen_sampler = create_offscreen_sampler(&logical_device);
        let postprocess_descriptor_set_layout =
            create_postprocess_descriptor_set_layout(offscreen_sampler, &logical_device);
        let msaa_samples = util::find_max_msaa_samples(&instance, physical_device);
        let (pipeline, pipeline_layout, pipeline_render_pass) = create_pipeline(
            descriptor_set_layout,
            msaa_samples,
            &instance,
            physical_device,
            &logical_device,
        );
        let (postprocess_pipeline, postprocess_pipeline_layout, postprocess_pass) =
            create_postprocess_pipeline(
                postprocess_descriptor_set_layout,
                swapchain_format,
                &logical_device,
            );
        let command_pool = create_command_pool(&queue_families, &logical_device);
        let command_buffers = create_command_buffers(&logical_device, command_pool);

        let color = create_color_resources(
            swapchain_extent,
            msaa_samples,
            &instance,
            physical_device,
            &logical_device,
        );

        let depth = create_depth_resources(
            swapchain_extent,
            msaa_samples,
            &instance,
            physical_device,
            &logical_device,
        );
        let offscreen = create_offscreen_resources(
            swapchain_extent,
            &instance,
            physical_device,
            &logical_device,
        );
        let offscreen_framebuffer = create_offscreen_framebuffer(
            pipeline_render_pass,
            offscreen.view,
            swapchain_extent,
            depth.view,
            color.view,
            &logical_device,
        );
        let framebuffers = create_framebuffers(
            postprocess_pass,
            swapchain_image_count,
            &swapchain_image_views,
            swapchain_extent,
            &logical_device,
        );

        let descriptor_pool = create_descriptor_pool(&logical_device);
        let postprocess_descriptor_pool = create_postprocess_descriptor_pool(&logical_device);
        let postprocess_descriptor_set = create_postprocess_descriptor_set(
            offscreen.view,
            postprocess_descriptor_set_layout,
            postprocess_descriptor_pool,
            &logical_device,
        );

        let light = create_uniform_buffer(&instance, physical_device, &logical_device);
        let noise_texture = util::generate_perlin_texture(
            512,
            16.,
            &instance,
            physical_device,
            &logical_device,
            graphics_queue,
            command_pool,
        );
        let noise_sampler = create_texture_sampler(1, &instance, physical_device, &logical_device);

        let mut objects = Vec::new();
        for model in models {
            let object = create_object(
                model,
                descriptor_set_layout,
                descriptor_pool,
                &light.buffers,
                &instance,
                physical_device,
                &logical_device,
                queues.graphics,
                command_pool,
            );
            objects.push(object);
        }

        let sync = create_sync(&logical_device);
        let projection = compute_projection(swapchain_extent);
        Renderer {
            _entry: entry,
            instance,
            extensions,
            debug_messenger,
            surface,
            physical_device,
            queue_families,
            surface_capabilities,
            surface_formats,
            present_modes,
            logical_device,
            queues,
            swapchain_extension,
            swapchain_format,
            swapchain_extent,
            swapchain,
            swapchain_image_views,
            descriptor_set_layout,
            postprocess_descriptor_set_layout,
            msaa_samples,
            pipeline,
            pipeline_layout,
            pipeline_render_pass,
            postprocess_pipeline,
            postprocess_pipeline_layout,
            postprocess_pass,
            command_pool,
            command_buffers,
            color,
            depth,
            offscreen,
            offscreen_framebuffer,
            offscreen_sampler,
            framebuffers,
            light,
            objects,
            descriptor_pool,
            postprocess_descriptor_pool,
            postprocess_descriptor_set,
            noise_texture,
            noise_sampler,
            sync,
            flight_index: 0,
            projection,
        }
    }

    pub fn recreate_swapchain(&mut self, window_size: PhysicalSize<u32>) {
        // First, wait for the GPU work to end. It's possible to pass an old swapchain while
        // creating the new one which results in a faster (?) transition, but in the interest of
        // simplicity let's skip that for now.
        unsafe { self.logical_device.device_wait_idle() }.unwrap();

        // This destroys swapchain resources including the framebuffer, but we should also consider
        // surface information obtained during physical device selection as outdated. These can
        // contain not only things like image formats, but also some sizes.
        self.cleanup_swapchain();

        // Query the surface information again.
        let surface_capabilities = unsafe {
            self.extensions
                .surface
                .get_physical_device_surface_capabilities(self.physical_device, self.surface)
        }
        .unwrap();
        let surface_formats = unsafe {
            self.extensions
                .surface
                .get_physical_device_surface_formats(self.physical_device, self.surface)
        }
        .unwrap();
        let present_modes = unsafe {
            self.extensions
                .surface
                .get_physical_device_surface_present_modes(self.physical_device, self.surface)
        }
        .unwrap();
        assert!(!surface_formats.is_empty());
        assert!(!present_modes.is_empty());

        let swapchain_image_count = select_swapchain_image_count(surface_capabilities);

        // Make sure the swapchain format is the same, if it weren't we'd need to recreate the
        // graphics pipeline too.
        let swapchain_format = select_swapchain_format(&surface_formats);
        assert_eq!(swapchain_format, self.swapchain_format);

        // Repeat creating the swapchain, except not using any Renderer members that heavily depend
        // on the swapchain (such as depth and color buffers).
        let swapchain_extent = select_swapchain_extent(surface_capabilities, window_size);
        let swapchain_present_mode = select_swapchain_present_mode(&present_modes);
        let swapchain = create_swapchain(
            swapchain_format,
            swapchain_extent,
            swapchain_present_mode,
            swapchain_image_count,
            &self.swapchain_extension,
            self.surface,
            surface_capabilities,
            &self.queue_families,
        );
        let swapchain_image_views = create_swapchain_image_views(
            swapchain,
            swapchain_format,
            &self.logical_device,
            &self.swapchain_extension,
        );
        let color = create_color_resources(
            swapchain_extent,
            self.msaa_samples,
            &self.instance,
            self.physical_device,
            &self.logical_device,
        );
        let depth = create_depth_resources(
            swapchain_extent,
            self.msaa_samples,
            &self.instance,
            self.physical_device,
            &self.logical_device,
        );
        let offscreen = create_offscreen_resources(
            swapchain_extent,
            &self.instance,
            self.physical_device,
            &self.logical_device,
        );
        let offscreen_framebuffer = create_offscreen_framebuffer(
            self.pipeline_render_pass,
            offscreen.view,
            swapchain_extent,
            depth.view,
            color.view,
            &self.logical_device,
        );
        let framebuffers = create_framebuffers(
            self.postprocess_pass,
            swapchain_image_count,
            &swapchain_image_views,
            swapchain_extent,
            &self.logical_device,
        );
        let postprocess_descriptor_set = create_postprocess_descriptor_set(
            offscreen.view,
            self.postprocess_descriptor_set_layout,
            self.postprocess_descriptor_pool,
            &self.logical_device,
        );
        let projection = compute_projection(swapchain_extent);

        // Doing the assignments at the end guarantees any operation won't fail in the middle, and
        // makes it possible to easily compare new values to old ones.
        self.surface_capabilities = surface_capabilities;
        self.surface_formats = surface_formats;
        self.present_modes = present_modes;
        self.swapchain_format = swapchain_format;
        self.swapchain_extent = swapchain_extent;
        self.swapchain = swapchain;
        self.swapchain_image_views = swapchain_image_views;
        self.color = color;
        self.depth = depth;
        self.offscreen = offscreen;
        self.offscreen_framebuffer = offscreen_framebuffer;
        self.framebuffers = framebuffers;
        self.postprocess_descriptor_set = postprocess_descriptor_set;
        self.projection = projection;
    }

    fn cleanup_swapchain(&mut self) {
        self.depth.cleanup(&self.logical_device);
        self.color.cleanup(&self.logical_device);
        self.offscreen.cleanup(&self.logical_device);
        unsafe {
            self.logical_device
                .free_descriptor_sets(
                    self.postprocess_descriptor_pool,
                    &[self.postprocess_descriptor_set],
                )
                .unwrap();
            for framebuffer in &self.framebuffers {
                self.logical_device.destroy_framebuffer(*framebuffer, None);
            }
            self.logical_device
                .destroy_framebuffer(self.offscreen_framebuffer, None);
            for image_view in &self.swapchain_image_views {
                self.logical_device.destroy_image_view(*image_view, None);
            }
            self.swapchain_extension
                .destroy_swapchain(self.swapchain, None);
        }
    }
}

impl Object {
    fn cleanup(&self, logical_device: &Device) {
        unsafe { logical_device.destroy_buffer(self.vertex_buffer, None) };
        unsafe { logical_device.free_memory(self.vertex_buffer_memory, None) };
        unsafe { logical_device.destroy_buffer(self.index_buffer, None) };
        unsafe { logical_device.free_memory(self.index_buffer_memory, None) };
        self.mvp.cleanup(logical_device);
        unsafe { logical_device.destroy_sampler(self.texture_sampler, None) };
        self.texture.cleanup(logical_device);
        self.material.cleanup(logical_device);
    }
}

impl<T> UniformBuffer<T> {
    fn cleanup(&self, logical_device: &Device) {
        for buffer in self.buffers {
            unsafe { logical_device.destroy_buffer(buffer, None) };
        }
        for memory in self.memories {
            unsafe { logical_device.free_memory(memory, None) };
        }
    }
}

impl ImageResources {
    fn cleanup(&self, logical_device: &Device) {
        unsafe {
            logical_device.destroy_image_view(self.view, None);
            logical_device.destroy_image(self.image, None);
            logical_device.free_memory(self.memory, None);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let dev = &self.logical_device;
            dev.device_wait_idle().unwrap();
            for fence in self.sync.in_flight {
                dev.destroy_fence(fence, None);
            }
            for semaphore in self.sync.render_finished {
                dev.destroy_semaphore(semaphore, None);
            }
            for semaphore in self.sync.image_available {
                dev.destroy_semaphore(semaphore, None);
            }
            dev.destroy_descriptor_pool(self.descriptor_pool, None);
            self.light.cleanup(dev);
            for object in &self.objects {
                object.cleanup(dev);
            }
            self.noise_texture.cleanup(dev);
            dev.destroy_command_pool(self.command_pool, None);
            dev.destroy_pipeline(self.pipeline, None);
            dev.destroy_pipeline(self.postprocess_pipeline, None);
            dev.destroy_render_pass(self.pipeline_render_pass, None);
            dev.destroy_render_pass(self.postprocess_pass, None);
            dev.destroy_pipeline_layout(self.pipeline_layout, None);
            dev.destroy_pipeline_layout(self.postprocess_pipeline_layout, None);
            dev.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            dev.destroy_descriptor_set_layout(self.postprocess_descriptor_set_layout, None);
        }
        self.cleanup_swapchain();
        unsafe {
            self.logical_device
                .destroy_descriptor_pool(self.postprocess_descriptor_pool, None);
            self.logical_device
                .destroy_sampler(self.offscreen_sampler, None);
            self.logical_device
                .destroy_sampler(self.noise_sampler, None);
            self.logical_device.destroy_device(None);
            self.extensions.surface.destroy_surface(self.surface, None);
            self.extensions
                .debug
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.instance.destroy_instance(None);
        }
    }
}

fn create_instance(window: &Window, entry: &Entry) -> Instance {
    // Set metadata of the app and the engine. May be used by the drivers to enable game-specific
    // and engine-specific optimizations, which won't happen, but let's set it to something sensible
    // anyway.
    let app_name = CString::new(VULKAN_APP_NAME).unwrap();
    let app_version = vk::make_api_version(
        0,
        VULKAN_APP_VERSION.0,
        VULKAN_APP_VERSION.1,
        VULKAN_APP_VERSION.2,
    );
    let engine_name = CString::new(VULKAN_ENGINE_NAME).unwrap();
    let engine_version = vk::make_api_version(
        0,
        VULKAN_ENGINE_VERSION.0,
        VULKAN_ENGINE_VERSION.1,
        VULKAN_ENGINE_VERSION.2,
    );
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(app_version)
        .engine_name(&engine_name)
        .engine_version(engine_version)
        .api_version(vk::API_VERSION_1_0);

    // Enable Vulkan validation layers. This should be later disabled in non-development builds.
    let layer_names = [b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const i8];

    // Vulkan doesn't appear to have any interesting extensions at this level, physical device
    // extensions are the interesting ones with raytracing and other stuff. This is just for
    // OS-specific windowing system interactions, and enabling debug logging for the validation
    // layers.
    let mut extension_names =
        ash_window::enumerate_required_extensions(window.window.raw_display_handle())
            .unwrap()
            .to_vec();
    extension_names.push(DebugUtils::name().as_ptr());

    let instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layer_names)
        .enabled_extension_names(&extension_names);
    unsafe { entry.create_instance(&instance_create_info, None) }.unwrap()
}

fn create_surface(window: &Window, entry: &Entry, instance: &Instance) -> vk::SurfaceKHR {
    unsafe {
        ash_window::create_surface(
            entry,
            instance,
            window.window.raw_display_handle(),
            window.window.raw_window_handle(),
            None,
        )
    }
    .unwrap()
}

fn create_logical_device(
    queue_families: &QueueFamilies,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Device {
    // Queues from the same family must be created at once, so we need to use a set to eliminate
    // duplicates. If the queue families are the same, we create only a single queue and keep
    // two handles. This needs to be remembered later when setting flags related to memory
    // access being exclusive to the queue or concurrent from many queues.
    let queue_indices = HashSet::from([queue_families.graphics, queue_families.present]);
    let queue_creates: Vec<_> = queue_indices
        .iter()
        .map(|queue_index| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*queue_index)
                .queue_priorities(&[1.])
                .build()
        })
        .collect();

    let physical_device_features = vk::PhysicalDeviceFeatures::builder().sampler_anisotropy(true);

    // Using validation layers on a device level shouldn't be necessary on newer Vulkan version
    // (since which one?), but it's good to keep it for compatibility.
    let layer_names = [b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const i8];

    unsafe {
        instance.create_device(
            physical_device,
            &vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_creates)
                .enabled_features(&physical_device_features)
                .enabled_layer_names(&layer_names)
                .enabled_extension_names(&[Swapchain::name().as_ptr()]),
            None,
        )
    }
    .unwrap()
}

fn select_swapchain_image_count(capabilities: vk::SurfaceCapabilitiesKHR) -> usize {
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

fn select_swapchain_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    for format in formats {
        if format.format == vk::Format::B8G8R8A8_SRGB
            && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        {
            return *format;
        }
    }
    formats[0]
}

fn select_swapchain_extent(
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

fn select_swapchain_present_mode(_available: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
    vk::PresentModeKHR::FIFO
}

fn create_swapchain(
    format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    present_mode: vk::PresentModeKHR,
    image_count: usize,
    extension: &Swapchain,
    surface: vk::SurfaceKHR,
    surface_capabilities: vk::SurfaceCapabilitiesKHR,
    queue_families: &QueueFamilies,
) -> vk::SwapchainKHR {
    // Create the swapchain for presenting images to display. Set to prefer triple buffering
    // right now, should be possible to change on laptops or integrated GPUs? Also requires
    // specifying a bunch of display-related parameters, which aren't very interesting as they
    // were mostly decided on previously.
    let queue_family_indices = [queue_families.graphics, queue_families.present];
    let create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(image_count as u32)
        .image_format(format.format)
        .image_color_space(format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);
    let create_info = if queue_families.graphics != queue_families.present {
        create_info
            .image_sharing_mode(vk::SharingMode::CONCURRENT)
            .queue_family_indices(&queue_family_indices)
    } else {
        create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
    };
    let create_info = create_info
        .pre_transform(surface_capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null());
    unsafe { extension.create_swapchain(&create_info, None) }.unwrap()
}

fn create_swapchain_image_views(
    swapchain: vk::SwapchainKHR,
    format: vk::SurfaceFormatKHR,
    logical_device: &Device,
    extension: &Swapchain,
) -> Vec<vk::ImageView> {
    // Create image views. Not really interesting for now, as I only use normal color settings.
    let images = unsafe { extension.get_swapchain_images(swapchain) }.unwrap();
    let mut image_views = vec![vk::ImageView::null(); images.len()];
    for i in 0..images.len() {
        image_views[i] = util::create_image_view(
            images[i],
            format.format,
            vk::ImageAspectFlags::COLOR,
            1,
            logical_device,
        );
    }
    image_views
}

fn create_descriptor_set_layout(logical_device: &Device) -> vk::DescriptorSetLayout {
    let mvp_layout_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);
    let sampler_layout_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_count(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);
    let material_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(2)
        .descriptor_count(1)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);
    let light_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(3)
        .descriptor_count(1)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);
    let layout_bindings = [
        *mvp_layout_binding,
        *sampler_layout_binding,
        *material_binding,
        *light_binding,
    ];
    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_bindings);
    unsafe { logical_device.create_descriptor_set_layout(&layout_info, None) }.unwrap()
}

fn create_postprocess_descriptor_set_layout(
    offscreen_sampler: vk::Sampler,
    logical_device: &Device,
) -> vk::DescriptorSetLayout {
    let render_samplers = [offscreen_sampler];
    let render_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_count(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .immutable_samplers(&render_samplers);
    let layout_bindings = [*render_binding];
    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_bindings);
    unsafe { logical_device.create_descriptor_set_layout(&layout_info, None) }.unwrap()
}

fn create_pipeline(
    descriptor_set_layout: vk::DescriptorSetLayout,
    msaa_samples: vk::SampleCountFlags,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
) -> (vk::Pipeline, vk::PipelineLayout, vk::RenderPass) {
    let color_attachment = *vk::AttachmentDescription::builder()
        .format(INTERNAL_HDR_FORMAT)
        .samples(msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let depth_attachment = *vk::AttachmentDescription::builder()
        .format(select_depth_format(instance, physical_device))
        .samples(msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
    let resolve_attachment = *vk::AttachmentDescription::builder()
        .format(INTERNAL_HDR_FORMAT)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    let pipeline = SimplePipeline {
        vertex_shader_path: "shaders/model.vert",
        fragment_shader_path: "shaders/model.frag",
        vertex_layout: Some(SimpleVertexLayout {
            stride: std::mem::size_of::<Vertex>(),
            attribute_descriptions: Vertex::attribute_descriptions(0),
        }),
        msaa_samples,
        descriptor_set_layout,
        color_attachment,
        depth_attachment: Some(depth_attachment),
        resolve_attachment: Some(resolve_attachment),
        logical_device,
    };
    build_simple_pipeline(pipeline)
}

fn create_postprocess_pipeline(
    descriptor_set_layout: vk::DescriptorSetLayout,
    swapchain_image_format: vk::SurfaceFormatKHR,
    logical_device: &Device,
) -> (vk::Pipeline, vk::PipelineLayout, vk::RenderPass) {
    let color_attachment = *vk::AttachmentDescription::builder()
        .format(swapchain_image_format.format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
    let pipeline = SimplePipeline {
        vertex_shader_path: "shaders/quad.vert",
        fragment_shader_path: "shaders/postprocess.frag",
        vertex_layout: None,
        msaa_samples: vk::SampleCountFlags::TYPE_1,
        descriptor_set_layout,
        color_attachment,
        depth_attachment: None,
        resolve_attachment: None,
        logical_device,
    };
    build_simple_pipeline(pipeline)
}

fn create_command_pool(queue_families: &QueueFamilies, logical_device: &Device) -> vk::CommandPool {
    let command_pool_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_families.graphics);
    unsafe { logical_device.create_command_pool(&command_pool_info, None) }.unwrap()
}

fn create_command_buffers(
    logical_device: &Device,
    command_pool: vk::CommandPool,
) -> [vk::CommandBuffer; FRAMES_IN_FLIGHT] {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(FRAMES_IN_FLIGHT as u32);
    unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info) }
        .unwrap()
        .try_into()
        .unwrap()
}

fn create_color_resources(
    swapchain_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
) -> ImageResources {
    let (image, memory) = util::create_image(
        INTERNAL_HDR_FORMAT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::ImageTiling::OPTIMAL,
        // Transient attachment lets the drivers lazily allocate memory for the framebuffer
        // attachment, and for some implementation this actually doesn't allocate memory at all. See
        // "Lazily ALlocated Memory" section of the Vulkan specification.
        vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
        swapchain_extent.width as usize,
        swapchain_extent.height as usize,
        1,
        msaa_samples,
        instance,
        physical_device,
        logical_device,
    );
    let view = util::create_image_view(
        image,
        INTERNAL_HDR_FORMAT,
        vk::ImageAspectFlags::COLOR,
        1,
        logical_device,
    );
    ImageResources {
        image,
        memory,
        view,
    }
}

fn create_offscreen_resources(
    swapchain_extent: vk::Extent2D,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
) -> ImageResources {
    let (image, memory) = util::create_image(
        INTERNAL_HDR_FORMAT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
        swapchain_extent.width as usize,
        swapchain_extent.height as usize,
        1,
        vk::SampleCountFlags::TYPE_1,
        instance,
        physical_device,
        logical_device,
    );
    let view = util::create_image_view(
        image,
        INTERNAL_HDR_FORMAT,
        vk::ImageAspectFlags::COLOR,
        1,
        logical_device,
    );
    ImageResources {
        image,
        memory,
        view,
    }
}

fn create_depth_resources(
    swapchain_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
) -> ImageResources {
    let format = select_depth_format(instance, physical_device);
    let (image, memory) = util::create_image(
        format,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        swapchain_extent.width as usize,
        swapchain_extent.height as usize,
        1,
        msaa_samples,
        instance,
        physical_device,
        logical_device,
    );
    let view = util::create_image_view(
        image,
        format,
        vk::ImageAspectFlags::DEPTH,
        1,
        logical_device,
    );
    ImageResources {
        image,
        memory,
        view,
    }
}

fn select_depth_format(instance: &Instance, physical_device: vk::PhysicalDevice) -> vk::Format {
    util::select_format(
        &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ],
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::ImageTiling::OPTIMAL,
        instance,
        physical_device,
    )
}

fn create_framebuffers(
    postprocess_pass: vk::RenderPass,
    swapchain_image_count: usize,
    swapchain_image_views: &[vk::ImageView],
    swapchain_extent: vk::Extent2D,
    logical_device: &Device,
) -> Vec<vk::Framebuffer> {
    let mut framebuffers = vec![vk::Framebuffer::null(); swapchain_image_count];
    for i in 0..swapchain_image_count {
        let attachments = [swapchain_image_views[i]];
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(postprocess_pass)
            .attachments(&attachments)
            .width(swapchain_extent.width)
            .height(swapchain_extent.height)
            .layers(1);
        let framebuffer =
            unsafe { logical_device.create_framebuffer(&framebuffer_info, None) }.unwrap();
        framebuffers[i] = framebuffer;
    }
    framebuffers
}

fn create_offscreen_framebuffer(
    pipeline_render_pass: vk::RenderPass,
    offscreen_image_view: vk::ImageView,
    swapchain_extent: vk::Extent2D,
    depth_image_view: vk::ImageView,
    color_image_view: vk::ImageView,
    logical_device: &Device,
) -> vk::Framebuffer {
    let attachments = [color_image_view, depth_image_view, offscreen_image_view];
    let framebuffer_info = vk::FramebufferCreateInfo::builder()
        .render_pass(pipeline_render_pass)
        .attachments(&attachments)
        .width(swapchain_extent.width)
        .height(swapchain_extent.height)
        .layers(1);
    unsafe { logical_device.create_framebuffer(&framebuffer_info, None) }.unwrap()
}

fn create_texture_sampler(
    mip_levels: usize,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
) -> vk::Sampler {
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };
    let sampler_info = vk::SamplerCreateInfo::builder()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .anisotropy_enable(true)
        .max_anisotropy(properties.limits.max_sampler_anisotropy)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .min_lod(0.)
        .max_lod(mip_levels as f32);
    unsafe { logical_device.create_sampler(&sampler_info, None) }.unwrap()
}

fn create_offscreen_sampler(logical_device: &Device) -> vk::Sampler {
    let sampler_info = vk::SamplerCreateInfo::builder()
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
        .unnormalized_coordinates(true);
    unsafe { logical_device.create_sampler(&sampler_info, None) }.unwrap()
}

fn create_object(
    model: &Model,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    light_buffers: &[vk::Buffer],
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
) -> Object {
    let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(
        &model.vertices,
        instance,
        physical_device,
        logical_device,
        graphics_queue,
        command_pool,
    );
    let (index_buffer, index_buffer_memory) = create_index_buffer(
        &model.indices,
        instance,
        physical_device,
        logical_device,
        graphics_queue,
        command_pool,
    );
    let mvp = create_uniform_buffer(instance, physical_device, logical_device);
    let (texture, texture_mipmaps) = util::load_texture(
        model.texture_path,
        instance,
        physical_device,
        logical_device,
        graphics_queue,
        command_pool,
    );
    let texture_sampler =
        create_texture_sampler(texture_mipmaps, instance, physical_device, logical_device);
    let material = create_uniform_buffer(instance, physical_device, logical_device);
    let descriptor_sets = create_descriptor_sets(
        descriptor_set_layout,
        descriptor_pool,
        &mvp.buffers,
        &material.buffers,
        light_buffers,
        texture.view,
        texture_sampler,
        logical_device,
    );
    Object {
        vertex_buffer,
        vertex_buffer_memory,
        vertex_count: model.vertices.len(),
        index_buffer,
        index_buffer_memory,
        mvp,
        texture,
        texture_sampler,
        material,
        descriptor_sets,
    }
}

fn create_vertex_buffer(
    vertex_data: &[Vertex],
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
) -> (vk::Buffer, vk::DeviceMemory) {
    let vertex_size = std::mem::size_of::<Vertex>();
    let vertex_count = vertex_data.len();
    let vertex_buffer_size = vertex_size * vertex_count;
    let (staging_buffer, staging_memory) = util::create_buffer(
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vertex_buffer_size,
        instance,
        physical_device,
        logical_device,
    );
    let (vertex_buffer, vertex_buffer_memory) = util::create_buffer(
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        vertex_buffer_size,
        instance,
        physical_device,
        logical_device,
    );
    util::with_mapped_slice(staging_memory, vertex_count, logical_device, |mapped| {
        MaybeUninit::write_slice(mapped, vertex_data);
    });
    util::copy_buffer(
        staging_buffer,
        vertex_buffer,
        vertex_buffer_size,
        logical_device,
        graphics_queue,
        command_pool,
    );
    unsafe { logical_device.destroy_buffer(staging_buffer, None) };
    unsafe { logical_device.free_memory(staging_memory, None) };
    (vertex_buffer, vertex_buffer_memory)
}

fn create_index_buffer(
    index_data: &[u32],
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
) -> (vk::Buffer, vk::DeviceMemory) {
    let index_size = std::mem::size_of_val(&index_data[0]);
    let index_buffer_size = index_size * index_data.len();
    let (staging_buffer, staging_memory) = util::create_buffer(
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        vk::BufferUsageFlags::TRANSFER_SRC,
        index_buffer_size,
        instance,
        physical_device,
        logical_device,
    );
    let (index_buffer, index_buffer_memory) = util::create_buffer(
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        index_buffer_size,
        instance,
        physical_device,
        logical_device,
    );
    util::with_mapped_slice(staging_memory, index_data.len(), logical_device, |mapped| {
        MaybeUninit::write_slice(mapped, index_data);
    });
    util::copy_buffer(
        staging_buffer,
        index_buffer,
        index_buffer_size,
        logical_device,
        graphics_queue,
        command_pool,
    );
    unsafe { logical_device.destroy_buffer(staging_buffer, None) };
    unsafe { logical_device.free_memory(staging_memory, None) };
    (index_buffer, index_buffer_memory)
}

fn create_uniform_buffer<T>(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
) -> UniformBuffer<T> {
    let mut buffers = [vk::Buffer::null(); FRAMES_IN_FLIGHT];
    let mut memories = [vk::DeviceMemory::null(); FRAMES_IN_FLIGHT];
    let mut mappings = [std::ptr::null_mut(); FRAMES_IN_FLIGHT];
    for i in 0..FRAMES_IN_FLIGHT {
        let buffer_size = std::mem::size_of::<T>();
        let (buffer, memory) = util::create_buffer(
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            buffer_size,
            instance,
            physical_device,
            logical_device,
        );
        let mapping = unsafe {
            logical_device.map_memory(memory, 0, buffer_size as u64, vk::MemoryMapFlags::empty())
        }
        .unwrap() as *mut T;
        buffers[i] = buffer;
        memories[i] = memory;
        mappings[i] = mapping;
    }
    UniformBuffer {
        buffers,
        memories,
        mappings,
    }
}

fn create_descriptor_pool(logical_device: &Device) -> vk::DescriptorPool {
    let pool_sizes = [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 6 * FRAMES_IN_FLIGHT as u32,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 2 * FRAMES_IN_FLIGHT as u32,
        },
    ];
    let pool_info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(&pool_sizes)
        .max_sets(2 * FRAMES_IN_FLIGHT as u32);
    unsafe { logical_device.create_descriptor_pool(&pool_info, None) }.unwrap()
}

fn create_postprocess_descriptor_pool(logical_device: &Device) -> vk::DescriptorPool {
    let pool_sizes = [vk::DescriptorPoolSize {
        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: 1,
    }];
    let pool_infp = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(&pool_sizes)
        .max_sets(1)
        .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);
    unsafe { logical_device.create_descriptor_pool(&pool_infp, None) }.unwrap()
}

fn create_descriptor_sets(
    layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    mvp_buffers: &[vk::Buffer],
    material_buffers: &[vk::Buffer],
    light_buffers: &[vk::Buffer],
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,
    logical_device: &Device,
) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
    let layouts = vec![layout; FRAMES_IN_FLIGHT];
    let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
        unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_alloc_info) }
            .unwrap()
            .try_into()
            .unwrap();
    for i in 0..FRAMES_IN_FLIGHT {
        let mvp_info = vk::DescriptorBufferInfo::builder()
            .buffer(mvp_buffers[i])
            .offset(0)
            .range(std::mem::size_of::<ModelViewProjection>() as u64);
        let image_info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(texture_image_view)
            .sampler(texture_sampler);
        let material_info = vk::DescriptorBufferInfo::builder()
            .buffer(material_buffers[i])
            .offset(0)
            .range(std::mem::size_of::<Material>() as u64);
        let light_info = vk::DescriptorBufferInfo::builder()
            .buffer(light_buffers[i])
            .offset(0)
            .range(std::mem::size_of::<Light>() as u64);
        let mvp_infos = [*mvp_info];
        let image_infos = [*image_info];
        let material_infos = [*material_info];
        let light_infos = [*light_info];
        let descriptor_writes = [
            *vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&mvp_infos),
            *vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_infos),
            *vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(2)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&material_infos),
            *vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(3)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&light_infos),
        ];
        unsafe { logical_device.update_descriptor_sets(&descriptor_writes, &[]) };
    }
    descriptor_sets
}

fn create_postprocess_descriptor_set(
    offscreen_view: vk::ImageView,
    layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    logical_device: &Device,
) -> vk::DescriptorSet {
    let layouts = [layout];
    let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let [descriptor_set]: [vk::DescriptorSet; 1] =
        unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_alloc_info) }
            .unwrap()
            .try_into()
            .unwrap();
    let image_info = vk::DescriptorImageInfo::builder()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(offscreen_view);
    let image_infos = [*image_info];
    let descriptor_writes = [*vk::WriteDescriptorSet::builder()
        .dst_set(descriptor_set)
        .dst_binding(0)
        .dst_array_element(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(&image_infos)];
    unsafe { logical_device.update_descriptor_sets(&descriptor_writes, &[]) };
    descriptor_set
}

fn create_sync(logical_device: &Device) -> Synchronization {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
    let mut image_available: [vk::Semaphore; FRAMES_IN_FLIGHT] = Default::default();
    let mut render_finished: [vk::Semaphore; FRAMES_IN_FLIGHT] = Default::default();
    let mut in_flight: [vk::Fence; FRAMES_IN_FLIGHT] = Default::default();
    for i in 0..FRAMES_IN_FLIGHT {
        image_available[i] =
            unsafe { logical_device.create_semaphore(&semaphore_info, None) }.unwrap();
        render_finished[i] =
            unsafe { logical_device.create_semaphore(&semaphore_info, None) }.unwrap();
        in_flight[i] = unsafe { logical_device.create_fence(&fence_info, None) }.unwrap();
    }
    Synchronization {
        image_available,
        render_finished,
        in_flight,
    }
}

fn compute_projection(swapchain_extent: vk::Extent2D) -> glm::Mat4 {
    let aspect_ratio = swapchain_extent.width as f32 / swapchain_extent.height as f32;
    let mut proj = glm::perspective_rh_zo(aspect_ratio, FRAC_PI_4, 0.1, 100.);
    proj[(1, 1)] *= -1.;
    proj
}
