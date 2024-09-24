use crate::renderer::debug::{begin_label, end_label};
use crate::renderer::util::{Dev, ImageResources};
use ash::vk;

pub struct Pass {
    pub debug_name: &'static str,
    pub debug_color: [u8; 3],
}

impl Pass {
    pub fn begin(
        &self,
        buf: vk::CommandBuffer,
        color: &ImageResources,
        depth: &ImageResources,
        extent: vk::Extent2D,
        dev: &Dev,
    ) {
        begin_label(buf, self.debug_name, self.debug_color, dev);

        let color_attachment_info = vk::RenderingAttachmentInfo::default()
            .image_view(color.view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0., 0., 0., 0.],
                },
            });
        let depth_attachment_info = vk::RenderingAttachmentInfo::default()
            .image_view(depth.view)
            .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .clear_value(vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue::default().depth(1.),
            });
        let rendering_info = vk::RenderingInfo::default()
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            })
            .color_attachments(std::array::from_ref(&color_attachment_info))
            .layer_count(1)
            .depth_attachment(&depth_attachment_info);
        unsafe { dev.cmd_begin_rendering(buf, &rendering_info) };
    }

    pub fn end(&self, buf: vk::CommandBuffer, dev: &Dev) {
        unsafe { dev.cmd_end_rendering(buf) };
        end_label(buf, dev);
    }
}
