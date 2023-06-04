use crate::model::Model;
use crate::renderer::debug::create_debug_messenger;
use crate::renderer::descriptors::{
    create_descriptor_metadata, Descriptor, DescriptorConfig, DescriptorKind, DescriptorMetadata,
    DescriptorValue,
};
use crate::renderer::device::{select_device, DeviceInfo};
use crate::renderer::graph::{create_pass, AttachmentConfig, Pass};
use crate::renderer::pipeline::{create_pipeline, Pipeline, PipelineConfig, VertexLayout};
use crate::renderer::raytracing::{create_blas, create_tlas};
use crate::renderer::swapchain::{create_swapchain, Swapchain};
use crate::renderer::traits::VertexOps;
use crate::renderer::uniform::{
    FragSettings, Light, Material, ModelViewProjection, Postprocessing,
};
use crate::renderer::util::{find_max_msaa_samples, Buffer, Ctx, Dev};
use crate::renderer::vertex::Vertex;
use crate::renderer::{
    Object, Renderer, Synchronization, UniformBuffer, VulkanExtensions, FRAMES_IN_FLIGHT,
};
use crate::window::Window;
use crate::{VULKAN_APP_NAME, VULKAN_APP_VERSION, VULKAN_ENGINE_NAME, VULKAN_ENGINE_VERSION};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{
    AccelerationStructure, BufferDeviceAddress, DeferredHostOperations, Surface,
    Swapchain as SwapchainKhr,
};
use ash::vk::{ExtDescriptorIndexingFn, KhrRayQueryFn, KhrShaderFloatControlsFn, KhrSpirv14Fn};
use ash::{vk, Device, Entry, Instance};
use log::warn;
use nalgebra::Matrix4;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::f32::consts::FRAC_PI_4;
use std::ffi::{CStr, CString};
use winit::dpi::PhysicalSize;

// Format used for passing HDR data between render passes to enable realistic differences in
// lighting parameters and improve postprocessing effect quality, not related to monitor HDR.
// Support for this format is required by the Vulkan specification.
const COLOR_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;

const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;

impl Renderer {
    pub fn new(window: &Window, models: &[&Model]) -> Renderer {
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
        let dev = Dev {
            logical: logical_device,
            physical: physical_device,
            instance,
        };
        let queue = unsafe { dev.get_device_queue(queue_family, 0) };
        let swapchain_ext = SwapchainKhr::new(&dev.instance, &dev);

        let msaa_samples = find_max_msaa_samples(&dev);
        let offscreen_sampler = create_offscreen_sampler(&dev);
        let postprocessing = UniformBuffer::create(&dev);

        let object_descriptor_metadata = create_object_descriptor_metadata(&dev);
        let postprocess_descriptor_metadata =
            create_postprocess_descriptor_metadata(offscreen_sampler, &dev);

        let (
            swapchain,
            object_pipeline,
            render,
            postprocess_pipeline,
            postprocess,
            postprocess_descriptor_sets,
            projection,
        ) = create_swapchain_all(
            window.window.inner_size(),
            &extensions.surface,
            &swapchain_ext,
            surface,
            msaa_samples,
            &postprocessing,
            &object_descriptor_metadata,
            &postprocess_descriptor_metadata,
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

        let mut objects = Vec::new();
        for model in models {
            let object = create_object(
                model,
                &object_descriptor_metadata,
                &light,
                &frag_settings,
                &ctx,
            );
            objects.push(object);
        }

        let blas = create_blas(&objects[0], &ctx);
        let tlas = create_tlas(&blas, &ctx);
        for object in &objects {
            for i in 0..FRAMES_IN_FLIGHT {
                let acceleration_structures = [tlas.acceleration_structure];
                let mut tlas_write = *vk::WriteDescriptorSetAccelerationStructureKHR::builder()
                    .acceleration_structures(&acceleration_structures);
                let mut descriptor_writes = [*vk::WriteDescriptorSet::builder()
                    .dst_set(object.descriptor_sets[i])
                    .dst_binding(4)
                    .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                    .push_next(&mut tlas_write)];
                descriptor_writes[0].descriptor_count = 1;
                unsafe { dev.update_descriptor_sets(&descriptor_writes, &[]) };
            }
        }

        Renderer {
            _entry: entry,
            extensions,
            debug_messenger,
            surface,
            dev,
            queue,
            swapchain_ext,
            msaa_samples,
            offscreen_sampler,
            postprocessing,
            object_descriptor_metadata,
            object_pipeline,
            render,
            postprocess_descriptor_metadata,
            postprocess_pipeline,
            postprocess,
            swapchain,
            postprocess_descriptor_sets,
            projection,
            command_pools,
            command_buffers,
            sync,
            flight_index: 0,
            light,
            frag_settings,
            objects,
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
            object_pipeline,
            render_pass,
            postprocess_pipeline,
            postprocess_pass,
            postprocess_descriptor_sets,
            projection,
        ) = create_swapchain_all(
            window_size,
            &self.extensions.surface,
            &self.swapchain_ext,
            self.surface,
            self.msaa_samples,
            &self.postprocessing,
            &self.object_descriptor_metadata,
            &self.postprocess_descriptor_metadata,
            &self.dev,
        );

        // Doing the assignments at the end guarantees any operation won't fail in the middle, and
        // makes it possible to easily compare new values to old ones.
        self.swapchain = swapchain;
        self.object_pipeline = object_pipeline;
        self.render = render_pass;
        self.postprocess_pipeline = postprocess_pipeline;
        self.postprocess = postprocess_pass;
        self.postprocess_descriptor_sets = postprocess_descriptor_sets;
        self.projection = projection;
    }

    pub fn recreate_planet(&mut self, planet_model: &Model) {
        let ctx = Ctx {
            dev: &self.dev,
            queue: self.queue,
            command_pool: self.command_pools[0],
        };
        let as_ext = AccelerationStructure::new(&self.dev.instance, &self.dev);
        unsafe { self.dev.device_wait_idle() }.unwrap();

        self.objects[0].cleanup(&self.dev, self.object_descriptor_metadata.pool);
        self.tlas.cleanup(&self.dev, &as_ext);
        self.blas.cleanup(&self.dev, &as_ext);

        self.objects[0] = create_object(
            planet_model,
            &self.object_descriptor_metadata,
            &self.light,
            &self.frag_settings,
            &ctx,
        );
        self.blas = create_blas(&self.objects[0], &ctx);
        self.tlas = create_tlas(&self.blas, &ctx);
        for object in &self.objects {
            for i in 0..FRAMES_IN_FLIGHT {
                let acceleration_structures = [self.tlas.acceleration_structure];
                let mut tlas_write = *vk::WriteDescriptorSetAccelerationStructureKHR::builder()
                    .acceleration_structures(&acceleration_structures);
                let mut descriptor_writes = [*vk::WriteDescriptorSet::builder()
                    .dst_set(object.descriptor_sets[i])
                    .dst_binding(4)
                    .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                    .push_next(&mut tlas_write)];
                descriptor_writes[0].descriptor_count = 1;
                unsafe { self.dev.update_descriptor_sets(&descriptor_writes, &[]) };
            }
        }
    }

    fn cleanup_swapchain(&mut self) {
        unsafe {
            self.dev
                .reset_descriptor_pool(
                    self.postprocess_descriptor_metadata.pool,
                    vk::DescriptorPoolResetFlags::empty(),
                )
                .unwrap();
            self.swapchain.cleanup(&self.dev);
            self.object_pipeline.cleanup(&self.dev);
            self.postprocess_pipeline.cleanup(&self.dev);
            self.render.cleanup(&self.dev);
            self.postprocess.cleanup(&self.dev);
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

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.dev.device_wait_idle().unwrap();

            drop(self.interface_renderer.take());
            for object in &self.objects {
                object.cleanup(&self.dev, self.object_descriptor_metadata.pool);
            }
            self.light.cleanup(&self.dev);
            self.frag_settings.cleanup(&self.dev);
            let as_ext = AccelerationStructure::new(&self.dev.instance, &self.dev);
            self.tlas.cleanup(&self.dev, &as_ext);
            self.blas.cleanup(&self.dev, &as_ext);
            self.sync.cleanup(&self.dev);
            for pool in &self.command_pools {
                self.dev.destroy_command_pool(*pool, None);
            }
            self.cleanup_swapchain();
            self.object_descriptor_metadata.cleanup(&self.dev);
            self.postprocess_descriptor_metadata.cleanup(&self.dev);
            self.postprocessing.cleanup(&self.dev);
            self.dev.destroy_sampler(self.offscreen_sampler, None);
            self.dev.destroy_device(None);
            self.extensions.surface.destroy_surface(self.surface, None);
            self.extensions
                .debug
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.dev.instance.destroy_instance(None);
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

    let layers = entry.enumerate_instance_layer_properties().unwrap();
    let mut layer_names = Vec::new();

    // Enable Vulkan validation layers. This should be later disabled in non-development builds.
    if let Some(layer) = find_layer(&layers, "VK_LAYER_KHRONOS_validation") {
        layer_names.push(layer);
    } else {
        warn!("vulkan validation layers not available");
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
        let layer_name_slice = unsafe {
            std::slice::from_raw_parts(
                layer.layer_name.as_ptr() as *const u8,
                layer.layer_name.len(),
            )
        };
        let layer_name = CStr::from_bytes_until_nul(layer_name_slice).unwrap();
        if layer_name.to_str().unwrap() == name {
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

    let physical_device_features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .fill_mode_non_solid(true);

    let mut bda_features =
        *vk::PhysicalDeviceBufferDeviceAddressFeaturesKHR::builder().buffer_device_address(true);
    let mut rq_features = *vk::PhysicalDeviceRayQueryFeaturesKHR::builder().ray_query(true);
    let mut as_features =
        *vk::PhysicalDeviceAccelerationStructureFeaturesKHR::builder().acceleration_structure(true);

    let extension_names = [
        AccelerationStructure::name().as_ptr(),
        BufferDeviceAddress::name().as_ptr(),
        DeferredHostOperations::name().as_ptr(),
        ExtDescriptorIndexingFn::name().as_ptr(),
        KhrRayQueryFn::name().as_ptr(),
        KhrShaderFloatControlsFn::name().as_ptr(),
        KhrSpirv14Fn::name().as_ptr(),
        SwapchainKhr::name().as_ptr(),
    ];

    unsafe {
        instance.create_device(
            physical_device,
            &vk::DeviceCreateInfo::builder()
                .queue_create_infos(std::slice::from_ref(&queue_create))
                .enabled_features(&physical_device_features)
                .enabled_extension_names(&extension_names)
                .push_next(&mut bda_features)
                .push_next(&mut rq_features)
                .push_next(&mut as_features),
            None,
        )
    }
    .unwrap()
}

fn create_swapchain_all(
    window_size: PhysicalSize<u32>,
    surface_ext: &Surface,
    swapchain_ext: &SwapchainKhr,
    surface: vk::SurfaceKHR,
    msaa_samples: vk::SampleCountFlags,
    postprocessing: &UniformBuffer<Postprocessing>,
    object_descriptor_metadata: &DescriptorMetadata,
    postprocess_descriptor_metadata: &DescriptorMetadata,
    dev: &Dev,
) -> (
    Swapchain,
    Pipeline,
    Pass,
    Pipeline,
    Pass,
    [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    Matrix4<f32>,
) {
    let swapchain = create_swapchain(surface, window_size, dev, surface_ext, swapchain_ext);
    let render = create_render_pass(msaa_samples, swapchain.extent, dev);
    let object_pipeline = create_object_pipeline(
        object_descriptor_metadata,
        msaa_samples,
        render.pass,
        swapchain.extent,
        dev,
    );
    let postprocess =
        create_postprocess_pass(swapchain.format.format, &swapchain, swapchain.extent, dev);
    let postprocess_pipeline = create_postprocess_pipeline(
        postprocess_descriptor_metadata,
        postprocess.pass,
        swapchain.extent,
        dev,
    );
    let postprocess_descriptor_sets = create_postprocess_descriptor_sets(
        render.resources[2].view,
        postprocessing,
        postprocess_descriptor_metadata,
        dev,
    );
    let projection = compute_projection(swapchain.extent);
    (
        swapchain,
        object_pipeline,
        render,
        postprocess_pipeline,
        postprocess,
        postprocess_descriptor_sets,
        projection,
    )
}

fn create_render_pass(msaa_samples: vk::SampleCountFlags, extent: vk::Extent2D, dev: &Dev) -> Pass {
    let attachments = [
        AttachmentConfig::new(COLOR_FORMAT)
            .samples(msaa_samples)
            .clear_color([0., 0., 0., 0.])
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
        AttachmentConfig::new(DEPTH_FORMAT)
            .samples(msaa_samples)
            .clear_depth(1.)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
        AttachmentConfig::new(COLOR_FORMAT)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .store(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .resolve()
            .usage(vk::ImageUsageFlags::SAMPLED),
    ];
    create_pass(extent, dev, &attachments)
}

fn create_postprocess_pass(
    format: vk::Format,
    swapchain: &Swapchain,
    extent: vk::Extent2D,
    dev: &Dev,
) -> Pass {
    let attachments = [AttachmentConfig::new(format)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .store(vk::ImageLayout::PRESENT_SRC_KHR)
        .swapchain(&swapchain.image_views)];
    create_pass(extent, dev, &attachments)
}

fn create_object_descriptor_metadata(dev: &Dev) -> DescriptorMetadata {
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
                kind: DescriptorKind::UniformBuffer,
                stage: vk::ShaderStageFlags::FRAGMENT,
            },
            Descriptor {
                kind: DescriptorKind::AccelerationStructure,
                stage: vk::ShaderStageFlags::FRAGMENT,
            },
        ],
        set_count: 3,
        dev,
    })
}

fn create_object_descriptor_sets(
    mvp: &UniformBuffer<ModelViewProjection>,
    material: &UniformBuffer<Material>,
    light: &UniformBuffer<Light>,
    frag_settings: &UniformBuffer<FragSettings>,
    metadata: &DescriptorMetadata,
    dev: &Dev,
) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
    metadata.create_sets(
        &[
            DescriptorValue::Buffer(mvp),
            DescriptorValue::Buffer(material),
            DescriptorValue::Buffer(light),
            DescriptorValue::Buffer(frag_settings),
            // TLAS needs to be written separately later.
        ],
        dev,
    )
}

fn create_postprocess_descriptor_metadata(sampler: vk::Sampler, dev: &Dev) -> DescriptorMetadata {
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
        dev,
    })
}

fn create_postprocess_descriptor_sets(
    offscreen_view: vk::ImageView,
    postprocessing: &UniformBuffer<Postprocessing>,
    metadata: &DescriptorMetadata,
    dev: &Dev,
) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
    metadata.create_sets(
        &[
            DescriptorValue::Image(offscreen_view),
            DescriptorValue::Buffer(postprocessing),
        ],
        dev,
    )
}

fn create_object_pipeline(
    descriptor_metadata: &DescriptorMetadata,
    msaa_samples: vk::SampleCountFlags,
    pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    dev: &Dev,
) -> Pipeline {
    create_pipeline(PipelineConfig {
        vertex_shader_path: "shaders/object.vert",
        fragment_shader_path: "shaders/object.frag",
        vertex_layout: Some(VertexLayout {
            stride: std::mem::size_of::<Vertex>(),
            attribute_descriptions: Vertex::attribute_descriptions(0),
        }),
        msaa_samples,
        polygon_mode: vk::PolygonMode::FILL,
        descriptor_layouts: &[descriptor_metadata.set_layout],
        depth_test: true,
        pass,
        dev,
        swapchain_extent,
    })
}

fn create_postprocess_pipeline(
    descriptors: &DescriptorMetadata,
    pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    dev: &Dev,
) -> Pipeline {
    create_pipeline(PipelineConfig {
        vertex_shader_path: "shaders/postprocess.vert",
        fragment_shader_path: "shaders/postprocess.frag",
        vertex_layout: None,
        msaa_samples: vk::SampleCountFlags::TYPE_1,
        polygon_mode: vk::PolygonMode::FILL,
        descriptor_layouts: &[descriptors.set_layout],
        depth_test: false,
        pass,
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

fn create_offscreen_sampler(dev: &Dev) -> vk::Sampler {
    let sampler_info = vk::SamplerCreateInfo::builder()
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
        .unnormalized_coordinates(true);
    unsafe { dev.create_sampler(&sampler_info, None) }.unwrap()
}

pub fn create_object(
    model: &Model,
    descriptor_metadata: &DescriptorMetadata,
    light: &UniformBuffer<Light>,
    frag_settings: &UniformBuffer<FragSettings>,
    ctx: &Ctx,
) -> Object {
    let vertex = create_vertex_buffer(&model.vertices, ctx);
    let index = create_index_buffer(&model.indices, ctx);
    let mvp = UniformBuffer::create(ctx.dev);
    let material = UniformBuffer::create(ctx.dev);
    let descriptor_sets = create_object_descriptor_sets(
        &mvp,
        &material,
        light,
        frag_settings,
        descriptor_metadata,
        ctx.dev,
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

fn create_vertex_buffer(vertex_data: &[Vertex], ctx: &Ctx) -> Buffer {
    let size = std::mem::size_of::<Vertex>() * vertex_data.len();
    let vertex = Buffer::create(
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::BufferUsageFlags::VERTEX_BUFFER
            | vk::BufferUsageFlags::TRANSFER_DST
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        size,
        ctx.dev,
    );
    vertex.fill_from_slice(vertex_data, ctx);
    vertex
}

fn create_index_buffer(index_data: &[u32], ctx: &Ctx) -> Buffer {
    let size = std::mem::size_of_val(&index_data[0]) * index_data.len();
    let index = Buffer::create(
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::BufferUsageFlags::INDEX_BUFFER
            | vk::BufferUsageFlags::TRANSFER_DST
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        size,
        ctx.dev,
    );
    index.fill_from_slice(index_data, ctx);
    index
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

fn compute_projection(swapchain_extent: vk::Extent2D) -> Matrix4<f32> {
    let aspect_ratio = swapchain_extent.width as f32 / swapchain_extent.height as f32;
    let mut proj = Matrix4::new_perspective(aspect_ratio, FRAC_PI_4, 0.01, 100000.);
    proj[(1, 1)] *= -1.;
    proj
}
