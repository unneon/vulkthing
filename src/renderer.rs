mod debug;
mod device;
pub mod gpu_data;
mod init;
mod shader;
mod traits;
mod util;

use crate::camera::Camera;
use crate::model::Model;
use crate::renderer::device::QueueFamilies;
use crate::renderer::gpu_data::UniformBufferObject;
use ash::extensions::khr::Swapchain;
use ash::{vk, Device, Entry, Instance};
use nalgebra_glm as glm;
use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
pub struct Renderer {
    entry: Entry,
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
    texture_mipmaps: usize,
    texture_sampler: vk::Sampler,
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    uniform_buffers: [vk::Buffer; FRAMES_IN_FLIGHT],
    uniform_buffer_memories: [vk::DeviceMemory; FRAMES_IN_FLIGHT],
    uniform_buffer_mapped: [*mut UniformBufferObject; FRAMES_IN_FLIGHT],
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    sync: Synchronization,
    frame_flight_index: usize,
}

struct Synchronization {
    image_available: [vk::Semaphore; FRAMES_IN_FLIGHT],
    render_finished: [vk::Semaphore; FRAMES_IN_FLIGHT],
    in_flight: [vk::Fence; FRAMES_IN_FLIGHT],
}

const FRAMES_IN_FLIGHT: usize = 2;

impl Renderer {
    pub fn draw_frame(&mut self, model: &Model, camera: &Camera) {
        draw_frame(
            &self.logical_device,
            self.sync.in_flight[self.frame_flight_index],
            self.swapchain,
            &self.swapchain_extension,
            self.swapchain_extent,
            self.sync.image_available[self.frame_flight_index],
            self.command_buffers[self.frame_flight_index],
            &self.framebuffers,
            self.pipeline,
            self.pipeline_render_pass,
            self.pipeline_layout,
            self.sync.render_finished[self.frame_flight_index],
            self.vertex_buffer,
            self.index_buffer,
            model.indices.len(),
            self.uniform_buffer_mapped[self.frame_flight_index],
            self.descriptor_sets[self.frame_flight_index],
            &camera,
            self.queues.graphics,
            self.queues.present,
        );
        self.frame_flight_index = (self.frame_flight_index + 1) % FRAMES_IN_FLIGHT;
    }
}

fn draw_frame(
    device: &Device,
    in_flight_fence: vk::Fence,
    swapchain: vk::SwapchainKHR,
    swapchain_extension: &Swapchain,
    swapchain_extent: vk::Extent2D,
    image_available_semaphore: vk::Semaphore,
    command_buffer: vk::CommandBuffer,
    framebuffers: &[vk::Framebuffer],
    pipeline: vk::Pipeline,
    pipeline_render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    render_finished_semaphore: vk::Semaphore,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    index_count: usize,
    ubo_ptr: *mut UniformBufferObject,
    descriptor_set: vk::DescriptorSet,
    camera: &Camera,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
) {
    unsafe { device.wait_for_fences(&[in_flight_fence], true, u64::MAX) }.unwrap();
    unsafe { device.reset_fences(&[in_flight_fence]) }.unwrap();
    // What is the second value?
    let image_index = unsafe {
        swapchain_extension.acquire_next_image(
            swapchain,
            u64::MAX,
            image_available_semaphore,
            vk::Fence::null(),
        )
    }
    .unwrap()
    .0;
    unsafe { device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty()) }
        .unwrap();
    record_command_buffer(
        device,
        command_buffer,
        image_index,
        framebuffers,
        swapchain_extent,
        pipeline,
        pipeline_render_pass,
        pipeline_layout,
        vertex_buffer,
        index_buffer,
        index_count,
        descriptor_set,
    );

    update_uniform_buffer(
        ubo_ptr,
        swapchain_extent.width as f32 / swapchain_extent.height as f32,
        camera,
    );

    let wait_semaphores = [image_available_semaphore];
    let command_buffers = [command_buffer];
    let signal_semaphores = [render_finished_semaphore];
    let submit_info = vk::SubmitInfo::builder()
        .wait_semaphores(&wait_semaphores)
        .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
        .command_buffers(&command_buffers)
        .signal_semaphores(&signal_semaphores);
    unsafe { device.queue_submit(graphics_queue, &[*submit_info], in_flight_fence) }.unwrap();

    let present_info_swapchains = [swapchain];
    let present_info_images = [image_index];
    let present_info = vk::PresentInfoKHR::builder()
        .wait_semaphores(&signal_semaphores)
        .swapchains(&present_info_swapchains)
        .image_indices(&present_info_images);
    unsafe { swapchain_extension.queue_present(present_queue, &present_info) }.unwrap();
}

fn record_command_buffer(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    image_index: u32,
    framebuffers: &[vk::Framebuffer],
    swapchain_extent: vk::Extent2D,
    pipeline: vk::Pipeline,
    pipeline_render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    index_count: usize,
    descriptor_set: vk::DescriptorSet,
) {
    let begin_info = vk::CommandBufferBeginInfo::builder();
    unsafe { device.begin_command_buffer(command_buffer, &begin_info) }.unwrap();

    let render_pass_info = vk::RenderPassBeginInfo::builder()
        .render_pass(pipeline_render_pass)
        .framebuffer(framebuffers[image_index as usize])
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
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
    unsafe {
        device.cmd_begin_render_pass(
            command_buffer,
            &render_pass_info,
            vk::SubpassContents::INLINE,
        )
    };

    unsafe { device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline) };

    let buffers = [vertex_buffer];
    let offsets = [0];
    unsafe { device.cmd_bind_vertex_buffers(command_buffer, 0, &buffers, &offsets) };

    unsafe { device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32) };

    let viewport = vk::Viewport {
        x: 0.,
        y: 0.,
        width: swapchain_extent.width as f32,
        height: swapchain_extent.height as f32,
        min_depth: 0.,
        max_depth: 1.,
    };
    unsafe { device.cmd_set_viewport(command_buffer, 0, &[viewport]) };

    let scissor = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: swapchain_extent,
    };
    unsafe { device.cmd_set_scissor(command_buffer, 0, &[scissor]) };

    unsafe {
        device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline_layout,
            0,
            &[descriptor_set],
            &[],
        )
    };

    unsafe { device.cmd_draw_indexed(command_buffer, index_count as u32, 1, 0, 0, 0) };

    unsafe { device.cmd_end_render_pass(command_buffer) };

    unsafe { device.end_command_buffer(command_buffer) }.unwrap();
}

fn update_uniform_buffer(ubo_ptr: *mut UniformBufferObject, aspect_ratio: f32, camera: &Camera) {
    let mut ubo = UniformBufferObject {
        model: glm::identity(),
        view: camera.view_matrix(),
        proj: glm::perspective_rh_zo(aspect_ratio, FRAC_PI_4, 0.1, 10.),
    };
    ubo.proj[(1, 1)] *= -1.;
    unsafe { ubo_ptr.write_volatile(ubo) };
}
