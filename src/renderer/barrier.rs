#![allow(clippy::wrong_self_convention)]

use crate::renderer::util::ImageResources;
use ash::vk;

pub struct ImageBarrier(vk::ImageMemoryBarrier2<'static>);

impl ImageResources {
    pub fn from_undefined(&self) -> ImageBarrier {
        ImageBarrier(
            vk::ImageMemoryBarrier2::default()
                .src_stage_mask(vk::PipelineStageFlags2::TOP_OF_PIPE)
                .src_access_mask(vk::AccessFlags2::empty())
                .old_layout(vk::ImageLayout::UNDEFINED)
                .image(self.image)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .level_count(1)
                        .layer_count(1),
                ),
        )
    }

    pub fn from_color_write(&self) -> ImageBarrier {
        ImageBarrier(
            vk::ImageMemoryBarrier2::default()
                .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
                .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .image(self.image)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .level_count(1)
                        .layer_count(1),
                ),
        )
    }
}

impl ImageBarrier {
    pub fn to_color_write(self) -> vk::ImageMemoryBarrier2<'static> {
        self.0
            .dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
            .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
    }

    pub fn to_depth(mut self) -> vk::ImageMemoryBarrier2<'static> {
        self.0.subresource_range.aspect_mask = vk::ImageAspectFlags::DEPTH;
        self.0
            .dst_stage_mask(
                vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
            )
            .new_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
    }

    pub fn to_present(self) -> vk::ImageMemoryBarrier2<'static> {
        self.0
            .dst_stage_mask(vk::PipelineStageFlags2::BOTTOM_OF_PIPE)
            .dst_access_mask(vk::AccessFlags2::empty())
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
    }
}
