use crate::renderer::shader::create_shader;
use crate::renderer::util::Dev;
use ash::{vk, Device};

pub struct Pipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
}

pub struct PipelineConfig<'a> {
    pub vertex_shader_path: &'a str,
    pub fragment_shader_path: &'a str,
    pub vertex_layout: Option<VertexLayout>,
    pub msaa_samples: vk::SampleCountFlags,
    pub polygon_mode: vk::PolygonMode,
    pub descriptor_layouts: &'a [vk::DescriptorSetLayout],
    pub depth_test: bool,
    pub pass: vk::RenderPass,
    pub dev: &'a Dev,
    pub swapchain_extent: vk::Extent2D,
}

pub struct VertexLayout {
    pub stride: usize,
    pub attribute_descriptions: Vec<vk::VertexInputAttributeDescription>,
}

impl Pipeline {
    pub fn cleanup(&self, dev: &Device) {
        unsafe {
            dev.destroy_pipeline(self.pipeline, None);
            dev.destroy_pipeline_layout(self.layout, None);
        }
    }
}

pub fn create_pipeline(config: PipelineConfig) -> Pipeline {
    // Build shaders from GLSL paths. This can build and cache SPIR-V by spawning glslc as a
    // subprocess.
    let vertex_shader = create_shader(
        config.vertex_shader_path,
        vk::ShaderStageFlags::VERTEX,
        config.dev,
    );
    let fragment_shader = create_shader(
        config.fragment_shader_path,
        vk::ShaderStageFlags::FRAGMENT,
        config.dev,
    );
    let shader_stages = [vertex_shader.stage_info, fragment_shader.stage_info];

    // Vertex data can be spread over many buffers for data locality reasons, and also be bound per
    // instance for instancing. But for this project I'll use either a bindless design or mesh
    // shaders eventually so this probably shouldn't matter. Not 100% sure.
    let (vertex_binding_descriptions, vertex_attribute_descriptions) =
        if let Some(vertex_layout) = config.vertex_layout {
            let vertex_binding_descriptions = vec![*vk::VertexInputBindingDescription::builder()
                .binding(0)
                .stride(vertex_layout.stride as u32)
                .input_rate(vk::VertexInputRate::VERTEX)];
            let vertex_attribute_descriptions = vertex_layout.attribute_descriptions;
            (vertex_binding_descriptions, vertex_attribute_descriptions)
        } else {
            (vec![], vec![])
        };
    let vertex_input = *vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&vertex_binding_descriptions)
        .vertex_attribute_descriptions(&vertex_attribute_descriptions);

    // Apparently triangle strips only make sense on older hardware, so I won't be using any other
    // options.
    let input_assembly = *vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    let viewport = vk::Viewport {
        x: 0.,
        y: 0.,
        width: config.swapchain_extent.width as f32,
        height: config.swapchain_extent.height as f32,
        min_depth: 0.,
        max_depth: 1.,
    };
    let scissor = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: config.swapchain_extent,
    };
    let viewport_state = *vk::PipelineViewportStateCreateInfo::builder()
        .viewports(std::slice::from_ref(&viewport))
        .scissors(std::slice::from_ref(&scissor));

    // Setting some conventions and whether fill or do wireframe. Wireframe could be useful later
    // for debugging, or maybe I'll just use the functionality in renderdoc.
    let rasterizer = *vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(config.polygon_mode)
        .line_width(1.)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false)
        .depth_bias_constant_factor(0.)
        .depth_bias_clamp(0.)
        .depth_bias_slope_factor(0.);

    // Advanced settings control some variable rate shading in polygon interiors? Probably don't
    // care with the visual style I'm going for, I should probably limit the number of MSAA samples
    // before diminishing returns kick in.
    let multisampling = *vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(config.msaa_samples)
        .min_sample_shading(1.)
        .sample_mask(&[])
        .alpha_to_coverage_enable(false)
        .alpha_to_one_enable(false);

    // Will only become relevant once I write some render passes for transparent objects.
    let color_blend_attachment = *vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(false);
    let color_blend_attachments = [color_blend_attachment];
    let color_blending = *vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(&color_blend_attachments);

    // Configuring conventions for the depth buffer. AMD FSR 2 has some recommendations to change
    // them from 0 1 to 0 infinity. I wonder what DLSS recommendations say.
    // TODO: AMD recommends using reversed 1 0 depth to improve float distribution?
    // TODO: AMD recommends to make the near value as high as possible.
    let depth_stencil = *vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(config.depth_test)
        .depth_write_enable(config.depth_test)
        .depth_compare_op(vk::CompareOp::LESS);

    // I would like to make these things static too, but it would require recreating the pipeline on
    // window resize. This doesn't sound too bad, games run in fullscreen anyway.
    let dynamic_state = *vk::PipelineDynamicStateCreateInfo::builder();

    // I think this is meant to be shared between multiple pipelines? You have to bind this along
    // with the descriptor set, so the intended use case is probably having a single descriptor set
    // and multiple associated pipelines that use it with slightly different parameters.
    let layout_create_info = *vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(config.descriptor_layouts)
        .push_constant_ranges(&[]);
    let layout = unsafe { config.dev.create_pipeline_layout(&layout_create_info, None) }.unwrap();

    // If Vulkan wasn't a C api where you have to pass array pointers, this entire function would be
    // a struct literal.
    let pipeline_info = *vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterizer)
        .multisample_state(&multisampling)
        .color_blend_state(&color_blending)
        .depth_stencil_state(&depth_stencil)
        .dynamic_state(&dynamic_state)
        .layout(layout)
        .render_pass(config.pass)
        .subpass(0);

    // Apparently creating pipelines can be batched? Probably worth it when there are many pipeline
    // combinations. This kind of starts happening already because I want to use different shaders
    // and vertex layout for procedurally generated objects, but let's assume the cost is reasonable
    // for now. Also, pipeline caches are a thing and probably reduce the impact of this on
    // subsequent loads.
    let pipeline = unsafe {
        config.dev.create_graphics_pipelines(
            vk::PipelineCache::null(),
            std::slice::from_ref(&pipeline_info),
            None,
        )
    }
    .unwrap()[0];

    Pipeline { pipeline, layout }
}
