use crate::cli::Args;
use crate::mesh::MeshData;
use crate::renderer::codegen::{
    create_descriptor_pools, create_descriptor_set_layouts, create_pipeline_layouts,
    create_pipelines, create_render_passes, create_samplers, create_shader_modules, create_shaders,
};
use crate::renderer::debug::{create_debug_messenger, set_label};
use crate::renderer::device::{select_device, DeviceInfo};
use crate::renderer::raytracing::{create_blas, create_tlas};
use crate::renderer::swapchain::{create_swapchain, Swapchain};
use crate::renderer::util::{sample_count, vulkan_str, Buffer, Ctx, Dev, ImageResources};
use crate::renderer::vertex::{GrassBlade, Star, Vertex};
use crate::renderer::{
    AsyncLoader, GrassChunk, MeshObject, Object, Renderer, RendererSettings, Synchronization,
    UniformBuffer, FRAMES_IN_FLIGHT,
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
        let debug_ext = DebugUtils::new(&entry, &instance);
        let surface_ext = Surface::new(&entry, &instance);
        let debug_messenger = create_debug_messenger(&debug_ext);
        let surface = create_surface(window, &entry, &instance);
        let DeviceInfo {
            physical_device,
            queue_family,
            supports_raytracing,
        } = select_device(surface, &instance, &surface_ext);
        let logical_device = create_logical_device(
            queue_family,
            supports_raytracing,
            &instance,
            physical_device,
        );
        let (acceleration_structure_ext, buffer_device_address_ext) = if supports_raytracing {
            let as_ext = AccelerationStructure::new(&instance, &logical_device);
            let bda_ext = BufferDeviceAddress::new(&instance, &logical_device);
            (Some(as_ext), Some(bda_ext))
        } else {
            (None, None)
        };
        let swapchain_ext = SwapchainKhr::new(&instance, &logical_device);
        let dev = Dev {
            logical: logical_device,
            physical: physical_device,
            instance,
            acceleration_structure_ext,
            buffer_device_address_ext,
            debug_ext,
            surface_ext,
            swapchain_ext,
        };
        let queue = unsafe { dev.get_device_queue(queue_family, 0) };
        let command_pools = create_command_pools(queue_family, &dev);
        let command_buffers = create_command_buffers(&command_pools, &dev);
        let sync = create_sync(&dev);
        let ctx = Ctx {
            dev: &dev,
            queue,
            command_pool: command_pools[0],
        };

        let msaa_samples = settings.msaa_samples;
        let samplers = create_samplers(&dev);

        let descriptor_set_layouts = create_descriptor_set_layouts(&samplers, &dev);
        let descriptor_pools = create_descriptor_pools(&descriptor_set_layouts, &dev);

        let swapchain = create_swapchain(surface, window.window.inner_size(), &dev);
        let passes = create_render_passes(&swapchain, msaa_samples, &dev);
        let lowres_bloom = create_lowres_bloom(&swapchain, &ctx);
        let atmosphere_descriptor_sets = descriptor_pools.alloc_deferred(
            passes.render.resources[0].view,
            lowres_bloom.view,
            &dev,
        );
        let gaussian_horizontal_descriptors =
            descriptor_pools.alloc_gaussian_horizontal(lowres_bloom.view, &dev);
        let gaussian_vertical_descriptors = descriptor_pools
            .alloc_gaussian_vertical(passes.gaussian_horizontal.resources[0].view, &dev);
        let postprocess_descriptor_sets = descriptor_pools.alloc_postprocess(
            passes.render.resources[0].view,
            passes.gaussian_vertical.resources[0].view,
            &dev,
        );
        let pipeline_layouts = create_pipeline_layouts(&descriptor_set_layouts, &dev);
        let shaders = create_shaders(supports_raytracing);
        let shader_modules = create_shader_modules(&shaders, &dev);
        let pipelines = create_pipelines(
            msaa_samples,
            &passes.render,
            &passes.gaussian_horizontal,
            &passes.gaussian_vertical,
            &passes.postprocess,
            sample_count(msaa_samples) as i32,
            1,
            0,
            0,
            1,
            &swapchain,
            &shader_modules,
            &pipeline_layouts,
            &dev,
        );
        shader_modules.cleanup(&dev);

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
            let descriptors = descriptor_pools.alloc_object(&mvp, &material, &dev);
            entities.push(Object {
                mvp,
                material,
                descriptors,
            });
        }
        let grass_mvp = UniformBuffer::create(&dev);
        let grass_material = UniformBuffer::create(&dev);
        let grass_descriptor_sets =
            descriptor_pools.alloc_object(&grass_mvp, &grass_material, &dev);
        let star_mvp = UniformBuffer::create(&dev);
        let star_material = UniformBuffer::create(&dev);
        let star_instances = Buffer::create(
            UNIFIED_MEMORY,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            world.stars.len() * std::mem::size_of::<Star>(),
            &dev,
        );
        let star_descriptor_sets = descriptor_pools.alloc_object(&star_mvp, &star_material, &dev);
        star_instances.generate_host_visible(world.stars.len(), &dev, |i| Star {
            model: world.stars[i].transform.model_matrix(),
            emit: world.stars[i].emit,
        });
        let skybox_mvp = UniformBuffer::create(&dev);
        let skybox_material = UniformBuffer::create(&dev);
        let skybox_descriptor_sets =
            descriptor_pools.alloc_object(&skybox_mvp, &skybox_material, &dev);
        let global = UniformBuffer::create(&dev);
        let global_descriptor_sets = descriptor_pools.alloc_global(&global, &tlas, &dev);

        Renderer {
            _entry: entry,
            debug_messenger,
            surface,
            dev,
            queue,
            supports_raytracing,
            msaa_samples,
            samplers,
            descriptor_set_layouts,
            descriptor_pools,
            pipeline_layouts,
            passes,
            lowres_bloom,
            atmosphere_descriptor_sets,
            swapchain,
            pipelines,
            gaussian_horizontal_descriptors,
            gaussian_vertical_descriptors,
            postprocess_descriptor_sets,
            command_pools,
            command_buffers,
            sync,
            flight_index: 0,
            grass_mvp,
            grass_material,
            mesh_objects,
            entities,
            grass_descriptor_sets,
            star_mvp,
            star_material,
            star_instances,
            star_descriptor_sets,
            skybox_mvp,
            skybox_material,
            skybox_descriptor_sets,
            grass_chunks: Arc::new(Mutex::new(Vec::new())),
            grass_blades_total: Arc::new(AtomicUsize::new(0)),
            global,
            global_descriptor_sets,
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
                self.passes.postprocess.pass,
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

        self.swapchain = create_swapchain(self.surface, window_size, &self.dev);
        self.passes = create_render_passes(&self.swapchain, self.msaa_samples, &self.dev);
        self.lowres_bloom = create_lowres_bloom(
            &self.swapchain,
            &Ctx {
                dev: &self.dev,
                command_pool: self.command_pools[0],
                queue: self.queue,
            },
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
            self.msaa_samples,
            &self.passes.render,
            &self.passes.gaussian_vertical,
            &self.passes.gaussian_vertical,
            &self.passes.postprocess,
            sample_count(self.msaa_samples) as i32,
            1,
            0,
            0,
            1,
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
        self.descriptor_pools.update_deferred(
            &self.atmosphere_descriptor_sets,
            self.passes.render.resources[0].view,
            self.lowres_bloom.view,
            &self.dev,
        );
        self.descriptor_pools.update_gaussian_horizontal(
            &self.gaussian_horizontal_descriptors,
            self.lowres_bloom.view,
            &self.dev,
        );
        self.descriptor_pools.update_gaussian_vertical(
            &self.gaussian_vertical_descriptors,
            self.passes.gaussian_horizontal.resources[0].view,
            &self.dev,
        );
        self.descriptor_pools.update_postprocess(
            &self.postprocess_descriptor_sets,
            self.passes.render.resources[0].view,
            self.passes.gaussian_vertical.resources[0].view,
            &self.dev,
        );
    }

    pub fn get_async_loader(&self) -> AsyncLoader {
        AsyncLoader {
            dev: self.dev.clone(),
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
            .extract_if(|chunk| predicate(chunk.id))
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
        self.passes.cleanup(&self.dev);
        self.lowres_bloom.cleanup(&self.dev);
    }
}

impl AsyncLoader {
    pub fn load_grass_chunk(&self, id: usize, blades_data: &[GrassBlade]) {
        trace!("loading grass chunk, \x1B[1mid\x1B[0m: {}", id);
        let blades = create_blade_buffer(blades_data, &self.dev);
        set_label(
            blades.buffer,
            &format!("Grass buffer chunk={id}"),
            &self.dev,
        );
        set_label(
            blades.memory,
            &format!("Grass memory chunk={id}"),
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
            self.star_mvp.cleanup(&self.dev);
            self.star_material.cleanup(&self.dev);
            self.star_instances.cleanup(&self.dev);
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
            self.grass_material.cleanup(&self.dev);
            self.skybox_mvp.cleanup(&self.dev);
            self.skybox_material.cleanup(&self.dev);
            self.global.cleanup(&self.dev);
            if let Some(tlas) = self.tlas.as_ref() {
                tlas.cleanup(&self.dev);
            }
            if let Some(blas) = self.blas.as_ref() {
                blas.cleanup(&self.dev);
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
            self.samplers.cleanup(&self.dev);
            self.dev.destroy_device(None);
            self.dev.surface_ext.destroy_surface(self.surface, None);
            self.dev
                .debug_ext
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

    let physical_device_features =
        vk::PhysicalDeviceFeatures::builder().fragment_stores_and_atomics(true);

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
    let size = std::mem::size_of_val(vertex_data);
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
    let size = std::mem::size_of_val(blades_data);
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

fn create_lowres_bloom(swapchain: &Swapchain, ctx: &Ctx) -> ImageResources {
    let bloom = ImageResources::create(
        vk::Format::R16G16B16A16_SFLOAT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED,
        vk::ImageAspectFlags::COLOR,
        vk::Extent2D {
            width: swapchain.extent.width / 2,
            height: swapchain.extent.height / 2,
        },
        vk::SampleCountFlags::TYPE_1,
        ctx.dev,
    );
    ctx.execute(|buf| {
        let barrier = *vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::GENERAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(bloom.image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE);
        unsafe {
            ctx.dev.cmd_pipeline_barrier(
                buf,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        };
    });
    bloom
}
