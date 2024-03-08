use crate::cli::Args;
use crate::config::{
    DEFAULT_VOXEL_MESHLET_MAX_COUNT, DEFAULT_VOXEL_TRIANGLE_MAX_COUNT,
    DEFAULT_VOXEL_VERTEX_MAX_COUNT,
};
use crate::mesh::MeshData;
use crate::renderer::codegen::{
    create_descriptor_pools, create_descriptor_set_layouts, create_pipeline_layouts,
    create_pipelines, create_render_passes, create_samplers, create_shader_modules, create_shaders,
    PASS_COUNT,
};
use crate::renderer::debug::create_debug_messenger;
use crate::renderer::device::{select_device, DeviceInfo};
use crate::renderer::swapchain::create_swapchain;
use crate::renderer::uniform::Star;
use crate::renderer::util::{vulkan_str, Buffer, Dev, StorageBuffer};
use crate::renderer::vertex::Vertex;
use crate::renderer::{
    MeshObject, Renderer, Synchronization, UniformBuffer, FRAMES_IN_FLIGHT, VRAM_VIA_BAR,
};
use crate::voxel::gpu_memory::VoxelGpuMemory;
use crate::window::Window;
use crate::world::World;
use crate::{VULKAN_APP_NAME, VULKAN_APP_VERSION, VULKAN_ENGINE_NAME, VULKAN_ENGINE_VERSION};
use ash::extensions::ext::{DebugUtils, MeshShader};
use ash::extensions::khr::{Surface, Swapchain as SwapchainKhr};
use ash::vk::{KhrShaderFloatControlsFn, KhrSpirv14Fn};
use ash::{vk, Device, Entry, Instance};
use log::{debug, warn};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::ffi::CString;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use winit::dpi::PhysicalSize;

impl Renderer {
    pub fn new(
        window: &Window,
        meshes: &[&MeshData<Vertex>],
        world: &World,
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
        } = select_device(surface, &instance, &surface_ext);
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let logical_device = create_logical_device(queue_family, &instance, physical_device);
        let swapchain_ext = SwapchainKhr::new(&instance, &logical_device);
        let mesh_ext = MeshShader::new(&instance, &logical_device);
        let dev = Dev {
            logical: logical_device,
            physical: physical_device,
            instance,
            debug_ext,
            surface_ext,
            swapchain_ext,
            mesh_ext,
        };
        let queue = unsafe { dev.get_device_queue(queue_family, 0) };
        let command_pools = create_command_pools(queue_family, &dev);
        let command_buffers = create_command_buffers(&command_pools, &dev);
        let sync = create_sync(&dev);

        let samplers = create_samplers(&dev);

        let descriptor_set_layouts = create_descriptor_set_layouts(&samplers, &dev);
        let descriptor_pools = create_descriptor_pools(&descriptor_set_layouts, &dev);

        let swapchain = create_swapchain(surface, window.window.inner_size(), &dev);
        let passes = create_render_passes(&swapchain, vk::SampleCountFlags::TYPE_1, &dev);
        let pipeline_layouts = create_pipeline_layouts(&descriptor_set_layouts, &dev);
        let shaders = create_shaders();
        let shader_modules = create_shader_modules(&shaders, &dev);
        let pipelines = create_pipelines(
            vk::SampleCountFlags::TYPE_1,
            &passes,
            &swapchain,
            &shader_modules,
            &pipeline_layouts,
            &dev,
        );
        shader_modules.cleanup(&dev);

        let mut mesh_objects = Vec::new();
        for mesh in meshes {
            let vertex = create_vertex_buffer(&mesh.vertices, &dev);
            let index = create_index_buffer(&mesh.indices, &dev);
            mesh_objects.push(MeshObject {
                triangle_count: mesh.vertices.len() / 3,
                vertex,
                index,
            });
        }

        let mut stars = StorageBuffer::new_array(VRAM_VIA_BAR, world.stars.len(), &dev);
        stars.generate(|i| Star {
            model: world.stars[i].transform.model_matrix(),
        });

        let query_pool = create_query_pool(&dev);

        let voxel_vertex_buffer =
            StorageBuffer::new_array(VRAM_VIA_BAR, DEFAULT_VOXEL_VERTEX_MAX_COUNT, &dev);
        let voxel_triangle_buffer =
            StorageBuffer::new_array(VRAM_VIA_BAR, DEFAULT_VOXEL_TRIANGLE_MAX_COUNT, &dev);
        let voxel_meshlet_buffer =
            StorageBuffer::new_array(VRAM_VIA_BAR, DEFAULT_VOXEL_MESHLET_MAX_COUNT, &dev);

        let global = UniformBuffer::create(&dev);
        let global_descriptor_sets = descriptor_pools.alloc_global(
            &global,
            &stars,
            &voxel_vertex_buffer,
            &voxel_triangle_buffer,
            &voxel_meshlet_buffer,
            &dev,
        );

        let voxel_meshlet_count = Arc::new(AtomicU32::new(0));
        let voxel_gpu_memory = Some(VoxelGpuMemory::new(
            voxel_meshlet_count.clone(),
            voxel_vertex_buffer,
            voxel_triangle_buffer,
            voxel_meshlet_buffer,
            dev.clone(),
        ));

        Renderer {
            _entry: entry,
            debug_messenger,
            surface,
            dev,
            queue,
            properties,
            samplers,
            descriptor_set_layouts,
            descriptor_pools,
            pipeline_layouts,
            passes,
            swapchain,
            pipelines,
            command_pools,
            command_buffers,
            sync,
            flight_index: 0,
            mesh_objects,
            stars,
            global,
            global_descriptor_sets,
            voxel_meshlet_count,
            voxel_gpu_memory,
            query_pool,
            frame_index: 0,
            pass_times: None,
            just_completed_first_render: false,
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
                self.passes.render.pass,
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

        self.recreate_pipelines();
    }

    pub fn recreate_pipelines(&mut self) {
        unsafe { self.dev.device_wait_idle() }.unwrap();
        self.pipelines.cleanup(&self.dev);
        let shaders = create_shaders();
        let shader_modules = create_shader_modules(&shaders, &self.dev);
        self.pipelines = create_pipelines(
            vk::SampleCountFlags::TYPE_1,
            &self.passes,
            &self.swapchain,
            &shader_modules,
            &self.pipeline_layouts,
            &self.dev,
        );
        shader_modules.cleanup(&self.dev);
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
        self.index.cleanup(dev);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.dev.device_wait_idle().unwrap();

            drop(self.interface_renderer.take());
            self.dev.destroy_query_pool(self.query_pool, None);
            self.stars.cleanup(&self.dev);
            for mesh in &self.mesh_objects {
                mesh.cleanup(&self.dev);
            }
            self.global.cleanup(&self.dev);
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
        .api_version(vk::API_VERSION_1_2);

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
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Device {
    let queue_create = *vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family)
        .queue_priorities(&[1.]);
    let queues = [queue_create];

    let extensions = vec![
        KhrShaderFloatControlsFn::name().as_ptr(),
        KhrSpirv14Fn::name().as_ptr(),
        MeshShader::name().as_ptr(),
        SwapchainKhr::name().as_ptr(),
    ];

    let features = *vk::PhysicalDeviceFeatures::builder()
        .fill_mode_non_solid(true)
        .fragment_stores_and_atomics(true)
        .shader_int16(true);
    let mut vk11_features =
        *vk::PhysicalDeviceVulkan11Features::builder().storage_buffer16_bit_access(true);
    let mut vk12_features = *vk::PhysicalDeviceVulkan12Features::builder()
        .shader_int8(true)
        .storage_buffer8_bit_access(true);
    let mut ms_features = *vk::PhysicalDeviceMeshShaderFeaturesEXT::builder()
        .mesh_shader(true)
        .task_shader(true);

    let create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queues)
        .enabled_features(&features)
        .enabled_extension_names(&extensions)
        .push_next(&mut vk11_features)
        .push_next(&mut vk12_features)
        .push_next(&mut ms_features);

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

pub fn create_vertex_buffer(vertex_data: &[Vertex], dev: &Dev) -> Buffer {
    let size = std::mem::size_of_val(vertex_data);
    let vertex = Buffer::create(VRAM_VIA_BAR, vk::BufferUsageFlags::VERTEX_BUFFER, size, dev);
    vertex.fill_from_slice_host_visible(vertex_data, dev);
    vertex
}

fn create_index_buffer(index_data: &[u32], dev: &Dev) -> Buffer {
    let size = std::mem::size_of_val(index_data);
    let vertex = Buffer::create(VRAM_VIA_BAR, vk::BufferUsageFlags::INDEX_BUFFER, size, dev);
    vertex.fill_from_slice_host_visible(index_data, dev);
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
