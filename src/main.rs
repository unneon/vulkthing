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

struct QueueDetails {
    graphics_family: u32,
    present_family: u32,
}

struct SwapchainDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl QueueDetails {
    fn query(
        instance: &Instance,
        device: vk::PhysicalDevice,
        surface: &Surface,
        surface_khr: vk::SurfaceKHR,
    ) -> VkResult<Option<QueueDetails>> {
        let queues = unsafe { instance.get_physical_device_queue_family_properties(device) };
        let Some(graphics_family) = QueueDetails::find_queue(&queues, |_, q| q.queue_flags.contains(vk::QueueFlags::GRAPHICS)) else {
            return Ok(None);
        };
        let Some(present_family) = QueueDetails::find_queue(&queues, |i, _| unsafe { surface.get_physical_device_surface_support(device, i, surface_khr) }
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
        device: vk::PhysicalDevice,
        surface: &Surface,
        surface_khr: vk::SurfaceKHR,
    ) -> VkResult<SwapchainDetails> {
        let capabilities =
            unsafe { surface.get_physical_device_surface_capabilities(device, surface_khr) }
                .unwrap();
        let formats =
            unsafe { surface.get_physical_device_surface_formats(device, surface_khr) }.unwrap();
        let present_modes =
            unsafe { surface.get_physical_device_surface_present_modes(device, surface_khr) }
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
        dbg!(no_image_limit);
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
        .build(&event_loop)
        .unwrap();

    // Load the Vulkan library, and check what extensions it implements. This should probably use
    // the dynamically loaded variant instead?
    let entry = unsafe { Entry::load() }.unwrap();
    println!("scanning for vulkan extensions:");
    for ext in entry.enumerate_instance_extension_properties(None).unwrap() {
        println!("  {ext:?}");
    }

    // Create a Vulkan instance. Mostly metadata of the app and the engine, as well as required
    // extensions.
    let app_info = vk::ApplicationInfo::builder()
        .application_name(VULKAN_APP_NAME)
        .application_version(VULKAN_APP_VERSION)
        .engine_name(VULKAN_ENGINE_NAME)
        .engine_version(VULKAN_ENGINE_VERSION)
        .api_version(vk::API_VERSION_1_0);
    let layer_names = [CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0")
        .unwrap()
        .as_ptr()];
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

    // Set up the callback for debug messages from validation layers. vulkan-tutorial.com also shows
    // how to enable this for creating instances, but ash example doesn't include this.
    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(vulkan_debug_callback));
    let debug_utils_loader = DebugUtils::new(&entry, &instance);
    let debug_call_back =
        unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None) }.unwrap();

    // Create the KHR extension surface from winit object. This must be done before selecting a
    // physical device to check whether is supports presenting to the display.
    let surface_khr = unsafe {
        ash_window::create_surface(
            &entry,
            &instance,
            window.raw_display_handle(),
            window.raw_window_handle(),
            None,
        )
    }
    .unwrap();
    let surface = Surface::new(&entry, &instance);

    // Select the GPU. For now, just select the first discrete GPU with graphics support. Later,
    // this should react better to iGPU, dGPU and iGPU+dGPU setups. In more complex setups, it would
    // be neat if you could start the game on any GPU, display a choice to the user and seamlessly
    // switch to a new physical device.
    let mut found = None;
    for device in unsafe { instance.enumerate_physical_devices() }.unwrap() {
        let properties = unsafe { instance.get_physical_device_properties(device) };
        let gpu_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
            .to_str()
            .unwrap();
        let Some(queues) = QueueDetails::query(&instance, device, &surface, surface_khr).unwrap() else {
            println!("rejected gpu, no suitable queues ({gpu_name})");
            continue;
        };
        let extensions = unsafe { instance.enumerate_device_extension_properties(device) }.unwrap();
        let has_swapchain_extension = extensions.iter().any(|ext| {
            let ext_name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
            ext_name == Swapchain::name()
        });
        if !has_swapchain_extension {
            println!("rejected gpu, no swapchain extension ({gpu_name})");
            continue;
        }
        // This query requires the swapchain extension to be present.
        let swapchains = SwapchainDetails::query(device, &surface, surface_khr).unwrap();
        if swapchains.formats.is_empty() || swapchains.present_modes.is_empty() {
            println!("rejected gpu, unsuitable swapchain ({gpu_name})");
            continue;
        }
        if found.is_none() {
            println!("accepted gpu: {gpu_name}");
            found = Some((device, queues, swapchains));
        } else {
            println!("rejected gpu, one already selected ({gpu_name})");
        }
    }
    let Some((physical_device, queues, swapchains)) = found else {
        panic!("gpu not found");
    };

    // Specify physical device extensions, required queues and create them. Probably should pick
    // queues more reasonably?
    let queue_indices = HashSet::from([queues.graphics_family, queues.present_family]);
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
    let device = unsafe {
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
    .unwrap();
    let graphics_queue = unsafe { device.get_device_queue(queues.graphics_family, 0) };
    let present_queue = unsafe { device.get_device_queue(queues.present_family, 0) };

    // Create the swapchain for presenting images to display. Set to prefer triple buffering right
    // now, should be possible to change on laptops or integrated GPUs?
    let format = swapchains.select_format();
    let present_mode = swapchains.select_present_mode();
    let extent = swapchains.select_swap_extent(&window);
    let image_count = swapchains.select_image_count();
    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface_khr)
        .min_image_count(image_count)
        .image_format(format.format)
        .image_color_space(format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);
    let queue_family_indices = [queues.graphics_family, queues.present_family];
    let swapchain_create_info = if queues.graphics_family != queues.present_family {
        swapchain_create_info
            .image_sharing_mode(vk::SharingMode::CONCURRENT)
            .queue_family_indices(&queue_family_indices)
    } else {
        swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
    };
    let swapchain_create_info = swapchain_create_info
        .pre_transform(swapchains.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null());
    let swapchain = Swapchain::new(&instance, &device);
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
        swapchain_image_views[i] =
            unsafe { device.create_image_view(&image_view_create, None) }.unwrap();
    }

    let vert_shader = make_shader(&device, include_bytes!("../shaders/triangle-vert.spv"));
    let frag_shader = make_shader(&device, include_bytes!("../shaders/triangle-frag.spv"));
    let vert_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader)
        .name(CStr::from_bytes_with_nul(b"main\0").unwrap());
    let frag_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader)
        .name(CStr::from_bytes_with_nul(b"main\0").unwrap());
    let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);
    let vertex_input = vk::PipelineVertexInputStateCreateInfo::builder();
    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);
    let viewport = vk::Viewport {
        x: 0.,
        y: 0.,
        width: swapchain_extent.width as f32,
        height: swapchain_extent.height as f32,
        min_depth: 0.,
        max_depth: 1.,
    };
    let scissor = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: swapchain_extent,
    };
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
    let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(&[*color_blend_attachment]);
    let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(&[])
        .push_constant_ranges(&[]);
    let pipeline_layout =
        unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }.unwrap();

    // Run the event loop. Winit delivers events, like key presses. After it finishes delivering
    // some batch of events, it sends a MainEventsCleared event, which means the application should
    // either render, or check whether it needs to rerender anything and possibly only request a
    // redraw of a specific window. Redrawing a window can also be requested by the operating
    // system, for example if the window size changes. For games, initially I'll render at both
    // events, but this probably needs to be changed to alter framebuffer size if the window is
    // resized?
    event_loop.run_return(move |event, _, control_flow| {
        control_flow.set_poll();
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => control_flow.set_exit(),
            Event::MainEventsCleared | Event::RedrawRequested(_) => {
                // render
                control_flow.set_exit();
            }
            _ => (),
        }
    });

    unsafe { device.destroy_pipeline_layout(pipeline_layout, None) };
    unsafe { device.destroy_shader_module(frag_shader, None) };
    unsafe { device.destroy_shader_module(vert_shader, None) };
    for image_view in swapchain_image_views {
        unsafe { device.destroy_image_view(image_view, None) };
    }
    unsafe { swapchain.destroy_swapchain(swapchain_khr, None) };
    unsafe { device.destroy_device(None) };
    unsafe { surface.destroy_surface(surface_khr, None) };
    unsafe { debug_utils_loader.destroy_debug_utils_messenger(debug_call_back, None) };
    unsafe { instance.destroy_instance(None) };
}

fn make_shader(device: &Device, code: &[u8]) -> vk::ShaderModule {
    let aligned_code = ash::util::read_spv(&mut std::io::Cursor::new(code)).unwrap();
    let shader_module_create = vk::ShaderModuleCreateInfo::builder().code(&aligned_code);
    unsafe { device.create_shader_module(&shader_module_create, None) }.unwrap()
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
