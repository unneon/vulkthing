use crate::renderer::util::Dev;
use ash::vk;

pub struct Pass {
    pub pass: vk::RenderPass,
    extent: vk::Extent2D,
    clears: Vec<vk::ClearValue>,
}

pub struct AttachmentConfig {
    format: vk::Format,
    samples: vk::SampleCountFlags,
    clear: Option<vk::ClearValue>,
    layout: vk::ImageLayout,
    final_layout: Option<vk::ImageLayout>,
    resolve: bool,
}

impl Pass {
    pub fn cleanup(&self, dev: &Dev) {
        unsafe { dev.destroy_render_pass(self.pass, None) };
    }

    pub fn begin(&self, framebuffer: vk::Framebuffer) -> vk::RenderPassBeginInfo {
        *vk::RenderPassBeginInfo::builder()
            .render_pass(self.pass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.extent,
            })
            .clear_values(&self.clears)
    }
}

impl AttachmentConfig {
    pub fn new(format: vk::Format) -> AttachmentConfig {
        AttachmentConfig {
            format,
            samples: vk::SampleCountFlags::TYPE_1,
            clear: None,
            layout: vk::ImageLayout::UNDEFINED,
            final_layout: None,
            resolve: false,
        }
    }

    pub fn samples(mut self, samples: vk::SampleCountFlags) -> Self {
        self.samples = samples;
        self
    }

    pub fn clear_color(mut self, color: [f32; 4]) -> Self {
        self.clear = Some(vk::ClearValue {
            color: vk::ClearColorValue { float32: color },
        });
        self
    }

    pub fn clear_depth(mut self, depth: f32) -> Self {
        self.clear = Some(vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue { depth, stencil: 0 },
        });
        self
    }

    pub fn layout(mut self, layout: vk::ImageLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn store(mut self, final_layout: vk::ImageLayout) -> Self {
        self.final_layout = Some(final_layout);
        self
    }

    pub fn resolve(mut self) -> Self {
        self.resolve = true;
        self
    }
}

pub fn create_pass(extent: vk::Extent2D, dev: &Dev, configs: &[AttachmentConfig]) -> Pass {
    let mut attachments = Vec::new();
    let mut color = None;
    let mut depth = None;
    let mut resolve = None;
    let mut clears = Vec::new();
    for (index, config) in configs.iter().enumerate() {
        let attachment = *vk::AttachmentDescription::builder()
            .format(config.format)
            .samples(config.samples)
            .load_op(if config.clear.is_some() {
                vk::AttachmentLoadOp::CLEAR
            } else {
                vk::AttachmentLoadOp::DONT_CARE
            })
            .store_op(if config.final_layout.is_some() {
                vk::AttachmentStoreOp::STORE
            } else {
                vk::AttachmentStoreOp::DONT_CARE
            })
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(config.final_layout.unwrap_or(config.layout));
        let reference = *vk::AttachmentReference::builder()
            .attachment(index as u32)
            .layout(config.layout);
        attachments.push(attachment);
        if config.layout == vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            && !config.resolve
            && color.is_none()
        {
            color = Some(reference);
        } else if config.layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            && depth.is_none()
        {
            depth = Some(reference);
        } else if config.layout == vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            && config.resolve
            && resolve.is_none()
        {
            resolve = Some(reference);
        } else {
            panic!(
                "unimplemented case {:?},{:?} {:?},{:?},{:?}",
                config.format,
                config.resolve,
                color.is_none(),
                depth.is_none(),
                resolve.is_none()
            );
        }
        if let Some(clear) = config.clear {
            clears.push(clear);
        }
    }
    let mut subpass =
        vk::SubpassDescription::builder().pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);
    if let Some(color) = color.as_ref() {
        subpass = subpass.color_attachments(std::slice::from_ref(color));
    }
    if let Some(depth) = depth.as_ref() {
        subpass = subpass.depth_stencil_attachment(depth);
    }
    if let Some(resolve) = resolve.as_ref() {
        subpass = subpass.resolve_attachments(std::slice::from_ref(resolve));
    }
    let subpass = *subpass;
    let create_info = *vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(std::slice::from_ref(&subpass));
    let pass = unsafe { dev.create_render_pass(&create_info, None) }.unwrap();
    Pass {
        pass,
        extent,
        clears,
    }
}
