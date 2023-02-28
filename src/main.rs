#![feature(const_cstr_methods)]

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::prelude::VkResult;
use ash::vk::ComponentSwizzle;
use ash::{vk, Device, Entry, Instance};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::borrow::Cow;
use std::collections::HashSet;
use std::ffi::CStr;
use std::ops::Deref;
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::WindowBuilder;

const WINDOW_TITLE: &str = "Vulkthing";
const WINDOW_SIZE: (usize, usize) = (800, 600);

const VULKAN_APP_NAME: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"Vulkthing\0") };
const VULKAN_APP_VERSION: u32 = vk::make_api_version(0, 0, 0, 0);
const VULKAN_ENGINE_NAME: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"Vulkthing\0") };
const VULKAN_ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 0, 0);

struct VulkanInstance {
    instance: Instance,
    ext: VulkanInstanceExts,
}

struct VulkanInstanceExts {
    debug: Arc<DebugUtils>,
    surface: Arc<Surface>,
}

struct VulkanDebug {
    ext: Arc<DebugUtils>,
    messenger: vk::DebugUtilsMessengerEXT,
}

struct VulkanSurface {
    ext: Arc<Surface>,
    surface: vk::SurfaceKHR,
}

struct VulkanPhysicalDevice {
    device: vk::PhysicalDevice,
    queues: QueueDetails,
    swapchain: SwapchainDetails,
}

struct QueueDetails {
    graphics_family: u32,
    present_family: u32,
}

struct SwapchainDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

struct VulkanLogicalDevice {
    device: Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
}

impl VulkanInstance {
    fn create(entry: &Entry, window: &winit::window::Window) -> VulkanInstance {
        // Set metadata of the app and the engine. May be used by the drivers to enable
        // game-specific and engine-specific optimizations, which won't happen, but let's set it to
        // something sensible anyway.
        let app_info = vk::ApplicationInfo::builder()
            .application_name(VULKAN_APP_NAME)
            .application_version(VULKAN_APP_VERSION)
            .engine_name(VULKAN_ENGINE_NAME)
            .engine_version(VULKAN_ENGINE_VERSION)
            .api_version(vk::API_VERSION_1_0);

        // Enable Vulkan validation layers. This should be later disabled in non-development builds.
        let layer_names = [CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0")
            .unwrap()
            .as_ptr()];

        // Vulkan doesn't appear to have any interesting extensions at this level, physical device
        // extensions are the interesting ones with raytracing and other stuff. This is just for
        // OS-specific windowing system interactions, and enabling debug logging for the validation
        // layers.
        let mut extension_names =
            ash_window::enumerate_required_extensions(window.raw_display_handle())
                .unwrap()
                .to_vec();
        extension_names.push(DebugUtils::name().as_ptr());

        let instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layer_names)
            .enabled_extension_names(&extension_names);
        let instance = unsafe { entry.create_instance(&instance_create_info, None) }.unwrap();

        // Load the extension function pointers. The DebugUtils extension was explicitly added to
        // extension_names list, and Surface is implied by enumerate_required_extensions.
        let debug = Arc::new(DebugUtils::new(&entry, &instance));
        let surface = Arc::new(Surface::new(&entry, &instance));
        let ext = VulkanInstanceExts { debug, surface };

        VulkanInstance { instance, ext }
    }
}

impl VulkanDebug {
    fn create(instance: &VulkanInstance) -> VulkanDebug {
        // Enable filtering by message severity and type. General and verbose levels seem to produce
        // too much noise related to physical device selection, so I turned them off.
        // vulkan-tutorial.com also shows how to enable this for creating instances, but the ash
        // example doesn't include this.
        let severity_filter = vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
            | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING;
        let type_filter = vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE;
        let info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(severity_filter)
            .message_type(type_filter)
            .pfn_user_callback(Some(vulkan_debug_callback));
        let messenger =
            unsafe { instance.ext.debug.create_debug_utils_messenger(&info, None) }.unwrap();
        VulkanDebug {
            ext: instance.ext.debug.clone(),
            messenger,
        }
    }
}

impl VulkanSurface {
    fn create(
        entry: &Entry,
        instance: &VulkanInstance,
        window: &winit::window::Window,
    ) -> VulkanSurface {
        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
        }
        .unwrap();
        VulkanSurface {
            ext: instance.ext.surface.clone(),
            surface,
        }
    }
}

impl VulkanPhysicalDevice {
    fn find(instance: &VulkanInstance, surface: &VulkanSurface) -> VulkanPhysicalDevice {
        // Select the GPU. For now, just select the first discrete GPU with graphics support. Later,
        // this should react better to iGPU, dGPU and iGPU+dGPU setups. In more complex setups, it would
        // be neat if you could start the game on any GPU, display a choice to the user and seamlessly
        // switch to a new physical device.
        let mut found = None;
        for device in unsafe { instance.enumerate_physical_devices() }.unwrap() {
            let properties = unsafe { instance.get_physical_device_properties(device) };
            let name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
                .to_str()
                .unwrap()
                .to_owned();

            // The GPU has to have a graphics queue. Otherwise there's no way to do any rendering
            // operations, so this must be some weird compute-only accelerator or something. This
            // also checks whether there is a present queue. This could be worked around using two
            // separate GPUs (or just one for headless benchmarking), but the OS should take care of
            // handling this sort of stuff between devices, probably?
            let Some(queues) = QueueDetails::query(&instance, device, &surface).unwrap() else {
                println!("rejected gpu, no suitable queues ({name})");
                continue;
            };

            // Check whether the GPU supports the swapchain extension. This should be implied by the
            // presence of the present queue, but we can check this explicitly.
            let extensions =
                unsafe { instance.enumerate_device_extension_properties(device) }.unwrap();
            let has_swapchain_extension = extensions.iter().any(|ext| {
                let ext_name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                ext_name == Swapchain::name()
            });
            if !has_swapchain_extension {
                println!("rejected gpu, no swapchain extension ({name})");
                continue;
            }

            // This queries some more details about swapchain support, and apparently this requires
            // the earlier extension check in order to be correct (not crash?). Also there shouldn't
            // be devices that support swapchains but no formats or present modes, but let's check
            // anyway because the tutorial does.
            let swapchain = SwapchainDetails::query(&instance, device, &surface).unwrap();
            if swapchain.formats.is_empty() || swapchain.present_modes.is_empty() {
                println!("rejected gpu, unsuitable swapchain ({name})");
                continue;
            }

            // Reject GPUs once we found one already. I've seen debug logs indicating some
            // Linux-specific sorting is going on, so it sounds like the options should be ordered
            // sensibly already? Might be a good idea to check on a iGPU+dGPU laptop.
            if found.is_some() {
                println!("rejected gpu, one already selected ({name})");
                continue;
            }

            // Let's not break, because getting logs about other GPUs could possibly help debug
            // performance problems related to GPU selection.
            println!("accepted gpu: {name}");
            found = Some(VulkanPhysicalDevice {
                device,
                queues,
                swapchain,
            });
        }

        let Some(physical_device) = found else {
            panic!("gpu not found");
        };
        physical_device
    }
}

impl VulkanLogicalDevice {
    fn create(
        instance: &VulkanInstance,
        physical_device: &VulkanPhysicalDevice,
    ) -> VulkanLogicalDevice {
        // Queues from the same family must be created at once, so we need to use a set to eliminate
        // duplicates. If the queue families are the same, we create only a single queue and keep
        // two handles. This needs to be remembered later when setting flags related to memory
        // access being exclusive to the queue or concurrent from many queues.
        let queue_indices = HashSet::from([
            physical_device.queues.graphics_family,
            physical_device.queues.present_family,
        ]);
        let queue_creates: Vec<_> = queue_indices
            .iter()
            .map(|queue_index| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(*queue_index)
                    .queue_priorities(&[1.])
                    .build()
            })
            .collect();

        let physical_device_features = vk::PhysicalDeviceFeatures::builder();

        // Using validation layers on a device level shouldn't be necessary on newer Vulkan version
        // (since which one?), but it's good to keep it for compatibility.
        let layer_names = [b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const i8];

        let device = unsafe {
            instance.create_device(
                physical_device.device,
                &vk::DeviceCreateInfo::builder()
                    .queue_create_infos(&queue_creates)
                    .enabled_features(&physical_device_features)
                    .enabled_layer_names(&layer_names)
                    .enabled_extension_names(&[Swapchain::name().as_ptr()]),
                None,
            )
        }
        .unwrap();
        let graphics_queue =
            unsafe { device.get_device_queue(physical_device.queues.graphics_family, 0) };
        let present_queue =
            unsafe { device.get_device_queue(physical_device.queues.present_family, 0) };
        VulkanLogicalDevice {
            device,
            graphics_queue,
            present_queue,
        }
    }
}

impl QueueDetails {
    fn query(
        instance: &VulkanInstance,
        device: vk::PhysicalDevice,
        surface: &VulkanSurface,
    ) -> VkResult<Option<QueueDetails>> {
        // Find the first queue that supports a given operation and return it. Not sure what to do
        // when there are multiple queues that support an operation? Also, graphics queue being
        // distinct from present queue is supposed to be somewhat rare, so not sure where can I test
        // it.
        let queues = unsafe { instance.get_physical_device_queue_family_properties(device) };
        let Some(graphics_family) = QueueDetails::find_queue(&queues, |_, q| q.queue_flags.contains(vk::QueueFlags::GRAPHICS)) else {
            return Ok(None);
        };
        let Some(present_family) = QueueDetails::find_queue(&queues, |i, _| unsafe { instance.ext.surface.get_physical_device_surface_support(device, i, surface.surface) }
            .unwrap()) else {
            return Ok(None);
        };
        Ok(Some(QueueDetails {
            graphics_family,
            present_family,
        }))
    }

    fn find_queue(
        queues: &[vk::QueueFamilyProperties],
        p: impl Fn(u32, &vk::QueueFamilyProperties) -> bool,
    ) -> Option<u32> {
        for (index, queue) in queues.iter().enumerate() {
            let index = index as u32;
            if p(index, queue) {
                return Some(index);
            }
        }
        None
    }
}

impl SwapchainDetails {
    fn query(
        instance: &VulkanInstance,
        device: vk::PhysicalDevice,
        surface: &VulkanSurface,
    ) -> VkResult<SwapchainDetails> {
        let capabilities = unsafe {
            instance
                .ext
                .surface
                .get_physical_device_surface_capabilities(device, surface.surface)
        }
        .unwrap();
        let formats = unsafe {
            instance
                .ext
                .surface
                .get_physical_device_surface_formats(device, surface.surface)
        }
        .unwrap();
        let present_modes = unsafe {
            instance
                .ext
                .surface
                .get_physical_device_surface_present_modes(device, surface.surface)
        }
        .unwrap();
        Ok(SwapchainDetails {
            capabilities,
            formats,
            present_modes,
        })
    }

    fn select_format(&self) -> vk::SurfaceFormatKHR {
        self.formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&self.formats[0])
            .clone()
    }

    fn select_present_mode(&self) -> vk::PresentModeKHR {
        self.present_modes
            .iter()
            .copied()
            .find(|m| *m == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO)
    }

    fn select_swap_extent(&self, window: &winit::window::Window) -> vk::Extent2D {
        if self.capabilities.current_extent.width != u32::MAX {
            return self.capabilities.current_extent;
        }
        let window_size = window.inner_size();
        vk::Extent2D {
            width: window_size.width.clamp(
                self.capabilities.min_image_extent.width,
                self.capabilities.max_image_extent.width,
            ),
            height: window_size.height.clamp(
                self.capabilities.min_image_extent.height,
                self.capabilities.max_image_extent.height,
            ),
        }
    }

    fn select_image_count(&self) -> u32 {
        let no_image_limit = self.capabilities.max_image_count == 0;
        self.capabilities.min_image_count
            + if no_image_limit
                || self.capabilities.min_image_count + 1 <= self.capabilities.max_image_count
            {
                1
            } else {
                0
            }
    }
}

impl Deref for VulkanInstance {
    type Target = Instance;

    fn deref(&self) -> &Instance {
        &self.instance
    }
}

impl Deref for VulkanLogicalDevice {
    type Target = Device;

    fn deref(&self) -> &Device {
        &self.device
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) };
    }
}

impl Drop for VulkanDebug {
    fn drop(&mut self) {
        unsafe { self.ext.destroy_debug_utils_messenger(self.messenger, None) };
    }
}

impl Drop for VulkanSurface {
    fn drop(&mut self) {
        unsafe { self.ext.destroy_surface(self.surface, None) };
    }
}

impl Drop for VulkanLogicalDevice {
    fn drop(&mut self) {
        unsafe { self.device.destroy_device(None) };
    }
}

fn main() {
    // Create the application window using winit. Use a predefined size for now, though games should
    // run in fullscreen eventually.
    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_inner_size(winit::dpi::LogicalSize::new(
            WINDOW_SIZE.0 as f64,
            WINDOW_SIZE.1 as f64,
        ))
        .with_resizable(false)
        .with_decorations(false)
        .build(&event_loop)
        .unwrap();

    // Load the Vulkan library. This should probably use the dynamically loaded variant instead?
    let entry = unsafe { Entry::load() }.unwrap();

    let instance = VulkanInstance::create(&entry, &window);
    let _debug = VulkanDebug::create(&instance);
    let surface = VulkanSurface::create(&entry, &instance, &window);
    let physical_device = VulkanPhysicalDevice::find(&instance, &surface);
    let logical_device = VulkanLogicalDevice::create(&instance, &physical_device);

    // Create the swapchain for presenting images to display. Set to prefer triple buffering right
    // now, should be possible to change on laptops or integrated GPUs?
    let format = physical_device.swapchain.select_format();
    let present_mode = physical_device.swapchain.select_present_mode();
    let extent = physical_device.swapchain.select_swap_extent(&window);
    let image_count = physical_device.swapchain.select_image_count();
    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface.surface)
        .min_image_count(image_count)
        .image_format(format.format)
        .image_color_space(format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);
    let queue_family_indices = [
        physical_device.queues.graphics_family,
        physical_device.queues.present_family,
    ];
    let swapchain_create_info =
        if physical_device.queues.graphics_family != physical_device.queues.present_family {
            swapchain_create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices)
        } else {
            swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        };
    let swapchain_create_info = swapchain_create_info
        .pre_transform(physical_device.swapchain.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null());
    let swapchain = Swapchain::new(&instance, &logical_device.device);
    let swapchain_khr =
        unsafe { swapchain.create_swapchain(&swapchain_create_info, None) }.unwrap();
    let swapchain_images = unsafe { swapchain.get_swapchain_images(swapchain_khr) }.unwrap();
    let swapchain_image_format = format.format;
    let swapchain_extent = extent;

    // Create image views. Not really interesting for now, as I only use normal color settings.
    let mut swapchain_image_views = vec![vk::ImageView::null(); swapchain_images.len()];
    for i in 0..swapchain_images.len() {
        let image_view_create = vk::ImageViewCreateInfo::builder()
            .image(swapchain_images[i])
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(swapchain_image_format)
            .components(vk::ComponentMapping {
                r: ComponentSwizzle::IDENTITY,
                g: ComponentSwizzle::IDENTITY,
                b: ComponentSwizzle::IDENTITY,
                a: ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });
        swapchain_image_views[i] = unsafe {
            logical_device
                .device
                .create_image_view(&image_view_create, None)
        }
        .unwrap();
    }

    let vert_shader = make_shader(
        &logical_device,
        include_bytes!("../shaders/triangle-vert.spv"),
    );
    let frag_shader = make_shader(
        &logical_device,
        include_bytes!("../shaders/triangle-frag.spv"),
    );
    let vert_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader)
        .name(CStr::from_bytes_with_nul(b"main\0").unwrap());
    let frag_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader)
        .name(CStr::from_bytes_with_nul(b"main\0").unwrap());
    let shader_stages = [*vert_shader_stage, *frag_shader_stage];
    let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);
    let vertex_input = vk::PipelineVertexInputStateCreateInfo::builder();
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
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_bias_enable(false)
        .depth_bias_constant_factor(0.)
        .depth_bias_clamp(0.)
        .depth_bias_slope_factor(0.);
    let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::TYPE_1)
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
    let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(&[])
        .push_constant_ranges(&[]);
    let pipeline_layout =
        unsafe { logical_device.create_pipeline_layout(&pipeline_layout_info, None) }.unwrap();

    let color_attachment = vk::AttachmentDescription::builder()
        .format(swapchain_image_format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let color_attachments = [*color_attachment_ref];
    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachments);
    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
    let attachments = [*color_attachment];
    let subpasses = [*subpass];
    let dependencies = [*dependency];
    let render_pass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&dependencies);
    let render_pass =
        unsafe { logical_device.create_render_pass(&render_pass_info, None) }.unwrap();

    let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterizer)
        .multisample_state(&multisampling)
        .color_blend_state(&color_blending)
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

    let mut swapchain_framebuffers = vec![vk::Framebuffer::null(); swapchain_image_views.len()];
    for i in 0..swapchain_image_views.len() {
        let attachments = [swapchain_image_views[i]];
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(swapchain_extent.width)
            .height(swapchain_extent.height)
            .layers(1);
        let framebuffer =
            unsafe { logical_device.create_framebuffer(&framebuffer_info, None) }.unwrap();
        swapchain_framebuffers[i] = framebuffer;
    }

    let command_pool_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(physical_device.queues.graphics_family);
    let command_pool =
        unsafe { logical_device.create_command_pool(&command_pool_info, None) }.unwrap();
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let command_buffer =
        unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info) }
            .unwrap()
            .into_iter()
            .next()
            .unwrap();

    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
    let image_available_semaphore =
        unsafe { logical_device.create_semaphore(&semaphore_info, None) }.unwrap();
    let render_finished_semaphore =
        unsafe { logical_device.create_semaphore(&semaphore_info, None) }.unwrap();
    let in_flight_fence = unsafe { logical_device.create_fence(&fence_info, None) }.unwrap();

    // Run the event loop. Winit delivers events, like key presses. After it finishes delivering
    // some batch of events, it sends a MainEventsCleared event, which means the application should
    // either render, or check whether it needs to rerender anything and possibly only request a
    // redraw of a specific window. Redrawing a window can also be requested by the operating
    // system, for example if the window size changes. For games, initially I'll render at both
    // events, but this probably needs to be changed to alter framebuffer size if the window is
    // resized?
    event_loop.run_return(|event, _, control_flow| {
        control_flow.set_poll();
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => control_flow.set_exit(),
            Event::MainEventsCleared | Event::RedrawRequested(_) => {
                // render
                draw_frame(
                    &logical_device,
                    in_flight_fence,
                    &swapchain,
                    swapchain_khr,
                    image_available_semaphore,
                    command_buffer,
                    render_pass,
                    &swapchain_framebuffers,
                    swapchain_extent,
                    pipeline,
                    render_finished_semaphore,
                );
            }
            _ => (),
        }
    });

    unsafe { logical_device.device_wait_idle() }.unwrap();

    unsafe { logical_device.destroy_fence(in_flight_fence, None) };
    unsafe { logical_device.destroy_semaphore(render_finished_semaphore, None) };
    unsafe { logical_device.destroy_semaphore(image_available_semaphore, None) };
    unsafe { logical_device.destroy_command_pool(command_pool, None) };
    for framebuffer in swapchain_framebuffers {
        unsafe { logical_device.destroy_framebuffer(framebuffer, None) };
    }
    unsafe { logical_device.destroy_pipeline(pipeline, None) };
    unsafe { logical_device.destroy_render_pass(render_pass, None) };
    unsafe { logical_device.destroy_pipeline_layout(pipeline_layout, None) };
    unsafe { logical_device.destroy_shader_module(frag_shader, None) };
    unsafe { logical_device.destroy_shader_module(vert_shader, None) };
    for image_view in swapchain_image_views {
        unsafe { logical_device.destroy_image_view(image_view, None) };
    }
    unsafe { swapchain.destroy_swapchain(swapchain_khr, None) };
}

fn make_shader(device: &Device, code: &[u8]) -> vk::ShaderModule {
    let aligned_code = ash::util::read_spv(&mut std::io::Cursor::new(code)).unwrap();
    let shader_module_create = vk::ShaderModuleCreateInfo::builder().code(&aligned_code);
    unsafe { device.create_shader_module(&shader_module_create, None) }.unwrap()
}

fn draw_frame(
    device: &VulkanLogicalDevice,
    in_flight_fence: vk::Fence,
    swapchain: &Swapchain,
    swapchain_khr: vk::SwapchainKHR,
    image_available_semaphore: vk::Semaphore,
    command_buffer: vk::CommandBuffer,
    render_pass: vk::RenderPass,
    swapchain_framebuffers: &[vk::Framebuffer],
    swapchain_extent: vk::Extent2D,
    pipeline: vk::Pipeline,
    render_finished_semaphore: vk::Semaphore,
) {
    unsafe { device.wait_for_fences(&[in_flight_fence], true, u64::MAX) }.unwrap();
    unsafe { device.reset_fences(&[in_flight_fence]) }.unwrap();
    // What is the second value?
    let image_index = unsafe {
        swapchain.acquire_next_image(
            swapchain_khr,
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
        render_pass,
        swapchain_framebuffers,
        swapchain_extent,
        pipeline,
    );

    let wait_semaphores = [image_available_semaphore];
    let command_buffers = [command_buffer];
    let signal_semaphores = [render_finished_semaphore];
    let submit_info = vk::SubmitInfo::builder()
        .wait_semaphores(&wait_semaphores)
        .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
        .command_buffers(&command_buffers)
        .signal_semaphores(&signal_semaphores);
    unsafe { device.queue_submit(device.graphics_queue, &[*submit_info], in_flight_fence) }
        .unwrap();

    let present_info_swapchains = [swapchain_khr];
    let present_info_images = [image_index];
    let present_info = vk::PresentInfoKHR::builder()
        .wait_semaphores(&signal_semaphores)
        .swapchains(&present_info_swapchains)
        .image_indices(&present_info_images);
    unsafe { swapchain.queue_present(device.present_queue, &present_info) }.unwrap();
}

fn record_command_buffer(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    image_index: u32,
    render_pass: vk::RenderPass,
    swapchain_framebuffers: &[vk::Framebuffer],
    swapchain_extent: vk::Extent2D,
    pipeline: vk::Pipeline,
) {
    let begin_info = vk::CommandBufferBeginInfo::builder();
    unsafe { device.begin_command_buffer(command_buffer, &begin_info) }.unwrap();

    let render_pass_info = vk::RenderPassBeginInfo::builder()
        .render_pass(render_pass)
        .framebuffer(swapchain_framebuffers[image_index as usize])
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
        })
        .clear_values(&[vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0., 0., 0., 0.],
            },
        }]);
    unsafe {
        device.cmd_begin_render_pass(
            command_buffer,
            &render_pass_info,
            vk::SubpassContents::INLINE,
        )
    };

    unsafe { device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline) };

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

    unsafe { device.cmd_draw(command_buffer, 3, 1, 0, 0) };

    unsafe { device.cmd_end_render_pass(command_buffer) };

    unsafe { device.end_command_buffer(command_buffer) }.unwrap();
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;
    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };
    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };
    println!(
        "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
    );
    vk::FALSE
}
