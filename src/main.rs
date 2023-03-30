#![feature(const_cstr_methods)]

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::prelude::VkResult;
use ash::{vk, Device, Entry, Instance};
use nalgebra_glm as glm;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::borrow::Cow;
use std::collections::{hash_map, HashMap, HashSet};
use std::f32::consts::{FRAC_PI_4, PI};
use std::ffi::CStr;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::time::Instant;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::WindowBuilder;

const WINDOW_TITLE: &str = "Vulkthing";
const WINDOW_SIZE: (usize, usize) = (1920, 1080);

const VULKAN_APP_NAME: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"Vulkthing\0") };
const VULKAN_APP_VERSION: u32 = vk::make_api_version(0, 0, 0, 0);
const VULKAN_ENGINE_NAME: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"Vulkthing\0") };
const VULKAN_ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 0, 0);

const MAX_FRAMES_IN_FLIGHT: usize = 2;

struct VulkanInstance<'a> {
    entry: &'a Entry,
    instance: Instance,
    ext: VulkanInstanceExts,
}

struct VulkanInstanceExts {
    debug: DebugUtils,
    surface: Surface,
}

struct VulkanDebug<'a> {
    instance: &'a VulkanInstance<'a>,
    messenger: vk::DebugUtilsMessengerEXT,
}

struct VulkanSurface<'a> {
    instance: &'a VulkanInstance<'a>,
    surface: vk::SurfaceKHR,
}

struct VulkanPhysicalDevice<'a> {
    instance: &'a VulkanInstance<'a>,
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

struct VulkanLogicalDevice<'a> {
    instance: &'a VulkanInstance<'a>,
    physical_device: &'a VulkanPhysicalDevice<'a>,
    device: Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
}

struct VulkanSwapchain<'a> {
    logical_device: &'a VulkanLogicalDevice<'a>,
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

struct Shader<'a> {
    logical_device: &'a VulkanLogicalDevice<'a>,
    module: vk::ShaderModule,
    stage: vk::PipelineShaderStageCreateInfo,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: glm::Vec3,
    color: glm::Vec3,
    tex_coord: glm::Vec2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct UniformBufferObject {
    model: glm::Mat4,
    view: glm::Mat4,
    proj: glm::Mat4,
}

impl<'a> VulkanInstance<'a> {
    fn create(entry: &'a Entry, window: &winit::window::Window) -> VulkanInstance<'a> {
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
        let layer_names = [b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const i8];

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
        let debug = DebugUtils::new(&entry, &instance);
        let surface = Surface::new(&entry, &instance);
        let ext = VulkanInstanceExts { debug, surface };

        VulkanInstance {
            entry,
            instance,
            ext,
        }
    }
}

impl<'a> VulkanDebug<'a> {
    fn create(instance: &'a VulkanInstance) -> VulkanDebug<'a> {
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
            instance,
            messenger,
        }
    }
}

impl<'a> VulkanSurface<'a> {
    fn create(instance: &'a VulkanInstance, window: &winit::window::Window) -> VulkanSurface<'a> {
        let surface = unsafe {
            ash_window::create_surface(
                &instance.entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
        }
        .unwrap();
        VulkanSurface { instance, surface }
    }
}

impl<'a> VulkanPhysicalDevice<'a> {
    fn find_for(surface: &VulkanSurface<'a>) -> VulkanPhysicalDevice<'a> {
        let instance = surface.instance;

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

            let supported_features = unsafe { instance.get_physical_device_features(device) };
            if supported_features.sampler_anisotropy == 0 {
                println!("rejected gpu, no sampler anisotropy feature");
                continue;
            }

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
            let swapchain =
                unsafe { SwapchainDetails::query(&instance, device, &surface) }.unwrap();
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
                instance,
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

impl<'a> VulkanLogicalDevice<'a> {
    fn create(physical_device: &'a VulkanPhysicalDevice<'a>) -> VulkanLogicalDevice<'a> {
        let instance = physical_device.instance;

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

        let physical_device_features =
            vk::PhysicalDeviceFeatures::builder().sampler_anisotropy(true);

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
            instance,
            physical_device,
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
    unsafe fn query(
        instance: &VulkanInstance,
        device: vk::PhysicalDevice,
        surface: &VulkanSurface,
    ) -> VkResult<SwapchainDetails> {
        let ext = &instance.ext.surface;
        let capabilities = ext.get_physical_device_surface_capabilities(device, surface.surface)?;
        let formats = ext.get_physical_device_surface_formats(device, surface.surface)?;
        let present_modes =
            ext.get_physical_device_surface_present_modes(device, surface.surface)?;
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
        vk::PresentModeKHR::MAILBOX
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
        let preferred_image_count = self.capabilities.min_image_count + 1;
        if no_image_limit {
            preferred_image_count
        } else {
            preferred_image_count.min(self.capabilities.max_image_count)
        }
    }
}

impl<'a> VulkanSwapchain<'a> {
    fn create(
        logical_device: &'a VulkanLogicalDevice<'a>,
        surface: &VulkanSurface,
        window: &winit::window::Window,
    ) -> VulkanSwapchain<'a> {
        assert!(std::ptr::eq(logical_device.instance, surface.instance));
        let instance = logical_device.instance;
        let physical_device = logical_device.physical_device;
        let ext = Swapchain::new(&instance, &logical_device.device);

        // Create the swapchain for presenting images to display. Set to prefer triple buffering
        // right now, should be possible to change on laptops or integrated GPUs? Also requires
        // specifying a bunch of display-related parameters, which aren't very interesting as they
        // were mostly decided on previously.
        let format = physical_device.swapchain.select_format();
        let present_mode = physical_device.swapchain.select_present_mode();
        let extent = physical_device.swapchain.select_swap_extent(&window);
        let image_count = physical_device.swapchain.select_image_count();
        let image_format = format.format;
        let queue_family_indices = [
            physical_device.queues.graphics_family,
            physical_device.queues.present_family,
        ];
        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.surface)
            .min_image_count(image_count)
            .image_format(image_format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);
        let create_info =
            if physical_device.queues.graphics_family != physical_device.queues.present_family {
                create_info
                    .image_sharing_mode(vk::SharingMode::CONCURRENT)
                    .queue_family_indices(&queue_family_indices)
            } else {
                create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            };
        let create_info = create_info
            .pre_transform(physical_device.swapchain.capabilities.current_transform)
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
        let shader_stages = [vert_shader.stage, frag_shader.stage];
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
        let set_layouts = [descriptor_set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&set_layouts)
            .push_constant_ranges(&[]);
        let pipeline_layout =
            unsafe { logical_device.create_pipeline_layout(&pipeline_layout_info, None) }.unwrap();

        let color_attachment = vk::AttachmentDescription::builder()
            .format(swapchain.image_format)
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
        let depth_attachment = *vk::AttachmentDescription::builder()
            .format(find_depth_format(logical_device.physical_device))
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        let depth_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments)
            .depth_stencil_attachment(&depth_attachment_ref);
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
        let attachments = [*color_attachment, depth_attachment];
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

impl<'a> Shader<'a> {
    fn compile(
        logical_device: &'a VulkanLogicalDevice,
        code: &'static [u8],
        stage_flags: vk::ShaderStageFlags,
    ) -> Self {
        let aligned_code = ash::util::read_spv(&mut std::io::Cursor::new(code)).unwrap();
        let module = unsafe {
            logical_device.device.create_shader_module(
                &vk::ShaderModuleCreateInfo::builder().code(&aligned_code),
                None,
            )
        }
        .unwrap();
        let stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(stage_flags)
            .module(module)
            .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
            .build();
        Shader {
            logical_device,
            stage,
            module,
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

impl Deref for VulkanInstance<'_> {
    type Target = Instance;

    fn deref(&self) -> &Instance {
        &self.instance
    }
}

impl Deref for VulkanLogicalDevice<'_> {
    type Target = Device;

    fn deref(&self) -> &Device {
        &self.device
    }
}

impl Drop for VulkanInstance<'_> {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) };
    }
}

impl Drop for VulkanDebug<'_> {
    fn drop(&mut self) {
        unsafe {
            self.instance
                .ext
                .debug
                .destroy_debug_utils_messenger(self.messenger, None)
        };
    }
}

impl Drop for VulkanSurface<'_> {
    fn drop(&mut self) {
        unsafe {
            self.instance
                .ext
                .surface
                .destroy_surface(self.surface, None)
        };
    }
}

impl Drop for VulkanLogicalDevice<'_> {
    fn drop(&mut self) {
        unsafe { self.device.destroy_device(None) };
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

impl Drop for Shader<'_> {
    fn drop(&mut self) {
        unsafe { self.logical_device.destroy_shader_module(self.module, None) };
    }
}

fn main() {
    let models = tobj::load_obj("assets/viking-room.obj", &Default::default())
        .unwrap()
        .0;
    let mut unique_vertices = HashMap::new();
    let mut vertex_data = Vec::new();
    let mut index_data = Vec::new();
    for model in models {
        for index in model.mesh.indices {
            let pos_offset = (3 * index) as usize;
            let tex_coord_offset = (2 * index) as usize;
            let vertex = Vertex {
                position: glm::vec3(
                    model.mesh.positions[pos_offset],
                    model.mesh.positions[pos_offset + 1],
                    model.mesh.positions[pos_offset + 2],
                ),
                color: glm::vec3(1., 1., 1.),
                tex_coord: glm::vec2(
                    model.mesh.texcoords[tex_coord_offset],
                    1.0 - model.mesh.texcoords[tex_coord_offset + 1],
                ),
            };
            let index = match unique_vertices.entry(vertex) {
                hash_map::Entry::Occupied(e) => *e.get(),
                hash_map::Entry::Vacant(e) => {
                    let index = vertex_data.len();
                    e.insert(index);
                    vertex_data.push(vertex);
                    index
                }
            };
            index_data.push(index as u32);
        }
    }

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
    let surface = VulkanSurface::create(&instance, &window);
    let physical_device = VulkanPhysicalDevice::find_for(&surface);
    let logical_device = VulkanLogicalDevice::create(&physical_device);
    let swapchain = VulkanSwapchain::create(&logical_device, &surface, &window);

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
    let descriptor_set_layout =
        unsafe { logical_device.create_descriptor_set_layout(&layout_info, None) }.unwrap();

    let pipeline = VulkanPipeline::create(&swapchain, descriptor_set_layout);

    let command_pool_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(physical_device.queues.graphics_family);
    let command_pool =
        unsafe { logical_device.create_command_pool(&command_pool_info, None) }.unwrap();
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);
    let command_buffers =
        unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info) }.unwrap();

    let (depth_image, depth_image_memory, depth_image_view) =
        create_depth_resources(&swapchain, logical_device.graphics_queue, command_pool);
    let framebuffers = create_framebuffers(&pipeline, depth_image_view);

    let (texture_image, texture_image_memory, mip_levels) =
        create_texture_image(&logical_device, logical_device.graphics_queue, command_pool);
    let texture_image_view = create_texture_image_view(&logical_device, texture_image, mip_levels);
    let texture_sampler = create_texture_sampler(&logical_device, mip_levels);

    let (vertex_buffer, vertex_buffer_memory) =
        create_vertex_buffer(&vertex_data, &logical_device, command_pool);
    let (index_buffer, index_buffer_memory) =
        create_index_buffer(&index_data, &logical_device, command_pool);

    let mut uniform_buffers = Vec::new();
    let mut uniform_buffer_memories = Vec::new();
    let mut uniform_buffer_mapped = Vec::new();
    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        let buffer_size = std::mem::size_of::<UniformBufferObject>();
        let (buffer, buffer_memory) = create_buffer(
            &logical_device,
            buffer_size,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        let buffer_mapped = unsafe {
            logical_device.device.map_memory(
                buffer_memory,
                0,
                buffer_size as u64,
                vk::MemoryMapFlags::empty(),
            )
        }
        .unwrap() as *mut UniformBufferObject;
        uniform_buffers.push(buffer);
        uniform_buffer_memories.push(buffer_memory);
        uniform_buffer_mapped.push(buffer_mapped);
    }

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
    let descriptor_pool =
        unsafe { logical_device.create_descriptor_pool(&pool_info, None) }.unwrap();

    let layouts = vec![descriptor_set_layout; MAX_FRAMES_IN_FLIGHT];
    let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptor_pool)
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

    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
    let image_available_semaphores: Vec<_> = (0..MAX_FRAMES_IN_FLIGHT)
        .map(|_| unsafe { logical_device.create_semaphore(&semaphore_info, None) }.unwrap())
        .collect();
    let render_finished_semaphore: Vec<_> = (0..MAX_FRAMES_IN_FLIGHT)
        .map(|_| unsafe { logical_device.create_semaphore(&semaphore_info, None) }.unwrap())
        .collect();
    let in_flight_fence: Vec<_> = (0..MAX_FRAMES_IN_FLIGHT)
        .map(|_| unsafe { logical_device.create_fence(&fence_info, None) }.unwrap())
        .collect();

    let start_time = Instant::now();

    let mut current_frame = 0;

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
                    in_flight_fence[current_frame],
                    &swapchain,
                    image_available_semaphores[current_frame],
                    command_buffers[current_frame],
                    &framebuffers,
                    &pipeline,
                    render_finished_semaphore[current_frame],
                    vertex_buffer,
                    index_buffer,
                    index_data.len(),
                    start_time,
                    uniform_buffer_mapped[current_frame],
                    descriptor_sets[current_frame],
                );
                current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
            }
            _ => (),
        }
    });

    unsafe { logical_device.device_wait_idle() }.unwrap();

    for fence in in_flight_fence {
        unsafe { logical_device.destroy_fence(fence, None) };
    }
    for semaphore in render_finished_semaphore {
        unsafe { logical_device.destroy_semaphore(semaphore, None) };
    }
    for semaphore in image_available_semaphores {
        unsafe { logical_device.destroy_semaphore(semaphore, None) };
    }
    unsafe { logical_device.destroy_command_pool(command_pool, None) };
    cleanup_swapchain(
        &logical_device,
        &framebuffers,
        depth_image,
        depth_image_memory,
        depth_image_view,
    );
    unsafe { logical_device.destroy_descriptor_pool(descriptor_pool, None) };
    unsafe { logical_device.destroy_descriptor_set_layout(descriptor_set_layout, None) };
    for buffer in uniform_buffers {
        unsafe { logical_device.destroy_buffer(buffer, None) };
    }
    for memory in uniform_buffer_memories {
        unsafe { logical_device.free_memory(memory, None) };
    }
    unsafe { logical_device.destroy_buffer(vertex_buffer, None) };
    unsafe { logical_device.destroy_buffer(index_buffer, None) };
    unsafe { logical_device.free_memory(vertex_buffer_memory, None) };
    unsafe { logical_device.free_memory(index_buffer_memory, None) };
    unsafe { logical_device.destroy_sampler(texture_sampler, None) };
    unsafe { logical_device.destroy_image_view(texture_image_view, None) };
    unsafe { logical_device.destroy_image(texture_image, None) };
    unsafe { logical_device.free_memory(texture_image_memory, None) };
}

fn create_framebuffers(
    pipeline: &VulkanPipeline,
    depth_image_view: vk::ImageView,
) -> Vec<vk::Framebuffer> {
    let mut framebuffers = vec![vk::Framebuffer::null(); pipeline.swapchain.image_count()];
    for i in 0..pipeline.swapchain.image_count() {
        let attachments = [pipeline.swapchain.image_views[i], depth_image_view];
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
            offset: std::mem::size_of::<glm::Vec3>() as u32
                + std::mem::size_of::<glm::Vec3>() as u32,
        },
    ]
}

fn find_memory_type(
    device: &VulkanPhysicalDevice,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> u32 {
    let memory = unsafe {
        device
            .instance
            .get_physical_device_memory_properties(device.device)
    };
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
    logical_device: &VulkanLogicalDevice,
    size: usize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_info = *vk::BufferCreateInfo::builder()
        .size(size as u64)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let buffer = unsafe { logical_device.create_buffer(&buffer_info, None) }.unwrap();
    let requirements = unsafe { logical_device.get_buffer_memory_requirements(buffer) };
    let memory_type_index = find_memory_type(
        &logical_device.physical_device,
        requirements.memory_type_bits,
        properties,
    );
    let alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type_index);
    let memory = unsafe { logical_device.device.allocate_memory(&alloc_info, None) }.unwrap();
    unsafe { logical_device.device.bind_buffer_memory(buffer, memory, 0) }.unwrap();
    (buffer, memory)
}

fn create_image(
    logical_device: &VulkanLogicalDevice,
    width: usize,
    height: usize,
    mip_levels: usize,
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
        .samples(vk::SampleCountFlags::TYPE_1);
    let image = unsafe { logical_device.create_image(&image_info, None) }.unwrap();

    let requirements = unsafe { logical_device.get_image_memory_requirements(image) };
    let memory_type = find_memory_type(
        logical_device.physical_device,
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
    physical_device: &VulkanPhysicalDevice,
) -> vk::Format {
    for format in candidates {
        let props = unsafe {
            physical_device
                .instance
                .get_physical_device_format_properties(physical_device.device, *format)
        };
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

fn find_depth_format(physical_device: &VulkanPhysicalDevice) -> vk::Format {
    find_supported_format(
        &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ],
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        physical_device,
    )
}

fn has_stencil_component(format: vk::Format) -> bool {
    format == vk::Format::D32_SFLOAT_S8_UINT || format == vk::Format::D24_UNORM_S8_UINT
}

fn create_depth_resources(
    swapchain: &VulkanSwapchain,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
) -> (vk::Image, vk::DeviceMemory, vk::ImageView) {
    let format = find_depth_format(swapchain.logical_device.physical_device);
    let (image, image_memory) = create_image(
        swapchain.logical_device,
        swapchain.extent.width as usize,
        swapchain.extent.height as usize,
        1,
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
    logical_device: &VulkanLogicalDevice,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
) -> (vk::Image, vk::DeviceMemory, usize) {
    let image = image::open("assets/viking-room.png").unwrap().to_rgba8();
    let pixel_count = image.width() as usize * image.height() as usize;
    let image_size = pixel_count * 4;
    let mip_levels = (image.width().max(image.height()) as f32).log2().floor() as usize + 1;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        logical_device,
        image_size,
        vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
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
        logical_device,
        image.width() as usize,
        image.height() as usize,
        mip_levels,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
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
    logical_device: &VulkanLogicalDevice,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
) {
    let format_properties = unsafe {
        logical_device
            .instance
            .get_physical_device_format_properties(logical_device.physical_device.device, format)
    };
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
    logical_device: &VulkanLogicalDevice,
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
    logical_device: &VulkanLogicalDevice,
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

fn create_texture_sampler(logical_device: &VulkanLogicalDevice, mip_levels: usize) -> vk::Sampler {
    let properties = unsafe {
        logical_device
            .instance
            .get_physical_device_properties(logical_device.physical_device.device)
    };
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
    logical_device: &VulkanLogicalDevice,
    command_pool: vk::CommandPool,
) -> (vk::Buffer, vk::DeviceMemory) {
    let vertex_size = std::mem::size_of::<Vertex>();
    let vertex_buffer_size = vertex_size * vertex_data.len();
    let (staging_buffer, staging_buffer_memory) = create_buffer(
        &logical_device,
        vertex_buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );
    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        &logical_device,
        vertex_buffer_size,
        vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );
    let staging_ptr = unsafe {
        logical_device.device.map_memory(
            staging_buffer_memory,
            0,
            vertex_buffer_size as u64,
            vk::MemoryMapFlags::empty(),
        )
    }
    .unwrap();
    unsafe { std::slice::from_raw_parts_mut(staging_ptr as *mut Vertex, vertex_data.len()) }
        .copy_from_slice(&vertex_data);
    unsafe { logical_device.device.unmap_memory(staging_buffer_memory) };
    copy_buffer(
        &logical_device,
        staging_buffer,
        vertex_buffer,
        vertex_buffer_size,
        command_pool,
        logical_device.graphics_queue,
    );
    unsafe { logical_device.destroy_buffer(staging_buffer, None) };
    unsafe { logical_device.free_memory(staging_buffer_memory, None) };
    (vertex_buffer, vertex_buffer_memory)
}

fn transition_image_layout(
    logical_device: &VulkanLogicalDevice,
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
    logical_device: &VulkanLogicalDevice,
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
    logical_device: &VulkanLogicalDevice,
    command_pool: vk::CommandPool,
) -> (vk::Buffer, vk::DeviceMemory) {
    let index_size = std::mem::size_of_val(&index_data[0]);
    let index_buffer_size = index_size * index_data.len();
    let (staging_buffer, staging_buffer_memory) = create_buffer(
        &logical_device,
        index_buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );
    let (index_buffer, index_buffer_memory) = create_buffer(
        &logical_device,
        index_buffer_size,
        vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );
    let staging_ptr = unsafe {
        logical_device.device.map_memory(
            staging_buffer_memory,
            0,
            index_buffer_size as u64,
            vk::MemoryMapFlags::empty(),
        )
    }
    .unwrap();
    unsafe { std::slice::from_raw_parts_mut(staging_ptr as *mut u32, index_data.len()) }
        .copy_from_slice(&index_data);
    unsafe { logical_device.device.unmap_memory(staging_buffer_memory) };
    copy_buffer(
        &logical_device,
        staging_buffer,
        index_buffer,
        index_buffer_size,
        command_pool,
        logical_device.graphics_queue,
    );
    unsafe { logical_device.destroy_buffer(staging_buffer, None) };
    unsafe { logical_device.free_memory(staging_buffer_memory, None) };
    (index_buffer, index_buffer_memory)
}

fn copy_buffer(
    logical_device: &VulkanLogicalDevice,
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
    logical_device: &VulkanLogicalDevice,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
    f: impl FnOnce(vk::CommandBuffer) -> R,
) -> R {
    let command_info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(command_pool)
        .command_buffer_count(1);
    let command_buffer = unsafe {
        logical_device
            .device
            .allocate_command_buffers(&command_info)
    }
    .unwrap()
    .into_iter()
    .next()
    .unwrap();

    let begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe {
        logical_device
            .device
            .begin_command_buffer(command_buffer, &begin_info)
    }
    .unwrap();

    let result = f(command_buffer);

    unsafe { logical_device.end_command_buffer(command_buffer) }.unwrap();

    let submit_buffers = [command_buffer];
    let submit_info = vk::SubmitInfo::builder().command_buffers(&submit_buffers);
    unsafe { logical_device.queue_submit(queue, &[*submit_info], vk::Fence::null()) }.unwrap();
    unsafe { logical_device.queue_wait_idle(queue) }.unwrap();
    unsafe { logical_device.free_command_buffers(command_pool, &[command_buffer]) };

    result
}

fn cleanup_swapchain(
    logical_device: &VulkanLogicalDevice,
    framebuffers: &[vk::Framebuffer],
    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    depth_image_view: vk::ImageView,
) {
    unsafe { logical_device.destroy_image_view(depth_image_view, None) };
    unsafe { logical_device.destroy_image(depth_image, None) };
    unsafe { logical_device.free_memory(depth_image_memory, None) };
    for framebuffer in framebuffers {
        unsafe { logical_device.destroy_framebuffer(*framebuffer, None) };
    }
}

fn draw_frame(
    device: &VulkanLogicalDevice,
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
    start_time: Instant,
    ubo_ptr: *mut UniformBufferObject,
    descriptor_set: vk::DescriptorSet,
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
        start_time,
        ubo_ptr,
        swapchain.extent.width as f32 / swapchain.extent.height as f32,
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

    let present_info_swapchains = [swapchain.swapchain];
    let present_info_images = [image_index];
    let present_info = vk::PresentInfoKHR::builder()
        .wait_semaphores(&signal_semaphores)
        .swapchains(&present_info_swapchains)
        .image_indices(&present_info_images);
    unsafe {
        swapchain
            .ext
            .queue_present(device.present_queue, &present_info)
    }
    .unwrap();
}

fn record_command_buffer(
    device: &VulkanLogicalDevice,
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

fn update_uniform_buffer(
    start_time: Instant,
    ubo_ptr: *mut UniformBufferObject,
    aspect_ratio: f32,
) {
    let current_time = Instant::now();
    let time = (current_time - start_time).as_secs_f32();
    let mut ubo = UniformBufferObject {
        model: glm::rotate(&glm::identity(), time * PI / 16., &glm::vec3(0., 0., 1.)),
        view: glm::look_at(&glm::vec3(2., 2., 2.), &glm::zero(), &glm::vec3(0., 0., 1.)),
        proj: glm::perspective_rh_zo(aspect_ratio, FRAC_PI_4, 0.1, 10.),
    };
    ubo.proj[(1, 1)] *= -1.;
    unsafe { ubo_ptr.write_volatile(ubo) };
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
