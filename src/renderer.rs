mod debug;
mod device;
mod lifecycle;
mod pipeline;
mod shader;
mod traits;
mod uniform;
mod util;
pub mod vertex;

use crate::renderer::uniform::{Filters, Light, Material, ModelViewProjection};
use crate::renderer::util::UniformBuffer;
use crate::world::{Entity, World};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::{vk, Device, Entry, Instance};
use imgui::{Condition, Drag};
use nalgebra::{Matrix4, Vector3};
use winit::dpi::PhysicalSize;

pub struct Renderer {
    // Immutable parts of the renderer. These can't change in the current design, but recovering
    // from GPU crashes might require doing something with these later?
    _entry: Entry,
    instance: Instance,
    extensions: VulkanExtensions,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    logical_device: Device,
    queue: vk::Queue,
    swapchain_extension: Swapchain,

    // Parameters of the renderer that are required early for creating more important objects.
    msaa_samples: vk::SampleCountFlags,
    offscreen_sampler: vk::Sampler,
    filters: UniformBuffer<Filters>,

    // Description of the main render pass. Doesn't contain any information about the objects yet,
    // only low-level data format descriptions.
    object_descriptor_set_layout: vk::DescriptorSetLayout,
    render_pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
    render_pipeline: vk::Pipeline,

    // Description of the postprocessing pass, and also the actual descriptor pool. Necessary,
    // because the postprocessing pass depends on swapchain extent and needs to have the descriptor
    // set updated after window resize.
    postprocess_descriptor_set_layout: vk::DescriptorSetLayout,
    postprocess_pipeline_layout: vk::PipelineLayout,
    postprocess_pass: vk::RenderPass,
    postprocess_pipeline: vk::Pipeline,
    postprocess_descriptor_pool: vk::DescriptorPool,

    // All resources that depend on swapchain extent (window size). So swapchain description, memory
    // used for all framebuffer attachments, framebuffers, and the mentioned postprocess descriptor
    // set. Projection matrix depends on the monitor aspect ratio, so it's included too.
    swapchain_extent: vk::Extent2D,
    swapchain: vk::SwapchainKHR,
    swapchain_image_views: Vec<vk::ImageView>,
    color: ImageResources,
    depth: ImageResources,
    offscreen: ImageResources,
    render_framebuffer: vk::Framebuffer,
    postprocess_framebuffers: Vec<vk::Framebuffer>,
    postprocess_descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    projection: Matrix4<f32>,

    // Vulkan objects actually used for command recording and synchronization. Also internal
    // renderer state for keeping track of concurrent frames.
    command_pools: [vk::CommandPool; FRAMES_IN_FLIGHT],
    command_buffers: [vk::CommandBuffer; FRAMES_IN_FLIGHT],
    sync: Synchronization,
    flight_index: usize,

    // And finally resources specific to this renderer. So various buffers related to objects we
    // actually render, their descriptor sets and the like.
    light: UniformBuffer<Light>,
    object_descriptor_pool: vk::DescriptorPool,
    objects: Vec<Object>,
    noise_texture: ImageResources,
    noise_sampler: vk::Sampler,

    pub imgui: imgui::Context,
    imgui_renderer: Option<imgui_rs_vulkan_renderer::Renderer>,
}

struct VulkanExtensions {
    debug: DebugUtils,
    surface: Surface,
}

struct ImageResources {
    image: vk::Image,
    memory: vk::DeviceMemory,
    view: vk::ImageView,
}

struct Synchronization {
    image_available: [vk::Semaphore; FRAMES_IN_FLIGHT],
    render_finished: [vk::Semaphore; FRAMES_IN_FLIGHT],
    in_flight: [vk::Fence; FRAMES_IN_FLIGHT],
}

struct Object {
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    index_count: usize,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    mvp: UniformBuffer<ModelViewProjection>,
    texture: ImageResources,
    texture_sampler: vk::Sampler,
    material: UniformBuffer<Material>,
    descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
}

const FRAMES_IN_FLIGHT: usize = 2;

impl Renderer {
    pub fn draw_frame(&mut self, world: &mut World, window_size: PhysicalSize<u32>) {
        let Some(image_index) = (unsafe { self.prepare_command_buffer(window_size) }) else {
            return;
        };
        self.build_ui(world);
        unsafe { self.record_command_buffer(image_index, world) };
        for entity in &world.entities {
            self.update_object_uniforms(world, entity);
        }
        self.update_light_uniform(world);
        self.submit_graphics();
        self.submit_present(image_index);

        self.flight_index = (self.flight_index + 1) % FRAMES_IN_FLIGHT;
    }

    fn build_ui(&mut self, world: &mut World) {
        let filters = self.filters.deref(self.flight_index);
        let mut color_filter = [
            filters.color_filter.x,
            filters.color_filter.y,
            filters.color_filter.z,
        ];
        let ui = self.imgui.frame();
        ui.window("World")
            .size([256., 128.], Condition::Always)
            .position([50., 50.], Condition::Once)
            .build(|| {
                ui.checkbox("Pause light movement", &mut world.light_pause);
            });
        ui.window("Postprocessing")
            .size([384., 256.], Condition::Always)
            .position([50., 300.], Condition::Once)
            .build(|| {
                Drag::new("Exposure")
                    .range(0., f32::INFINITY)
                    .speed(0.01)
                    .build(ui, &mut filters.exposure);
                ui.slider("Temperature", -1.67, 1.67, &mut filters.temperature);
                ui.slider("Tint", -1.67, 1.67, &mut filters.tint);
                Drag::new("Contrast")
                    .range(0., f32::INFINITY)
                    .speed(0.01)
                    .build(ui, &mut filters.contrast);
                Drag::new("Brightness")
                    .range(0., f32::INFINITY)
                    .speed(0.01)
                    .build(ui, &mut filters.brightness);
                ui.color_edit3("Color filter", &mut color_filter);
                Drag::new("Saturation")
                    .range(0., f32::INFINITY)
                    .speed(0.01)
                    .build(ui, &mut filters.saturation);
                Drag::new("Gamma")
                    .range(0., f32::INFINITY)
                    .speed(0.01)
                    .build(ui, &mut filters.gamma);
            });
        filters.color_filter = Vector3::new(color_filter[0], color_filter[1], color_filter[2]);
    }

    unsafe fn prepare_command_buffer(&mut self, window_size: PhysicalSize<u32>) -> Option<u32> {
        let dev = &self.logical_device;
        let image_available = self.sync.image_available[self.flight_index];
        let in_flight = self.sync.in_flight[self.flight_index];

        dev.wait_for_fences(&[in_flight], true, u64::MAX).unwrap();

        let acquire_result = self.swapchain_extension.acquire_next_image(
            self.swapchain,
            u64::MAX,
            image_available,
            vk::Fence::null(),
        );
        if acquire_result == Err(vk::Result::ERROR_OUT_OF_DATE_KHR) {
            self.recreate_swapchain(window_size);
            return None;
        }
        let (image_index, _is_suboptimal) = acquire_result.unwrap();

        dev.reset_fences(&[in_flight]).unwrap();
        dev.reset_command_pool(
            self.command_pools[self.flight_index],
            vk::CommandPoolResetFlags::empty(),
        )
        .unwrap();

        Some(image_index)
    }

    unsafe fn record_command_buffer(&mut self, image_index: u32, world: &World) {
        let buf = self.command_buffers[self.flight_index];

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        self.logical_device
            .begin_command_buffer(buf, &begin_info)
            .unwrap();
        self.record_render_pass(buf, world);
        self.record_postprocess_pass(buf, image_index);
        self.logical_device.end_command_buffer(buf).unwrap();
    }

    unsafe fn record_render_pass(&self, buf: vk::CommandBuffer, world: &World) {
        let dev = &self.logical_device;
        let pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.render_framebuffer)
            // I don't quite understand when render area should be anything else. It seems like
            // scissor already offers the same functionality?
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            })
            .clear_values(&[
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0., 0., 0., 0.],
                    },
                },
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.,
                        stencil: 0,
                    },
                },
            ]);
        dev.cmd_begin_render_pass(buf, &pass_info, vk::SubpassContents::INLINE);

        dev.cmd_bind_pipeline(buf, vk::PipelineBindPoint::GRAPHICS, self.render_pipeline);

        for entity in &world.entities {
            let object = &self.objects[entity.gpu_object];
            dev.cmd_bind_descriptor_sets(
                buf,
                vk::PipelineBindPoint::GRAPHICS,
                self.render_pipeline_layout,
                0,
                &[object.descriptor_sets[self.flight_index]],
                &[],
            );
            dev.cmd_bind_vertex_buffers(buf, 0, &[object.vertex_buffer], &[0]);
            dev.cmd_bind_index_buffer(buf, object.index_buffer, 0, vk::IndexType::UINT32);
            dev.cmd_draw_indexed(buf, object.index_count as u32, 1, 0, 0, 0);
        }

        dev.cmd_end_render_pass(buf);
    }

    unsafe fn record_postprocess_pass(&mut self, buf: vk::CommandBuffer, image_index: u32) {
        let dev = &self.logical_device;
        let pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.postprocess_pass)
            .framebuffer(self.postprocess_framebuffers[image_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            });
        dev.cmd_begin_render_pass(buf, &pass_info, vk::SubpassContents::INLINE);

        dev.cmd_bind_pipeline(
            buf,
            vk::PipelineBindPoint::GRAPHICS,
            self.postprocess_pipeline,
        );

        dev.cmd_bind_descriptor_sets(
            buf,
            vk::PipelineBindPoint::GRAPHICS,
            self.postprocess_pipeline_layout,
            0,
            &[self.postprocess_descriptor_sets[self.flight_index]],
            &[],
        );
        dev.cmd_draw(buf, 6, 1, 0, 0);

        self.imgui_renderer
            .as_mut()
            .unwrap()
            .cmd_draw(buf, &self.imgui.render())
            .unwrap();

        dev.cmd_end_render_pass(buf);
    }

    fn update_object_uniforms(&self, world: &World, entity: &Entity) {
        let model = Matrix4::identity()
            .prepend_translation(&entity.position)
            .prepend_nonuniform_scaling(&entity.scale);
        let mvp = ModelViewProjection {
            model,
            view: world.camera.view_matrix(),
            proj: self.projection,
        };
        let material = Material { emit: entity.emit };
        self.objects[entity.gpu_object]
            .mvp
            .write(self.flight_index, mvp);
        self.objects[entity.gpu_object]
            .material
            .write(self.flight_index, material);
    }

    fn update_light_uniform(&self, world: &World) {
        let light = Light {
            color: world.light.color,
            position: world.light.position,
            ambient_strength: world.light.ambient_strength,
        };
        self.light.write(self.flight_index, light);
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
            self.logical_device.queue_submit(
                self.queue,
                &[*submit_info],
                self.sync.in_flight[self.flight_index],
            )
        }
        .unwrap();
    }

    fn submit_present(&self, image_index: u32) {
        let render_finished = self.sync.render_finished[self.flight_index];

        let wait_semaphores = [render_finished];
        let swapchains = [self.swapchain];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);
        unsafe {
            self.swapchain_extension
                .queue_present(self.queue, &present_info)
        }
        .unwrap();
    }
}
