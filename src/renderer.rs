mod debug;
mod device;
pub mod gpu_data;
mod lifecycle;
mod shader;
mod traits;
mod util;

use crate::camera::Camera;
use crate::renderer::device::QueueFamilies;
use crate::renderer::gpu_data::UniformBufferObject;
use ash::extensions::khr::Swapchain;
use ash::{vk, Device, Entry, Instance};
use nalgebra_glm as glm;
use std::f32::consts::FRAC_PI_4;
use winit::dpi::PhysicalSize;

pub struct Renderer {
    _entry: Entry,
    instance: Instance,
    extensions: util::VulkanExtensions,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    queue_families: QueueFamilies,
    surface_capabilities: vk::SurfaceCapabilitiesKHR,
    surface_formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
    logical_device: Device,
    queues: util::Queues,
    swapchain_extension: Swapchain,
    swapchain_image_count: usize,
    swapchain_format: vk::SurfaceFormatKHR,
    swapchain_extent: vk::Extent2D,
    swapchain: vk::SwapchainKHR,
    swapchain_image_views: Vec<vk::ImageView>,
    descriptor_set_layout: vk::DescriptorSetLayout,
    msaa_samples: vk::SampleCountFlags,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    pipeline_render_pass: vk::RenderPass,
    command_pool: vk::CommandPool,
    command_buffers: [vk::CommandBuffer; FRAMES_IN_FLIGHT],
    color: util::ImageResources,
    depth: util::ImageResources,
    framebuffers: Vec<vk::Framebuffer>,
    texture: util::ImageResources,
    texture_sampler: vk::Sampler,
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    vertex_count: usize,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    light_vb: vk::Buffer,
    light_vbm: vk::DeviceMemory,
    light_vc: usize,
    light_ib: vk::Buffer,
    light_ibm: vk::DeviceMemory,
    uniform_buffers: [vk::Buffer; FRAMES_IN_FLIGHT],
    uniform_buffer_memories: [vk::DeviceMemory; FRAMES_IN_FLIGHT],
    uniform_buffer_mapped: [*mut UniformBufferObject; FRAMES_IN_FLIGHT],
    light_ub: [vk::Buffer; FRAMES_IN_FLIGHT],
    light_ubm: [vk::DeviceMemory; FRAMES_IN_FLIGHT],
    light_ubp: [*mut UniformBufferObject; FRAMES_IN_FLIGHT],
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    light_ds: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    sync: Synchronization,
    flight_index: usize,
}

struct Synchronization {
    image_available: [vk::Semaphore; FRAMES_IN_FLIGHT],
    render_finished: [vk::Semaphore; FRAMES_IN_FLIGHT],
    in_flight: [vk::Fence; FRAMES_IN_FLIGHT],
}

const FRAMES_IN_FLIGHT: usize = 2;

impl Renderer {
    pub fn draw_frame(&mut self, camera: &Camera, window_size: PhysicalSize<u32>, timestamp: f32) {
        let Some(image_index) = (unsafe { self.prepare_command_buffer(window_size) }) else {
            return;
        };
        unsafe { self.record_command_buffer(image_index) };
        self.update_uniform_buffer(camera);
        self.update_light_ub(camera, timestamp);
        self.submit_graphics();
        self.submit_present(image_index);

        self.flight_index = (self.flight_index + 1) % FRAMES_IN_FLIGHT;
    }

    unsafe fn prepare_command_buffer(&mut self, window_size: PhysicalSize<u32>) -> Option<u32> {
        let dev = &self.logical_device;
        let command_buffer = self.command_buffers[self.flight_index];
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
        dev.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
            .unwrap();

        Some(image_index)
    }

    unsafe fn record_command_buffer(&self, image_index: u32) {
        let dev = &self.logical_device;
        let buf = self.command_buffers[self.flight_index];

        dev.begin_command_buffer(buf, &vk::CommandBufferBeginInfo::builder())
            .unwrap();

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.pipeline_render_pass)
            .framebuffer(self.framebuffers[image_index as usize])
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
        dev.cmd_begin_render_pass(buf, &render_pass_info, vk::SubpassContents::INLINE);

        dev.cmd_bind_pipeline(buf, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

        let viewport = vk::Viewport {
            x: 0.,
            y: 0.,
            width: self.swapchain_extent.width as f32,
            height: self.swapchain_extent.height as f32,
            min_depth: 0.,
            max_depth: 1.,
        };
        dev.cmd_set_viewport(buf, 0, &[viewport]);

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain_extent,
        };
        dev.cmd_set_scissor(buf, 0, &[scissor]);

        dev.cmd_bind_descriptor_sets(
            buf,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline_layout,
            0,
            &[self.descriptor_sets[self.flight_index]],
            &[],
        );
        dev.cmd_bind_vertex_buffers(buf, 0, &[self.vertex_buffer], &[0]);
        dev.cmd_bind_index_buffer(buf, self.index_buffer, 0, vk::IndexType::UINT32);
        dev.cmd_draw_indexed(buf, self.vertex_count as u32, 1, 0, 0, 0);

        dev.cmd_bind_descriptor_sets(
            buf,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline_layout,
            0,
            &[self.light_ds[self.flight_index]],
            &[],
        );
        dev.cmd_bind_vertex_buffers(buf, 0, &[self.light_vb], &[0]);
        dev.cmd_bind_index_buffer(buf, self.light_ib, 0, vk::IndexType::UINT32);
        dev.cmd_draw_indexed(buf, self.light_vc as u32, 1, 0, 0, 0);

        dev.cmd_end_render_pass(buf);

        dev.end_command_buffer(buf).unwrap();
    }

    fn update_uniform_buffer(&self, camera: &Camera) {
        let aspect_ratio = self.swapchain_extent.width as f32 / self.swapchain_extent.height as f32;
        let mut ubo = UniformBufferObject {
            model: glm::identity(),
            view: camera.view_matrix(),
            proj: glm::perspective_rh_zo(aspect_ratio, FRAC_PI_4, 0.1, 100.),
        };
        ubo.proj[(1, 1)] *= -1.;
        unsafe { self.uniform_buffer_mapped[self.flight_index].write_volatile(ubo) };
    }

    fn update_light_ub(&self, camera: &Camera, timestamp: f32) {
        let aspect_ratio = self.swapchain_extent.width as f32 / self.swapchain_extent.height as f32;
        let model = glm::identity();
        let model = glm::rotate_z(&model, timestamp * 2.);
        let model = glm::translate(&model, &glm::vec3(-4., 0., 2.));
        let model = glm::scale(&model, &glm::vec3(0.2, 0.2, 0.2));
        let mut ubo = UniformBufferObject {
            model,
            view: camera.view_matrix(),
            proj: glm::perspective_rh_zo(aspect_ratio, FRAC_PI_4, 0.1, 100.),
        };
        ubo.proj[(1, 1)] *= -1.;
        unsafe { self.light_ubp[self.flight_index].write_volatile(ubo) };
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
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);
        unsafe {
            self.logical_device.queue_submit(
                self.queues.graphics,
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
                .queue_present(self.queues.present, &present_info)
        }
        .unwrap();
    }
}
