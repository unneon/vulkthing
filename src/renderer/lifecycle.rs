use crate::cli::Args;
use crate::mesh::MeshData;
use crate::renderer::codegen::{
    create_descriptor_pools, create_descriptor_set_layouts, create_pipeline_layouts,
    create_pipelines, create_samplers, create_shader_modules, create_shaders,
};
use crate::renderer::debug::{create_debug_messenger, set_label};
use crate::renderer::device::{select_device, DeviceInfo};
use crate::renderer::graph::{create_pass, AttachmentConfig, Pass, PassConfig};
use crate::renderer::raytracing::{create_blas, create_tlas};
use crate::renderer::swapchain::{create_swapchain, Swapchain};
use crate::renderer::util::{vulkan_str, Buffer, Ctx, Dev};
use crate::renderer::vertex::{GrassBlade, Vertex};
use crate::renderer::{
    AsyncLoader, GrassChunk, MeshObject, Object, Renderer, RendererSettings, Synchronization,
    UniformBuffer, VulkanExtensions, FRAMES_IN_FLIGHT,
};
use crate::window::Window;
use crate::world::World;
use crate::{VULKAN_APP_NAME, VULKAN_APP_VERSION, VULKAN_ENGINE_NAME, VULKAN_ENGINE_VERSION};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{
    AccelerationStructure, BufferDeviceAddress, DeferredHostOperations, Surface,
    Swapchain as SwapchainKhr,
};
use ash::vk::{ExtDescriptorIndexingFn, KhrRayQueryFn, KhrShaderFloatControlsFn, KhrSpirv14Fn};
use ash::{vk, Device, Entry, Instance};
use log::{debug, trace, warn};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::ffi::CString;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use winit::dpi::PhysicalSize;

// Format used for passing HDR data between render passes to enable realistic differences in
// lighting parameters and improve postprocessing effect quality, not related to monitor HDR.
// Support for this format is required by the Vulkan specification.
const COLOR_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;

const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;

pub const UNIFIED_MEMORY: vk::MemoryPropertyFlags = vk::MemoryPropertyFlags::from_raw(
    vk::MemoryPropertyFlags::DEVICE_LOCAL.as_raw()
        | vk::MemoryPropertyFlags::HOST_VISIBLE.as_raw()
        | vk::MemoryPropertyFlags::HOST_COHERENT.as_raw(),
);

impl Renderer {
    pub fn new(
        window: &Window,
        meshes: &[&MeshData],
        world: &World,
        settings: &RendererSettings,
        args: &Args,
    ) -> Renderer {
        let entry = unsafe { Entry::load() }.unwrap();
        let instance = create_instance(window, &entry, args);
        let extensions = VulkanExtensions {
            debug: DebugUtils::new(&entry, &instance),
            surface: Surface::new(&entry, &instance),
        };
        let debug_messenger = create_debug_messenger(&extensions.debug);
        let surface = create_surface(window, &entry, &instance);
        let DeviceInfo {
            physical_device,
            queue_family,
            supports_raytracing,
        } = select_device(&instance, &extensions.surface, surface);
        let logical_device = create_logical_device(
            queue_family,
            supports_raytracing,
            &instance,
            physical_device,
        );
        let dev = Dev {
            logical: logical_device,
            physical: physical_device,
            instance,
        };
        let queue = unsafe { dev.get_device_queue(queue_family, 0) };
        let swapchain_ext = SwapchainKhr::new(&dev.instance, &dev);

        let msaa_samples = settings.msaa_samples;
        let samplers = create_samplers(&dev);
        let atmosphere_uniform = UniformBuffer::create(&dev);
        let gaussian_uniform = UniformBuffer::create(&dev);
        let postprocessing = UniformBuffer::create(&dev);
        let camera = UniformBuffer::create(&dev);

        let descriptor_set_layouts = create_descriptor_set_layouts(&samplers, &dev);
        let descriptor_pools = create_descriptor_pools(&descriptor_set_layouts, &dev);

        let swapchain = create_swapchain(
            surface,
            window.window.inner_size(),
            &dev,
            &extensions.surface,
            &swapchain_ext,
        );
        let render = create_render_pass(msaa_samples, swapchain.extent, &dev);
        let gaussian = create_gaussian_pass(swapchain.extent, &dev);
        let postprocess =
            create_postprocess_pass(swapchain.format.format, &swapchain, swapchain.extent, &dev);
        let atmosphere_descriptor_sets = descriptor_pools.alloc_atmosphere(
            render.resources[0].view,
            render.resources[1].view,
            &atmosphere_uniform,
            &camera,
            &dev,
        );
        let gaussian_descriptor_sets =
            descriptor_pools.alloc_gaussian(render.resources[3].view, &gaussian_uniform, &dev);
        let postprocess_descriptor_sets = descriptor_pools.alloc_postprocess(
            render.resources[3].view,
            gaussian.resources[0].view,
            &postprocessing,
            &dev,
        );
        let pipeline_layouts = create_pipeline_layouts(&descriptor_set_layouts, &dev);
        let shaders = create_shaders(supports_raytracing);
        let shader_modules = create_shader_modules(&shaders, &dev);
        let pipelines = create_pipelines(
            &render,
            &gaussian,
            &postprocess,
            msaa_samples,
            &swapchain,
            &shader_modules,
            &pipeline_layouts,
            &dev,
        );
        shader_modules.cleanup(&dev);

        let command_pools = create_command_pools(queue_family, &dev);
        let command_buffers = create_command_buffers(&command_pools, &dev);
        let sync = create_sync(&dev);
        let ctx = Ctx {
            dev: &dev,
            queue,
            command_pool: command_pools[0],
        };

        let light = UniformBuffer::create(&dev);
        let frag_settings = UniformBuffer::create(&dev);

        let mut mesh_objects = Vec::new();
        for mesh in meshes {
            let vertex = create_vertex_buffer(&mesh.vertices, supports_raytracing, &dev);
            mesh_objects.push(MeshObject {
                triangle_count: mesh.vertices.len() / 3,
                vertex,
            });
        }

        let (tlas, blas) = if supports_raytracing {
            let blas = create_blas(&mesh_objects[0], &ctx);
            let tlas = create_tlas(&world.planet().model_matrix(world), &blas, &ctx);
            (Some(tlas), Some(blas))
        } else {
            (None, None)
        };

        let mut entities = Vec::new();
        for _ in world.entities() {
            let mvp = UniformBuffer::create(&dev);
            let material = UniformBuffer::create(&dev);
            let descriptors =
                descriptor_pools.alloc_object(&mvp, &material, &light, &frag_settings, &tlas, &dev);
            entities.push(Object {
                mvp,
                material,
                descriptors,
            });
        }
        let grass_mvp = UniformBuffer::create(&dev);
        let grass_uniform = UniformBuffer::create(&dev);
        let grass_descriptor_sets = descriptor_pools.alloc_grass(
            &grass_mvp,
            &grass_uniform,
            &light,
            &frag_settings,
            &tlas,
            &dev,
        );
        let skybox_mvp = UniformBuffer::create(&dev);
        let skybox_descriptor_sets = descriptor_pools.alloc_skybox(&skybox_mvp, &dev);

        Renderer {
            _entry: entry,
            extensions,
            debug_messenger,
            surface,
            dev,
            queue,
            swapchain_ext,
            supports_raytracing,
            msaa_samples,
            samplers,
            atmosphere_uniform,
            gaussian_uniform,
            postprocessing,
            camera,
            descriptor_set_layouts,
            descriptor_pools,
            pipeline_layouts,
            render,
            gaussian,
            atmosphere_descriptor_sets,
            postprocess,
            swapchain,
            pipelines,
            gaussian_descriptor_sets,
            postprocess_descriptor_sets,
            command_pools,
            command_buffers,
            sync,
            flight_index: 0,
            grass_mvp,
            grass_uniform,
            light,
            frag_settings,
            mesh_objects,
            entities,
            grass_descriptor_sets,
            skybox_mvp,
            skybox_descriptor_sets,
            grass_chunks: Arc::new(Mutex::new(Vec::new())),
            grass_blades_total: Arc::new(AtomicUsize::new(0)),
            blas,
            tlas,
            interface_renderer: None,
        }
    }

    pub fn create_interface_renderer(&mut self, imgui: &mut imgui::Context) {
        self.interface_renderer = Some(
            imgui_rs_vulkan_renderer::Renderer::with_default_allocator(
                &self.dev.instance,
                self.dev.physical,
                self.dev.logical.clone(),
                self.queue,
                self.command_pools[0],
                self.postprocess.pass,
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
        unsafe { self.dev.device_wait_idle() }.unwrap();

        // This destroys swapchain resources including the framebuffer, but we should also consider
        // surface information obtained during physical device selection as outdated. These can
        // contain not only things like image formats, but also some sizes.
        self.cleanup_swapchain();

        self.swapchain = create_swapchain(
            self.surface,
            window_size,
            &self.dev,
            &self.extensions.surface,
            &self.swapchain_ext,
        );
        self.render = create_render_pass(self.msaa_samples, self.swapchain.extent, &self.dev);
        self.gaussian = create_gaussian_pass(self.swapchain.extent, &self.dev);
        self.postprocess = create_postprocess_pass(
            self.swapchain.format.format,
            &self.swapchain,
            self.swapchain.extent,
            &self.dev,
        );

        self.update_offscreen_descriptors();
        self.recreate_pipelines();
    }

    pub fn recreate_pipelines(&mut self) {
        unsafe { self.dev.device_wait_idle() }.unwrap();
        self.pipelines.cleanup(&self.dev);
        let shaders = create_shaders(self.supports_raytracing);
        let shader_modules = create_shader_modules(&shaders, &self.dev);
        self.pipelines = create_pipelines(
            &self.render,
            &self.gaussian,
            &self.postprocess,
            self.msaa_samples,
            &self.swapchain,
            &shader_modules,
            &self.pipeline_layouts,
            &self.dev,
        );
        shader_modules.cleanup(&self.dev);
    }

    fn update_offscreen_descriptors(&self) {
        // Trying to update only some of the descriptors can cause DEVICE_LOST for some reason. It
        // happens even if I split the initial update right after creation into two parts, so it
        // doesn't sound like something involving state. It might be worth it to figure out what's
        // happening before I rewrite the descriptor set codegen to use batching properly. Or maybe
        // I'll switch to VK_EXT_descriptor_buffer completely?
        self.descriptor_pools.update_atmosphere(
            &self.atmosphere_descriptor_sets,
            self.render.resources[0].view,
            self.render.resources[1].view,
            &self.atmosphere_uniform,
            &self.camera,
            &self.dev,
        );
        self.descriptor_pools.update_gaussian(
            &self.gaussian_descriptor_sets,
            self.render.resources[3].view,
            &self.gaussian_uniform,
            &self.dev,
        );
        self.descriptor_pools.update_postprocess(
            &self.postprocess_descriptor_sets,
            self.render.resources[3].view,
            self.gaussian.resources[0].view,
            &self.postprocessing,
            &self.dev,
        );
    }

    pub fn get_async_loader(&self) -> AsyncLoader {
        AsyncLoader {
            dev: self.dev.clone(),
            debug_ext: self.extensions.debug.clone(),
            grass_chunks: self.grass_chunks.clone(),
            grass_blades_total: self.grass_blades_total.clone(),
        }
    }

    pub fn unload_grass_chunks(
        &mut self,
        mut predicate: impl FnMut(usize) -> bool,
        mut on_unload: impl FnMut(usize),
    ) {
        let mut first = true;
        for chunk in self
            .grass_chunks
            .lock()
            .unwrap()
            .drain_filter(|chunk| predicate(chunk.id))
        {
            trace!("unloading grass chunk, \x1B[1mid\x1B[0m: {}", chunk.id);
            self.grass_blades_total
                .fetch_sub(chunk.blade_count, Ordering::Relaxed);
            on_unload(chunk.id);
            if first {
                unsafe { self.dev.device_wait_idle() }.unwrap();
                first = false;
            }
            chunk.cleanup(&self.dev);
        }
    }

    fn cleanup_swapchain(&mut self) {
        self.swapchain.cleanup(&self.dev);
        self.render.cleanup(&self.dev);
        self.gaussian.cleanup(&self.dev);
        self.postprocess.cleanup(&self.dev);
    }
}

impl AsyncLoader {
    pub fn load_grass_chunk(&self, id: usize, blades_data: &[GrassBlade]) {
        trace!("loading grass chunk, \x1B[1mid\x1B[0m: {}", id);
        let blades = create_blade_buffer(blades_data, &self.dev);
        set_label(
            blades.buffer,
            &format!("Grass buffer chunk={id}"),
            &self.debug_ext,
            &self.dev,
        );
        set_label(
            blades.memory,
            &format!("Grass memory chunk={id}"),
            &self.debug_ext,
            &self.dev,
        );
        self.grass_chunks.lock().unwrap().push(GrassChunk {
            id,
            blades,
            blade_count: blades_data.len(),
        });
        self.grass_blades_total
            .fetch_add(blades_data.len(), Ordering::Relaxed);
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

impl MeshObject {
    pub fn cleanup(&self, dev: &Device) {
        self.vertex.cleanup(dev);
    }
}

impl Object {
    pub fn cleanup(&self, dev: &Device) {
        self.mvp.cleanup(dev);
        self.material.cleanup(dev);
    }
}

impl GrassChunk {
    pub fn cleanup(&self, dev: &Device) {
        self.blades.cleanup(dev);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.dev.device_wait_idle().unwrap();

            drop(self.interface_renderer.take());
            for entity in &self.entities {
                entity.cleanup(&self.dev);
            }
            for mesh in &self.mesh_objects {
                mesh.cleanup(&self.dev);
            }
            for grass_chunk in self.grass_chunks.lock().unwrap().iter() {
                grass_chunk.cleanup(&self.dev);
            }
            self.grass_mvp.cleanup(&self.dev);
            self.grass_uniform.cleanup(&self.dev);
            self.skybox_mvp.cleanup(&self.dev);
            self.light.cleanup(&self.dev);
            self.frag_settings.cleanup(&self.dev);
            let as_ext = AccelerationStructure::new(&self.dev.instance, &self.dev);
            if let Some(tlas) = self.tlas.as_ref() {
                tlas.cleanup(&self.dev, &as_ext);
            }
            if let Some(blas) = self.blas.as_ref() {
                blas.cleanup(&self.dev, &as_ext);
            }
            self.sync.cleanup(&self.dev);
            for pool in &self.command_pools {
                self.dev.destroy_command_pool(*pool, None);
            }
            self.cleanup_swapchain();
            self.pipelines.cleanup(&self.dev);
            self.pipeline_layouts.cleanup(&self.dev);
            self.descriptor_pools.cleanup(&self.dev);
            self.descriptor_set_layouts.cleanup(&self.dev);
            self.atmosphere_uniform.cleanup(&self.dev);
            self.gaussian_uniform.cleanup(&self.dev);
            self.postprocessing.cleanup(&self.dev);
            self.camera.cleanup(&self.dev);
            self.samplers.cleanup(&self.dev);
            self.dev.destroy_device(None);
            self.extensions.surface.destroy_surface(self.surface, None);
            self.extensions
                .debug
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.dev.instance.destroy_instance(None);
        }
    }
}

fn create_instance(window: &Window, entry: &Entry, args: &Args) -> Instance {
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

    let layers = entry.enumerate_instance_layer_properties().unwrap();
    let mut layer_names = Vec::new();

    // Enable Vulkan validation layers by default. This should be later changed in non-development
    // builds.
    if !args.disable_validation {
        if let Some(layer) = find_layer(&layers, "VK_LAYER_KHRONOS_validation") {
            layer_names.push(layer);
        } else {
            warn!("vulkan validation layers not available");
        }
    } else {
        debug!("vulkan validation layers disabled");
    }

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

fn find_layer(layers: &[vk::LayerProperties], name: &str) -> Option<*const i8> {
    for layer in layers {
        if vulkan_str(&layer.layer_name) == name {
            return Some(layer.layer_name.as_ptr());
        }
    }
    None
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
    supports_raytracing: bool,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Device {
    let queue_create = *vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family)
        .queue_priorities(&[1.]);
    let queues = [queue_create];

    let physical_device_features = vk::PhysicalDeviceFeatures::builder().sample_rate_shading(true);

    let mut extensions = vec![SwapchainKhr::name().as_ptr()];

    let mut create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queues)
        .enabled_features(&physical_device_features);

    let mut bda_features;
    let mut rq_features;
    let mut as_features;
    if supports_raytracing {
        bda_features = *vk::PhysicalDeviceBufferDeviceAddressFeaturesKHR::builder()
            .buffer_device_address(true);
        rq_features = *vk::PhysicalDeviceRayQueryFeaturesKHR::builder().ray_query(true);
        as_features = *vk::PhysicalDeviceAccelerationStructureFeaturesKHR::builder()
            .acceleration_structure(true);
        extensions.extend_from_slice(&[
            AccelerationStructure::name().as_ptr(),
            BufferDeviceAddress::name().as_ptr(),
            DeferredHostOperations::name().as_ptr(),
            ExtDescriptorIndexingFn::name().as_ptr(),
            KhrRayQueryFn::name().as_ptr(),
            KhrShaderFloatControlsFn::name().as_ptr(),
            KhrSpirv14Fn::name().as_ptr(),
        ]);
        create_info = create_info
            .push_next(&mut bda_features)
            .push_next(&mut rq_features)
            .push_next(&mut as_features);
    }

    let create_info = *create_info.enabled_extension_names(&extensions);

    unsafe { instance.create_device(physical_device, &create_info, None) }.unwrap()
}

fn create_render_pass(msaa_samples: vk::SampleCountFlags, extent: vk::Extent2D, dev: &Dev) -> Pass {
    create_pass(PassConfig {
        debug_name: "Forward rendering pass",
        debug_color: [160, 167, 161],
        attachments: &[
            AttachmentConfig::new(COLOR_FORMAT)
                .samples(msaa_samples)
                .clear_color([0., 0., 0., 0.])
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .input_to(1)
                .transient(),
            AttachmentConfig::new(vk::Format::R32G32B32A32_SFLOAT)
                .samples(msaa_samples)
                .clear_color([0., 0., 0., 0.])
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .input_to(1)
                .transient(),
            AttachmentConfig::new(DEPTH_FORMAT)
                .samples(msaa_samples)
                .clear_depth(1.)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .transient(),
            AttachmentConfig::new(COLOR_FORMAT)
                .samples(msaa_samples)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .store(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .usage(vk::ImageUsageFlags::SAMPLED)
                .subpass(1),
        ],
        dependencies: &[vk::SubpassDependency {
            src_subpass: 0,
            dst_subpass: 1,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
            src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_access_mask: vk::AccessFlags::INPUT_ATTACHMENT_READ,
            dependency_flags: vk::DependencyFlags::BY_REGION,
        }],
        extent,
        dev,
    })
}

fn create_gaussian_pass(extent: vk::Extent2D, dev: &Dev) -> Pass {
    create_pass(PassConfig {
        debug_name: "Gaussian pass",
        debug_color: [244, 244, 247],
        attachments: &[AttachmentConfig::new(COLOR_FORMAT)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .store(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .usage(vk::ImageUsageFlags::SAMPLED)],
        dependencies: &[],
        extent,
        dev,
    })
}

fn create_postprocess_pass(
    format: vk::Format,
    swapchain: &Swapchain,
    extent: vk::Extent2D,
    dev: &Dev,
) -> Pass {
    create_pass(PassConfig {
        debug_name: "Postprocess pass",
        debug_color: [210, 206, 203],
        attachments: &[AttachmentConfig::new(format)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .store(vk::ImageLayout::PRESENT_SRC_KHR)
            .swapchain(&swapchain.image_views)],
        dependencies: &[],
        extent,
        dev,
    })
}

fn create_command_pools(queue_family: u32, dev: &Dev) -> [vk::CommandPool; FRAMES_IN_FLIGHT] {
    let command_pool_info = vk::CommandPoolCreateInfo::builder().queue_family_index(queue_family);
    let mut pools = [vk::CommandPool::null(); FRAMES_IN_FLIGHT];
    for pool in &mut pools {
        *pool = unsafe { dev.create_command_pool(&command_pool_info, None) }.unwrap();
    }
    pools
}

fn create_command_buffers(
    command_pools: &[vk::CommandPool; FRAMES_IN_FLIGHT],
    dev: &Dev,
) -> [vk::CommandBuffer; FRAMES_IN_FLIGHT] {
    let mut buffers = [vk::CommandBuffer::null(); FRAMES_IN_FLIGHT];
    for (i, buffer) in buffers.iter_mut().enumerate() {
        let buffer_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pools[i])
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        *buffer = unsafe { dev.allocate_command_buffers(&buffer_info) }.unwrap()[0];
    }
    buffers
}

fn create_vertex_buffer(vertex_data: &[Vertex], supports_raytracing: bool, dev: &Dev) -> Buffer {
    let size = std::mem::size_of::<Vertex>() * vertex_data.len();
    let vertex = Buffer::create(
        UNIFIED_MEMORY,
        vk::BufferUsageFlags::VERTEX_BUFFER
            | if supports_raytracing {
                vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                    | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
            } else {
                vk::BufferUsageFlags::empty()
            },
        size,
        dev,
    );
    vertex.fill_from_slice_host_visible(vertex_data, dev);
    vertex
}

fn create_blade_buffer(blades_data: &[GrassBlade], dev: &Dev) -> Buffer {
    let size = std::mem::size_of::<GrassBlade>() * blades_data.len();
    let blades = Buffer::create(
        UNIFIED_MEMORY,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        size,
        dev,
    );
    blades.fill_from_slice_host_visible(blades_data, dev);
    blades
}

fn create_sync(dev: &Dev) -> Synchronization {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
    let mut image_available: [vk::Semaphore; FRAMES_IN_FLIGHT] = Default::default();
    let mut render_finished: [vk::Semaphore; FRAMES_IN_FLIGHT] = Default::default();
    let mut in_flight: [vk::Fence; FRAMES_IN_FLIGHT] = Default::default();
    for i in 0..FRAMES_IN_FLIGHT {
        image_available[i] = unsafe { dev.create_semaphore(&semaphore_info, None) }.unwrap();
        render_finished[i] = unsafe { dev.create_semaphore(&semaphore_info, None) }.unwrap();
        in_flight[i] = unsafe { dev.create_fence(&fence_info, None) }.unwrap();
    }
    Synchronization {
        image_available,
        render_finished,
        in_flight,
    }
}
