use crate::renderer::codegen::PASS_COUNT;
use crate::renderer::debug::begin_label;
use crate::renderer::util::{Dev, ImageResources};
use ash::vk;

pub struct Pass {
    pub debug_name: &'static str,
    pub debug_color: [u8; 3],
    pub pass: vk::RenderPass,
    pub extent: vk::Extent2D,
    pub clears: Vec<vk::ClearValue>,
    pub resources: Vec<ImageResources>,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub direct_to_swapchain: bool,
    pub index: usize,
}

impl Pass {
    pub fn cleanup(&self, dev: &Dev) {
        unsafe {
            for framebuffer in &self.framebuffers {
                dev.destroy_framebuffer(*framebuffer, None);
            }
            for resource in &self.resources {
                resource.cleanup(dev);
            }
            dev.destroy_render_pass(self.pass, None);
        }
    }

    #[allow(unused)]
    pub fn begin(
        &self,
        buf: vk::CommandBuffer,
        dev: &Dev,
        query_pool: vk::QueryPool,
        flight_index: usize,
    ) {
        assert!(!self.direct_to_swapchain);
        self.generic_begin(buf, self.framebuffers[0], dev, query_pool, flight_index);
    }

    pub fn begin_to_swapchain(
        &self,
        buf: vk::CommandBuffer,
        image_index: usize,
        dev: &Dev,
        query_pool: vk::QueryPool,
        flight_index: usize,
    ) {
        assert!(self.direct_to_swapchain);
        self.generic_begin(
            buf,
            self.framebuffers[image_index],
            dev,
            query_pool,
            flight_index,
        );
    }

    fn generic_begin(
        &self,
        buf: vk::CommandBuffer,
        framebuffer: vk::Framebuffer,
        dev: &Dev,
        query_pool: vk::QueryPool,
        flight_index: usize,
    ) {
        unsafe {
            dev.cmd_write_timestamp(
                buf,
                vk::PipelineStageFlags::ALL_COMMANDS,
                query_pool,
                (flight_index * (PASS_COUNT + 1) + self.index) as u32,
            )
        };
        let info = *vk::RenderPassBeginInfo::builder()
            .render_pass(self.pass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.extent,
            })
            .clear_values(&self.clears);
        begin_label(buf, self.debug_name, self.debug_color, dev);
        unsafe { dev.cmd_begin_render_pass(buf, &info, vk::SubpassContents::INLINE) };
    }
}
