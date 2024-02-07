pub mod codegen;
pub mod debug;
mod device;
mod graph;
pub mod lifecycle;
mod raytracing;
mod shader;
mod swapchain;
pub mod uniform;
pub mod util;
pub mod vertex;

use crate::renderer::codegen::{
    DescriptorPools, DescriptorSetLayouts, Passes, PipelineLayouts, Pipelines, Samplers, PASS_COUNT,
};
use crate::renderer::debug::{begin_label, end_label};
use crate::renderer::graph::Pass;
use crate::renderer::raytracing::RaytraceResources;
use crate::renderer::swapchain::Swapchain;
use crate::renderer::uniform::{
    Atmosphere, Camera, Gaussian, Global, Material, PostprocessUniform, Tonemapper, Transform,
};
use crate::renderer::util::{timestamp_difference_to_duration, Buffer, Dev, UniformBuffer};
use crate::world::World;
use ash::{vk, Entry};
use imgui::DrawData;
use nalgebra::{Matrix4, Vector2, Vector3};
use std::f32::consts::FRAC_PI_4;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use winit::dpi::PhysicalSize;

pub struct Renderer {
    // Immutable parts of the renderer. These can't change in the current design, but recovering
    // from GPU crashes might require doing something with these later?
    _entry: Entry,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    surface: vk::SurfaceKHR,
    pub dev: Dev,
    queue: vk::Queue,
    supports_raytracing: bool,
    properties: vk::PhysicalDeviceProperties,

    // Parameters of the renderer that are required early for creating more important objects.
    samplers: Samplers,

    // Description of the main render pass. Doesn't contain any information about the objects yet,
    // only low-level data format descriptions.
    descriptor_set_layouts: DescriptorSetLayouts,
    descriptor_pools: DescriptorPools,
    pipeline_layouts: PipelineLayouts,
    passes: Passes,

    // All resources that depend on swapchain extent (window size). So swapchain description, memory
    // used for all framebuffer attachments, framebuffers, and the mentioned postprocess descriptor
    // set. Projection matrix depends on the monitor aspect ratio, so it's included too.
    pub swapchain: Swapchain,
    pipelines: Pipelines,
    atmosphere_descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    extract_descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    gaussian_horizontal_descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    gaussian_vertical_descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    postprocess_descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],

    // Vulkan objects actually used for command recording and synchronization. Also internal
    // renderer state for keeping track of concurrent frames.
    command_pools: [vk::CommandPool; FRAMES_IN_FLIGHT],
    command_buffers: [vk::CommandBuffer; FRAMES_IN_FLIGHT],
    sync: Synchronization,
    flight_index: usize,

    // And finally resources specific to this renderer. So various buffers related to objects we
    // actually render, their descriptor sets and the like.
    mesh_objects: Vec<MeshObject>,
    entities: Vec<Object>,
    star_transform: UniformBuffer<Transform>,
    star_material: UniformBuffer<Material>,
    star_instances: Buffer,
    star_descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    skybox_transform: UniformBuffer<Transform>,
    skybox_material: UniformBuffer<Material>,
    skybox_descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    global: UniformBuffer<Global>,
    global_descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],

    tlas: Option<RaytraceResources>,
    blas: Option<RaytraceResources>,

    query_pool: vk::QueryPool,
    frame_index: usize,
    pub pass_times: Option<[Duration; PASS_COUNT]>,
    pub just_completed_first_render: bool,

    interface_renderer: Option<imgui_rs_vulkan_renderer::Renderer>,

    pub voxel_chunks: Option<Arc<AtomicU64>>,
    voxel_transform: UniformBuffer<Transform>,
    voxel_material: UniformBuffer<Material>,
    voxel_descriptor_set: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    pub voxel_buffer: Buffer,
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
    transform: UniformBuffer<Transform>,
    material: UniformBuffer<Material>,
    descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
}

pub struct RendererSettings {
    pub atmosphere_in_scattering_samples: usize,
    pub atmosphere_optical_depth_samples: usize,
    pub atmosphere_wavelengths: Vector3<f32>,
    pub depth_near: f32,
    pub depth_far: f32,
    pub enable_atmosphere: bool,
    pub enable_ray_tracing: bool,
    pub postprocess: PostprocessSettings,
}

pub struct PostprocessSettings {
    pub color_filter: Vector3<f32>,
    pub bloom_exponent_coefficient: f32,
    pub bloom_radius: usize,
    pub bloom_strength: f32,
    pub bloom_threshold: f32,
    pub exposure: f32,
    pub temperature: f32,
    pub tint: f32,
    pub contrast: f32,
    pub brightness: f32,
    pub saturation: f32,
    pub tonemapper: Tonemapper,
    pub gamma: f32,
}

pub const VRAM_VIA_BAR: vk::MemoryPropertyFlags = vk::MemoryPropertyFlags::from_raw(
    vk::MemoryPropertyFlags::DEVICE_LOCAL.as_raw()
        | vk::MemoryPropertyFlags::HOST_VISIBLE.as_raw()
        | vk::MemoryPropertyFlags::HOST_COHERENT.as_raw(),
);

pub const FRAMES_IN_FLIGHT: usize = 2;

// Format used for passing HDR data between render passes to enable realistic differences in
// lighting parameters and improve postprocessing effect quality, not related to monitor HDR.
// Support for this format is required by the Vulkan specification.
const COLOR_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;

const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;

impl Renderer {
    pub fn draw_frame(
        &mut self,
        world: &World,
        settings: &RendererSettings,
        window_size: PhysicalSize<u32>,
        ui_draw: &DrawData,
    ) {
        let Some(image_index) = (unsafe { self.prepare_command_buffer(window_size) }) else {
            return;
        };
        unsafe { self.record_command_buffer(image_index, world, ui_draw) };
        self.pass_times = self.query_timestamps();
        self.update_voxel_uniform(world, settings);
        for entity_id in 0..world.entities().len() {
            self.update_object_uniforms(world, entity_id, settings);
        }
        self.update_star_uniform(world, settings);
        self.update_skybox_uniform(world, settings);
        self.update_global_uniform(world, settings, window_size);
        self.submit_graphics();
        self.submit_present(image_index);

        self.flight_index = (self.flight_index + 1) % FRAMES_IN_FLIGHT;
        self.frame_index += 1;
    }

    unsafe fn prepare_command_buffer(&mut self, window_size: PhysicalSize<u32>) -> Option<usize> {
        let image_available = self.sync.image_available[self.flight_index];
        let in_flight = self.sync.in_flight[self.flight_index];

        self.dev
            .wait_for_fences(&[in_flight], true, u64::MAX)
            .unwrap();

        self.just_completed_first_render = self.frame_index == FRAMES_IN_FLIGHT;

        let acquire_result = self.dev.swapchain_ext.acquire_next_image(
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
        self.reset_timestamps(buf);
        self.record_render_pass(buf, world);
        self.record_atmosphere_pass(buf);
        self.record_extract_pass(buf);
        self.record_gaussian_passes(buf);
        self.record_postprocess_pass(buf, image_index, ui_draw);
        self.write_timestamp(buf, PASS_COUNT, vk::PipelineStageFlags::ALL_COMMANDS);
        self.dev.end_command_buffer(buf).unwrap();
    }

    unsafe fn record_render_pass(&self, buf: vk::CommandBuffer, world: &World) {
        self.passes
            .render
            .begin(buf, &self.dev, self.query_pool, self.flight_index);

        if let Some(voxel_chunks) = self.voxel_chunks.as_ref() {
            let vertex_count = voxel_chunks.load(Ordering::SeqCst) as u32;

            begin_label(buf, "Voxel draws", [255, 0, 0], &self.dev);
            self.bind_graphics_pipeline(buf, self.pipelines.object);
            self.bind_descriptor_sets(
                buf,
                self.pipeline_layouts.object,
                &self.voxel_descriptor_set,
            );
            unsafe {
                self.dev
                    .cmd_bind_vertex_buffers(buf, 0, &[self.voxel_buffer.buffer], &[0])
            };
            unsafe { self.dev.cmd_draw(buf, vertex_count, 1, 0, 0) };
            end_label(buf, &self.dev);
        }

        begin_label(buf, "Entity draws", [57, 65, 62], &self.dev);
        self.bind_graphics_pipeline(buf, self.pipelines.object);
        for (entity, gpu_entity) in world.entities().iter().zip(&self.entities) {
            let mesh = &self.mesh_objects[entity.mesh_id()];
            self.bind_descriptor_sets(buf, self.pipeline_layouts.object, &gpu_entity.descriptors);
            mesh.bind_vertex(buf, &self.dev);
            mesh.draw(1, buf, &self.dev);
        }
        end_label(buf, &self.dev);

        begin_label(buf, "Star draws", [213, 204, 184], &self.dev);
        self.bind_graphics_pipeline(buf, self.pipelines.star);
        self.bind_descriptor_sets(buf, self.pipeline_layouts.star, &self.star_descriptor_sets);
        self.mesh_objects[2].bind_vertex_instanced(&self.star_instances, buf, &self.dev);
        self.mesh_objects[2].draw(world.stars.len(), buf, &self.dev);
        end_label(buf, &self.dev);

        begin_label(buf, "Skybox draw", [129, 147, 164], &self.dev);
        self.bind_graphics_pipeline(buf, self.pipelines.skybox);
        self.bind_descriptor_sets(
            buf,
            self.pipeline_layouts.skybox,
            &self.skybox_descriptor_sets,
        );
        self.mesh_objects[1].bind_vertex(buf, &self.dev);
        self.mesh_objects[1].draw(1, buf, &self.dev);
        end_label(buf, &self.dev);

        self.dev.cmd_end_render_pass(buf);
        end_label(buf, &self.dev);
    }

    unsafe fn record_atmosphere_pass(&mut self, buf: vk::CommandBuffer) {
        self.passes
            .atmosphere
            .begin(buf, &self.dev, self.query_pool, self.flight_index);

        self.bind_graphics_pipeline(buf, self.pipelines.atmosphere);
        self.bind_descriptor_sets(
            buf,
            self.pipeline_layouts.atmosphere,
            &self.atmosphere_descriptors,
        );
        self.dev.cmd_draw(buf, 6, 1, 0, 0);

        self.dev.cmd_end_render_pass(buf);
        end_label(buf, &self.dev);
    }

    unsafe fn record_extract_pass(&mut self, buf: vk::CommandBuffer) {
        self.passes
            .extract
            .begin(buf, &self.dev, self.query_pool, self.flight_index);

        self.bind_graphics_pipeline(buf, self.pipelines.extract);
        self.bind_descriptor_sets(
            buf,
            self.pipeline_layouts.extract,
            &self.extract_descriptors,
        );
        self.dev.cmd_draw(buf, 6, 1, 0, 0);

        self.dev.cmd_end_render_pass(buf);
        end_label(buf, &self.dev);
    }

    unsafe fn record_gaussian_passes(&mut self, buf: vk::CommandBuffer) {
        self.passes
            .gaussian_horizontal
            .begin(buf, &self.dev, self.query_pool, self.flight_index);

        self.bind_graphics_pipeline(buf, self.pipelines.gaussian_horizontal);
        self.bind_descriptor_sets(
            buf,
            self.pipeline_layouts.gaussian_horizontal,
            &self.gaussian_horizontal_descriptors,
        );
        self.dev.cmd_draw(buf, 6, 1, 0, 0);

        self.dev.cmd_end_render_pass(buf);
        end_label(buf, &self.dev);

        self.passes
            .gaussian_vertical
            .begin(buf, &self.dev, self.query_pool, self.flight_index);

        self.bind_graphics_pipeline(buf, self.pipelines.gaussian_vertical);
        self.bind_descriptor_sets(
            buf,
            self.pipeline_layouts.gaussian_vertical,
            &self.gaussian_vertical_descriptors,
        );
        self.dev.cmd_draw(buf, 6, 1, 0, 0);

        self.dev.cmd_end_render_pass(buf);
        end_label(buf, &self.dev);
    }

    unsafe fn record_postprocess_pass(
        &mut self,
        buf: vk::CommandBuffer,
        image_index: usize,
        ui_draw: &DrawData,
    ) {
        self.passes.postprocess.begin_to_swapchain(
            buf,
            image_index,
            &self.dev,
            self.query_pool,
            self.flight_index,
        );

        begin_label(buf, "Postprocess draw", [210, 206, 203], &self.dev);
        self.bind_graphics_pipeline(buf, self.pipelines.postprocess);
        self.bind_descriptor_sets(
            buf,
            self.pipeline_layouts.postprocess,
            &self.postprocess_descriptor_sets,
        );
        self.dev.cmd_draw(buf, 6, 1, 0, 0);
        end_label(buf, &self.dev);

        // TODO: Fix drawing SRGB interface to linear color space.
        begin_label(buf, "Debugging interface draw", [63, 70, 73], &self.dev);
        self.interface_renderer
            .as_mut()
            .unwrap()
            .cmd_draw(buf, ui_draw)
            .unwrap();
        end_label(buf, &self.dev);

        self.dev.cmd_end_render_pass(buf);
        end_label(buf, &self.dev);
    }

    fn update_voxel_uniform(&self, world: &World, settings: &RendererSettings) {
        let transform = Transform {
            model: Matrix4::identity(),
            view: world.view_matrix(),
            proj: self.projection_matrix(settings),
        };
        let material = Material {
            emit: Vector3::from_element(0.01),
            metallic: 0.,
            ao: 0.,
            roughness: 1.,
            albedo: Vector3::from_element(0.9),
        };
        self.voxel_material.write(self.flight_index, &material);
        self.voxel_transform.write(self.flight_index, &transform);
    }

    fn update_object_uniforms(&self, world: &World, entity_id: usize, settings: &RendererSettings) {
        let entity = &world.entities()[entity_id];
        let transform = Transform {
            model: entity.model_matrix(),
            view: world.view_matrix(),
            proj: self.projection_matrix(settings),
        };
        let material = Material {
            albedo: entity.albedo(),
            metallic: entity.metallic(),
            emit: entity.emit(),
            roughness: entity.roughness(),
            ao: entity.ao(),
        };
        self.entities[entity_id]
            .transform
            .write(self.flight_index, &transform);
        self.entities[entity_id]
            .material
            .write(self.flight_index, &material);
    }

    fn update_star_uniform(&self, world: &World, settings: &RendererSettings) {
        let transform = Transform {
            model: Matrix4::identity(),
            view: world.view_matrix(),
            proj: self.projection_matrix(settings),
        };
        self.star_transform.write(self.flight_index, &transform);
    }

    fn update_skybox_uniform(&self, world: &World, settings: &RendererSettings) {
        let transform = Transform {
            model: Matrix4::new_scaling(32000.),
            view: world.view_matrix(),
            proj: self.projection_matrix(settings),
        };
        self.skybox_transform.write(self.flight_index, &transform);
    }

    fn update_global_uniform(
        &self,
        world: &World,
        settings: &RendererSettings,
        window_size: PhysicalSize<u32>,
    ) {
        self.global.write(
            self.flight_index,
            &Global {
                light: world.light(),
                settings: uniform::Settings {
                    use_ray_tracing: settings.enable_ray_tracing,
                    _pad0: [0; 3],
                },
                atmosphere: Atmosphere {
                    enable: settings.enable_atmosphere,
                    _pad0: [0; 3],
                    scatter_point_count: settings.atmosphere_in_scattering_samples as u32,
                    optical_depth_point_count: settings.atmosphere_optical_depth_samples as u32,
                    density_falloff: world.atmosphere.density_falloff,
                    // TODO
                    planet_position: Vector3::zeros(),
                    planet_radius: 0.,
                    sun_position: world.sun().transform.translation,
                    scale: world.atmosphere.scale,
                    wavelengths: settings.atmosphere_wavelengths,
                    scattering_strength: world.atmosphere.scattering_strength,
                    henyey_greenstein_g: world.atmosphere.henyey_greenstein_g,
                },
                gaussian: Gaussian {
                    threshold: settings.postprocess.bloom_threshold,
                    radius: settings.postprocess.bloom_radius as i32,
                    exponent_coefficient: settings.postprocess.bloom_exponent_coefficient,
                },
                postprocessing: PostprocessUniform {
                    color_filter: settings.postprocess.color_filter,
                    bloom_constant: settings.postprocess.bloom_strength,
                    exposure: settings.postprocess.exposure,
                    temperature: settings.postprocess.temperature,
                    tint: settings.postprocess.tint,
                    contrast: settings.postprocess.contrast,
                    brightness: settings.postprocess.brightness,
                    saturation: settings.postprocess.saturation,
                    tonemapper: settings.postprocess.tonemapper,
                    gamma: settings.postprocess.gamma,
                },
                camera: Camera {
                    view_matrix: world.view_matrix(),
                    projection_matrix: self.projection_matrix(settings),
                    inverse_view_matrix: world.view_matrix().try_inverse().unwrap(),
                    inverse_projection_matrix: self
                        .projection_matrix(settings)
                        .try_inverse()
                        .unwrap(),
                    resolution: Vector2::new(window_size.width as f32, window_size.height as f32),
                    _pad0: [0., 0.],
                    position: world.camera.position(),
                },
            },
        );
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
        unsafe {
            self.dev
                .swapchain_ext
                .queue_present(self.queue, &present_info)
        }
        .unwrap();
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

    fn bind_graphics_pipeline(&self, buf: vk::CommandBuffer, pipeline: vk::Pipeline) {
        unsafe {
            self.dev
                .cmd_bind_pipeline(buf, vk::PipelineBindPoint::GRAPHICS, pipeline)
        };
    }

    #[allow(dead_code)]
    fn bind_compute_pipeline(&self, buf: vk::CommandBuffer, pipeline: vk::Pipeline) {
        unsafe {
            self.dev
                .cmd_bind_pipeline(buf, vk::PipelineBindPoint::COMPUTE, pipeline)
        };
    }

    fn bind_descriptor_sets(
        &self,
        buf: vk::CommandBuffer,
        layout: vk::PipelineLayout,
        sets: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],
    ) {
        unsafe {
            self.dev.cmd_bind_descriptor_sets(
                buf,
                vk::PipelineBindPoint::GRAPHICS,
                layout,
                0,
                &[
                    sets[self.flight_index],
                    self.global_descriptor_sets[self.flight_index],
                ],
                &[],
            )
        };
    }

    fn reset_timestamps(&self, buf: vk::CommandBuffer) {
        unsafe {
            self.dev.cmd_reset_query_pool(
                buf,
                self.query_pool,
                (self.flight_index * (PASS_COUNT + 1)) as u32,
                (PASS_COUNT + 1) as u32,
            )
        };
    }

    fn write_timestamp(&self, buf: vk::CommandBuffer, index: usize, stage: vk::PipelineStageFlags) {
        unsafe {
            self.dev.cmd_write_timestamp(
                buf,
                stage,
                self.query_pool,
                (self.flight_index * (PASS_COUNT + 1) + index) as u32,
            )
        };
    }

    fn query_timestamps(&self) -> Option<[Duration; PASS_COUNT]> {
        // CPU can't wait for current frame metrics because it has to prepare command buffers for
        // the next frame, the query results are delayed by FRAMES_IN_FLIGHT frames.
        if self.frame_index < FRAMES_IN_FLIGHT {
            return None;
        }

        let mut timestamps = [0; PASS_COUNT + 1];
        unsafe {
            self.dev.get_query_pool_results(
                self.query_pool,
                (self.flight_index * (PASS_COUNT + 1)) as u32,
                (PASS_COUNT + 1) as u32,
                &mut timestamps,
                vk::QueryResultFlags::TYPE_64,
            )
        }
        .unwrap();

        let mut pass_times = [Duration::ZERO; PASS_COUNT];
        for i in 0..PASS_COUNT {
            pass_times[i] = timestamp_difference_to_duration(
                timestamps[i + 1] - timestamps[i],
                &self.properties,
            );
        }
        Some(pass_times)
    }
}

impl MeshObject {
    fn bind_vertex(&self, buf: vk::CommandBuffer, dev: &Dev) {
        unsafe { dev.cmd_bind_vertex_buffers(buf, 0, &[self.vertex.buffer], &[0]) };
    }

    fn bind_vertex_instanced(&self, instances: &Buffer, buf: vk::CommandBuffer, dev: &Dev) {
        unsafe {
            dev.cmd_bind_vertex_buffers(buf, 0, &[self.vertex.buffer, instances.buffer], &[0, 0])
        };
    }

    fn draw(&self, instance_count: usize, buf: vk::CommandBuffer, dev: &Dev) {
        unsafe {
            dev.cmd_draw(
                buf,
                3 * self.triangle_count as u32,
                instance_count as u32,
                0,
                0,
            )
        };
    }
}
