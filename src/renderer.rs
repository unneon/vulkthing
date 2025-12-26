mod barrier;
pub mod codegen;
pub mod debug;
mod device;
pub mod lifecycle;
mod pass;
mod shader;
mod swapchain;
pub mod util;
pub mod vertex;

use crate::gpu::std140::{Atmosphere, Camera, Debug, Global, VoxelMaterial, Voxels};
use crate::gpu::std430::Star;
#[cfg(feature = "dev-menu")]
use crate::interface::EnumInterface;
use crate::renderer::codegen::{Passes, Pipelines, Samplers};
use crate::renderer::debug::{begin_label, end_label};
use crate::renderer::pass::Pass;
use crate::renderer::swapchain::Swapchain;
use crate::renderer::util::{
    timestamp_difference_to_duration, Buffer, Dev, ImageResources, StorageBuffer, UniformBuffer,
};
use crate::voxel::gpu::VoxelGpuMemory;
use crate::voxel::VoxelsConfig;
use crate::world::World;
use ash::{vk, Entry};
#[cfg(feature = "dev-menu")]
use imgui::DrawData;
use nalgebra::{Matrix4, Vector2, Vector3};
use std::f32::consts::FRAC_PI_4;
use std::sync::atomic::{AtomicU32, Ordering};
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
    properties: vk::PhysicalDeviceProperties,

    // Parameters of the renderer that are required early for creating more important objects.
    samplers: Samplers,

    // Description of the main render pass. Doesn't contain any information about the objects yet,
    // only low-level data format descriptions.
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    pipeline_layout: vk::PipelineLayout,
    passes: Passes,

    // All resources that depend on swapchain extent (window size). So swapchain description, memory
    // used for all framebuffer attachments, framebuffers, and the mentioned postprocess descriptor
    // set. Projection matrix depends on the monitor aspect ratio, so it's included too.
    pub swapchain: Swapchain,
    pipelines: Pipelines,
    depth: ImageResources,

    // Vulkan objects actually used for command recording and synchronization. Also internal
    // renderer state for keeping track of concurrent frames.
    command_pools: [vk::CommandPool; FRAMES_IN_FLIGHT],
    command_buffers: [vk::CommandBuffer; FRAMES_IN_FLIGHT],
    sync: Synchronization,
    flight_index: usize,

    // And finally resources specific to this renderer. So various buffers related to objects we
    // actually render, their descriptor sets and the like.
    mesh_objects: Vec<MeshObject>,
    stars: StorageBuffer<[Star]>,
    global: UniformBuffer<Global>,
    descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],

    voxel_meshlet_count: Arc<AtomicU32>,
    pub voxel_gpu_memory: Option<Box<dyn VoxelGpuMemory>>,

    query_pool: vk::QueryPool,
    frame_index: usize,
    pub frametime: Option<Duration>,
    pub just_completed_first_render: bool,

    #[cfg(feature = "dev-menu")]
    interface_renderer: Option<imgui_rs_vulkan_renderer::Renderer>,
}

struct Synchronization {
    image_available: [vk::Semaphore; FRAMES_IN_FLIGHT],
    render_finished: [vk::Semaphore; FRAMES_IN_FLIGHT],
    in_flight: [vk::Fence; FRAMES_IN_FLIGHT],
}

pub struct MeshObject {
    triangle_count: usize,
    vertex: Buffer,
    index: Buffer,
}

pub struct RendererSettings {
    pub voxel_rendering: VoxelRendering,
    pub atmosphere_in_scattering_samples: usize,
    pub atmosphere_optical_depth_samples: usize,
    pub atmosphere_wavelengths: Vector3<f32>,
    pub depth_near: f32,
    pub depth_far: f32,
    pub enable_atmosphere: bool,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum VoxelRendering {
    Classic,
    MeshShaders,
    RayTracing,
}

#[derive(Clone)]
pub struct DeviceSupport {
    mesh_shaders: bool,
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
#[allow(dead_code)]
const COLOR_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;

const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;

impl Renderer {
    pub fn draw_frame(
        &mut self,
        world: &World,
        voxels: &VoxelsConfig,
        settings: &RendererSettings,
        window_size: PhysicalSize<u32>,
        #[cfg(feature = "dev-menu")] ui_draw: &DrawData,
    ) {
        let Some(image_index) = (unsafe { self.prepare_command_buffer(window_size) }) else {
            return;
        };
        unsafe {
            self.record_command_buffer(
                image_index,
                world,
                settings,
                #[cfg(feature = "dev-menu")]
                ui_draw,
            )
        };
        self.frametime = self.query_timestamp();
        self.update_global_uniform(
            world,
            voxels,
            self.voxel_meshlet_count.load(Ordering::SeqCst),
            settings,
            window_size,
        );
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
        settings: &RendererSettings,
        #[cfg(feature = "dev-menu")] ui_draw: &DrawData,
    ) {
        let buf = self.command_buffers[self.flight_index];

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        self.dev.begin_command_buffer(buf, &begin_info).unwrap();
        self.reset_timestamps(buf);
        self.write_timestamp(buf, 0, vk::PipelineStageFlags::ALL_COMMANDS);
        self.record_render_pass(
            image_index,
            buf,
            world,
            settings,
            #[cfg(feature = "dev-menu")]
            ui_draw,
        );
        self.write_timestamp(buf, 1, vk::PipelineStageFlags::ALL_COMMANDS);
        self.dev.end_command_buffer(buf).unwrap();
    }

    unsafe fn record_render_pass(
        &mut self,
        image_index: usize,
        buf: vk::CommandBuffer,
        world: &World,
        settings: &RendererSettings,
        #[cfg(feature = "dev-menu")] ui_draw: &DrawData,
    ) {
        let color = &self.swapchain.images[image_index];
        let depth = &self.depth;

        self.barriers(
            buf,
            &[
                color.from_undefined().to_color_write(),
                depth.from_undefined().to_depth(),
            ],
        );

        self.passes
            .render
            .begin(buf, color, depth, self.swapchain.extent, &self.dev);

        self.bind_descriptor_set(buf);

        match settings.voxel_rendering {
            VoxelRendering::Classic => todo!(),
            VoxelRendering::MeshShaders => {
                let voxel_meshlet_count = self.voxel_meshlet_count.load(Ordering::SeqCst);
                begin_label(buf, "Voxel draws (mesh shaders)", [255, 0, 0], &self.dev);
                self.bind_graphics_pipeline(buf, self.pipelines.voxel);
                self.draw_mesh_shaders(buf, voxel_meshlet_count.div_ceil(64));
                end_label(buf, &self.dev);

                if voxel_meshlet_count > 0 {
                    begin_label(buf, "Debug voxel triangle draw", [238, 186, 11], &self.dev);
                    self.bind_graphics_pipeline(buf, self.pipelines.debug_voxel_triangle);
                    self.draw_mesh_shaders(buf, 1);
                    end_label(buf, &self.dev);

                    begin_label(buf, "Debug voxel world bound draw", [255, 78, 0], &self.dev);
                    self.bind_graphics_pipeline(buf, self.pipelines.debug_voxel_world_bound);
                    self.draw_mesh_shaders(buf, 1);
                    end_label(buf, &self.dev);

                    begin_label(buf, "Debug voxel screen bound draw", [113, 0, 0], &self.dev);
                    self.bind_graphics_pipeline(buf, self.pipelines.debug_voxel_screen_bound);
                    self.draw_mesh_shaders(buf, 1);
                    end_label(buf, &self.dev);
                }
            }
            VoxelRendering::RayTracing => {
                begin_label(buf, "Voxel draws (ray tracing)", [255, 0, 0], &self.dev);
                self.bind_graphics_pipeline(buf, self.pipelines.voxel_rt);
                unsafe { self.dev.cmd_draw(buf, 6, 1, 0, 0) };
                end_label(buf, &self.dev);
            }
        }

        begin_label(buf, "Sun draw", [156, 85, 35], &self.dev);
        self.bind_graphics_pipeline(buf, self.pipelines.sun);
        self.mesh_objects[1].bind_vertex(buf, &self.dev);
        self.mesh_objects[1].draw(1, buf, &self.dev);
        end_label(buf, &self.dev);

        begin_label(buf, "Star draws", [213, 204, 184], &self.dev);
        self.bind_graphics_pipeline(buf, self.pipelines.star);
        self.mesh_objects[0].bind_vertex(buf, &self.dev);
        self.mesh_objects[0].draw(world.stars.len(), buf, &self.dev);
        end_label(buf, &self.dev);

        begin_label(buf, "Skybox draw", [129, 147, 164], &self.dev);
        self.bind_graphics_pipeline(buf, self.pipelines.skybox);
        unsafe { self.dev.cmd_draw(buf, 6, 1, 0, 0) };
        end_label(buf, &self.dev);

        #[cfg(feature = "dev-menu")]
        {
            // TODO: Fix drawing SRGB interface to linear color space.
            begin_label(buf, "Debugging interface draw", [63, 70, 73], &self.dev);
            self.interface_renderer
                .as_mut()
                .unwrap()
                .cmd_draw(buf, ui_draw)
                .unwrap();
            end_label(buf, &self.dev);
        }

        self.passes.render.end(buf, &self.dev);

        self.barriers(buf, &[color.from_color_write().to_present()]);
    }

    fn update_global_uniform(
        &self,
        world: &World,
        voxels: &VoxelsConfig,
        voxel_meshlet_count: u32,
        settings: &RendererSettings,
        window_size: PhysicalSize<u32>,
    ) {
        let mut materials = [VoxelMaterial {
            albedo: Vector3::zeros(),
            roughness: 0.,
            emit: Vector3::zeros(),
            metallic: 0.,
        }; 256];
        materials[1] = VoxelMaterial {
            albedo: Vector3::new(0.55, 0.6, 0.66),
            roughness: 1.,
            emit: Vector3::zeros(),
            metallic: 0.,
        };
        materials[2] = VoxelMaterial {
            albedo: Vector3::new(0.62, 0.4, 0.24),
            roughness: 1.,
            emit: Vector3::zeros(),
            metallic: 0.,
        };
        materials[3] = VoxelMaterial {
            albedo: Vector3::new(0.63, 0.81, 0.42),
            roughness: 1.,
            emit: Vector3::zeros(),
            metallic: 0.,
        };
        self.global.write(
            self.flight_index,
            &Global {
                voxels: Voxels {
                    chunk_size: voxels.chunk_size as u32,
                    meshlet_count: voxel_meshlet_count,
                    root_svo_index: 0,
                    root_svo_side: 64,
                    root_svo_base: Vector3::zeros(),
                },
                light: world.light(),
                atmosphere: Atmosphere {
                    enable: if settings.enable_atmosphere { 1 } else { 0 },
                    scatter_point_count: settings.atmosphere_in_scattering_samples as u32,
                    optical_depth_point_count: settings.atmosphere_optical_depth_samples as u32,
                    density_falloff: world.atmosphere.density_falloff,
                    planet_position: Vector3::new(
                        world.camera.position().x,
                        world.camera.position().y,
                        -world.atmosphere.planet_radius,
                    ),
                    planet_radius: world.atmosphere.planet_radius,
                    sun_position: world.sun().transform.translation,
                    scale: world.atmosphere.scale,
                    wavelengths: settings.atmosphere_wavelengths,
                    scattering_strength: world.atmosphere.scattering_strength,
                    henyey_greenstein_g: world.atmosphere.henyey_greenstein_g,
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
                    depth_near: settings.depth_near,
                    depth_far: settings.depth_far,
                    position: world.camera.position(),
                    _pad0: 0.,
                    direction: world.camera.view_direction(),
                },
                materials,
                debug: Debug { meshlet_id: 0 },
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
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::FRAGMENT_SHADER])
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);
        unsafe {
            self.dev.queue_submit(
                self.queue,
                &[submit_info],
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
        let present_info = vk::PresentInfoKHR::default()
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

    pub fn wait_idle(&self) {
        unsafe {
            self.dev.device_wait_idle().unwrap();
        };
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

    fn bind_descriptor_set(&self, buf: vk::CommandBuffer) {
        unsafe {
            self.dev.cmd_bind_descriptor_sets(
                buf,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.descriptor_sets[self.flight_index]],
                &[],
            )
        };
    }

    fn draw_mesh_shaders(&self, buf: vk::CommandBuffer, count: u32) {
        unsafe { self.dev.mesh_ext.cmd_draw_mesh_tasks(buf, count, 1, 1) };
    }

    fn barriers(&self, buf: vk::CommandBuffer, barriers: &[vk::ImageMemoryBarrier2]) {
        let dependency_info = vk::DependencyInfo::default().image_memory_barriers(barriers);
        unsafe { self.dev.cmd_pipeline_barrier2(buf, &dependency_info) }
    }

    fn reset_timestamps(&self, buf: vk::CommandBuffer) {
        unsafe {
            self.dev
                .cmd_reset_query_pool(buf, self.query_pool, (2 * self.flight_index) as u32, 2)
        };
    }

    fn write_timestamp(&self, buf: vk::CommandBuffer, index: usize, stage: vk::PipelineStageFlags) {
        unsafe {
            self.dev.cmd_write_timestamp(
                buf,
                stage,
                self.query_pool,
                (2 * self.flight_index + index) as u32,
            )
        };
    }

    fn query_timestamp(&self) -> Option<Duration> {
        // CPU can't wait for current frame metrics because it has to prepare command buffers for
        // the next frame, the query results are delayed by FRAMES_IN_FLIGHT frames.
        if self.frame_index < FRAMES_IN_FLIGHT {
            return None;
        }

        let mut timestamps = [0; 2];
        unsafe {
            self.dev.get_query_pool_results(
                self.query_pool,
                (2 * self.flight_index) as u32,
                &mut timestamps,
                vk::QueryResultFlags::TYPE_64,
            )
        }
        .unwrap();

        Some(timestamp_difference_to_duration(
            timestamps[1] - timestamps[0],
            &self.properties,
        ))
    }
}

impl MeshObject {
    fn bind_vertex(&self, buf: vk::CommandBuffer, dev: &Dev) {
        unsafe { dev.cmd_bind_vertex_buffers(buf, 0, &[self.vertex.buffer], &[0]) };
        unsafe { dev.cmd_bind_index_buffer(buf, self.index.buffer, 0, vk::IndexType::UINT32) };
    }

    fn draw(&self, instance_count: usize, buf: vk::CommandBuffer, dev: &Dev) {
        unsafe {
            dev.cmd_draw_indexed(
                buf,
                3 * self.triangle_count as u32,
                instance_count as u32,
                0,
                0,
                0,
            )
        };
    }
}

#[cfg(feature = "dev-menu")]
impl EnumInterface for VoxelRendering {
    const VALUES: &'static [Self] = &[
        VoxelRendering::Classic,
        VoxelRendering::MeshShaders,
        VoxelRendering::RayTracing,
    ];

    fn label(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed(match self {
            VoxelRendering::Classic => "Classic",
            VoxelRendering::MeshShaders => "Mesh shaders",
            VoxelRendering::RayTracing => "Ray tracing",
        })
    }
}
