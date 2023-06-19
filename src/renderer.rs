mod codegen;
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
use crate::renderer::codegen::{DescriptorSetLayouts, Samplers};
use crate::renderer::debug::{begin_label, end_label};
use crate::renderer::descriptors::DescriptorMetadata;
use crate::renderer::graph::Pass;
use crate::renderer::pipeline::Pipeline;
use crate::renderer::raytracing::RaytraceResources;
use crate::renderer::swapchain::Swapchain;
use crate::renderer::uniform::{
    Atmosphere, Camera, FragSettings, Gaussian, GrassUniform, Light, Material, ModelViewProjection,
    Postprocessing,
};
use crate::renderer::util::{Buffer, Dev, UniformBuffer};
use crate::world::World;
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
    swapchain_ext: SwapchainKhr,
    supports_raytracing: bool,

    // Parameters of the renderer that are required early for creating more important objects.
    pub msaa_samples: vk::SampleCountFlags,
    samplers: Samplers,
    atmosphere_uniform: UniformBuffer<Atmosphere>,
    gaussian_uniform: UniformBuffer<Gaussian>,
    postprocessing: UniformBuffer<Postprocessing>,
    camera: UniformBuffer<Camera>,

    // Description of the main render pass. Doesn't contain any information about the objects yet,
    // only low-level data format descriptions.
    descriptor_set_layouts: DescriptorSetLayouts,
    object_descriptor_metadata: DescriptorMetadata,
    grass_descriptor_metadata: DescriptorMetadata,
    skybox_descriptor_metadata: DescriptorMetadata,
    render: Pass,

    atmosphere_descriptor_metadata: DescriptorMetadata,

    gaussian_descriptor_metadata: DescriptorMetadata,
    gaussian: Pass,

    // Description of the postprocessing pass, and also the actual descriptor pool. Necessary,
    // because the postprocessing pass depends on swapchain extent and needs to have the descriptor
    // set updated after window resize.
    postprocess_descriptor_metadata: DescriptorMetadata,
    postprocess: Pass,

    // All resources that depend on swapchain extent (window size). So swapchain description, memory
    // used for all framebuffer attachments, framebuffers, and the mentioned postprocess descriptor
    // set. Projection matrix depends on the monitor aspect ratio, so it's included too.
    pub swapchain: Swapchain,
    pipelines: Pipelines,
    atmosphere_descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    gaussian_descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    postprocess_descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],

    // Vulkan objects actually used for command recording and synchronization. Also internal
    // renderer state for keeping track of concurrent frames.
    command_pools: [vk::CommandPool; FRAMES_IN_FLIGHT],
    command_buffers: [vk::CommandBuffer; FRAMES_IN_FLIGHT],
    sync: Synchronization,
    flight_index: usize,

    // And finally resources specific to this renderer. So various buffers related to objects we
    // actually render, their descriptor sets and the like.
    grass_mvp: UniformBuffer<ModelViewProjection>,
    grass_uniform: UniformBuffer<GrassUniform>,
    light: UniformBuffer<Light>,
    frag_settings: UniformBuffer<FragSettings>,
    mesh_objects: Vec<MeshObject>,
    entities: Vec<Object>,
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

struct Pipelines {
    object: Pipeline,
    grass: Pipeline,
    skybox: Pipeline,
    atmosphere: Pipeline,
    gaussian: Pipeline,
    postprocess: Pipeline,
}

struct Synchronization {
    image_available: [vk::Semaphore; FRAMES_IN_FLIGHT],
    render_finished: [vk::Semaphore; FRAMES_IN_FLIGHT],
    in_flight: [vk::Fence; FRAMES_IN_FLIGHT],
}

pub struct MeshObject {
    triangle_count: usize,
    vertex: Buffer,
}

pub struct Object {
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
    debug_ext: DebugUtils,
    grass_chunks: Arc<Mutex<Vec<GrassChunk>>>,
    grass_blades_total: Arc<AtomicUsize>,
}

pub struct RendererSettings {
    pub depth_near: f32,
    pub depth_far: f32,
    pub msaa_samples: vk::SampleCountFlags,
}

const FRAMES_IN_FLIGHT: usize = 8;

impl Renderer {
    pub fn draw_frame(
        &mut self,
        world: &World,
        grass: &Grass,
        settings: &RendererSettings,
        frag_settings: &FragSettings,
        atmosphere: &Atmosphere,
        gaussian: &Gaussian,
        postprocessing: &Postprocessing,
        window_size: PhysicalSize<u32>,
        ui_draw: &DrawData,
    ) {
        let Some(image_index) = (unsafe { self.prepare_command_buffer(window_size) }) else {
            return;
        };
        unsafe { self.record_command_buffer(image_index, world, ui_draw) };
        for entity_id in 0..world.entities().len() {
            self.update_object_uniforms(world, entity_id, settings);
        }
        self.update_grass_uniform(grass, world, settings);
        self.update_skybox_uniform(world, settings);
        self.light.write(self.flight_index, &world.light());
        self.frag_settings.write(self.flight_index, frag_settings);
        self.atmosphere_uniform.write(self.flight_index, atmosphere);
        self.gaussian_uniform.write(self.flight_index, gaussian);
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
        self.record_gaussian_pass(buf);
        self.record_postprocess_pass(buf, image_index, ui_draw);
        self.dev.end_command_buffer(buf).unwrap();
    }

    unsafe fn record_render_pass(&self, buf: vk::CommandBuffer, world: &World) {
        self.render.begin(buf, &self.dev, &self.extensions.debug);

        begin_label(buf, "Entity draws", [57, 65, 62], &self.extensions.debug);
        self.bind_pipeline(buf, &self.pipelines.object);
        for (entity, gpu_entity) in world.entities().iter().zip(&self.entities) {
            let mesh = &self.mesh_objects[entity.mesh_id()];
            self.bind_descriptor_sets(buf, &self.pipelines.object, &gpu_entity.descriptor_sets);
            self.dev
                .cmd_bind_vertex_buffers(buf, 0, &[mesh.vertex.buffer], &[0]);
            self.dev
                .cmd_draw(buf, 3 * mesh.triangle_count as u32, 1, 0, 0);
        }
        end_label(buf, &self.extensions.debug);

        begin_label(buf, "Grass draws", [100, 142, 55], &self.extensions.debug);
        self.bind_pipeline(buf, &self.pipelines.grass);
        self.bind_descriptor_sets(buf, &self.pipelines.grass, &self.grass_descriptor_sets);
        for grass_chunk in self.grass_chunks.lock().unwrap().iter() {
            self.dev.cmd_bind_vertex_buffers(
                buf,
                0,
                &[
                    self.mesh_objects[3].vertex.buffer,
                    grass_chunk.blades.buffer,
                ],
                &[0, 0],
            );
            self.dev.cmd_draw(
                buf,
                3 * self.mesh_objects[3].triangle_count as u32,
                grass_chunk.blade_count as u32,
                0,
                0,
            );
        }
        end_label(buf, &self.extensions.debug);

        begin_label(buf, "Skybox draw", [129, 147, 164], &self.extensions.debug);
        self.bind_pipeline(buf, &self.pipelines.skybox);
        self.bind_descriptor_sets(buf, &self.pipelines.skybox, &self.skybox_descriptor_sets);
        self.dev
            .cmd_bind_vertex_buffers(buf, 0, &[self.mesh_objects[1].vertex.buffer], &[0]);
        self.dev
            .cmd_draw(buf, 3 * self.mesh_objects[1].triangle_count as u32, 1, 0, 0);
        end_label(buf, &self.extensions.debug);

        self.dev.cmd_next_subpass(buf, vk::SubpassContents::INLINE);

        begin_label(
            buf,
            "Atmosphere draw",
            [84, 115, 144],
            &self.extensions.debug,
        );
        self.bind_pipeline(buf, &self.pipelines.atmosphere);
        self.bind_descriptor_sets(
            buf,
            &self.pipelines.atmosphere,
            &self.atmosphere_descriptor_sets,
        );
        self.dev.cmd_draw(buf, 6, 1, 0, 0);
        end_label(buf, &self.extensions.debug);

        self.dev.cmd_end_render_pass(buf);
        end_label(buf, &self.extensions.debug);
    }

    unsafe fn record_gaussian_pass(&mut self, buf: vk::CommandBuffer) {
        self.gaussian.begin(buf, &self.dev, &self.extensions.debug);

        self.bind_pipeline(buf, &self.pipelines.gaussian);
        self.bind_descriptor_sets(
            buf,
            &self.pipelines.gaussian,
            &self.gaussian_descriptor_sets,
        );
        self.dev.cmd_draw(buf, 6, 1, 0, 0);

        self.dev.cmd_end_render_pass(buf);
        end_label(buf, &self.extensions.debug);
    }

    unsafe fn record_postprocess_pass(
        &mut self,
        buf: vk::CommandBuffer,
        image_index: usize,
        ui_draw: &DrawData,
    ) {
        self.postprocess
            .begin_to_swapchain(buf, image_index, &self.dev, &self.extensions.debug);

        begin_label(
            buf,
            "Postprocess draw",
            [210, 206, 203],
            &self.extensions.debug,
        );
        self.bind_pipeline(buf, &self.pipelines.postprocess);
        self.bind_descriptor_sets(
            buf,
            &self.pipelines.postprocess,
            &self.postprocess_descriptor_sets,
        );
        self.dev.cmd_draw(buf, 6, 1, 0, 0);
        end_label(buf, &self.extensions.debug);

        begin_label(
            buf,
            "Debugging interface draw",
            [63, 70, 73],
            &self.extensions.debug,
        );
        self.interface_renderer
            .as_mut()
            .unwrap()
            .cmd_draw(buf, ui_draw)
            .unwrap();
        end_label(buf, &self.extensions.debug);

        self.dev.cmd_end_render_pass(buf);
        end_label(buf, &self.extensions.debug);
    }

    fn update_object_uniforms(&self, world: &World, entity_id: usize, settings: &RendererSettings) {
        let entity = &world.entities()[entity_id];
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
        self.entities[entity_id].mvp.write(self.flight_index, &mvp);
        self.entities[entity_id]
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
            model: Matrix4::new_scaling(32000.),
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
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::FRAGMENT_SHADER])
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

    fn bind_pipeline(&self, buf: vk::CommandBuffer, pipeline: &Pipeline) {
        unsafe {
            self.dev
                .cmd_bind_pipeline(buf, vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline)
        };
    }

    fn bind_descriptor_sets(
        &self,
        buf: vk::CommandBuffer,
        pipeline: &Pipeline,
        sets: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],
    ) {
        unsafe {
            self.dev.cmd_bind_descriptor_sets(
                buf,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.layout,
                0,
                &[sets[self.flight_index]],
                &[],
            )
        };
    }
}
