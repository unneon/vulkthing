use crate::cli::Args;
use crate::config::{
    DEFAULT_VOXEL_MESHLET_MAX_COUNT, DEFAULT_VOXEL_OCTREE_MAX_COUNT,
    DEFAULT_VOXEL_TRIANGLE_MAX_COUNT, DEFAULT_VOXEL_VERTEX_MAX_COUNT,
};
use crate::gpu::std430::Star;
use crate::mesh::MeshData;
use crate::renderer::codegen::{
    alloc_descriptor_set, create_descriptor_pool, create_descriptor_set_layout, create_pipelines,
    create_samplers, create_shader_modules,
};
use crate::renderer::debug::{create_debug_messenger, set_label};
use crate::renderer::device::{select_device, DeviceInfo};
use crate::renderer::swapchain::create_swapchain;
use crate::renderer::util::{vulkan_str, Buffer, Dev, ImageResources, StorageBuffer};
use crate::renderer::vertex::Vertex;
use crate::renderer::{
    DeviceSupport, MeshObject, Renderer, Synchronization, UniformBuffer, DEPTH_FORMAT,
    FRAMES_IN_FLIGHT, VRAM_VIA_BAR,
};
use crate::voxel::gpu::meshlets::VoxelMeshletMemory;
use crate::voxel::gpu::{VoxelGpuMemory, EMPTY_ROOT};
use crate::world::World;
use crate::{VULKAN_APP_NAME, VULKAN_APP_VERSION, VULKAN_ENGINE_NAME, VULKAN_ENGINE_VERSION};
use ash::ext::{debug_utils, mesh_shader};
use ash::khr::{surface, swapchain};
use ash::{vk, Device, Entry, Instance};
use log::{debug, warn};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::ffi::CString;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

impl Renderer {
    pub fn new(
        window: &Window,
        meshes: &[&MeshData<Vertex>],
        world: &World,
        args: &Args,
    ) -> Renderer {
        let entry = unsafe { Entry::load() }.unwrap();
        let instance = create_instance(window, &entry, args);
        let debug_ext_instance = debug_utils::Instance::new(&entry, &instance);
        let debug_messenger = create_debug_messenger(&debug_ext_instance);
        let surface_ext = surface::Instance::new(&entry, &instance);
        let surface = create_surface(window, &entry, &instance);
        let DeviceInfo {
            physical_device,
            queue_family,
        } = select_device(surface, &instance, &surface_ext);
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let mut ms_features = vk::PhysicalDeviceMeshShaderFeaturesEXT::default();
        let mut features = vk::PhysicalDeviceFeatures2::default().push_next(&mut ms_features);
        unsafe { instance.get_physical_device_features2(physical_device, &mut features) };
        let device_support = DeviceSupport {
            mesh_shaders: (ms_features.mesh_shader != 0) && (ms_features.task_shader != 0),
        };
        if !device_support.mesh_shaders {
            warn!("mesh shaders not available");
        }
        let logical_device =
            create_logical_device(queue_family, &instance, physical_device, &device_support);
        let debug_ext = debug_utils::Device::new(&instance, &logical_device);
        let swapchain_ext = swapchain::Device::new(&instance, &logical_device);
        let mesh_ext = mesh_shader::Device::new(&instance, &logical_device);
        let dev = Dev {
            logical: logical_device,
            physical: physical_device,
            instance,
            debug_ext,
            debug_ext_instance,
            surface_ext,
            swapchain_ext,
            mesh_ext,
            support: device_support,
        };
        let queue = unsafe { dev.get_device_queue(queue_family, 0) };
        let command_pools = create_command_pools(queue_family, &dev);
        let command_buffers = create_command_buffers(&command_pools, &dev);

        let samplers = create_samplers(&dev);

        let descriptor_set_layout = create_descriptor_set_layout(&samplers, &dev);
        let descriptor_pool = create_descriptor_pool(descriptor_set_layout, &dev);

        let swapchain = create_swapchain(surface, window.inner_size(), &dev);
        let sync = create_sync(swapchain.images.len(), &dev);
        let depth = create_depth(swapchain.extent, &dev);
        let pipeline_layout = create_pipeline_layout(descriptor_set_layout, &dev);
        let shader_modules = create_shader_modules(&dev);
        let pipelines = create_pipelines(
            vk::SampleCountFlags::TYPE_1,
            &swapchain,
            &shader_modules,
            pipeline_layout,
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
        let mut voxel_octree_buffer =
            StorageBuffer::new_array(VRAM_VIA_BAR, DEFAULT_VOXEL_OCTREE_MAX_COUNT, &dev);
        voxel_octree_buffer.generate(|_| EMPTY_ROOT);

        let global = UniformBuffer::create(&dev);
        let global_descriptor_sets = alloc_descriptor_set(
            &global,
            &stars,
            &voxel_vertex_buffer,
            &voxel_triangle_buffer,
            &voxel_meshlet_buffer,
            &voxel_octree_buffer,
            &dev,
            descriptor_set_layout,
            descriptor_pool,
        );

        let voxel_meshlet_count = Arc::new(AtomicU32::new(0));
        let voxel_gpu_memory = Box::new(VoxelMeshletMemory::new(
            voxel_meshlet_count.clone(),
            voxel_vertex_buffer,
            voxel_triangle_buffer,
            voxel_meshlet_buffer,
            voxel_octree_buffer,
            dev.clone(),
        )) as Box<dyn VoxelGpuMemory>;

        Renderer {
            _entry: entry,
            debug_messenger,
            surface,
            dev,
            queue,
            properties,
            samplers,
            descriptor_set_layout,
            descriptor_pool,
            pipeline_layout,
            swapchain,
            pipelines,
            depth,
            command_pools,
            command_buffers,
            sync,
            flight_index: 0,
            mesh_objects,
            stars,
            global,
            descriptor_sets: global_descriptor_sets,
            voxel_meshlet_count,
            voxel_gpu_memory: Some(voxel_gpu_memory),
            query_pool,
            frame_index: 0,
            frametime: None,
            just_completed_first_render: false,
            #[cfg(feature = "dev-menu")]
            interface_renderer: None,
        }
    }

    #[cfg(feature = "dev-menu")]
    pub fn create_interface_renderer(&mut self, imgui: &mut imgui::Context) {
        self.interface_renderer = Some(
            imgui_rs_vulkan_renderer::Renderer::with_default_allocator(
                &self.dev.instance,
                self.dev.physical,
                self.dev.logical.clone(),
                self.queue,
                self.command_pools[0],
                imgui_rs_vulkan_renderer::DynamicRendering {
                    color_attachment_format: self.swapchain.format.format,
                    depth_attachment_format: Some(DEPTH_FORMAT),
                },
                imgui,
                Some(imgui_rs_vulkan_renderer::Options {
                    in_flight_frames: FRAMES_IN_FLIGHT,
                    enable_depth_test: false,
                    enable_depth_write: false,
                    subpass: 0,
                    sample_count: vk::SampleCountFlags::TYPE_1,
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
        self.depth = create_depth(self.swapchain.extent, &self.dev);

        self.recreate_pipelines();
    }

    pub fn recreate_pipelines(&mut self) {
        unsafe { self.dev.device_wait_idle() }.unwrap();
        self.pipelines.cleanup(&self.dev);
        let shader_modules = create_shader_modules(&self.dev);
        self.pipelines = create_pipelines(
            vk::SampleCountFlags::TYPE_1,
            &self.swapchain,
            &shader_modules,
            self.pipeline_layout,
            &self.dev,
        );
        shader_modules.cleanup(&self.dev);
    }

    fn cleanup_swapchain(&mut self) {
        self.swapchain.cleanup(&self.dev);
        self.depth.cleanup(&self.dev);
    }
}

impl Synchronization {
    fn cleanup(&self, dev: &Device) {
        for fence in self.in_flight {
            unsafe { dev.destroy_fence(fence, None) };
        }
        for semaphore in &self.render_finished {
            unsafe { dev.destroy_semaphore(*semaphore, None) };
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

            #[cfg(feature = "dev-menu")]
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
            self.dev.destroy_pipeline_layout(self.pipeline_layout, None);
            self.dev.destroy_descriptor_pool(self.descriptor_pool, None);
            self.dev
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.samplers.cleanup(&self.dev);
            self.dev.destroy_device(None);
            self.dev.surface_ext.destroy_surface(self.surface, None);
            self.dev
                .debug_ext_instance
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
    let app_info = vk::ApplicationInfo::default()
        .application_name(&app_name)
        .application_version(app_version)
        .engine_name(&engine_name)
        .engine_version(engine_version)
        .api_version(vk::API_VERSION_1_3);

    let layers = unsafe { entry.enumerate_instance_layer_properties() }.unwrap();
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
    };

    // Vulkan doesn't appear to have any interesting extensions at this level, physical device
    // extensions are the interesting ones with raytracing and other stuff. This is just for
    // OS-specific windowing system interactions, and enabling debug logging for the validation
    // layers.
    let mut extension_names =
        ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
            .unwrap()
            .to_vec();
    extension_names.push(debug_utils::NAME.as_ptr());

    let instance_create_info = vk::InstanceCreateInfo::default()
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
            window.display_handle().unwrap().as_raw(),
            window.window_handle().unwrap().as_raw(),
            None,
        )
    }
    .unwrap()
}

fn create_logical_device(
    queue_family: u32,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    device_support: &DeviceSupport,
) -> Device {
    let queue_create = vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_family)
        .queue_priorities(&[1.]);
    let queues = [queue_create];

    let mut extensions = vec![swapchain::NAME.as_ptr()];
    if device_support.mesh_shaders {
        extensions.extend_from_slice(&[
            mesh_shader::NAME.as_ptr(),
            ash::khr::shader_float_controls::NAME.as_ptr(),
            ash::khr::spirv_1_4::NAME.as_ptr(),
        ]);
    }

    // TODO: I'm not sure why some of these features are required.
    let features = vk::PhysicalDeviceFeatures::default()
        .fill_mode_non_solid(true)
        .fragment_stores_and_atomics(true)
        .shader_int16(true)
        .shader_int64(true)
        .vertex_pipeline_stores_and_atomics(true);
    let mut vk11_features = vk::PhysicalDeviceVulkan11Features::default()
        .shader_draw_parameters(true)
        .storage_buffer16_bit_access(true)
        .uniform_and_storage_buffer16_bit_access(true);
    let mut vk12_features = vk::PhysicalDeviceVulkan12Features::default()
        .buffer_device_address(true)
        .shader_int8(true)
        .storage_buffer8_bit_access(true)
        .timeline_semaphore(true)
        .uniform_and_storage_buffer8_bit_access(true)
        .vulkan_memory_model(true)
        .vulkan_memory_model_device_scope(true);
    let mut vk13_features = vk::PhysicalDeviceVulkan13Features::default()
        .dynamic_rendering(true)
        .maintenance4(true)
        .synchronization2(true);
    let mut ms_features = vk::PhysicalDeviceMeshShaderFeaturesEXT::default()
        .mesh_shader(device_support.mesh_shaders)
        .task_shader(device_support.mesh_shaders);

    let mut create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queues)
        .enabled_features(&features)
        .enabled_extension_names(&extensions)
        .push_next(&mut vk11_features)
        .push_next(&mut vk12_features)
        .push_next(&mut vk13_features);
    if device_support.mesh_shaders {
        create_info = create_info.push_next(&mut ms_features);
    }

    unsafe { instance.create_device(physical_device, &create_info, None) }.unwrap()
}

fn create_pipeline_layout(layout: vk::DescriptorSetLayout, dev: &Dev) -> vk::PipelineLayout {
    let create_info =
        vk::PipelineLayoutCreateInfo::default().set_layouts(std::array::from_ref(&layout));
    unsafe { dev.create_pipeline_layout(&create_info, None).unwrap() }
}

fn create_depth(extent: vk::Extent2D, dev: &Dev) -> ImageResources {
    ImageResources::create(
        DEPTH_FORMAT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
        vk::ImageAspectFlags::DEPTH,
        extent,
        vk::SampleCountFlags::TYPE_1,
        dev,
    )
}

fn create_command_pools(queue_family: u32, dev: &Dev) -> [vk::CommandPool; FRAMES_IN_FLIGHT] {
    let command_pool_info = vk::CommandPoolCreateInfo::default().queue_family_index(queue_family);
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
        let buffer_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pools[i])
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        *buffer = unsafe { dev.allocate_command_buffers(&buffer_info) }.unwrap()[0];
    }
    buffers
}

pub fn create_vertex_buffer(vertex_data: &[Vertex], dev: &Dev) -> Buffer {
    let size = std::mem::size_of_val(vertex_data);
    let mut vertex = Buffer::create(VRAM_VIA_BAR, vk::BufferUsageFlags::VERTEX_BUFFER, size, dev);
    vertex.fill_from_slice_host_visible(vertex_data, dev);
    vertex
}

fn create_index_buffer(index_data: &[u32], dev: &Dev) -> Buffer {
    let size = std::mem::size_of_val(index_data);
    let mut vertex = Buffer::create(VRAM_VIA_BAR, vk::BufferUsageFlags::INDEX_BUFFER, size, dev);
    vertex.fill_from_slice_host_visible(index_data, dev);
    vertex
}

fn create_sync(image_count: usize, dev: &Dev) -> Synchronization {
    let semaphore_info = vk::SemaphoreCreateInfo::default();
    let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
    let mut image_available: [vk::Semaphore; FRAMES_IN_FLIGHT] = Default::default();
    let mut in_flight: [vk::Fence; FRAMES_IN_FLIGHT] = Default::default();
    for i in 0..FRAMES_IN_FLIGHT {
        image_available[i] = unsafe { dev.create_semaphore(&semaphore_info, None) }.unwrap();
        set_label(image_available[i], &format!("image_available[{i}]"), dev);
        in_flight[i] = unsafe { dev.create_fence(&fence_info, None) }.unwrap();
        set_label(in_flight[i], &format!("in_flight[{i}]"), dev);
    }
    let render_finished = (0..image_count)
        .map(|i| {
            let render_finished = unsafe { dev.create_semaphore(&semaphore_info, None) }.unwrap();
            set_label(render_finished, &format!("render_finished[{i}]"), dev);
            render_finished
        })
        .collect();
    Synchronization {
        image_available,
        render_finished,
        in_flight,
    }
}

fn create_query_pool(dev: &Dev) -> vk::QueryPool {
    let create_info = vk::QueryPoolCreateInfo::default()
        .query_type(vk::QueryType::TIMESTAMP)
        .query_count((2 * FRAMES_IN_FLIGHT) as u32);
    unsafe { dev.create_query_pool(&create_info, None) }.unwrap()
}
