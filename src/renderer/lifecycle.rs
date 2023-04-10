use crate::model::Model;
use crate::renderer::debug::create_debug_messenger;
use crate::renderer::device::{select_device, DeviceInfo, QueueFamilies};
use crate::renderer::shader::Shader;
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

impl Renderer {
    pub fn new(window: &Window, building_model: &Model, sun_model: &Model) -> Renderer {
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
        let msaa_samples = util::find_max_msaa_samples(&instance, physical_device);
        let (pipeline, pipeline_layout, pipeline_render_pass) = create_pipeline(
            descriptor_set_layout,
            swapchain_format,
            msaa_samples,
            &instance,
            physical_device,
            &logical_device,
        );
        let command_pool = create_command_pool(&queue_families, &logical_device);
        let command_buffers = create_command_buffers(&logical_device, command_pool);

        let color = create_color_resources(
            swapchain_format,
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
            queues.graphics,
            command_pool,
        );
        let framebuffers = create_framebuffers(
            pipeline_render_pass,
            swapchain_image_count,
            &swapchain_image_views,
            swapchain_extent,
            depth.view,
            color.view,
            &logical_device,
        );

        let (texture, texture_mipmaps) = util::load_texture(
            building_model.texture_path,
            &instance,
            physical_device,
            &logical_device,
            queues.graphics,
            command_pool,
        );
        let texture_sampler =
            create_texture_sampler(texture_mipmaps, &instance, physical_device, &logical_device);

        let descriptor_pool = create_descriptor_pool(&logical_device);

        let light = create_uniform_buffer(&instance, physical_device, &logical_device);

        let building = create_object(
            building_model,
            descriptor_set_layout,
            descriptor_pool,
            &light.buffers,
            texture.view,
            texture_sampler,
            &instance,
            physical_device,
            &logical_device,
            queues.graphics,
            command_pool,
        );
        let sun = create_object(
            sun_model,
            descriptor_set_layout,
            descriptor_pool,
            &light.buffers,
            texture.view,
            texture_sampler,
            &instance,
            physical_device,
            &logical_device,
            queues.graphics,
            command_pool,
        );

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
            swapchain_image_count,
            swapchain_format,
            swapchain_extent,
            swapchain,
            swapchain_image_views,
            descriptor_set_layout,
            msaa_samples,
            pipeline,
            pipeline_layout,
            pipeline_render_pass,
            command_pool,
            command_buffers,
            color,
            depth,
            framebuffers,
            texture,
            texture_sampler,
            light,
            building,
            sun,
            descriptor_pool,
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
            swapchain_format,
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
            self.queues.graphics,
            self.command_pool,
        );
        let framebuffers = create_framebuffers(
            self.pipeline_render_pass,
            swapchain_image_count,
            &swapchain_image_views,
            swapchain_extent,
            depth.view,
            color.view,
            &self.logical_device,
        );
        let projection = compute_projection(swapchain_extent);

        // Doing the assignments at the end guarantees any operation won't fail in the middle, and
        // makes it possible to easily compare new values to old ones.
        self.surface_capabilities = surface_capabilities;
        self.surface_formats = surface_formats;
        self.present_modes = present_modes;
        self.swapchain_image_count = swapchain_image_count;
        self.swapchain_format = swapchain_format;
        self.swapchain_extent = swapchain_extent;
        self.swapchain = swapchain;
        self.swapchain_image_views = swapchain_image_views;
        self.color = color;
        self.depth = depth;
        self.framebuffers = framebuffers;
        self.projection = projection;
    }

    fn cleanup_swapchain(&mut self) {
        self.depth.cleanup(&self.logical_device);
        self.color.cleanup(&self.logical_device);
        unsafe {
            for framebuffer in &self.framebuffers {
                self.logical_device.destroy_framebuffer(*framebuffer, None);
            }
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
            self.building.cleanup(dev);
            self.sun.cleanup(dev);
            dev.destroy_sampler(self.texture_sampler, None);
            self.texture.cleanup(&self.logical_device);
            dev.destroy_command_pool(self.command_pool, None);
            dev.destroy_pipeline(self.pipeline, None);
            dev.destroy_render_pass(self.pipeline_render_pass, None);
            dev.destroy_pipeline_layout(self.pipeline_layout, None);
            dev.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
        self.cleanup_swapchain();
        unsafe {
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
    let no_image_limit = capabilities.max_image_count == 0;
    let preferred_image_count = capabilities.min_image_count as usize + 1;
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

fn create_pipeline(
    descriptor_set_layout: vk::DescriptorSetLayout,
    swapchain_image_format: vk::SurfaceFormatKHR,
    msaa_samples: vk::SampleCountFlags,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
) -> (vk::Pipeline, vk::PipelineLayout, vk::RenderPass) {
    let vert_shader = Shader::compile(
        logical_device,
        include_bytes!("../../shaders/triangle-vert.spv"),
        vk::ShaderStageFlags::VERTEX,
    );
    let frag_shader = Shader::compile(
        logical_device,
        include_bytes!("../../shaders/triangle-frag.spv"),
        vk::ShaderStageFlags::FRAGMENT,
    );
    let shader_stages = [vert_shader.stage_info, frag_shader.stage_info];
    let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);
    let vertex_attribute_descriptions = Vertex::attribute_descriptions(0);
    let vertex_binding_descriptions = [*vk::VertexInputBindingDescription::builder()
        .binding(0)
        .stride(std::mem::size_of::<Vertex>() as u32)
        .input_rate(vk::VertexInputRate::VERTEX)];
    let vertex_input = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&vertex_binding_descriptions)
        .vertex_attribute_descriptions(&vertex_attribute_descriptions);
    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);
    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewport_count(1)
        .scissor_count(1);
    let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false)
        .depth_bias_constant_factor(0.)
        .depth_bias_clamp(0.)
        .depth_bias_slope_factor(0.);
    let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(msaa_samples)
        .min_sample_shading(1.)
        .sample_mask(&[])
        .alpha_to_coverage_enable(false)
        .alpha_to_one_enable(false);
    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(false);
    let color_blend_attachments = [*color_blend_attachment];
    let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(&color_blend_attachments);
    let set_layouts = [descriptor_set_layout];
    let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(&set_layouts)
        .push_constant_ranges(&[]);
    let pipeline_layout =
        unsafe { logical_device.create_pipeline_layout(&pipeline_layout_info, None) }.unwrap();

    let color_attachment = vk::AttachmentDescription::builder()
        .format(swapchain_image_format.format)
        .samples(msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let color_attachments = [*color_attachment_ref];
    let depth_attachment = *vk::AttachmentDescription::builder()
        .format(select_depth_format(instance, physical_device))
        .samples(msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
    let depth_attachment_ref = vk::AttachmentReference::builder()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
    let color_attachment_resolve = *vk::AttachmentDescription::builder()
        .format(swapchain_image_format.format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
    let color_attachment_resolve_ref = *vk::AttachmentReference::builder()
        .attachment(2)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let resolve_attachments = [color_attachment_resolve_ref];
    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachments)
        .depth_stencil_attachment(&depth_attachment_ref)
        .resolve_attachments(&resolve_attachments);
    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        );
    let attachments = [
        *color_attachment,
        depth_attachment,
        color_attachment_resolve,
    ];
    let subpasses = [*subpass];
    let dependencies = [*dependency];
    let render_pass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&dependencies);
    let render_pass =
        unsafe { logical_device.create_render_pass(&render_pass_info, None) }.unwrap();

    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS)
        .depth_bounds_test_enable(false)
        .min_depth_bounds(0.)
        .max_depth_bounds(1.)
        .stencil_test_enable(false)
        .front(vk::StencilOpState::default())
        .back(vk::StencilOpState::default());

    let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterizer)
        .multisample_state(&multisampling)
        .color_blend_state(&color_blending)
        .depth_stencil_state(&depth_stencil)
        .dynamic_state(&dynamic_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0);
    let pipeline = unsafe {
        logical_device.create_graphics_pipelines(vk::PipelineCache::null(), &[*pipeline_info], None)
    }
    .unwrap()
    .into_iter()
    .next()
    .unwrap();

    (pipeline, pipeline_layout, render_pass)
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
    swapchain_format: vk::SurfaceFormatKHR,
    swapchain_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
) -> ImageResources {
    let (image, memory) = util::create_image(
        swapchain_format.format,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::ImageTiling::OPTIMAL,
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
        swapchain_format.format,
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
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
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
    // This is apparently done by the render pass anyway, but the tutorial leaves it in to show how
    // to do this explicitly.
    util::transition_image_layout(
        image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        format,
        1,
        logical_device,
        graphics_queue,
        command_pool,
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
    pipeline_render_pass: vk::RenderPass,
    swapchain_image_count: usize,
    swapchain_image_views: &[vk::ImageView],
    swapchain_extent: vk::Extent2D,
    depth_image_view: vk::ImageView,
    color_image_view: vk::ImageView,
    logical_device: &Device,
) -> Vec<vk::Framebuffer> {
    let mut framebuffers = vec![vk::Framebuffer::null(); swapchain_image_count];
    for i in 0..swapchain_image_count {
        let attachments = [color_image_view, depth_image_view, swapchain_image_views[i]];
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(pipeline_render_pass)
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
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .anisotropy_enable(true)
        .max_anisotropy(properties.limits.max_sampler_anisotropy)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .compare_op(vk::CompareOp::ALWAYS)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .min_lod(0.)
        .max_lod(mip_levels as f32)
        .mip_lod_bias(0.);
    unsafe { logical_device.create_sampler(&sampler_info, None) }.unwrap()
}

fn create_object(
    model: &Model,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    light_buffers: &[vk::Buffer],
    texture_view: vk::ImageView,
    texture_sampler: vk::Sampler,
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
    let material = create_uniform_buffer(instance, physical_device, logical_device);
    let descriptor_sets = create_descriptor_sets(
        descriptor_set_layout,
        descriptor_pool,
        &mvp.buffers,
        &material.buffers,
        light_buffers,
        texture_view,
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
    let descriptor_sets: [vk::DescriptorSet; 2] =
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
