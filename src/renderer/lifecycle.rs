use crate::model::Model;
use crate::renderer::debug::create_debug_messenger;
use crate::renderer::descriptors::{
    create_descriptor_metadata, Descriptor, DescriptorConfig, DescriptorKind, DescriptorMetadata,
    DescriptorValue,
};
use crate::renderer::device::{select_device, DeviceInfo};
use crate::renderer::pipeline::{create_pipeline, Pipeline, PipelineConfig, VertexLayout};
use crate::renderer::traits::VertexOps;
use crate::renderer::uniform::{Filters, Light, Material, ModelViewProjection};
use crate::renderer::util::{onetime_commands, Buffer};
use crate::renderer::vertex::Vertex;
use crate::renderer::{
    util, ImageResources, Object, RaytraceResources, Renderer, Synchronization, UniformBuffer,
    VulkanExtensions, FRAMES_IN_FLIGHT,
};
use crate::window::Window;
use crate::{VULKAN_APP_NAME, VULKAN_APP_VERSION, VULKAN_ENGINE_NAME, VULKAN_ENGINE_VERSION};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{
    AccelerationStructure, BufferDeviceAddress, DeferredHostOperations, Surface, Swapchain,
};
use ash::vk::{
    ExtDescriptorIndexingFn, KhrRayQueryFn, KhrShaderFloatControlsFn, KhrSpirv14Fn, Packed24_8,
};
use ash::{vk, Device, Entry, Instance};
use nalgebra::Matrix4;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::f32::consts::FRAC_PI_4;
use std::ffi::CString;
use std::mem::MaybeUninit;
use winit::dpi::PhysicalSize;

// Format used for passing HDR data between render passes to enable realistic differences in
// lighting parameters and improve postprocessing effect quality, not related to monitor HDR.
// Support for this format is required by the Vulkan specification.
const INTERNAL_HDR_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;

const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;

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
            queue_family,
        } = select_device(&instance, &extensions.surface, surface);
        let logical_device = create_logical_device(queue_family, &instance, physical_device);
        let queue = unsafe { logical_device.get_device_queue(queue_family, 0) };
        let swapchain_ext = Swapchain::new(&instance, &logical_device);

        let msaa_samples = util::find_max_msaa_samples(&instance, physical_device);
        let offscreen_sampler = create_offscreen_sampler(&logical_device);
        let filters = UniformBuffer::create(&instance, physical_device, &logical_device);

        let object_descriptor_metadata = create_object_descriptor_metadata(&logical_device);
        let postprocess_descriptor_metadata =
            create_postprocess_descriptor_metadata(offscreen_sampler, &logical_device);

        let (
            swapchain_extent,
            swapchain,
            swapchain_image_views,
            color,
            depth,
            offscreen,
            object_pipeline,
            render_pass,
            render_framebuffer,
            postprocess_pipeline,
            postprocess_pass,
            postprocess_framebuffers,
            postprocess_descriptor_sets,
            projection,
        ) = create_swapchain_all(
            window.window.inner_size(),
            &instance,
            &extensions.surface,
            &swapchain_ext,
            physical_device,
            &logical_device,
            surface,
            msaa_samples,
            &filters,
            &object_descriptor_metadata,
            &postprocess_descriptor_metadata,
        );

        let command_pools = create_command_pools(queue_family, &logical_device);
        let command_buffers = create_command_buffers(&logical_device, &command_pools);
        let sync = create_sync(&logical_device);

        let light = UniformBuffer::create(&instance, physical_device, &logical_device);

        let mut objects = Vec::new();
        for model in models {
            let object = create_object(
                model,
                &object_descriptor_metadata,
                &light,
                &instance,
                physical_device,
                &logical_device,
                queue,
                command_pools[0],
            );
            objects.push(object);
        }

        let blas = create_blas(
            &objects[0],
            &instance,
            physical_device,
            &logical_device,
            queue,
            command_pools[0],
        );
        let tlas = create_tlas(
            &blas,
            &instance,
            physical_device,
            &logical_device,
            queue,
            command_pools[0],
        );
        for object in &objects {
            for i in 0..FRAMES_IN_FLIGHT {
                let acceleration_structures = [tlas.acceleration_structure];
                let mut tlas_write = *vk::WriteDescriptorSetAccelerationStructureKHR::builder()
                    .acceleration_structures(&acceleration_structures);
                let mut descriptor_writes = [*vk::WriteDescriptorSet::builder()
                    .dst_set(object.descriptor_sets[i])
                    .dst_binding(3)
                    .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                    .push_next(&mut tlas_write)];
                descriptor_writes[0].descriptor_count = 1;
                unsafe { logical_device.update_descriptor_sets(&descriptor_writes, &[]) };
            }
        }

        Renderer {
            _entry: entry,
            instance,
            extensions,
            debug_messenger,
            surface,
            physical_device,
            logical_device,
            queue,
            swapchain_ext,
            msaa_samples,
            offscreen_sampler,
            filters,
            object_descriptor_metadata,
            object_pipeline,
            render_pass,
            postprocess_descriptor_metadata,
            postprocess_pipeline,
            postprocess_pass,
            swapchain_extent,
            swapchain,
            swapchain_image_views,
            color,
            depth,
            offscreen,
            render_framebuffer,
            postprocess_framebuffers,
            postprocess_descriptor_sets,
            projection,
            command_pools,
            command_buffers,
            sync,
            flight_index: 0,
            light,
            objects,
            blas,
            tlas,
            interface_renderer: None,
        }
    }

    pub fn create_interface_renderer(&mut self, imgui: &mut imgui::Context) {
        self.interface_renderer = Some(
            imgui_rs_vulkan_renderer::Renderer::with_default_allocator(
                &self.instance,
                self.physical_device,
                self.logical_device.clone(),
                self.queue,
                self.command_pools[0],
                self.postprocess_pass,
                imgui,
                Some(imgui_rs_vulkan_renderer::Options {
                    in_flight_frames: FRAMES_IN_FLIGHT,
                    enable_depth_test: false,
                    enable_depth_write: false,
                }),
            )
            .unwrap(),
        );
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

        let (
            swapchain_extent,
            swapchain,
            swapchain_image_views,
            color,
            depth,
            offscreen,
            object_pipeline,
            render_pass,
            render_framebuffer,
            postprocess_pipeline,
            postprocess_pass,
            postprocess_framebuffers,
            postprocess_descriptor_sets,
            projection,
        ) = create_swapchain_all(
            window_size,
            &self.instance,
            &self.extensions.surface,
            &self.swapchain_ext,
            self.physical_device,
            &self.logical_device,
            self.surface,
            self.msaa_samples,
            &self.filters,
            &self.object_descriptor_metadata,
            &self.postprocess_descriptor_metadata,
        );

        // Doing the assignments at the end guarantees any operation won't fail in the middle, and
        // makes it possible to easily compare new values to old ones.
        self.swapchain_extent = swapchain_extent;
        self.swapchain = swapchain;
        self.swapchain_image_views = swapchain_image_views;
        self.color = color;
        self.depth = depth;
        self.offscreen = offscreen;
        self.object_pipeline = object_pipeline;
        self.render_pass = render_pass;
        self.render_framebuffer = render_framebuffer;
        self.postprocess_pipeline = postprocess_pipeline;
        self.postprocess_pass = postprocess_pass;
        self.postprocess_framebuffers = postprocess_framebuffers;
        self.postprocess_descriptor_sets = postprocess_descriptor_sets;
        self.projection = projection;
    }

    pub fn recreate_planet(&mut self, planet_model: &Model) {
        unsafe { self.logical_device.device_wait_idle() }.unwrap();
        self.objects[0].cleanup(&self.logical_device, self.object_descriptor_metadata.pool);
        self.objects[0] = create_object(
            planet_model,
            &self.object_descriptor_metadata,
            &self.light,
            &self.instance,
            self.physical_device,
            &self.logical_device,
            self.queue,
            self.command_pools[0],
        );
    }

    fn cleanup_swapchain(&mut self) {
        let dev = &self.logical_device;
        unsafe {
            dev.reset_descriptor_pool(
                self.postprocess_descriptor_metadata.pool,
                vk::DescriptorPoolResetFlags::empty(),
            )
            .unwrap();
            for framebuffer in &self.postprocess_framebuffers {
                dev.destroy_framebuffer(*framebuffer, None);
            }
            dev.destroy_framebuffer(self.render_framebuffer, None);
            self.offscreen.cleanup(dev);
            self.depth.cleanup(dev);
            self.color.cleanup(dev);
            for image_view in &self.swapchain_image_views {
                dev.destroy_image_view(*image_view, None);
            }
            self.swapchain_ext.destroy_swapchain(self.swapchain, None);
            self.object_pipeline.cleanup(dev);
            self.postprocess_pipeline.cleanup(dev);
            dev.destroy_render_pass(self.postprocess_pass, None);
            dev.destroy_render_pass(self.render_pass, None);
        }
    }
}

impl Synchronization {
    fn cleanup(&self, dev: &Device) {
        for fence in self.in_flight {
            unsafe { dev.destroy_fence(fence, None) };
        }
        for semaphore in self.render_finished {
            unsafe { dev.destroy_semaphore(semaphore, None) };
        }
        for semaphore in self.image_available {
            unsafe { dev.destroy_semaphore(semaphore, None) };
        }
    }
}

impl Object {
    pub fn cleanup(&self, dev: &Device, pool: vk::DescriptorPool) {
        unsafe { dev.free_descriptor_sets(pool, &self.descriptor_sets) }.unwrap();
        self.vertex.cleanup(dev);
        self.index.cleanup(dev);
        self.mvp.cleanup(dev);
        self.material.cleanup(dev);
    }
}

impl ImageResources {
    fn cleanup(&self, dev: &Device) {
        unsafe {
            dev.destroy_image_view(self.view, None);
            dev.destroy_image(self.image, None);
            dev.free_memory(self.memory, None);
        }
    }
}

impl RaytraceResources {
    fn cleanup(&self, dev: &Device, as_ext: &AccelerationStructure) {
        unsafe { as_ext.destroy_acceleration_structure(self.acceleration_structure, None) };
        self.buffer.cleanup(dev);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let dev = &self.logical_device;
            dev.device_wait_idle().unwrap();

            drop(self.interface_renderer.take());

            for object in &self.objects {
                object.cleanup(dev, self.object_descriptor_metadata.pool);
            }
            self.light.cleanup(dev);
            let as_ext = AccelerationStructure::new(&self.instance, &self.logical_device);
            self.tlas.cleanup(dev, &as_ext);
            self.blas.cleanup(dev, &as_ext);

            self.sync.cleanup(dev);
            for pool in &self.command_pools {
                dev.destroy_command_pool(*pool, None);
            }

            drop(dev);
            self.cleanup_swapchain();
            let dev = &self.logical_device;

            self.object_descriptor_metadata.cleanup(dev);
            self.postprocess_descriptor_metadata.cleanup(dev);

            self.filters.cleanup(dev);
            dev.destroy_sampler(self.offscreen_sampler, None);

            dev.destroy_device(None);
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
        .api_version(vk::API_VERSION_1_1);

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
    queue_family: u32,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Device {
    let queue_create = *vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family)
        .queue_priorities(&[1.]);

    let physical_device_features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .fill_mode_non_solid(true);

    let mut bda_features =
        *vk::PhysicalDeviceBufferDeviceAddressFeaturesKHR::builder().buffer_device_address(true);
    let mut rq_features = *vk::PhysicalDeviceRayQueryFeaturesKHR::builder().ray_query(true);
    let mut as_features =
        *vk::PhysicalDeviceAccelerationStructureFeaturesKHR::builder().acceleration_structure(true);

    // Using validation layers on a device level shouldn't be necessary on newer Vulkan version
    // (since which one?), but it's good to keep it for compatibility.
    let layer_names = [b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const i8];

    let extension_names = [
        AccelerationStructure::name().as_ptr(),
        BufferDeviceAddress::name().as_ptr(),
        DeferredHostOperations::name().as_ptr(),
        ExtDescriptorIndexingFn::name().as_ptr(),
        KhrRayQueryFn::name().as_ptr(),
        KhrShaderFloatControlsFn::name().as_ptr(),
        KhrSpirv14Fn::name().as_ptr(),
        Swapchain::name().as_ptr(),
    ];

    unsafe {
        instance.create_device(
            physical_device,
            &vk::DeviceCreateInfo::builder()
                .queue_create_infos(std::slice::from_ref(&queue_create))
                .enabled_features(&physical_device_features)
                .enabled_layer_names(&layer_names)
                .enabled_extension_names(&extension_names)
                .push_next(&mut bda_features)
                .push_next(&mut rq_features)
                .push_next(&mut as_features),
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
    vk::PresentModeKHR::MAILBOX
}

fn create_swapchain(
    format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    present_mode: vk::PresentModeKHR,
    image_count: usize,
    extension: &Swapchain,
    surface: vk::SurfaceKHR,
    surface_capabilities: vk::SurfaceCapabilitiesKHR,
) -> vk::SwapchainKHR {
    // Create the swapchain for presenting images to display. Set to prefer triple buffering
    // right now, should be possible to change on laptops or integrated GPUs? Also requires
    // specifying a bunch of display-related parameters, which aren't very interesting as they
    // were mostly decided on previously.
    let create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(image_count as u32)
        .image_format(format.format)
        .image_color_space(format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
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

fn create_swapchain_all(
    window_size: PhysicalSize<u32>,
    instance: &Instance,
    surface_ext: &Surface,
    swapchain_ext: &Swapchain,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
    surface: vk::SurfaceKHR,
    msaa_samples: vk::SampleCountFlags,
    filters: &UniformBuffer<Filters>,
    object_descriptor_metadata: &DescriptorMetadata,
    postprocess_descriptor_metadata: &DescriptorMetadata,
) -> (
    vk::Extent2D,
    vk::SwapchainKHR,
    Vec<vk::ImageView>,
    ImageResources,
    ImageResources,
    ImageResources,
    Pipeline,
    vk::RenderPass,
    vk::Framebuffer,
    Pipeline,
    vk::RenderPass,
    Vec<vk::Framebuffer>,
    [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    Matrix4<f32>,
) {
    // Query the surface information again.
    let surface_capabilities =
        unsafe { surface_ext.get_physical_device_surface_capabilities(physical_device, surface) }
            .unwrap();
    let surface_formats = {
        unsafe { surface_ext.get_physical_device_surface_formats(physical_device, surface) }
            .unwrap()
    };
    let present_modes =
        unsafe { surface_ext.get_physical_device_surface_present_modes(physical_device, surface) }
            .unwrap();
    assert!(!present_modes.is_empty());

    let swapchain_image_count = select_swapchain_image_count(surface_capabilities);

    // Repeat creating the swapchain, except not using any Renderer members that heavily depend
    // on the swapchain (such as depth and color buffers).
    let swapchain_format = select_swapchain_format(&surface_formats);
    let swapchain_extent = select_swapchain_extent(surface_capabilities, window_size);
    let swapchain_present_mode = select_swapchain_present_mode(&present_modes);
    let swapchain = create_swapchain(
        swapchain_format,
        swapchain_extent,
        swapchain_present_mode,
        swapchain_image_count,
        swapchain_ext,
        surface,
        surface_capabilities,
    );
    let swapchain_image_views =
        create_swapchain_image_views(swapchain, swapchain_format, logical_device, swapchain_ext);
    let color = create_color_resources(
        swapchain_extent,
        msaa_samples,
        instance,
        physical_device,
        logical_device,
    );
    let depth = create_depth_resources(
        swapchain_extent,
        msaa_samples,
        instance,
        physical_device,
        logical_device,
    );
    let offscreen =
        create_offscreen_resources(swapchain_extent, instance, physical_device, logical_device);
    let render_pass = create_render_pass(msaa_samples, logical_device);
    let object_pipeline = create_object_pipeline(
        object_descriptor_metadata,
        msaa_samples,
        render_pass,
        swapchain_extent,
        logical_device,
    );
    let postprocess_pass = create_postprocess_pass(swapchain_format, &logical_device);
    let postprocess_pipeline = create_postprocess_pipeline(
        postprocess_descriptor_metadata,
        postprocess_pass,
        swapchain_extent,
        logical_device,
    );
    let offscreen_framebuffer = create_offscreen_framebuffer(
        render_pass,
        offscreen.view,
        swapchain_extent,
        depth.view,
        color.view,
        logical_device,
    );
    let framebuffers = create_framebuffers(
        postprocess_pass,
        swapchain_image_count,
        &swapchain_image_views,
        swapchain_extent,
        logical_device,
    );
    let postprocess_descriptor_sets = create_postprocess_descriptor_sets(
        offscreen.view,
        &filters,
        &postprocess_descriptor_metadata,
        logical_device,
    );
    let projection = compute_projection(swapchain_extent);
    (
        swapchain_extent,
        swapchain,
        swapchain_image_views,
        color,
        depth,
        offscreen,
        object_pipeline,
        render_pass,
        offscreen_framebuffer,
        postprocess_pipeline,
        postprocess_pass,
        framebuffers,
        postprocess_descriptor_sets,
        projection,
    )
}

fn create_render_pass(
    msaa_samples: vk::SampleCountFlags,
    logical_device: &Device,
) -> vk::RenderPass {
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
        .format(DEPTH_FORMAT)
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
    let attachments = [color_attachment, depth_attachment, resolve_attachment];

    let color_reference = *vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let depth_reference = *vk::AttachmentReference::builder()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
    let resolve_reference = *vk::AttachmentReference::builder()
        .attachment(2)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let subpass = *vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(std::slice::from_ref(&color_reference))
        .depth_stencil_attachment(&depth_reference)
        .resolve_attachments(std::slice::from_ref(&resolve_reference));

    let create_info = *vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(std::slice::from_ref(&subpass));
    unsafe { logical_device.create_render_pass(&create_info, None) }.unwrap()
}

fn create_postprocess_pass(
    swapchain_format: vk::SurfaceFormatKHR,
    logical_device: &Device,
) -> vk::RenderPass {
    let color_attachment = *vk::AttachmentDescription::builder()
        .format(swapchain_format.format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_reference = *vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let subpass = *vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(std::slice::from_ref(&color_reference));

    let create_info = *vk::RenderPassCreateInfo::builder()
        .attachments(&std::slice::from_ref(&color_attachment))
        .subpasses(std::slice::from_ref(&subpass));
    unsafe { logical_device.create_render_pass(&create_info, None) }.unwrap()
}

fn create_object_descriptor_metadata(logical_device: &Device) -> DescriptorMetadata {
    create_descriptor_metadata(DescriptorConfig {
        descriptors: vec![
            Descriptor {
                kind: DescriptorKind::UniformBuffer,
                stage: vk::ShaderStageFlags::VERTEX,
            },
            Descriptor {
                kind: DescriptorKind::UniformBuffer,
                stage: vk::ShaderStageFlags::FRAGMENT,
            },
            Descriptor {
                kind: DescriptorKind::UniformBuffer,
                stage: vk::ShaderStageFlags::FRAGMENT,
            },
            Descriptor {
                kind: DescriptorKind::AccelerationStructure,
                stage: vk::ShaderStageFlags::FRAGMENT,
            },
        ],
        set_count: 2,
        logical_device,
    })
}

fn create_object_descriptor_sets(
    mvp: &UniformBuffer<ModelViewProjection>,
    material: &UniformBuffer<Material>,
    light: &UniformBuffer<Light>,
    metadata: &DescriptorMetadata,
    logical_device: &Device,
) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
    metadata.create_sets(
        &[
            DescriptorValue::Buffer(mvp),
            DescriptorValue::Buffer(material),
            DescriptorValue::Buffer(light),
            // TLAS needs to be written separately later.
        ],
        logical_device,
    )
}

fn create_postprocess_descriptor_metadata(
    sampler: vk::Sampler,
    logical_device: &Device,
) -> DescriptorMetadata {
    create_descriptor_metadata(DescriptorConfig {
        descriptors: vec![
            Descriptor {
                kind: DescriptorKind::ImmutableSampler { sampler },
                stage: vk::ShaderStageFlags::FRAGMENT,
            },
            Descriptor {
                kind: DescriptorKind::UniformBuffer,
                stage: vk::ShaderStageFlags::FRAGMENT,
            },
        ],
        set_count: 1,
        logical_device,
    })
}

fn create_postprocess_descriptor_sets(
    offscreen_view: vk::ImageView,
    filters: &UniformBuffer<Filters>,
    metadata: &DescriptorMetadata,
    logical_device: &Device,
) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
    metadata.create_sets(
        &[
            DescriptorValue::Image(offscreen_view),
            DescriptorValue::Buffer(filters),
        ],
        logical_device,
    )
}

fn create_object_pipeline(
    descriptor_metadata: &DescriptorMetadata,
    msaa_samples: vk::SampleCountFlags,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    logical_device: &Device,
) -> Pipeline {
    create_pipeline(PipelineConfig {
        vertex_shader_path: "shaders/model.vert",
        fragment_shader_path: "shaders/model.frag",
        vertex_layout: Some(VertexLayout {
            stride: std::mem::size_of::<Vertex>(),
            attribute_descriptions: Vertex::attribute_descriptions(0),
        }),
        msaa_samples,
        polygon_mode: vk::PolygonMode::FILL,
        descriptor_set_layout: descriptor_metadata.set_layout,
        depth_test: true,
        pass: render_pass,
        logical_device,
        swapchain_extent,
    })
}

fn create_postprocess_pipeline(
    descriptors: &DescriptorMetadata,
    postprocess_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    logical_device: &Device,
) -> Pipeline {
    create_pipeline(PipelineConfig {
        vertex_shader_path: "shaders/quad.vert",
        fragment_shader_path: "shaders/postprocess.frag",
        vertex_layout: None,
        msaa_samples: vk::SampleCountFlags::TYPE_1,
        polygon_mode: vk::PolygonMode::FILL,
        descriptor_set_layout: descriptors.set_layout,
        depth_test: false,
        pass: postprocess_pass,
        logical_device,
        swapchain_extent,
    })
}

fn create_command_pools(
    queue_family: u32,
    logical_device: &Device,
) -> [vk::CommandPool; FRAMES_IN_FLIGHT] {
    let command_pool_info = vk::CommandPoolCreateInfo::builder().queue_family_index(queue_family);
    let mut pools = [vk::CommandPool::null(); FRAMES_IN_FLIGHT];
    for pool in &mut pools {
        *pool = unsafe { logical_device.create_command_pool(&command_pool_info, None) }.unwrap();
    }
    pools
}

fn create_command_buffers(
    logical_device: &Device,
    command_pools: &[vk::CommandPool; FRAMES_IN_FLIGHT],
) -> [vk::CommandBuffer; FRAMES_IN_FLIGHT] {
    let mut buffers = [vk::CommandBuffer::null(); FRAMES_IN_FLIGHT];
    for (i, buffer) in buffers.iter_mut().enumerate() {
        let buffer_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pools[i])
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        *buffer = unsafe { logical_device.allocate_command_buffers(&buffer_info) }.unwrap()[0];
    }
    buffers
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
    let (image, memory) = util::create_image(
        DEPTH_FORMAT,
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
        DEPTH_FORMAT,
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

fn create_offscreen_sampler(logical_device: &Device) -> vk::Sampler {
    let sampler_info = vk::SamplerCreateInfo::builder()
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
        .unnormalized_coordinates(true);
    unsafe { logical_device.create_sampler(&sampler_info, None) }.unwrap()
}

pub fn create_object(
    model: &Model,
    descriptor_metadata: &DescriptorMetadata,
    light_buffer: &UniformBuffer<Light>,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
) -> Object {
    let vertex = create_vertex_buffer(
        &model.vertices,
        instance,
        physical_device,
        logical_device,
        queue,
        command_pool,
    );
    let index = create_index_buffer(
        &model.indices,
        instance,
        physical_device,
        logical_device,
        queue,
        command_pool,
    );
    let mvp = UniformBuffer::create(instance, physical_device, logical_device);
    let material = UniformBuffer::create(instance, physical_device, logical_device);
    let descriptor_sets = create_object_descriptor_sets(
        &mvp,
        &material,
        light_buffer,
        &descriptor_metadata,
        logical_device,
    );
    Object {
        triangle_count: model.indices.len() / 3,
        raw_vertex_count: model.vertices.len(),
        vertex,
        index,
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
    queue: vk::Queue,
    command_pool: vk::CommandPool,
) -> Buffer {
    let vertex_size = std::mem::size_of::<Vertex>();
    let vertex_count = vertex_data.len();
    let vertex_buffer_size = vertex_size * vertex_count;
    let staging = Buffer::create(
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vertex_buffer_size,
        instance,
        physical_device,
        logical_device,
    );
    let vertex = Buffer::create(
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::BufferUsageFlags::VERTEX_BUFFER
            | vk::BufferUsageFlags::TRANSFER_DST
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        vertex_buffer_size,
        instance,
        physical_device,
        logical_device,
    );
    util::with_mapped_slice(staging.memory, vertex_count, logical_device, |mapped| {
        MaybeUninit::write_slice(mapped, vertex_data);
    });
    util::copy_buffer(
        staging.buffer,
        vertex.buffer,
        vertex_buffer_size,
        logical_device,
        queue,
        command_pool,
    );
    staging.cleanup(logical_device);
    vertex
}

fn create_index_buffer(
    index_data: &[u32],
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
) -> Buffer {
    let index_size = std::mem::size_of_val(&index_data[0]);
    let index_buffer_size = index_size * index_data.len();
    let staging = Buffer::create(
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        vk::BufferUsageFlags::TRANSFER_SRC,
        index_buffer_size,
        instance,
        physical_device,
        logical_device,
    );
    let index = Buffer::create(
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::BufferUsageFlags::INDEX_BUFFER
            | vk::BufferUsageFlags::TRANSFER_DST
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        index_buffer_size,
        instance,
        physical_device,
        logical_device,
    );
    util::with_mapped_slice(staging.memory, index_data.len(), logical_device, |mapped| {
        MaybeUninit::write_slice(mapped, index_data);
    });
    util::copy_buffer(
        staging.buffer,
        index.buffer,
        index_buffer_size,
        logical_device,
        queue,
        command_pool,
    );
    staging.cleanup(logical_device);
    index
}

fn create_blas(
    object: &Object,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
) -> RaytraceResources {
    let as_ext = AccelerationStructure::new(&instance, &logical_device);
    let bda_ext = BufferDeviceAddress::new(&instance, &logical_device);

    let vertex_address = object.vertex.device_address(&bda_ext);
    let index_address = object.index.device_address(&bda_ext);
    let triangles = *vk::AccelerationStructureGeometryTrianglesDataKHR::builder()
        .vertex_format(vk::Format::R32G32B32_SFLOAT)
        .vertex_data(vk::DeviceOrHostAddressConstKHR {
            device_address: vertex_address,
        })
        .vertex_stride(std::mem::size_of::<Vertex>() as u64)
        .index_type(vk::IndexType::UINT32)
        .index_data(vk::DeviceOrHostAddressConstKHR {
            device_address: index_address,
        })
        .transform_data(vk::DeviceOrHostAddressConstKHR::default())
        .max_vertex(object.raw_vertex_count as u32);
    let geometry = *vk::AccelerationStructureGeometryKHR::builder()
        .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
        .flags(vk::GeometryFlagsKHR::OPAQUE)
        .geometry(vk::AccelerationStructureGeometryDataKHR { triangles });
    let range_info = *vk::AccelerationStructureBuildRangeInfoKHR::builder()
        .first_vertex(0)
        .primitive_count(object.triangle_count as u32)
        .primitive_offset(0)
        .transform_offset(0);

    let geometries = [geometry];
    let mut blas_info = *vk::AccelerationStructureBuildGeometryInfoKHR::builder()
        .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
        .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
        .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
        .geometries(&geometries);

    let size_info = unsafe {
        as_ext.get_acceleration_structure_build_sizes(
            vk::AccelerationStructureBuildTypeKHR::DEVICE,
            &blas_info,
            &[range_info.primitive_count],
        )
    };

    let scratch = Buffer::create(
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::STORAGE_BUFFER,
        size_info.build_scratch_size as usize,
        &instance,
        physical_device,
        &logical_device,
    );

    let blas_buffer = Buffer::create(
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
        size_info.acceleration_structure_size as usize,
        &instance,
        physical_device,
        &logical_device,
    );
    let blas_create_info = *vk::AccelerationStructureCreateInfoKHR::builder()
        .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
        .size(size_info.acceleration_structure_size)
        .buffer(blas_buffer.buffer);
    let blas = unsafe { as_ext.create_acceleration_structure(&blas_create_info, None) }.unwrap();

    blas_info.dst_acceleration_structure = blas;
    blas_info.scratch_data.device_address = scratch.device_address(&bda_ext);

    let blas_range_infos = [range_info];
    let all_blas_build_infos = [blas_info];
    let all_blas_range_infos = [blas_range_infos.as_slice()];
    onetime_commands(&logical_device, queue, command_pool, |command_buffer| {
        unsafe {
            as_ext.cmd_build_acceleration_structures(
                command_buffer,
                &all_blas_build_infos,
                &all_blas_range_infos,
            )
        };
    });

    scratch.cleanup(&logical_device);

    RaytraceResources {
        acceleration_structure: blas,
        buffer: blas_buffer,
        primitive_count: object.triangle_count,
    }
}

fn create_tlas(
    blas: &RaytraceResources,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
) -> RaytraceResources {
    let as_ext = AccelerationStructure::new(&instance, &logical_device);
    let bda_ext = BufferDeviceAddress::new(&instance, &logical_device);

    let instanced = vk::AccelerationStructureInstanceKHR {
        transform: vk::TransformMatrixKHR {
            matrix: [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0.],
        },
        instance_custom_index_and_mask: Packed24_8::new(0, 0xff),
        instance_shader_binding_table_record_offset_and_flags: Packed24_8::new(
            0,
            vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE.as_raw() as u8,
        ),
        acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
            device_handle: unsafe {
                as_ext.get_acceleration_structure_device_address(
                    &vk::AccelerationStructureDeviceAddressInfoKHR::builder()
                        .acceleration_structure(blas.acceleration_structure),
                )
            },
        },
    };

    let instances_buffer = Buffer::create(
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
            | vk::BufferUsageFlags::TRANSFER_DST,
        std::mem::size_of::<vk::AccelerationStructureInstanceKHR>(),
        instance,
        physical_device,
        logical_device,
    );
    instances_buffer.fill_from_slice(
        &[instanced],
        instance,
        physical_device,
        logical_device,
        queue,
        command_pool,
    );
    let instances_address = instances_buffer.device_address(&bda_ext);

    let instances_vk = *vk::AccelerationStructureGeometryInstancesDataKHR::builder().data(
        vk::DeviceOrHostAddressConstKHR {
            device_address: instances_address,
        },
    );

    let tlas_geometry = *vk::AccelerationStructureGeometryKHR::builder()
        .geometry_type(vk::GeometryTypeKHR::INSTANCES)
        .geometry(vk::AccelerationStructureGeometryDataKHR {
            instances: instances_vk,
        });
    let tlas_geometries = [tlas_geometry];
    let mut tlas_info = *vk::AccelerationStructureBuildGeometryInfoKHR::builder()
        .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
        .geometries(&tlas_geometries)
        .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
        .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL);

    let tlas_size_info = unsafe {
        as_ext.get_acceleration_structure_build_sizes(
            vk::AccelerationStructureBuildTypeKHR::DEVICE,
            &tlas_info,
            &[blas.primitive_count as u32],
        )
    };

    let tlas_buffer = Buffer::create(
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
        tlas_size_info.acceleration_structure_size as usize,
        &instance,
        physical_device,
        &logical_device,
    );
    let tlas_create_info = *vk::AccelerationStructureCreateInfoKHR::builder()
        .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
        .size(tlas_size_info.acceleration_structure_size)
        .buffer(tlas_buffer.buffer);
    let tlas = unsafe { as_ext.create_acceleration_structure(&tlas_create_info, None) }.unwrap();

    let tlas_scratch = Buffer::create(
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::STORAGE_BUFFER,
        tlas_size_info.build_scratch_size as usize,
        &instance,
        physical_device,
        &logical_device,
    );

    tlas_info.dst_acceleration_structure = tlas;
    tlas_info.scratch_data.device_address = tlas_scratch.device_address(&bda_ext);

    let tlas_range_info = *vk::AccelerationStructureBuildRangeInfoKHR::builder()
        .first_vertex(0)
        .primitive_count(1)
        .primitive_offset(0)
        .transform_offset(0);
    let tlas_range_infos = [tlas_range_info];
    let all_tlas_build_infos = [tlas_info];
    let all_tlas_range_infos = [tlas_range_infos.as_slice()];
    onetime_commands(&logical_device, queue, command_pool, |command_buffer| {
        unsafe {
            as_ext.cmd_build_acceleration_structures(
                command_buffer,
                &all_tlas_build_infos,
                &all_tlas_range_infos,
            )
        };
    });

    tlas_scratch.cleanup(&logical_device);
    instances_buffer.cleanup(&logical_device);

    RaytraceResources {
        acceleration_structure: tlas,
        buffer: tlas_buffer,
        primitive_count: 1,
    }
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

fn compute_projection(swapchain_extent: vk::Extent2D) -> Matrix4<f32> {
    let aspect_ratio = swapchain_extent.width as f32 / swapchain_extent.height as f32;
    let mut proj = Matrix4::new_perspective(aspect_ratio, FRAC_PI_4, 1., 10000.);
    proj[(1, 1)] *= -1.;
    proj
}
