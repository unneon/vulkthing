use crate::renderer::util::{Dev, ImageResources};
use ash::vk;

pub struct Pass {
    pub pass: vk::RenderPass,
    extent: vk::Extent2D,
    clears: Vec<vk::ClearValue>,
    pub resources: Vec<ImageResources>,
    framebuffers: Vec<vk::Framebuffer>,
    direct_to_swapchain: bool,
}

pub struct AttachmentConfig<'a> {
    format: vk::Format,
    samples: vk::SampleCountFlags,
    clear: Option<vk::ClearValue>,
    layout: vk::ImageLayout,
    final_layout: Option<vk::ImageLayout>,
    image_flags: vk::ImageUsageFlags,
    swapchain: Option<&'a [vk::ImageView]>,
    subpass: usize,
    input_to: Vec<usize>,
    transient: bool,
    store: bool,
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

    pub fn begin(&self) -> vk::RenderPassBeginInfo {
        assert!(!self.direct_to_swapchain);
        self.generic_begin(self.framebuffers[0])
    }

    pub fn begin_to_swapchain(&self, image_index: usize) -> vk::RenderPassBeginInfo {
        assert!(self.direct_to_swapchain);
        self.generic_begin(self.framebuffers[image_index])
    }

    fn generic_begin(&self, framebuffer: vk::Framebuffer) -> vk::RenderPassBeginInfo {
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

impl<'a> AttachmentConfig<'a> {
    pub fn new(format: vk::Format) -> Self {
        AttachmentConfig {
            format,
            samples: vk::SampleCountFlags::TYPE_1,
            clear: None,
            layout: vk::ImageLayout::UNDEFINED,
            final_layout: None,
            image_flags: vk::ImageUsageFlags::empty(),
            swapchain: None,
            subpass: 0,
            input_to: Vec::new(),
            transient: false,
            store: false,
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

    pub fn final_layout(mut self, layout: vk::ImageLayout) -> Self {
        self.final_layout = Some(layout);
        self
    }

    pub fn store(mut self, final_layout: vk::ImageLayout) -> Self {
        self.final_layout = Some(final_layout);
        self.store = true;
        self
    }

    pub fn usage(mut self, usage: vk::ImageUsageFlags) -> Self {
        self.image_flags = usage;
        self
    }

    pub fn swapchain(mut self, swapchain: &'a [vk::ImageView]) -> Self {
        self.swapchain = Some(swapchain);
        self
    }

    pub fn subpass(mut self, index: usize) -> Self {
        self.subpass = index;
        self
    }

    pub fn input_to(mut self, subpass: usize) -> Self {
        self.input_to.push(subpass);
        self
    }

    pub fn transient(mut self) -> Self {
        self.transient = true;
        self
    }
}

pub fn create_pass(
    extent: vk::Extent2D,
    dev: &Dev,
    configs: &[AttachmentConfig],
    dependencies: &[vk::SubpassDependency],
) -> Pass {
    let subpass_count = configs.iter().map(|config| config.subpass).max().unwrap() + 1;
    let mut attachments = Vec::new();
    let mut color = vec![Vec::new(); subpass_count];
    let mut depth = vec![None; subpass_count];
    let mut input = vec![Vec::new(); subpass_count];
    let mut clears = Vec::new();
    let mut resources = Vec::new();
    let mut framebuffer_attachments = Vec::new();
    let mut swapchain_index = None;
    let mut swapchain_views = None;
    for (index, config) in configs.iter().enumerate() {
        let attachment = *vk::AttachmentDescription::builder()
            .format(config.format)
            .samples(config.samples)
            .load_op(if config.clear.is_some() {
                vk::AttachmentLoadOp::CLEAR
            } else {
                vk::AttachmentLoadOp::DONT_CARE
            })
            .store_op(if config.store {
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
        if config.layout == vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL {
            color[config.subpass].push(reference);
        } else if config.layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            && depth[config.subpass].is_none()
        {
            depth[config.subpass] = Some(reference);
        } else {
            panic!(
                "unimplemented case {:?} {:?}",
                config.format,
                depth[config.subpass].is_none(),
            );
        }
        for as_input in &config.input_to {
            let input_reference = *vk::AttachmentReference::builder()
                .attachment(index as u32)
                .layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            input[*as_input].push(input_reference);
        }
        if let Some(clear) = config.clear {
            clears.push(clear);
        }
        if config.swapchain.is_none() {
            let mut flags = config.image_flags;
            if config.layout == vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL {
                flags |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
            } else if config.layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
                flags |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
            }
            if config.transient {
                flags |= vk::ImageUsageFlags::TRANSIENT_ATTACHMENT;
            }
            if !config.input_to.is_empty() {
                flags |= vk::ImageUsageFlags::INPUT_ATTACHMENT;
            }
            let resource = ImageResources::create(
                config.format,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                vk::ImageTiling::OPTIMAL,
                flags,
                if config.layout == vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL {
                    vk::ImageAspectFlags::COLOR
                } else {
                    vk::ImageAspectFlags::DEPTH
                },
                extent,
                config.samples,
                dev,
            );
            framebuffer_attachments.push(resource.view);
            resources.push(resource);
        } else {
            framebuffer_attachments.push(vk::ImageView::null());
            swapchain_index = Some(index);
            swapchain_views = config.swapchain;
        }
    }
    let mut subpasses = Vec::new();
    for subpass_index in 0..subpass_count {
        let mut subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color[subpass_index])
            .input_attachments(&input[subpass_index]);
        if let Some(depth) = depth[subpass_index].as_ref() {
            subpass = subpass.depth_stencil_attachment(depth);
        }
        subpasses.push(*subpass);
    }
    let create_info = *vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(dependencies);
    let pass = unsafe { dev.create_render_pass(&create_info, None) }.unwrap();
    let mut framebuffers = Vec::new();
    let info = *vk::FramebufferCreateInfo::builder()
        .render_pass(pass)
        .attachments(&framebuffer_attachments)
        .width(extent.width)
        .height(extent.height)
        .layers(1);
    if let Some(swapchain_index) = swapchain_index {
        for image in swapchain_views.unwrap() {
            unsafe { *(info.p_attachments.add(swapchain_index) as *mut vk::ImageView) = *image };
            let framebuffer = unsafe { dev.create_framebuffer(&info, None) }.unwrap();
            framebuffers.push(framebuffer);
        }
    } else {
        let framebuffer = unsafe { dev.create_framebuffer(&info, None) }.unwrap();
        framebuffers.push(framebuffer);
    }
    Pass {
        pass,
        extent,
        clears,
        resources,
        framebuffers,
        direct_to_swapchain: swapchain_index.is_some(),
    }
}
