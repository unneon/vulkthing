use crate::cli::Args;
use crate::config::DEFAULT_STAR_COUNT;
use crate::mesh::MeshData;
use crate::renderer::codegen::{
    create_descriptor_set_layouts, create_samplers, DescriptorSetLayouts,
};
use crate::renderer::debug::{create_debug_messenger, set_label};
use crate::renderer::descriptors::{
    create_descriptor_metadata, Descriptor, DescriptorConfig, DescriptorKind, DescriptorMetadata,
    DescriptorValue,
};
use crate::renderer::device::{select_device, DeviceInfo};
use crate::renderer::graph::{create_pass, AttachmentConfig, Pass, PassConfig};
use crate::renderer::pipeline::{create_pipeline, Pipeline, PipelineConfig};
use crate::renderer::raytracing::{create_blas, create_tlas, RaytraceResources};
use crate::renderer::shader::SpecializationConstant;
use crate::renderer::swapchain::{create_swapchain, Swapchain};
use crate::renderer::uniform::{
    Atmosphere, Camera, FragSettings, Gaussian, GrassUniform, Light, Material, ModelViewProjection,
    Postprocessing,
};
use crate::renderer::util::{sample_count, vulkan_str, Buffer, Ctx, Dev};
use crate::renderer::vertex::{GrassBlade, Vertex};
use crate::renderer::{
    AsyncLoader, GrassChunk, MeshObject, Object, Pipelines, Renderer, RendererSettings,
    Synchronization, UniformBuffer, VulkanExtensions, FRAMES_IN_FLIGHT,
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

        let object_descriptor_metadata = create_object_descriptor_metadata(
            descriptor_set_layouts.object,
            supports_raytracing,
            &dev,
        );
        let grass_descriptor_metadata = create_grass_descriptor_metadata(
            descriptor_set_layouts.grass,
            supports_raytracing,
            &dev,
        );
        let skybox_descriptor_metadata =
            create_skybox_descriptor_metadata(descriptor_set_layouts.skybox, &dev);
        let atmosphere_descriptor_metadata =
            create_atmosphere_descriptor_metadata(descriptor_set_layouts.atmosphere, &dev);
        let gaussian_descriptor_metadata = create_gaussian_descriptor_metadata(
            descriptor_set_layouts.gaussian,
            samplers.pixel,
            &dev,
        );
        let postprocess_descriptor_metadata = create_postprocess_descriptor_metadata(
            descriptor_set_layouts.postprocess,
            samplers.pixel,
            &dev,
        );

        let (
            swapchain,
            render,
            atmosphere_descriptor_sets,
            gaussian,
            gaussian_descriptor_sets,
            postprocess,
            postprocess_descriptor_sets,
        ) = create_swapchain_all(
            window.window.inner_size(),
            &extensions.surface,
            &swapchain_ext,
            surface,
            msaa_samples,
            &atmosphere_uniform,
            &gaussian_uniform,
            &postprocessing,
            &camera,
            &atmosphere_descriptor_metadata,
            &gaussian_descriptor_metadata,
            &postprocess_descriptor_metadata,
            &dev,
        );
        let pipelines = create_pipelines(
            &descriptor_set_layouts,
            &render,
            &gaussian,
            &postprocess,
            msaa_samples,
            &swapchain,
            supports_raytracing,
            &dev,
        );

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
            mesh_objects.push(create_mesh(mesh, supports_raytracing, &dev));
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
            entities.push(create_entity(
                &object_descriptor_metadata,
                &light,
                &frag_settings,
                tlas.as_ref(),
                &dev,
            ));
        }
        let grass_mvp = UniformBuffer::create(&dev);
        let grass_uniform = UniformBuffer::create(&dev);
        let grass_descriptor_sets = create_grass_descriptor_sets(
            &grass_mvp,
            &grass_uniform,
            &light,
            &frag_settings,
            tlas.as_ref(),
            &grass_descriptor_metadata,
            &dev,
        );
        let skybox_mvp = UniformBuffer::create(&dev);
        let skybox_descriptor_sets =
            create_skybox_descriptor_sets(&skybox_mvp, &skybox_descriptor_metadata, &dev);

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
            object_descriptor_metadata,
            grass_descriptor_metadata,
            skybox_descriptor_metadata,
            render,
            atmosphere_descriptor_metadata,
            gaussian_descriptor_metadata,
            gaussian,
            atmosphere_descriptor_sets,
            postprocess_descriptor_metadata,
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

        let (
            swapchain,
            render_pass,
            atmosphere_descriptor_sets,
            gaussian,
            gaussian_descriptor_sets,
            postprocess_pass,
            postprocess_descriptor_sets,
        ) = create_swapchain_all(
            window_size,
            &self.extensions.surface,
            &self.swapchain_ext,
            self.surface,
            self.msaa_samples,
            &self.atmosphere_uniform,
            &self.gaussian_uniform,
            &self.postprocessing,
            &self.camera,
            &self.atmosphere_descriptor_metadata,
            &self.gaussian_descriptor_metadata,
            &self.postprocess_descriptor_metadata,
            &self.dev,
        );

        // Doing the assignments at the end guarantees any operation won't fail in the middle, and
        // makes it possible to easily compare new values to old ones.
        self.swapchain = swapchain;
        self.render = render_pass;
        self.atmosphere_descriptor_sets = atmosphere_descriptor_sets;
        self.gaussian = gaussian;
        self.gaussian_descriptor_sets = gaussian_descriptor_sets;
        self.postprocess = postprocess_pass;
        self.postprocess_descriptor_sets = postprocess_descriptor_sets;

        self.recreate_pipelines();
    }

    pub fn recreate_pipelines(&mut self) {
        unsafe { self.dev.device_wait_idle() }.unwrap();
        self.cleanup_pipelines();
        self.pipelines = create_pipelines(
            &self.descriptor_set_layouts,
            &self.render,
            &self.gaussian,
            &self.postprocess,
            self.msaa_samples,
            &self.swapchain,
            self.supports_raytracing,
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
        unsafe {
            self.dev
                .reset_descriptor_pool(
                    self.gaussian_descriptor_metadata.pool,
                    vk::DescriptorPoolResetFlags::empty(),
                )
                .unwrap();
            self.dev
                .reset_descriptor_pool(
                    self.postprocess_descriptor_metadata.pool,
                    vk::DescriptorPoolResetFlags::empty(),
                )
                .unwrap();
            self.dev
                .reset_descriptor_pool(
                    self.atmosphere_descriptor_metadata.pool,
                    vk::DescriptorPoolResetFlags::empty(),
                )
                .unwrap();
            self.swapchain.cleanup(&self.dev);
            self.render.cleanup(&self.dev);
            self.gaussian.cleanup(&self.dev);
            self.postprocess.cleanup(&self.dev);
        }
    }

    fn cleanup_pipelines(&mut self) {
        self.pipelines.object.cleanup(&self.dev);
        self.pipelines.grass.cleanup(&self.dev);
        self.pipelines.skybox.cleanup(&self.dev);
        self.pipelines.atmosphere.cleanup(&self.dev);
        self.pipelines.gaussian.cleanup(&self.dev);
        self.pipelines.postprocess.cleanup(&self.dev);
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
            self.cleanup_pipelines();
            self.descriptor_set_layouts.cleanup(&self.dev);
            self.object_descriptor_metadata.cleanup(&self.dev);
            self.grass_descriptor_metadata.cleanup(&self.dev);
            self.skybox_descriptor_metadata.cleanup(&self.dev);
            self.atmosphere_descriptor_metadata.cleanup(&self.dev);
            self.gaussian_descriptor_metadata.cleanup(&self.dev);
            self.postprocess_descriptor_metadata.cleanup(&self.dev);
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

fn create_swapchain_all(
    window_size: PhysicalSize<u32>,
    surface_ext: &Surface,
    swapchain_ext: &SwapchainKhr,
    surface: vk::SurfaceKHR,
    msaa_samples: vk::SampleCountFlags,
    atmosphere_uniform: &UniformBuffer<Atmosphere>,
    gaussian_uniform: &UniformBuffer<Gaussian>,
    postprocessing: &UniformBuffer<Postprocessing>,
    camera: &UniformBuffer<Camera>,
    atmosphere_descriptor_metadata: &DescriptorMetadata,
    gaussian_descriptor_metadata: &DescriptorMetadata,
    postprocess_descriptor_metadata: &DescriptorMetadata,
    dev: &Dev,
) -> (
    Swapchain,
    Pass,
    [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    Pass,
    [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    Pass,
    [vk::DescriptorSet; FRAMES_IN_FLIGHT],
) {
    let swapchain = create_swapchain(surface, window_size, dev, surface_ext, swapchain_ext);
    let render = create_render_pass(msaa_samples, swapchain.extent, dev);
    let atmosphere_descriptor_sets = create_atmosphere_descriptor_sets(
        render.resources[0].view,
        render.resources[1].view,
        atmosphere_uniform,
        camera,
        atmosphere_descriptor_metadata,
        dev,
    );
    let gaussian = create_gaussian_pass(swapchain.extent, dev);
    let gaussian_descriptor_sets = create_gaussian_descriptor_sets(
        render.resources[3].view,
        gaussian_uniform,
        gaussian_descriptor_metadata,
        dev,
    );
    let postprocess =
        create_postprocess_pass(swapchain.format.format, &swapchain, swapchain.extent, dev);
    let postprocess_descriptor_sets = create_postprocess_descriptor_sets(
        render.resources[3].view,
        gaussian.resources[0].view,
        postprocessing,
        postprocess_descriptor_metadata,
        dev,
    );
    (
        swapchain,
        render,
        atmosphere_descriptor_sets,
        gaussian,
        gaussian_descriptor_sets,
        postprocess,
        postprocess_descriptor_sets,
    )
}

fn create_pipelines(
    descriptor_set_layouts: &DescriptorSetLayouts,
    render: &Pass,
    gaussian: &Pass,
    postprocess: &Pass,
    msaa_samples: vk::SampleCountFlags,
    swapchain: &Swapchain,
    supports_raytracing: bool,
    dev: &Dev,
) -> Pipelines {
    let object = create_object_pipeline(
        descriptor_set_layouts.object,
        msaa_samples,
        render.pass,
        swapchain.extent,
        supports_raytracing,
        dev,
    );
    let grass = create_grass_pipeline(
        descriptor_set_layouts.grass,
        msaa_samples,
        render.pass,
        swapchain.extent,
        supports_raytracing,
        dev,
    );
    let skybox = create_skybox_pipeline(
        descriptor_set_layouts.skybox,
        msaa_samples,
        render.pass,
        swapchain.extent,
        dev,
    );
    let atmosphere = create_atmosphere_pipeline(
        descriptor_set_layouts.atmosphere,
        render.pass,
        swapchain.extent,
        msaa_samples,
        dev,
    );
    let gaussian = create_gaussian_pipeline(
        descriptor_set_layouts.gaussian,
        gaussian.pass,
        swapchain.extent,
        msaa_samples,
        dev,
    );
    let postprocess = create_postprocess_pipeline(
        descriptor_set_layouts.postprocess,
        postprocess.pass,
        swapchain.extent,
        supports_raytracing,
        msaa_samples,
        dev,
    );
    Pipelines {
        object,
        grass,
        skybox,
        atmosphere,
        gaussian,
        postprocess,
    }
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

fn create_object_descriptor_metadata(
    layout: vk::DescriptorSetLayout,
    supports_raytracing: bool,
    dev: &Dev,
) -> DescriptorMetadata {
    let mut descriptors = vec![
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
            kind: DescriptorKind::UniformBuffer,
            stage: vk::ShaderStageFlags::FRAGMENT,
        },
    ];
    if supports_raytracing {
        descriptors.push(Descriptor {
            kind: DescriptorKind::AccelerationStructure,
            stage: vk::ShaderStageFlags::FRAGMENT,
        });
    }
    create_descriptor_metadata(DescriptorConfig {
        descriptors,
        layout,
        set_count: 3 + DEFAULT_STAR_COUNT,
        dev,
    })
}

fn create_grass_descriptor_metadata(
    layout: vk::DescriptorSetLayout,
    supports_raytracing: bool,
    dev: &Dev,
) -> DescriptorMetadata {
    let mut descriptors = vec![
        Descriptor {
            kind: DescriptorKind::UniformBuffer,
            stage: vk::ShaderStageFlags::VERTEX,
        },
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
    ];
    if supports_raytracing {
        descriptors.push(Descriptor {
            kind: DescriptorKind::AccelerationStructure,
            stage: vk::ShaderStageFlags::FRAGMENT,
        });
    }
    create_descriptor_metadata(DescriptorConfig {
        descriptors,
        layout,
        set_count: 1,
        dev,
    })
}

fn create_skybox_descriptor_metadata(
    layout: vk::DescriptorSetLayout,
    dev: &Dev,
) -> DescriptorMetadata {
    let descriptors = vec![Descriptor {
        kind: DescriptorKind::UniformBuffer,
        stage: vk::ShaderStageFlags::VERTEX,
    }];
    create_descriptor_metadata(DescriptorConfig {
        descriptors,
        layout,
        set_count: 1,
        dev,
    })
}

fn create_object_descriptor_sets(
    mvp: &UniformBuffer<ModelViewProjection>,
    material: &UniformBuffer<Material>,
    light: &UniformBuffer<Light>,
    frag_settings: &UniformBuffer<FragSettings>,
    tlas: Option<&RaytraceResources>,
    metadata: &DescriptorMetadata,
    dev: &Dev,
) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
    let mut values = vec![
        DescriptorValue::Buffer(mvp),
        DescriptorValue::Buffer(material),
        DescriptorValue::Buffer(light),
        DescriptorValue::Buffer(frag_settings),
    ];
    if let Some(tlas) = tlas {
        values.push(DescriptorValue::AccelerationStructure(
            tlas.acceleration_structure,
        ));
    }
    metadata.create_sets(&values, dev)
}

fn create_grass_descriptor_sets(
    mvp: &UniformBuffer<ModelViewProjection>,
    grass_uniform: &UniformBuffer<GrassUniform>,
    light: &UniformBuffer<Light>,
    frag_settings: &UniformBuffer<FragSettings>,
    tlas: Option<&RaytraceResources>,
    metadata: &DescriptorMetadata,
    dev: &Dev,
) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
    let mut values = vec![
        DescriptorValue::Buffer(mvp),
        DescriptorValue::Buffer(grass_uniform),
        DescriptorValue::Buffer(light),
        DescriptorValue::Buffer(frag_settings),
    ];
    if let Some(tlas) = tlas {
        values.push(DescriptorValue::AccelerationStructure(
            tlas.acceleration_structure,
        ));
    }
    metadata.create_sets(&values, dev)
}

fn create_skybox_descriptor_sets(
    mvp: &UniformBuffer<ModelViewProjection>,
    metadata: &DescriptorMetadata,
    dev: &Dev,
) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
    metadata.create_sets(&[DescriptorValue::Buffer(mvp)], dev)
}

fn create_atmosphere_descriptor_metadata(
    layout: vk::DescriptorSetLayout,
    dev: &Dev,
) -> DescriptorMetadata {
    let descriptors = vec![
        Descriptor {
            kind: DescriptorKind::InputAttachment,
            stage: vk::ShaderStageFlags::FRAGMENT,
        },
        Descriptor {
            kind: DescriptorKind::InputAttachment,
            stage: vk::ShaderStageFlags::FRAGMENT,
        },
        Descriptor {
            kind: DescriptorKind::UniformBuffer,
            stage: vk::ShaderStageFlags::FRAGMENT,
        },
        Descriptor {
            kind: DescriptorKind::UniformBuffer,
            stage: vk::ShaderStageFlags::FRAGMENT,
        },
    ];
    create_descriptor_metadata(DescriptorConfig {
        descriptors,
        layout,
        set_count: 1,
        dev,
    })
}

fn create_atmosphere_descriptor_sets(
    offscreen_view: vk::ImageView,
    position_view: vk::ImageView,
    atmosphere: &UniformBuffer<Atmosphere>,
    camera: &UniformBuffer<Camera>,
    metadata: &DescriptorMetadata,
    dev: &Dev,
) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
    metadata.create_sets(
        &[
            DescriptorValue::InputAttachment(offscreen_view),
            DescriptorValue::InputAttachment(position_view),
            DescriptorValue::Buffer(atmosphere),
            DescriptorValue::Buffer(camera),
        ],
        dev,
    )
}

fn create_gaussian_descriptor_metadata(
    layout: vk::DescriptorSetLayout,
    sampler: vk::Sampler,
    dev: &Dev,
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
        layout,
        set_count: 1,
        dev,
    })
}

fn create_gaussian_descriptor_sets(
    offscreen_view: vk::ImageView,
    gaussian_uniform: &UniformBuffer<Gaussian>,
    metadata: &DescriptorMetadata,
    dev: &Dev,
) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
    metadata.create_sets(
        &[
            DescriptorValue::Image(offscreen_view),
            DescriptorValue::Buffer(gaussian_uniform),
        ],
        dev,
    )
}

fn create_postprocess_descriptor_metadata(
    layout: vk::DescriptorSetLayout,
    sampler: vk::Sampler,
    dev: &Dev,
) -> DescriptorMetadata {
    create_descriptor_metadata(DescriptorConfig {
        descriptors: vec![
            Descriptor {
                kind: DescriptorKind::ImmutableSampler { sampler },
                stage: vk::ShaderStageFlags::FRAGMENT,
            },
            Descriptor {
                kind: DescriptorKind::ImmutableSampler { sampler },
                stage: vk::ShaderStageFlags::FRAGMENT,
            },
            Descriptor {
                kind: DescriptorKind::UniformBuffer,
                stage: vk::ShaderStageFlags::FRAGMENT,
            },
        ],
        layout,
        set_count: 1,
        dev,
    })
}

fn create_postprocess_descriptor_sets(
    offscreen_view: vk::ImageView,
    bloom_view: vk::ImageView,
    postprocessing: &UniformBuffer<Postprocessing>,
    metadata: &DescriptorMetadata,
    dev: &Dev,
) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
    metadata.create_sets(
        &[
            DescriptorValue::Image(offscreen_view),
            DescriptorValue::Image(bloom_view),
            DescriptorValue::Buffer(postprocessing),
        ],
        dev,
    )
}

fn create_object_pipeline(
    layout: vk::DescriptorSetLayout,
    msaa_samples: vk::SampleCountFlags,
    pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    supports_raytracing: bool,
    dev: &Dev,
) -> Pipeline {
    create_pipeline(PipelineConfig {
        vertex_shader_path: "shaders/object.vert",
        vertex_specialization: &[],
        fragment_shader_path: "shaders/object.frag",
        fragment_specialization: &[],
        vertex_bindings: &[vk::VertexInputBindingDescription {
            binding: 0,
            stride: 24,
            input_rate: vk::VertexInputRate::VERTEX,
        }],
        vertex_attributes: &[
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
                offset: 12,
            },
        ],
        msaa_samples,
        cull_mode: vk::CullModeFlags::BACK,
        descriptor_layouts: &[layout],
        color_attachment_count: 2,
        depth_test: true,
        pass,
        subpass: 0,
        supports_raytracing,
        dev,
        swapchain_extent,
    })
}

fn create_grass_pipeline(
    layout: vk::DescriptorSetLayout,
    msaa_samples: vk::SampleCountFlags,
    pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    supports_raytracing: bool,
    dev: &Dev,
) -> Pipeline {
    create_pipeline(PipelineConfig {
        vertex_shader_path: "shaders/grass.vert",
        vertex_specialization: &[],
        fragment_shader_path: "shaders/grass.frag",
        fragment_specialization: &[],
        vertex_bindings: &[
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: 24,
                input_rate: vk::VertexInputRate::VERTEX,
            },
            vk::VertexInputBindingDescription {
                binding: 1,
                stride: 64,
                input_rate: vk::VertexInputRate::INSTANCE,
            },
        ],
        vertex_attributes: &[
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
                offset: 12,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 2,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 3,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 12,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 4,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 24,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 5,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 36,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 6,
                format: vk::Format::R32_SFLOAT,
                offset: 48,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 7,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 52,
            },
        ],
        msaa_samples,
        cull_mode: vk::CullModeFlags::NONE,
        descriptor_layouts: &[layout],
        color_attachment_count: 2,
        depth_test: true,
        pass,
        subpass: 0,
        supports_raytracing,
        dev,
        swapchain_extent,
    })
}

fn create_skybox_pipeline(
    layout: vk::DescriptorSetLayout,
    msaa_samples: vk::SampleCountFlags,
    pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    dev: &Dev,
) -> Pipeline {
    create_pipeline(PipelineConfig {
        vertex_shader_path: "shaders/skybox.vert",
        vertex_specialization: &[],
        fragment_shader_path: "shaders/skybox.frag",
        fragment_specialization: &[],
        vertex_bindings: &[vk::VertexInputBindingDescription {
            binding: 0,
            stride: 24,
            input_rate: vk::VertexInputRate::VERTEX,
        }],
        vertex_attributes: &[vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        }],
        msaa_samples,
        cull_mode: vk::CullModeFlags::FRONT,
        descriptor_layouts: &[layout],
        color_attachment_count: 2,
        depth_test: true,
        pass,
        subpass: 0,
        supports_raytracing: false,
        dev,
        swapchain_extent,
    })
}

fn create_atmosphere_pipeline(
    layout: vk::DescriptorSetLayout,
    pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags,
    dev: &Dev,
) -> Pipeline {
    create_pipeline(PipelineConfig {
        vertex_shader_path: "shaders/atmosphere.vert",
        vertex_specialization: &[],
        fragment_shader_path: "shaders/atmosphere.frag",
        fragment_specialization: &[SpecializationConstant {
            id: 0,
            value: sample_count(msaa_samples) as i32,
        }],
        vertex_bindings: &[],
        vertex_attributes: &[],
        msaa_samples,
        cull_mode: vk::CullModeFlags::BACK,
        descriptor_layouts: &[layout],
        color_attachment_count: 1,
        depth_test: false,
        pass,
        subpass: 1,
        supports_raytracing: false,
        dev,
        swapchain_extent,
    })
}

fn create_gaussian_pipeline(
    layout: vk::DescriptorSetLayout,
    pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags,
    dev: &Dev,
) -> Pipeline {
    create_pipeline(PipelineConfig {
        vertex_shader_path: "shaders/gaussian.vert",
        vertex_specialization: &[],
        fragment_shader_path: "shaders/gaussian.frag",
        fragment_specialization: &[SpecializationConstant {
            id: 0,
            value: sample_count(msaa_samples) as i32,
        }],
        vertex_bindings: &[],
        vertex_attributes: &[],
        msaa_samples: vk::SampleCountFlags::TYPE_1,
        cull_mode: vk::CullModeFlags::BACK,
        descriptor_layouts: &[layout],
        color_attachment_count: 1,
        depth_test: false,
        pass,
        subpass: 0,
        supports_raytracing: false,
        dev,
        swapchain_extent,
    })
}

fn create_postprocess_pipeline(
    layout: vk::DescriptorSetLayout,
    pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    supports_raytracing: bool,
    msaa_samples: vk::SampleCountFlags,
    dev: &Dev,
) -> Pipeline {
    create_pipeline(PipelineConfig {
        vertex_shader_path: "shaders/postprocess.vert",
        vertex_specialization: &[],
        fragment_shader_path: "shaders/postprocess.frag",
        fragment_specialization: &[SpecializationConstant {
            id: 0,
            value: sample_count(msaa_samples) as i32,
        }],
        vertex_bindings: &[],
        vertex_attributes: &[],
        msaa_samples: vk::SampleCountFlags::TYPE_1,
        cull_mode: vk::CullModeFlags::BACK,
        descriptor_layouts: &[layout],
        color_attachment_count: 1,
        depth_test: false,
        pass,
        subpass: 0,
        supports_raytracing,
        dev,
        swapchain_extent,
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

pub fn create_mesh(model: &MeshData, supports_raytracing: bool, dev: &Dev) -> MeshObject {
    let vertex = create_vertex_buffer(&model.vertices, supports_raytracing, dev);
    MeshObject {
        triangle_count: model.vertices.len() / 3,
        vertex,
    }
}

pub fn create_entity(
    descriptor_metadata: &DescriptorMetadata,
    light: &UniformBuffer<Light>,
    frag_settings: &UniformBuffer<FragSettings>,
    tlas: Option<&RaytraceResources>,
    dev: &Dev,
) -> Object {
    let mvp = UniformBuffer::create(dev);
    let material = UniformBuffer::create(dev);
    let descriptor_sets = create_object_descriptor_sets(
        &mvp,
        &material,
        light,
        frag_settings,
        tlas,
        descriptor_metadata,
        dev,
    );
    Object {
        mvp,
        material,
        descriptor_sets,
    }
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
