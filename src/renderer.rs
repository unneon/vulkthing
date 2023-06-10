mod debug;
mod descriptors;
mod device;
mod graph;
mod lifecycle;
mod pipeline;
mod raytracing;
mod shader;
mod swapchain;
pub mod uniform;
mod util;
pub mod vertex;

use crate::grass::Grass;
use crate::renderer::descriptors::DescriptorMetadata;
use crate::renderer::graph::Pass;
use crate::renderer::pipeline::Pipeline;
use crate::renderer::raytracing::RaytraceResources;
use crate::renderer::swapchain::Swapchain;
use crate::renderer::uniform::{
    Camera, FragSettings, GrassUniform, Light, Material, ModelViewProjection, Postprocessing,
};
use crate::renderer::util::{Buffer, Dev, UniformBuffer};
use crate::world::{Entity, World};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain as SwapchainKhr};
use ash::{vk, Entry};
use imgui::DrawData;
use nalgebra::{Matrix4, Vector3};
use std::f32::consts::FRAC_PI_4;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use winit::dpi::PhysicalSize;

pub struct Renderer {
    // Immutable parts of the renderer. These can't change in the current design, but recovering
    // from GPU crashes might require doing something with these later?
    _entry: Entry,
    extensions: VulkanExtensions,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    surface: vk::SurfaceKHR,
    dev: Dev,
    queue: vk::Queue,
    transfer_queue: vk::Queue,
    swapchain_ext: SwapchainKhr,
    supports_raytracing: bool,

    // Parameters of the renderer that are required early for creating more important objects.
    msaa_samples: vk::SampleCountFlags,
    offscreen_sampler: vk::Sampler,
    postprocessing: UniformBuffer<Postprocessing>,
    camera: UniformBuffer<Camera>,

    // Description of the main render pass. Doesn't contain any information about the objects yet,
    // only low-level data format descriptions.
    object_descriptor_metadata: DescriptorMetadata,
    grass_descriptor_metadata: DescriptorMetadata,
    skybox_descriptor_metadata: DescriptorMetadata,
    object_pipeline: Pipeline,
    grass_pipeline: Pipeline,
    skybox_pipeline: Pipeline,
    render: Pass,

    // Description of the postprocessing pass, and also the actual descriptor pool. Necessary,
    // because the postprocessing pass depends on swapchain extent and needs to have the descriptor
    // set updated after window resize.
    postprocess_descriptor_metadata: DescriptorMetadata,
    postprocess_pipeline: Pipeline,
    postprocess: Pass,

    // All resources that depend on swapchain extent (window size). So swapchain description, memory
    // used for all framebuffer attachments, framebuffers, and the mentioned postprocess descriptor
    // set. Projection matrix depends on the monitor aspect ratio, so it's included too.
    pub swapchain: Swapchain,
    postprocess_descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],

    // Vulkan objects actually used for command recording and synchronization. Also internal
    // renderer state for keeping track of concurrent frames.
    command_pools: [vk::CommandPool; FRAMES_IN_FLIGHT],
    command_buffers: [vk::CommandBuffer; FRAMES_IN_FLIGHT],
    transfer_command_pool: vk::CommandPool,
    sync: Synchronization,
    flight_index: usize,

    // And finally resources specific to this renderer. So various buffers related to objects we
    // actually render, their descriptor sets and the like.
    grass_vertex_count: usize,
    grass_vertex: Buffer,
    grass_mvp: UniformBuffer<ModelViewProjection>,
    grass_uniform: UniformBuffer<GrassUniform>,
    light: UniformBuffer<Light>,
    frag_settings: UniformBuffer<FragSettings>,
    objects: Vec<Object>,
    grass_chunks: Arc<Mutex<Vec<GrassChunk>>>,
    pub grass_blades_total: Arc<AtomicUsize>,
    grass_descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    skybox_mvp: UniformBuffer<ModelViewProjection>,
    skybox_descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],

    tlas: Option<RaytraceResources>,
    blas: Option<RaytraceResources>,

    interface_renderer: Option<imgui_rs_vulkan_renderer::Renderer>,
}

struct VulkanExtensions {
    debug: DebugUtils,
    surface: Surface,
}

struct Synchronization {
    image_available: [vk::Semaphore; FRAMES_IN_FLIGHT],
    render_finished: [vk::Semaphore; FRAMES_IN_FLIGHT],
    in_flight: [vk::Fence; FRAMES_IN_FLIGHT],
}

pub struct Object {
    triangle_count: usize,
    raw_vertex_count: usize,
    vertex: Buffer,
    mvp: UniformBuffer<ModelViewProjection>,
    material: UniformBuffer<Material>,
    descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
}

pub struct GrassChunk {
    id: usize,
    blade_count: usize,
    blades: Buffer,
}

pub struct AsyncLoader {
    dev: Dev,
    transfer_queue: vk::Queue,
    transfer_command_pool: vk::CommandPool,
    grass_chunks: Arc<Mutex<Vec<GrassChunk>>>,
    grass_blades_total: Arc<AtomicUsize>,
}

pub struct RendererSettings {
    pub depth_near: f32,
    pub depth_far: f32,
}

const FRAMES_IN_FLIGHT: usize = 2;

impl Renderer {
    pub fn draw_frame(
        &mut self,
        world: &World,
        grass: &Grass,
        settings: &RendererSettings,
        frag_settings: &FragSettings,
        postprocessing: &Postprocessing,
        window_size: PhysicalSize<u32>,
        ui_draw: &DrawData,
    ) {
        let Some(image_index) = (unsafe { self.prepare_command_buffer(window_size) }) else {
            return;
        };
        unsafe { self.record_command_buffer(image_index, world, ui_draw) };
        for entity in world.entities() {
            self.update_object_uniforms(world, entity, settings);
        }
        self.update_grass_uniform(grass, world, settings);
        self.update_skybox_uniform(world, settings);
        self.light.write(self.flight_index, &world.light());
        self.frag_settings.write(self.flight_index, frag_settings);
        self.postprocessing.write(self.flight_index, postprocessing);
        self.update_camera_uniform(world);
        self.submit_graphics();
        self.submit_present(image_index);

        self.flight_index = (self.flight_index + 1) % FRAMES_IN_FLIGHT;
    }

    unsafe fn prepare_command_buffer(&mut self, window_size: PhysicalSize<u32>) -> Option<usize> {
        let image_available = self.sync.image_available[self.flight_index];
        let in_flight = self.sync.in_flight[self.flight_index];

        self.dev
            .wait_for_fences(&[in_flight], true, u64::MAX)
            .unwrap();

        let acquire_result = self.swapchain_ext.acquire_next_image(
            self.swapchain.handle,
            u64::MAX,
            image_available,
            vk::Fence::null(),
        );
        if acquire_result == Err(vk::Result::ERROR_OUT_OF_DATE_KHR) {
            self.recreate_swapchain(window_size);
            return None;
        }
        let (image_index, _is_suboptimal) = acquire_result.unwrap();

        self.dev.reset_fences(&[in_flight]).unwrap();
        self.dev
            .reset_command_pool(
                self.command_pools[self.flight_index],
                vk::CommandPoolResetFlags::empty(),
            )
            .unwrap();

        Some(image_index as usize)
    }

    unsafe fn record_command_buffer(
        &mut self,
        image_index: usize,
        world: &World,
        ui_draw: &DrawData,
    ) {
        let buf = self.command_buffers[self.flight_index];

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        self.dev.begin_command_buffer(buf, &begin_info).unwrap();
        self.record_render_pass(buf, world);
        self.record_postprocess_pass(buf, image_index, ui_draw);
        self.dev.end_command_buffer(buf).unwrap();
    }

    unsafe fn record_render_pass(&self, buf: vk::CommandBuffer, world: &World) {
        let pass = self.render.begin();
        self.dev
            .cmd_begin_render_pass(buf, &pass, vk::SubpassContents::INLINE);

        self.dev.cmd_bind_pipeline(
            buf,
            vk::PipelineBindPoint::GRAPHICS,
            self.object_pipeline.pipeline,
        );
        for entity in world.entities() {
            let object = &self.objects[entity.gpu_object()];
            self.dev.cmd_bind_descriptor_sets(
                buf,
                vk::PipelineBindPoint::GRAPHICS,
                self.object_pipeline.layout,
                0,
                &[object.descriptor_sets[self.flight_index]],
                &[],
            );
            self.dev
                .cmd_bind_vertex_buffers(buf, 0, &[object.vertex.buffer], &[0]);

            self.dev
                .cmd_draw(buf, 3 * object.triangle_count as u32, 1, 0, 0);
        }

        self.dev.cmd_bind_pipeline(
            buf,
            vk::PipelineBindPoint::GRAPHICS,
            self.grass_pipeline.pipeline,
        );
        self.dev.cmd_bind_descriptor_sets(
            buf,
            vk::PipelineBindPoint::GRAPHICS,
            self.grass_pipeline.layout,
            0,
            &[self.grass_descriptor_sets[self.flight_index]],
            &[],
        );
        for grass_chunk in self.grass_chunks.lock().unwrap().iter() {
            self.dev.cmd_bind_vertex_buffers(
                buf,
                0,
                &[self.grass_vertex.buffer, grass_chunk.blades.buffer],
                &[0, 0],
            );
            self.dev.cmd_draw(
                buf,
                self.grass_vertex_count as u32,
                grass_chunk.blade_count as u32,
                0,
                0,
            );
        }

        self.dev.cmd_bind_pipeline(
            buf,
            vk::PipelineBindPoint::GRAPHICS,
            self.skybox_pipeline.pipeline,
        );
        self.dev.cmd_bind_descriptor_sets(
            buf,
            vk::PipelineBindPoint::GRAPHICS,
            self.skybox_pipeline.layout,
            0,
            &[self.skybox_descriptor_sets[self.flight_index]],
            &[],
        );
        self.dev
            .cmd_bind_vertex_buffers(buf, 0, &[self.objects[1].vertex.buffer], &[0]);
        self.dev
            .cmd_draw(buf, 3 * self.objects[1].triangle_count as u32, 1, 0, 0);

        self.dev.cmd_end_render_pass(buf);
    }

    unsafe fn record_postprocess_pass(
        &mut self,
        buf: vk::CommandBuffer,
        image_index: usize,
        ui_draw: &DrawData,
    ) {
        let pass = self.postprocess.begin_to_swapchain(image_index);
        self.dev
            .cmd_begin_render_pass(buf, &pass, vk::SubpassContents::INLINE);

        self.dev.cmd_bind_pipeline(
            buf,
            vk::PipelineBindPoint::GRAPHICS,
            self.postprocess_pipeline.pipeline,
        );
        self.dev.cmd_bind_descriptor_sets(
            buf,
            vk::PipelineBindPoint::GRAPHICS,
            self.postprocess_pipeline.layout,
            0,
            &[self.postprocess_descriptor_sets[self.flight_index]],
            &[],
        );
        self.dev.cmd_draw(buf, 6, 1, 0, 0);

        self.interface_renderer
            .as_mut()
            .unwrap()
            .cmd_draw(buf, ui_draw)
            .unwrap();

        self.dev.cmd_end_render_pass(buf);
    }

    fn update_object_uniforms(&self, world: &World, entity: &Entity, settings: &RendererSettings) {
        let mvp = ModelViewProjection {
            model: entity.model_matrix(world),
            view: world.view_matrix(),
            proj: self.projection_matrix(settings),
        };
        let material = Material {
            diffuse: entity.diffuse(),
            _pad0: 0.,
            emit: entity.emit(),
        };
        self.objects[entity.gpu_object()]
            .mvp
            .write(self.flight_index, &mvp);
        self.objects[entity.gpu_object()]
            .material
            .write(self.flight_index, &material);
    }

    fn update_grass_uniform(&self, grass: &Grass, world: &World, settings: &RendererSettings) {
        let mvp = ModelViewProjection {
            model: world.planet().model_matrix(world),
            view: world.view_matrix(),
            proj: self.projection_matrix(settings),
        };
        let grass = GrassUniform {
            height_average: grass.height_average,
            height_max_variance: grass.height_max_variance,
            width: grass.width,
            time: world.time,
            sway_direction: Vector3::new(0., 1., 0.),
            sway_frequency: grass.sway_frequency,
            sway_amplitude: grass.sway_amplitude,
        };
        self.grass_mvp.write(self.flight_index, &mvp);
        self.grass_uniform.write(self.flight_index, &grass);
    }

    fn update_skybox_uniform(&self, world: &World, settings: &RendererSettings) {
        let mvp = ModelViewProjection {
            model: Matrix4::new_scaling(8192.),
            view: world.view_matrix(),
            proj: self.projection_matrix(settings),
        };
        self.skybox_mvp.write(self.flight_index, &mvp);
    }

    fn update_camera_uniform(&self, world: &World) {
        let position = world.camera.position();
        self.camera.write(self.flight_index, &Camera { position });
    }

    fn submit_graphics(&self) {
        let command_buffer = self.command_buffers[self.flight_index];
        let image_available = self.sync.image_available[self.flight_index];
        let render_finished = self.sync.render_finished[self.flight_index];

        let wait_semaphores = [image_available];
        let command_buffers = [command_buffer];
        let signal_semaphores = [render_finished];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::TOP_OF_PIPE])
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);
        unsafe {
            self.dev.queue_submit(
                self.queue,
                &[*submit_info],
                self.sync.in_flight[self.flight_index],
            )
        }
        .unwrap();
    }

    fn submit_present(&self, image_index: usize) {
        let render_finished = self.sync.render_finished[self.flight_index];

        let wait_semaphores = [render_finished];
        let swapchains = [self.swapchain.handle];
        let image_indices = [image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);
        unsafe { self.swapchain_ext.queue_present(self.queue, &present_info) }.unwrap();
    }

    fn projection_matrix(&self, settings: &RendererSettings) -> Matrix4<f32> {
        let aspect_ratio = self.swapchain.extent.width as f32 / self.swapchain.extent.height as f32;
        let mut proj = Matrix4::new_perspective(
            aspect_ratio,
            FRAC_PI_4,
            settings.depth_near,
            settings.depth_far,
        );
        proj[(1, 1)] *= -1.;
        proj
    }
}
