use crate::cli::Args;
use crate::config::DEFAULT_VOXEL_VERTEX_MEMORY;
use crate::mesh::MeshData;
use crate::renderer::codegen::{
    create_descriptor_pools, create_descriptor_set_layouts, create_pipeline_layouts,
    create_pipelines, create_render_passes, create_samplers, create_shader_modules, create_shaders,
    PASS_COUNT,
};
use crate::renderer::debug::create_debug_messenger;
use crate::renderer::device::{select_device, DeviceInfo};
use crate::renderer::raytracing::{create_blas, create_tlas};
use crate::renderer::swapchain::create_swapchain;
use crate::renderer::util::{vulkan_str, Buffer, Ctx, Dev};
use crate::renderer::vertex::{Star, Vertex};
use crate::renderer::{
    MeshObject, Object, Renderer, Synchronization, UniformBuffer, FRAMES_IN_FLIGHT, VRAM_VIA_BAR,
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
use log::{debug, warn};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::ffi::CString;
use winit::dpi::PhysicalSize;

impl Renderer {
    pub fn new(window: &Window, meshes: &[&MeshData], world: &World, args: &Args) -> Renderer {
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
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
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

        let samplers = create_samplers(&dev);

        let descriptor_set_layouts = create_descriptor_set_layouts(&samplers, &dev);
        let descriptor_pools = create_descriptor_pools(&descriptor_set_layouts, &dev);

        let swapchain = create_swapchain(surface, window.window.inner_size(), &dev);
        let passes = create_render_passes(&swapchain, vk::SampleCountFlags::TYPE_1, &dev);
        let atmosphere_descriptors = descriptor_pools.alloc_atmosphere(
            passes.render.resources[0].view,
            passes.render.resources[1].view,
            &dev,
        );
        let extract_descriptors =
            descriptor_pools.alloc_extract(passes.atmosphere.resources[0].view, &dev);
        let gaussian_horizontal_descriptors =
            descriptor_pools.alloc_gaussian(passes.extract.resources[0].view, &dev);
        let gaussian_vertical_descriptors =
            descriptor_pools.alloc_gaussian(passes.gaussian_horizontal.resources[0].view, &dev);
        let postprocess_descriptor_sets = descriptor_pools.alloc_postprocess(
            passes.atmosphere.resources[0].view,
            passes.gaussian_vertical.resources[0].view,
            &dev,
        );
        let pipeline_layouts = create_pipeline_layouts(&descriptor_set_layouts, &dev);
        let shaders = create_shaders(supports_raytracing);
        let shader_modules = create_shader_modules(&shaders, &dev);
        let pipelines = create_pipelines(
            vk::SampleCountFlags::TYPE_1,
            &passes,
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
            let tlas = create_tlas(&world.entities()[0].model_matrix(), &blas, &ctx);
            (Some(tlas), Some(blas))
        } else {
            (None, None)
        };
        // let (tlas, blas) = (None, None);

        let mut entities = Vec::new();
        for _ in world.entities() {
            let transform = UniformBuffer::create(&dev);
            let material = UniformBuffer::create(&dev);
            let descriptors = descriptor_pools.alloc_object(&transform, &material, &dev);
            entities.push(Object {
                transform,
                material,
                descriptors,
            });
        }
        let star_transform = UniformBuffer::create(&dev);
        let star_material = UniformBuffer::create(&dev);
        let star_instances = Buffer::create(
            VRAM_VIA_BAR,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            world.stars.len() * std::mem::size_of::<Star>(),
            &dev,
        );
        let star_descriptor_sets =
            descriptor_pools.alloc_object(&star_transform, &star_material, &dev);
        star_instances.generate_host_visible(&dev, |i| Star {
            model: world.stars[i].transform.model_matrix(),
            emit: world.stars[i].emit,
        });
        let skybox_transform = UniformBuffer::create(&dev);
        let skybox_material = UniformBuffer::create(&dev);
        let skybox_descriptor_sets =
            descriptor_pools.alloc_object(&skybox_transform, &skybox_material, &dev);
        let global = UniformBuffer::create(&dev);
        let global_descriptor_sets = descriptor_pools.alloc_global(&global, &tlas, &dev);

        let query_pool = create_query_pool(&dev);

        let voxel_transform = UniformBuffer::create(&dev);
        let voxel_material = UniformBuffer::create(&dev);
        let voxel_descriptor_set =
            descriptor_pools.alloc_object(&voxel_transform, &voxel_material, &dev);
        let voxel_buffer = Buffer::create(
            VRAM_VIA_BAR,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            DEFAULT_VOXEL_VERTEX_MEMORY,
            &dev,
        );

        Renderer {
            _entry: entry,
            debug_messenger,
            surface,
            dev,
            queue,
            supports_raytracing,
            properties,
            samplers,
            descriptor_set_layouts,
            descriptor_pools,
            pipeline_layouts,
            passes,
            atmosphere_descriptors,
            extract_descriptors,
            swapchain,
            pipelines,
            gaussian_horizontal_descriptors,
            gaussian_vertical_descriptors,
            postprocess_descriptor_sets,
            command_pools,
            command_buffers,
            sync,
            flight_index: 0,
            mesh_objects,
            entities,
            star_transform,
            star_material,
            star_instances,
            star_descriptor_sets,
            skybox_transform,
            skybox_material,
            skybox_descriptor_sets,
            global,
            global_descriptor_sets,
            blas,
            tlas,
            query_pool,
            frame_index: 0,
            pass_times: None,
            just_completed_first_render: false,
            interface_renderer: None,
            voxels_shared: None,
            voxel_transform,
            voxel_material,
            voxel_descriptor_set,
            voxel_buffer,
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
        self.passes =
            create_render_passes(&self.swapchain, vk::SampleCountFlags::TYPE_1, &self.dev);

        self.update_offscreen_descriptors();
        self.recreate_pipelines();
    }

    pub fn recreate_pipelines(&mut self) {
        unsafe { self.dev.device_wait_idle() }.unwrap();
        self.pipelines.cleanup(&self.dev);
        let shaders = create_shaders(self.supports_raytracing);
        let shader_modules = create_shader_modules(&shaders, &self.dev);
        self.pipelines = create_pipelines(
            vk::SampleCountFlags::TYPE_1,
            &self.passes,
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
        self.descriptor_pools.update_atmosphere(
            &self.atmosphere_descriptors,
            self.passes.render.resources[0].view,
            self.passes.render.resources[1].view,
            &self.dev,
        );
        self.descriptor_pools.update_extract(
            &self.extract_descriptors,
            self.passes.atmosphere.resources[0].view,
            &self.dev,
        );
        self.descriptor_pools.update_gaussian(
            &self.gaussian_horizontal_descriptors,
            self.passes.extract.resources[0].view,
            &self.dev,
        );
        self.descriptor_pools.update_gaussian(
            &self.gaussian_vertical_descriptors,
            self.passes.gaussian_horizontal.resources[0].view,
            &self.dev,
        );
        self.descriptor_pools.update_postprocess(
            &self.postprocess_descriptor_sets,
            self.passes.atmosphere.resources[0].view,
            self.passes.gaussian_vertical.resources[0].view,
            &self.dev,
        );
    }

    fn cleanup_swapchain(&mut self) {
        self.swapchain.cleanup(&self.dev);
        self.passes.cleanup(&self.dev);
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
        self.transform.cleanup(dev);
        self.material.cleanup(dev);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.dev.device_wait_idle().unwrap();

            drop(self.interface_renderer.take());
            self.dev.destroy_query_pool(self.query_pool, None);
            self.voxel_material.cleanup(&self.dev);
            self.voxel_transform.cleanup(&self.dev);
            self.voxel_buffer.cleanup(&self.dev);
            self.star_transform.cleanup(&self.dev);
            self.star_material.cleanup(&self.dev);
            self.star_instances.cleanup(&self.dev);
            for entity in &self.entities {
                entity.cleanup(&self.dev);
            }
            for mesh in &self.mesh_objects {
                mesh.cleanup(&self.dev);
            }
            self.skybox_transform.cleanup(&self.dev);
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

pub fn create_vertex_buffer(
    vertex_data: &[Vertex],
    supports_raytracing: bool,
    dev: &Dev,
) -> Buffer {
    let size = std::mem::size_of_val(vertex_data);
    let vertex = Buffer::create(
        VRAM_VIA_BAR,
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

fn create_query_pool(dev: &Dev) -> vk::QueryPool {
    let create_info = *vk::QueryPoolCreateInfo::builder()
        .query_type(vk::QueryType::TIMESTAMP)
        .query_count(((PASS_COUNT + 1) * FRAMES_IN_FLIGHT) as u32);
    unsafe { dev.create_query_pool(&create_info, None) }.unwrap()
}
