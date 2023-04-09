mod debug;
mod device;
mod shader;

use crate::camera::Camera;
use crate::input::InputState;
use crate::model::{Model, Vertex};
use crate::renderer::debug::create_debug_messenger;
use crate::renderer::device::{select_device, QueueFamilies};
use crate::renderer::shader::Shader;
use crate::window::Window;
use crate::{VULKAN_APP_NAME, VULKAN_APP_VERSION, VULKAN_ENGINE_NAME, VULKAN_ENGINE_VERSION};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::{vk, Device, Entry, Instance};
use nalgebra_glm as glm;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::collections::HashSet;
use std::f32::consts::FRAC_PI_4;
use std::hash::{Hash, Hasher};
use std::time::Instant;
use winit::event::{DeviceEvent, Event, StartCause, WindowEvent};
use winit::platform::run_return::EventLoopExtRunReturn;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

struct VulkanSwapchain<'a> {
    logical_device: &'a Device,
    ext: Swapchain,
    swapchain: vk::SwapchainKHR,
    image_format: vk::Format,
    extent: vk::Extent2D,
    image_views: Vec<vk::ImageView>,
}

struct VulkanPipeline<'a> {
    swapchain: &'a VulkanSwapchain<'a>,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct UniformBufferObject {
    model: glm::Mat4,
    view: glm::Mat4,
    proj: glm::Mat4,
}

fn select_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    formats
        .iter()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or(&formats[0])
        .clone()
}

fn select_present_mode() -> vk::PresentModeKHR {
    vk::PresentModeKHR::FIFO
}

fn select_swap_extent(capabilities: vk::SurfaceCapabilitiesKHR, window: &Window) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        return capabilities.current_extent;
    }
    let window_size = window.window.inner_size();
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

fn select_image_count(capabilities: vk::SurfaceCapabilitiesKHR) -> u32 {
    let no_image_limit = capabilities.max_image_count == 0;
    let preferred_image_count = capabilities.min_image_count + 1;
    if no_image_limit {
        preferred_image_count
    } else {
        preferred_image_count.min(capabilities.max_image_count)
    }
}

impl<'a> VulkanSwapchain<'a> {
    fn create(
        renderer: &Renderer,
        logical_device: &'a Device,
        surface: vk::SurfaceKHR,
        window: &Window,
        instance: &Instance,
    ) -> VulkanSwapchain<'a> {
        let ext = Swapchain::new(&instance, logical_device);

        // Create the swapchain for presenting images to display. Set to prefer triple buffering
        // right now, should be possible to change on laptops or integrated GPUs? Also requires
        // specifying a bunch of display-related parameters, which aren't very interesting as they
        // were mostly decided on previously.
        let format = select_format(&renderer.surface_formats);
        let present_mode = select_present_mode();
        let extent = select_swap_extent(renderer.surface_capabilities, window);
        let image_count = select_image_count(renderer.surface_capabilities);
        let image_format = format.format;
        let queue_family_indices = [
            renderer.queue_families.graphics,
            renderer.queue_families.present,
        ];
        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(image_format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);
        let create_info = if renderer.queue_families.graphics != renderer.queue_families.present {
            create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices)
        } else {
            create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        };
        let create_info = create_info
            .pre_transform(renderer.surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());
        let swapchain = unsafe { ext.create_swapchain(&create_info, None) }.unwrap();
        let images = unsafe { ext.get_swapchain_images(swapchain) }.unwrap();

        // Create image views. Not really interesting for now, as I only use normal color settings.
        let mut image_views = vec![vk::ImageView::null(); images.len()];
        for i in 0..images.len() {
            image_views[i] = create_image_view(
                logical_device,
                images[i],
                image_format,
                vk::ImageAspectFlags::COLOR,
                1,
            );
        }

        VulkanSwapchain {
            logical_device,
            ext,
            swapchain,
            image_format,
            extent,
            image_views,
        }
    }

    fn image_count(&self) -> usize {
        self.image_views.len()
    }
}

impl<'a> VulkanPipeline<'a> {
    fn create(
        swapchain: &'a VulkanSwapchain,
        descriptor_set_layout: vk::DescriptorSetLayout,
        msaa_samples: vk::SampleCountFlags,
        physical_device: vk::PhysicalDevice,
        instance: &Instance,
    ) -> VulkanPipeline<'a> {
        let logical_device = swapchain.logical_device;
        let vert_shader = Shader::compile(
            logical_device,
            include_bytes!("../shaders/triangle-vert.spv"),
            vk::ShaderStageFlags::VERTEX,
        );
        let frag_shader = Shader::compile(
            logical_device,
            include_bytes!("../shaders/triangle-frag.spv"),
            vk::ShaderStageFlags::FRAGMENT,
        );
        let shader_stages = [vert_shader.stage_info, frag_shader.stage_info];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);
        let vertex_binding_descriptions = [Vertex::get_binding_description()];
        let vertex_attribute_descriptions = get_attribute_descriptions();
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
            .format(swapchain.image_format)
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
            .format(find_depth_format(physical_device, instance))
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
            .format(swapchain.image_format)
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
            logical_device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[*pipeline_info],
                None,
            )
        }
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

        VulkanPipeline {
            swapchain,
            pipeline,
            pipeline_layout,
            render_pass,
        }
    }
}

impl Vertex {
    fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let p = self as *const Vertex as *const [u8; std::mem::size_of::<Vertex>()];
        p.hash(state);
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        let p1 = self as *const Vertex as *const [u8; std::mem::size_of::<Vertex>()];
        let p2 = other as *const Vertex as *const [u8; std::mem::size_of::<Vertex>()];
        unsafe { *p1 == *p2 }
    }
}

impl Drop for VulkanSwapchain<'_> {
    fn drop(&mut self) {
        for image_view in &self.image_views {
            unsafe { self.logical_device.destroy_image_view(*image_view, None) };
        }
        unsafe { self.ext.destroy_swapchain(self.swapchain, None) };
    }
}

impl Drop for VulkanPipeline<'_> {
    fn drop(&mut self) {
        unsafe {
            self.swapchain
                .logical_device
                .destroy_pipeline(self.pipeline, None)
        };
        unsafe {
            self.swapchain
                .logical_device
                .destroy_render_pass(self.render_pass, None)
        };
        unsafe {
            self.swapchain
                .logical_device
                .destroy_pipeline_layout(self.pipeline_layout, None)
        };
    }
}

pub struct Renderer {
    entry: Entry,
    instance: Instance,
    extensions: VulkanExtensions,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    queue_families: QueueFamilies,
    surface_capabilities: vk::SurfaceCapabilitiesKHR,
    surface_formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
    logical_device: Device,
    queues: Queues,
    // swapchain_extension: Swapchain,
    // swapchain: vk::SwapchainKHR,
    // swapchain_format: vk::Format,
    // swapchain_extent: vk::Extent2D,
    // swapchain_image_views: Vec<vk::ImageView>,
    // descriptor_set_layout: vk::DescriptorSetLayout,
    // msaa_samples: vk::SampleCountFlags,
    // pipeline: vk::Pipeline,
    // pipeline_layout: vk::PipelineLayout,
    // pipeline_render_pass: vk::RenderPass,
    // command_pool: vk::CommandPool,
    // command_buffers: [vk::CommandBuffer; MAX_FRAMES_IN_FLIGHT],
    // color: ImageResources,
    // depth: ImageResources,
    // framebuffers: [vk::Framebuffer; MAX_FRAMES_IN_FLIGHT],
    // texture: ImageResources,
    // texture_mipmaps: usize,
    // texture_sampler: vk::Sampler,
    // vertex_buffer: vk::Buffer,
    // vertex_buffer_memory: vk::DeviceMemory,
    // index_buffer: vk::Buffer,
    // index_buffer_memory: vk::DeviceMemory,
    // uniform_buffers: [vk::Buffer; MAX_FRAMES_IN_FLIGHT],
    // uniform_buffer_memories: [vk::DeviceMemory; MAX_FRAMES_IN_FLIGHT],
    // descriptor_pool: vk::DescriptorPool,
    // descriptor_set: vk::DescriptorSet,
    // sync: Synchronization,
    // frame_flight_index: usize,
}

pub struct VulkanExtensions {
    debug: DebugUtils,
    surface: Surface,
}

pub struct Queues {
    graphics: vk::Queue,
    present: vk::Queue,
}

pub struct ImageResources {
    image: vk::Image,
    memory: vk::DeviceMemory,
    view: vk::ImageView,
}

struct Synchronization {
    image_available: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
    render_finished: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
    in_flight: [vk::Fence; MAX_FRAMES_IN_FLIGHT],
}

impl Renderer {
    pub fn new(window: &Window) -> Renderer {
        let entry = unsafe { Entry::load() }.unwrap();
        let instance = create_instance(&entry, window);
        let extensions = VulkanExtensions {
            debug: DebugUtils::new(&entry, &instance),
            surface: Surface::new(&entry, &instance),
        };
        let debug_messenger = create_debug_messenger(&extensions.debug);
        let surface = create_surface(window, &entry, &instance);
        let device_info = select_device(&instance, &extensions.surface, surface);
        let logical_device = create_logical_device(
            &device_info.queue_families,
            &instance,
            device_info.physical_device,
        );
        let graphics_queue =
            unsafe { logical_device.get_device_queue(device_info.queue_families.graphics, 0) };
        let present_queue =
            unsafe { logical_device.get_device_queue(device_info.queue_families.present, 0) };
        Renderer {
            entry,
            instance,
            extensions,
            debug_messenger,
            surface,
            physical_device: device_info.physical_device,
            queue_families: device_info.queue_families,
            surface_capabilities: device_info.surface_capabilities,
            surface_formats: device_info.surface_formats,
            present_modes: device_info.present_modes,
            logical_device,
            queues: Queues {
                graphics: graphics_queue,
                present: present_queue,
            },
            // swapchain: vk::SwapchainKHR,
            // swapchain_format: vk::Format,
            // swapchain_extent: vk::Extent2D,
            // swapchain_image_views: Vec<vk::ImageView>,
            // descriptor_set_layout: vk::DescriptorSetLayout,
            // msaa_samples: vk::SampleCountFlags,
            // pipeline: vk::Pipeline,
            // pipeline_layout: vk::PipelineLayout,
            // pipeline_render_pass: vk::RenderPass,
            // command_pool: vk::CommandPool,
            // command_buffers: [vk::CommandBuffer; MAX_FRAMES_IN_FLIGHT],
            // color: ImageResources,
            // depth: ImageResources,
            // framebuffers: [vk::Framebuffer; MAX_FRAMES_IN_FLIGHT],
            // texture: ImageResources,
            // texture_mipmaps: usize,
            // texture_sampler: vk::Sampler,
            // vertex_buffer: vk::Buffer,
            // vertex_buffer_memory: vk::DeviceMemory,
            // index_buffer: vk::Buffer,
            // index_buffer_memory: vk::DeviceMemory,
            // uniform_buffers: [vk::Buffer; MAX_FRAMES_IN_FLIGHT],
            // uniform_buffer_memories: [vk::DeviceMemory; MAX_FRAMES_IN_FLIGHT],
            // descriptor_pool: vk::DescriptorPool,
            // descriptor_set: vk::DescriptorSet,
            // sync: Synchronization,
            // frame_flight_index: usize,
        }
    }
}

pub fn run_renderer(mut window: Window, model: Model) {
    let renderer = Renderer::new(&window);

    let swapchain = VulkanSwapchain::create(
        &renderer,
        &renderer.logical_device,
        renderer.surface,
        &window,
        &renderer.instance,
    );
    let descriptor_set_layout = create_descriptor_set_layout(&renderer.logical_device);
    let msaa_samples = get_max_usable_sample_count(renderer.physical_device, &renderer.instance);
    let pipeline = VulkanPipeline::create(
        &swapchain,
        descriptor_set_layout,
        msaa_samples,
        renderer.physical_device,
        &renderer.instance,
    );
    let command_pool = create_command_pool(&renderer.queue_families, &renderer.logical_device);
    let command_buffers = create_command_buffers(command_pool, &renderer.logical_device);

    let (color_image, color_image_memory, color_image_view) = create_color_resources(
        &swapchain,
        msaa_samples,
        &renderer.instance,
        renderer.physical_device,
    );

    let (depth_image, depth_image_memory, depth_image_view) = create_depth_resources(
        &swapchain,
        renderer.queues.graphics,
        command_pool,
        msaa_samples,
        renderer.physical_device,
        &renderer.instance,
    );
    let framebuffers = create_framebuffers(&pipeline, depth_image_view, color_image_view);

    let (texture_image, texture_image_memory, mip_levels) = create_texture_image(
        model.texture_path,
        &renderer.logical_device,
        renderer.queues.graphics,
        command_pool,
        &renderer.instance,
        renderer.physical_device,
    );
    let texture_image_view =
        create_texture_image_view(&renderer.logical_device, texture_image, mip_levels);
    let texture_sampler = create_texture_sampler(
        &renderer.logical_device,
        mip_levels,
        &renderer.instance,
        renderer.physical_device,
    );

    let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(
        &model.vertices,
        &renderer.logical_device,
        command_pool,
        &renderer.instance,
        renderer.physical_device,
        renderer.queues.graphics,
    );
    let (index_buffer, index_buffer_memory) = create_index_buffer(
        &model.indices,
        &renderer.logical_device,
        command_pool,
        &renderer.instance,
        renderer.physical_device,
        renderer.queues.graphics,
    );

    let (uniform_buffers, uniform_buffer_memories, uniform_buffer_mapped) = create_uniform_buffer(
        &renderer.logical_device,
        &renderer.instance,
        renderer.physical_device,
    );

    let descriptor_pool = create_descriptor_pool(&renderer.logical_device);
    let descriptor_sets = create_descriptor_sets(
        descriptor_set_layout,
        descriptor_pool,
        &uniform_buffers,
        texture_image_view,
        texture_sampler,
        &renderer.logical_device,
    );

    let sync = create_sync(&renderer.logical_device);

    let mut current_frame = 0;
    let mut input_state = InputState::new();
    let mut camera = Camera {
        position: glm::vec3(-2., 0., 0.),
        yaw: 0.,
    };
    let mut last_update = Instant::now();

    // Run the event loop. Winit delivers events, like key presses. After it finishes delivering
    // some batch of events, it sends a MainEventsCleared event, which means the application should
    // either render, or check whether it needs to rerender anything and possibly only request a
    // redraw of a specific window. Redrawing a window can also be requested by the operating
    // system, for example if the window size changes. For games, initially I'll render at both
    // events, but this probably needs to be changed to alter framebuffer size if the window is
    // resized?
    window.event_loop.run_return(|event, _, control_flow| {
        match event {
            Event::NewEvents(StartCause::Init) => (),
            // Can be used for collecting frame timing information later. Specifically, this makes
            // it possible to measure frame times accounting for things like having multiple input
            // events before a redraw request.
            Event::NewEvents(StartCause::Poll) => (),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => input_state.apply_keyboard(input),
                WindowEvent::CloseRequested => control_flow.set_exit(),
                _ => (),
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => input_state.apply_mouse(delta),
                _ => (),
            },
            // This is an indication that it's now allowed to create a graphics context, but the
            // limitation only applies on some platforms (Android).
            Event::Resumed => (),
            Event::MainEventsCleared => {
                let curr_update = Instant::now();
                let delta_time = (curr_update - last_update).as_secs_f32();
                last_update = curr_update;
                camera.apply_input(&input_state, delta_time);
                input_state.reset_after_frame();
                draw_frame(
                    &renderer.logical_device,
                    sync.in_flight[current_frame],
                    &swapchain,
                    sync.image_available[current_frame],
                    command_buffers[current_frame],
                    &framebuffers,
                    &pipeline,
                    sync.render_finished[current_frame],
                    vertex_buffer,
                    index_buffer,
                    model.indices.len(),
                    uniform_buffer_mapped[current_frame],
                    descriptor_sets[current_frame],
                    &camera,
                    renderer.queues.graphics,
                    renderer.queues.present,
                );
                current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
            }
            // This event is only sent after MainEventsCleared, during which we render
            // unconditionally.
            Event::RedrawRequested(_) => (),
            // This happens after redraws of all windows are finished, which isn't really applicable
            // to games.
            Event::RedrawEventsCleared => (),
            // Eventually, I should change this from a run_return invocation to normal run, and
            // handle all the Vulkan resource teardown during this event.
            Event::LoopDestroyed => (),
            _ => (),
        }
    });

    unsafe { renderer.logical_device.device_wait_idle() }.unwrap();

    for fence in sync.in_flight {
        unsafe { renderer.logical_device.destroy_fence(fence, None) };
    }
    for semaphore in sync.render_finished {
        unsafe { renderer.logical_device.destroy_semaphore(semaphore, None) };
    }
    for semaphore in sync.image_available {
        unsafe { renderer.logical_device.destroy_semaphore(semaphore, None) };
    }
    unsafe {
        renderer
            .logical_device
            .destroy_descriptor_pool(descriptor_pool, None)
    };
    for buffer in uniform_buffers {
        unsafe { renderer.logical_device.destroy_buffer(buffer, None) };
    }
    for memory in uniform_buffer_memories {
        unsafe { renderer.logical_device.free_memory(memory, None) };
    }
    unsafe { renderer.logical_device.destroy_buffer(index_buffer, None) };
    unsafe {
        renderer
            .logical_device
            .free_memory(index_buffer_memory, None)
    };
    unsafe { renderer.logical_device.destroy_buffer(vertex_buffer, None) };
    unsafe {
        renderer
            .logical_device
            .free_memory(vertex_buffer_memory, None)
    };
    unsafe {
        renderer
            .logical_device
            .destroy_sampler(texture_sampler, None)
    };
    unsafe {
        renderer
            .logical_device
            .destroy_image_view(texture_image_view, None)
    };
    unsafe { renderer.logical_device.destroy_image(texture_image, None) };
    unsafe {
        renderer
            .logical_device
            .free_memory(texture_image_memory, None)
    };
    for framebuffer in &framebuffers {
        unsafe {
            renderer
                .logical_device
                .destroy_framebuffer(*framebuffer, None)
        };
    }
    unsafe {
        renderer
            .logical_device
            .destroy_image_view(depth_image_view, None)
    };
    unsafe { renderer.logical_device.destroy_image(depth_image, None) };
    unsafe {
        renderer
            .logical_device
            .free_memory(depth_image_memory, None)
    };
    unsafe {
        renderer
            .logical_device
            .destroy_image_view(color_image_view, None)
    };
    unsafe { renderer.logical_device.destroy_image(color_image, None) };
    unsafe {
        renderer
            .logical_device
            .free_memory(color_image_memory, None)
    };
    unsafe {
        renderer
            .logical_device
            .destroy_command_pool(command_pool, None)
    };
    drop(pipeline);
    unsafe {
        renderer
            .logical_device
            .destroy_descriptor_set_layout(descriptor_set_layout, None)
    };
    drop(swapchain);
    unsafe { renderer.logical_device.destroy_device(None) };
    unsafe {
        renderer
            .extensions
            .surface
            .destroy_surface(renderer.surface, None)
    };
    unsafe {
        renderer
            .extensions
            .debug
            .destroy_debug_utils_messenger(renderer.debug_messenger, None)
    };
    unsafe { renderer.instance.destroy_instance(None) };
}

fn create_instance(entry: &Entry, window: &Window) -> Instance {
    // Set metadata of the app and the engine. May be used by the drivers to enable game-specific
    // and engine-specific optimizations, which won't happen, but let's set it to something sensible
    // anyway.
    let app_info = vk::ApplicationInfo::builder()
        .application_name(VULKAN_APP_NAME)
        .application_version(VULKAN_APP_VERSION)
        .engine_name(VULKAN_ENGINE_NAME)
        .engine_version(VULKAN_ENGINE_VERSION)
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
            &entry,
            &instance,
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

fn create_framebuffers(
    pipeline: &VulkanPipeline,
    depth_image_view: vk::ImageView,
    color_image_view: vk::ImageView,
) -> Vec<vk::Framebuffer> {
    let mut framebuffers = vec![vk::Framebuffer::null(); pipeline.swapchain.image_count()];
    for i in 0..pipeline.swapchain.image_count() {
        let attachments = [
            color_image_view,
            depth_image_view,
            pipeline.swapchain.image_views[i],
        ];
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(pipeline.render_pass)
            .attachments(&attachments)
            .width(pipeline.swapchain.extent.width)
            .height(pipeline.swapchain.extent.height)
            .layers(1);
        let framebuffer = unsafe {
            pipeline
                .swapchain
                .logical_device
                .create_framebuffer(&framebuffer_info, None)
        }
        .unwrap();
        framebuffers[i] = framebuffer;
    }
    framebuffers
}

fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
    [
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: std::mem::size_of::<glm::Vec3>() as u32,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 2,
            format: vk::Format::R32G32_SFLOAT,
            offset: std::mem::size_of::<glm::Vec3>() as u32 * 2,
        },
    ]
}

fn find_memory_type(
    instance: &Instance,
    device: vk::PhysicalDevice,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> u32 {
    let memory = unsafe { instance.get_physical_device_memory_properties(device) };
    for i in 0..memory.memory_type_count {
        if type_filter & (1 << i) != 0
            && !(memory.memory_types[i as usize].property_flags & properties).is_empty()
        {
            return i;
        }
    }
    panic!(
        "no good memory type_filter={type_filter} properties={properties:?} {:#?}",
        properties
    );
}

fn create_buffer(
    logical_device: &Device,
    size: usize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_info = *vk::BufferCreateInfo::builder()
        .size(size as u64)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let buffer = unsafe { logical_device.create_buffer(&buffer_info, None) }.unwrap();
    let requirements = unsafe { logical_device.get_buffer_memory_requirements(buffer) };
    let memory_type_index = find_memory_type(
        instance,
        physical_device,
        requirements.memory_type_bits,
        properties,
    );
    let alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type_index);
    let memory = unsafe { logical_device.allocate_memory(&alloc_info, None) }.unwrap();
    unsafe { logical_device.bind_buffer_memory(buffer, memory, 0) }.unwrap();
    (buffer, memory)
}

fn create_image(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
    width: usize,
    height: usize,
    mip_levels: usize,
    samples: vk::SampleCountFlags,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    memory: vk::MemoryPropertyFlags,
) -> (vk::Image, vk::DeviceMemory) {
    let image_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            width: width as u32,
            height: height as u32,
            depth: 1,
        })
        .mip_levels(mip_levels as u32)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(samples);
    let image = unsafe { logical_device.create_image(&image_info, None) }.unwrap();

    let requirements = unsafe { logical_device.get_image_memory_requirements(image) };
    let memory_type = find_memory_type(
        instance,
        physical_device,
        requirements.memory_type_bits,
        memory,
    );
    let alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type);
    let image_memory = unsafe { logical_device.allocate_memory(&alloc_info, None) }.unwrap();
    unsafe { logical_device.bind_image_memory(image, image_memory, 0) }.unwrap();

    (image, image_memory)
}

fn find_supported_format(
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
    physical_device: vk::PhysicalDevice,
    instance: &Instance,
) -> vk::Format {
    for format in candidates {
        let props =
            unsafe { instance.get_physical_device_format_properties(physical_device, *format) };
        if tiling == vk::ImageTiling::LINEAR
            && (props.linear_tiling_features & features) == features
        {
            return *format;
        } else if tiling == vk::ImageTiling::OPTIMAL
            && (props.optimal_tiling_features & features) == features
        {
            return *format;
        }
    }
    panic!("no supported format");
}

fn create_color_resources(
    swapchain: &VulkanSwapchain,
    msaa_samples: vk::SampleCountFlags,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> (vk::Image, vk::DeviceMemory, vk::ImageView) {
    let format = swapchain.image_format;
    let (image, image_memory) = create_image(
        instance,
        physical_device,
        swapchain.logical_device,
        swapchain.extent.width as usize,
        swapchain.extent.height as usize,
        1,
        msaa_samples,
        format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );
    let image_view = create_image_view(
        swapchain.logical_device,
        image,
        format,
        vk::ImageAspectFlags::COLOR,
        1,
    );
    (image, image_memory, image_view)
}

fn find_depth_format(physical_device: vk::PhysicalDevice, instance: &Instance) -> vk::Format {
    find_supported_format(
        &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ],
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        physical_device,
        instance,
    )
}

fn has_stencil_component(format: vk::Format) -> bool {
    format == vk::Format::D32_SFLOAT_S8_UINT || format == vk::Format::D24_UNORM_S8_UINT
}

fn create_depth_resources(
    swapchain: &VulkanSwapchain,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    msaa_samples: vk::SampleCountFlags,
    physical_device: vk::PhysicalDevice,
    instance: &Instance,
) -> (vk::Image, vk::DeviceMemory, vk::ImageView) {
    let format = find_depth_format(physical_device, instance);
    let (image, image_memory) = create_image(
        instance,
        physical_device,
        swapchain.logical_device,
        swapchain.extent.width as usize,
        swapchain.extent.height as usize,
        1,
        msaa_samples,
        format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );
    let image_view = create_image_view(
        swapchain.logical_device,
        image,
        format,
        vk::ImageAspectFlags::DEPTH,
        1,
    );
    transition_image_layout(
        swapchain.logical_device,
        graphics_queue,
        command_pool,
        image,
        format,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        1,
    );
    (image, image_memory, image_view)
}

fn create_texture_image(
    path: &str,
    logical_device: &Device,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> (vk::Image, vk::DeviceMemory, usize) {
    let image = image::open(path).unwrap().to_rgba8();
    let pixel_count = image.width() as usize * image.height() as usize;
    let image_size = pixel_count * 4;
    let mip_levels = (image.width().max(image.height()) as f32).log2().floor() as usize + 1;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        logical_device,
        image_size,
        vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        instance,
        physical_device,
    );
    let staging_ptr = unsafe {
        logical_device.map_memory(
            staging_buffer_memory,
            0,
            image_size as u64,
            vk::MemoryMapFlags::empty(),
        )
    }
    .unwrap();
    unsafe {
        std::ptr::copy_nonoverlapping(image.as_ptr(), staging_ptr as *mut u8, image_size);
    }
    unsafe { logical_device.unmap_memory(staging_buffer_memory) };

    let (texture_image, texture_image_memory) = create_image(
        instance,
        physical_device,
        logical_device,
        image.width() as usize,
        image.height() as usize,
        mip_levels,
        vk::SampleCountFlags::TYPE_1,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::SAMPLED,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    transition_image_layout(
        logical_device,
        graphics_queue,
        command_pool,
        texture_image,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        mip_levels,
    );
    copy_buffer_to_image(
        logical_device,
        graphics_queue,
        command_pool,
        staging_buffer,
        texture_image,
        image.width() as usize,
        image.height() as usize,
    );
    generate_mipmaps(
        texture_image,
        vk::Format::R8G8B8A8_SRGB,
        image.width() as usize,
        image.height() as usize,
        mip_levels,
        logical_device,
        graphics_queue,
        command_pool,
        instance,
        physical_device,
    );

    unsafe { logical_device.destroy_buffer(staging_buffer, None) };
    unsafe { logical_device.free_memory(staging_buffer_memory, None) };

    (texture_image, texture_image_memory, mip_levels)
}

fn generate_mipmaps(
    image: vk::Image,
    format: vk::Format,
    tex_width: usize,
    tex_height: usize,
    mip_levels: usize,
    logical_device: &Device,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) {
    let format_properties =
        unsafe { instance.get_physical_device_format_properties(physical_device, format) };
    assert!(format_properties
        .optimal_tiling_features
        .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR));

    single_time_commands(
        logical_device,
        graphics_queue,
        command_pool,
        move |command_buffer| {
            let mut barrier = *vk::ImageMemoryBarrier::builder()
                .image(image)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0, // Will be set before submitting each command.
                    base_array_layer: 0,
                    layer_count: 1,
                    level_count: 1,
                });
            let mut mip_width = tex_width;
            let mut mip_height = tex_height;
            for i in 1..mip_levels {
                barrier.subresource_range.base_mip_level = i as u32 - 1;
                barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
                barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
                barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
                barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;
                unsafe {
                    logical_device.cmd_pipeline_barrier(
                        command_buffer,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[barrier],
                    )
                };

                let blit = vk::ImageBlit::builder()
                    .src_offsets([
                        vk::Offset3D { x: 0, y: 0, z: 0 },
                        vk::Offset3D {
                            x: mip_width as i32,
                            y: mip_height as i32,
                            z: 1,
                        },
                    ])
                    .src_subresource(vk::ImageSubresourceLayers {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        mip_level: i as u32 - 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .dst_offsets([
                        vk::Offset3D { x: 0, y: 0, z: 0 },
                        vk::Offset3D {
                            x: if mip_width > 1 {
                                mip_width as i32 / 2
                            } else {
                                1
                            },
                            y: if mip_height > 1 {
                                mip_height as i32 / 2
                            } else {
                                1
                            },
                            z: 1,
                        },
                    ])
                    .dst_subresource(vk::ImageSubresourceLayers {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        mip_level: i as u32,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                unsafe {
                    logical_device.cmd_blit_image(
                        command_buffer,
                        image,
                        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                        image,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[*blit],
                        vk::Filter::LINEAR,
                    )
                };

                barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
                barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
                barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;
                unsafe {
                    logical_device.cmd_pipeline_barrier(
                        command_buffer,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::FRAGMENT_SHADER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[barrier],
                    )
                };

                if mip_width > 1 {
                    mip_width /= 2;
                }
                if mip_height > 1 {
                    mip_height /= 2;
                }
            }

            barrier.subresource_range.base_mip_level = mip_levels as u32 - 1;
            barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;
            unsafe {
                logical_device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                )
            };
        },
    );
}

fn create_image_view(
    logical_device: &Device,
    image: vk::Image,
    format: vk::Format,
    aspect_mask: vk::ImageAspectFlags,
    mip_levels: usize,
) -> vk::ImageView {
    let view_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: 0,
            level_count: mip_levels as u32,
            base_array_layer: 0,
            layer_count: 1,
        });
    unsafe { logical_device.create_image_view(&view_info, None) }.unwrap()
}

fn create_texture_image_view(
    logical_device: &Device,
    texture_image: vk::Image,
    mip_levels: usize,
) -> vk::ImageView {
    create_image_view(
        logical_device,
        texture_image,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageAspectFlags::COLOR,
        mip_levels,
    )
}

fn create_texture_sampler(
    logical_device: &Device,
    mip_levels: usize,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
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

fn create_vertex_buffer(
    vertex_data: &[Vertex],
    logical_device: &Device,
    command_pool: vk::CommandPool,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    graphics_queue: vk::Queue,
) -> (vk::Buffer, vk::DeviceMemory) {
    let vertex_size = std::mem::size_of::<Vertex>();
    let vertex_buffer_size = vertex_size * vertex_data.len();
    let (staging_buffer, staging_buffer_memory) = create_buffer(
        &logical_device,
        vertex_buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        instance,
        physical_device,
    );
    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        &logical_device,
        vertex_buffer_size,
        vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        instance,
        physical_device,
    );
    let staging_ptr = unsafe {
        logical_device.map_memory(
            staging_buffer_memory,
            0,
            vertex_buffer_size as u64,
            vk::MemoryMapFlags::empty(),
        )
    }
    .unwrap();
    unsafe { std::slice::from_raw_parts_mut(staging_ptr as *mut Vertex, vertex_data.len()) }
        .copy_from_slice(&vertex_data);
    unsafe { logical_device.unmap_memory(staging_buffer_memory) };
    copy_buffer(
        &logical_device,
        staging_buffer,
        vertex_buffer,
        vertex_buffer_size,
        command_pool,
        graphics_queue,
    );
    unsafe { logical_device.destroy_buffer(staging_buffer, None) };
    unsafe { logical_device.free_memory(staging_buffer_memory, None) };
    (vertex_buffer, vertex_buffer_memory)
}

fn transition_image_layout(
    logical_device: &Device,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    image: vk::Image,
    format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    mip_levels: usize,
) {
    single_time_commands(
        logical_device,
        graphics_queue,
        command_pool,
        move |command_buffer| {
            let barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(old_layout)
                .new_layout(new_layout)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
                    {
                        if has_stencil_component(format) {
                            vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
                        } else {
                            vk::ImageAspectFlags::DEPTH
                        }
                    } else {
                        vk::ImageAspectFlags::COLOR
                    },
                    base_mip_level: 0,
                    level_count: mip_levels as u32,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            let (barrier, source_stage, destination_stage) = if old_layout
                == vk::ImageLayout::UNDEFINED
                && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            {
                (
                    barrier
                        .src_access_mask(vk::AccessFlags::empty())
                        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE),
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                )
            } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
                && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            {
                (
                    barrier
                        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .dst_access_mask(vk::AccessFlags::SHADER_READ),
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                )
            } else if old_layout == vk::ImageLayout::UNDEFINED
                && new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            {
                (
                    barrier
                        .src_access_mask(vk::AccessFlags::empty())
                        .dst_access_mask(
                            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        ),
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                )
            } else {
                panic!("unsupported layout transition");
            };

            unsafe {
                logical_device.cmd_pipeline_barrier(
                    command_buffer,
                    source_stage,
                    destination_stage,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[*barrier],
                )
            };
        },
    );
}

fn copy_buffer_to_image(
    logical_device: &Device,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    buffer: vk::Buffer,
    image: vk::Image,
    width: usize,
    height: usize,
) {
    single_time_commands(
        logical_device,
        graphics_queue,
        command_pool,
        move |command_buffer| {
            let region = vk::BufferImageCopy {
                buffer_offset: 0,
                buffer_row_length: 0,
                buffer_image_height: 0,
                image_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
                image_extent: vk::Extent3D {
                    width: width as u32,
                    height: height as u32,
                    depth: 1,
                },
            };

            unsafe {
                logical_device.cmd_copy_buffer_to_image(
                    command_buffer,
                    buffer,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[region],
                )
            };
        },
    );
}

fn create_index_buffer(
    index_data: &[u32],
    logical_device: &Device,
    command_pool: vk::CommandPool,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    graphics_queue: vk::Queue,
) -> (vk::Buffer, vk::DeviceMemory) {
    let index_size = std::mem::size_of_val(&index_data[0]);
    let index_buffer_size = index_size * index_data.len();
    let (staging_buffer, staging_buffer_memory) = create_buffer(
        &logical_device,
        index_buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        instance,
        physical_device,
    );
    let (index_buffer, index_buffer_memory) = create_buffer(
        &logical_device,
        index_buffer_size,
        vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        instance,
        physical_device,
    );
    let staging_ptr = unsafe {
        logical_device.map_memory(
            staging_buffer_memory,
            0,
            index_buffer_size as u64,
            vk::MemoryMapFlags::empty(),
        )
    }
    .unwrap();
    unsafe { std::slice::from_raw_parts_mut(staging_ptr as *mut u32, index_data.len()) }
        .copy_from_slice(&index_data);
    unsafe { logical_device.unmap_memory(staging_buffer_memory) };
    copy_buffer(
        &logical_device,
        staging_buffer,
        index_buffer,
        index_buffer_size,
        command_pool,
        graphics_queue,
    );
    unsafe { logical_device.destroy_buffer(staging_buffer, None) };
    unsafe { logical_device.free_memory(staging_buffer_memory, None) };
    (index_buffer, index_buffer_memory)
}

fn create_uniform_buffer(
    logical_device: &Device,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> (
    Vec<vk::Buffer>,
    Vec<vk::DeviceMemory>,
    Vec<*mut UniformBufferObject>,
) {
    let mut buffers = Vec::new();
    let mut memories = Vec::new();
    let mut mappings = Vec::new();
    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        let buffer_size = std::mem::size_of::<UniformBufferObject>();
        let (buffer, memory) = create_buffer(
            &logical_device,
            buffer_size,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            instance,
            physical_device,
        );
        let mapping = unsafe {
            logical_device.map_memory(memory, 0, buffer_size as u64, vk::MemoryMapFlags::empty())
        }
        .unwrap() as *mut UniformBufferObject;
        buffers.push(buffer);
        memories.push(memory);
        mappings.push(mapping);
    }
    (buffers, memories, mappings)
}

fn create_descriptor_set_layout(logical_device: &Device) -> vk::DescriptorSetLayout {
    let ubo_layout_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);
    let sampler_layout_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_count(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);
    let layout_bindings = [*ubo_layout_binding, *sampler_layout_binding];
    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&layout_bindings);
    unsafe { logical_device.create_descriptor_set_layout(&layout_info, None) }.unwrap()
}

fn create_descriptor_pool(logical_device: &Device) -> vk::DescriptorPool {
    let pool_sizes = [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: MAX_FRAMES_IN_FLIGHT as u32,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: MAX_FRAMES_IN_FLIGHT as u32,
        },
    ];
    let pool_info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(&pool_sizes)
        .max_sets(MAX_FRAMES_IN_FLIGHT as u32);
    unsafe { logical_device.create_descriptor_pool(&pool_info, None) }.unwrap()
}

fn create_descriptor_sets(
    layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    uniform_buffers: &[vk::Buffer],
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,
    logical_device: &Device,
) -> Vec<vk::DescriptorSet> {
    let layouts = vec![layout; MAX_FRAMES_IN_FLIGHT];
    let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let descriptor_sets =
        unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_alloc_info) }.unwrap();
    for i in 0..MAX_FRAMES_IN_FLIGHT {
        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(uniform_buffers[i])
            .offset(0)
            .range(std::mem::size_of::<UniformBufferObject>() as u64);
        let buffer_infos = [*buffer_info];
        let image_info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(texture_image_view)
            .sampler(texture_sampler);
        let image_infos = [*image_info];
        let descriptor_writes = [
            *vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos),
            *vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_infos),
        ];
        unsafe { logical_device.update_descriptor_sets(&descriptor_writes, &[]) };
    }
    descriptor_sets
}

fn create_sync<'a>(logical_device: &Device) -> Synchronization {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
    let mut image_available: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT] = Default::default();
    let mut render_finished: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT] = Default::default();
    let mut in_flight: [vk::Fence; MAX_FRAMES_IN_FLIGHT] = Default::default();
    for i in 0..MAX_FRAMES_IN_FLIGHT {
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

fn get_max_usable_sample_count(
    physical_device: vk::PhysicalDevice,
    instance: &Instance,
) -> vk::SampleCountFlags {
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };
    let counts = properties.limits.framebuffer_color_sample_counts
        & properties.limits.framebuffer_depth_sample_counts;
    if counts.contains(vk::SampleCountFlags::TYPE_64) {
        return vk::SampleCountFlags::TYPE_64;
    }
    if counts.contains(vk::SampleCountFlags::TYPE_32) {
        return vk::SampleCountFlags::TYPE_32;
    }
    if counts.contains(vk::SampleCountFlags::TYPE_16) {
        return vk::SampleCountFlags::TYPE_16;
    }
    if counts.contains(vk::SampleCountFlags::TYPE_8) {
        return vk::SampleCountFlags::TYPE_8;
    }
    if counts.contains(vk::SampleCountFlags::TYPE_4) {
        return vk::SampleCountFlags::TYPE_4;
    }
    if counts.contains(vk::SampleCountFlags::TYPE_2) {
        return vk::SampleCountFlags::TYPE_2;
    }
    vk::SampleCountFlags::TYPE_1
}

fn create_command_pool(queue_families: &QueueFamilies, logical_device: &Device) -> vk::CommandPool {
    let command_pool_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_families.graphics);
    unsafe { logical_device.create_command_pool(&command_pool_info, None) }.unwrap()
}

fn create_command_buffers(
    command_pool: vk::CommandPool,
    logical_device: &Device,
) -> Vec<vk::CommandBuffer> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);
    unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info) }.unwrap()
}

fn copy_buffer(
    logical_device: &Device,
    src: vk::Buffer,
    dst: vk::Buffer,
    len: usize,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
) {
    single_time_commands(
        logical_device,
        graphics_queue,
        command_pool,
        move |command_buffer| {
            let copy_region = vk::BufferCopy::builder()
                .src_offset(0)
                .dst_offset(0)
                .size(len as u64);
            unsafe { logical_device.cmd_copy_buffer(command_buffer, src, dst, &[*copy_region]) };
        },
    );
}

fn single_time_commands<R>(
    logical_device: &Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
    f: impl FnOnce(vk::CommandBuffer) -> R,
) -> R {
    let command_info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(command_pool)
        .command_buffer_count(1);
    let command_buffer = unsafe { logical_device.allocate_command_buffers(&command_info) }
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    let begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { logical_device.begin_command_buffer(command_buffer, &begin_info) }.unwrap();

    let result = f(command_buffer);

    unsafe { logical_device.end_command_buffer(command_buffer) }.unwrap();

    let submit_buffers = [command_buffer];
    let submit_info = vk::SubmitInfo::builder().command_buffers(&submit_buffers);
    unsafe { logical_device.queue_submit(queue, &[*submit_info], vk::Fence::null()) }.unwrap();
    unsafe { logical_device.queue_wait_idle(queue) }.unwrap();
    unsafe { logical_device.free_command_buffers(command_pool, &[command_buffer]) };

    result
}

fn draw_frame(
    device: &Device,
    in_flight_fence: vk::Fence,
    swapchain: &VulkanSwapchain,
    image_available_semaphore: vk::Semaphore,
    command_buffer: vk::CommandBuffer,
    framebuffers: &[vk::Framebuffer],
    pipeline: &VulkanPipeline,
    render_finished_semaphore: vk::Semaphore,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    index_count: usize,
    ubo_ptr: *mut UniformBufferObject,
    descriptor_set: vk::DescriptorSet,
    camera: &Camera,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
) {
    unsafe { device.wait_for_fences(&[in_flight_fence], true, u64::MAX) }.unwrap();
    unsafe { device.reset_fences(&[in_flight_fence]) }.unwrap();
    // What is the second value?
    let image_index = unsafe {
        swapchain.ext.acquire_next_image(
            swapchain.swapchain,
            u64::MAX,
            image_available_semaphore,
            vk::Fence::null(),
        )
    }
    .unwrap()
    .0;
    unsafe { device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty()) }
        .unwrap();
    record_command_buffer(
        device,
        command_buffer,
        image_index,
        framebuffers,
        swapchain.extent,
        &pipeline,
        vertex_buffer,
        index_buffer,
        index_count,
        descriptor_set,
    );

    update_uniform_buffer(
        ubo_ptr,
        swapchain.extent.width as f32 / swapchain.extent.height as f32,
        camera,
    );

    let wait_semaphores = [image_available_semaphore];
    let command_buffers = [command_buffer];
    let signal_semaphores = [render_finished_semaphore];
    let submit_info = vk::SubmitInfo::builder()
        .wait_semaphores(&wait_semaphores)
        .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
        .command_buffers(&command_buffers)
        .signal_semaphores(&signal_semaphores);
    unsafe { device.queue_submit(graphics_queue, &[*submit_info], in_flight_fence) }.unwrap();

    let present_info_swapchains = [swapchain.swapchain];
    let present_info_images = [image_index];
    let present_info = vk::PresentInfoKHR::builder()
        .wait_semaphores(&signal_semaphores)
        .swapchains(&present_info_swapchains)
        .image_indices(&present_info_images);
    unsafe { swapchain.ext.queue_present(present_queue, &present_info) }.unwrap();
}

fn record_command_buffer(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    image_index: u32,
    framebuffers: &[vk::Framebuffer],
    swapchain_extent: vk::Extent2D,
    pipeline: &VulkanPipeline,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    index_count: usize,
    descriptor_set: vk::DescriptorSet,
) {
    let begin_info = vk::CommandBufferBeginInfo::builder();
    unsafe { device.begin_command_buffer(command_buffer, &begin_info) }.unwrap();

    let render_pass_info = vk::RenderPassBeginInfo::builder()
        .render_pass(pipeline.render_pass)
        .framebuffer(framebuffers[image_index as usize])
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
        })
        .clear_values(&[
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0., 0., 0., 0.],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.,
                    stencil: 0,
                },
            },
        ]);
    unsafe {
        device.cmd_begin_render_pass(
            command_buffer,
            &render_pass_info,
            vk::SubpassContents::INLINE,
        )
    };

    unsafe {
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline.pipeline,
        )
    };

    let buffers = [vertex_buffer];
    let offsets = [0];
    unsafe { device.cmd_bind_vertex_buffers(command_buffer, 0, &buffers, &offsets) };

    unsafe { device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32) };

    let viewport = vk::Viewport {
        x: 0.,
        y: 0.,
        width: swapchain_extent.width as f32,
        height: swapchain_extent.height as f32,
        min_depth: 0.,
        max_depth: 1.,
    };
    unsafe { device.cmd_set_viewport(command_buffer, 0, &[viewport]) };

    let scissor = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: swapchain_extent,
    };
    unsafe { device.cmd_set_scissor(command_buffer, 0, &[scissor]) };

    unsafe {
        device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline.pipeline_layout,
            0,
            &[descriptor_set],
            &[],
        )
    };

    unsafe { device.cmd_draw_indexed(command_buffer, index_count as u32, 1, 0, 0, 0) };

    unsafe { device.cmd_end_render_pass(command_buffer) };

    unsafe { device.end_command_buffer(command_buffer) }.unwrap();
}

fn update_uniform_buffer(ubo_ptr: *mut UniformBufferObject, aspect_ratio: f32, camera: &Camera) {
    let mut ubo = UniformBufferObject {
        model: glm::identity(),
        view: camera.view_matrix(),
        proj: glm::perspective_rh_zo(aspect_ratio, FRAC_PI_4, 0.1, 10.),
    };
    ubo.proj[(1, 1)] *= -1.;
    unsafe { ubo_ptr.write_volatile(ubo) };
}
