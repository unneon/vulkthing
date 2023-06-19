// Code generated from renderer.kdl.

use crate::renderer::shader::create_shader;
use crate::renderer::util::Dev;
use crate::renderer::Pass;
use crate::renderer::Swapchain;
use ash::vk;
use std::ffi::CStr;
use std::mem::MaybeUninit;

pub struct Samplers {
    pub pixel: vk::Sampler,
}

pub struct DescriptorSetLayouts {
    pub object: vk::DescriptorSetLayout,
    pub grass: vk::DescriptorSetLayout,
    pub skybox: vk::DescriptorSetLayout,
    pub atmosphere: vk::DescriptorSetLayout,
    pub gaussian: vk::DescriptorSetLayout,
    pub postprocess: vk::DescriptorSetLayout,
}

pub struct PipelineLayouts {
    pub object: vk::PipelineLayout,
    pub grass: vk::PipelineLayout,
    pub skybox: vk::PipelineLayout,
    pub atmosphere: vk::PipelineLayout,
    pub gaussian: vk::PipelineLayout,
    pub postprocess: vk::PipelineLayout,
}

#[repr(C)]
pub struct Pipelines {
    pub object: vk::Pipeline,
    pub grass: vk::Pipeline,
    pub skybox: vk::Pipeline,
    pub atmosphere: vk::Pipeline,
    pub gaussian: vk::Pipeline,
    pub postprocess: vk::Pipeline,
}

#[repr(C)]
struct Scratch {
    pixel_sampler: vk::SamplerCreateInfo,
    object_bindings: [vk::DescriptorSetLayoutBinding; 5],
    object_layout: vk::DescriptorSetLayoutCreateInfo,
    grass_bindings: [vk::DescriptorSetLayoutBinding; 5],
    grass_layout: vk::DescriptorSetLayoutCreateInfo,
    skybox_bindings: [vk::DescriptorSetLayoutBinding; 1],
    skybox_layout: vk::DescriptorSetLayoutCreateInfo,
    atmosphere_bindings: [vk::DescriptorSetLayoutBinding; 4],
    atmosphere_layout: vk::DescriptorSetLayoutCreateInfo,
    gaussian_bindings: [vk::DescriptorSetLayoutBinding; 2],
    gaussian_layout: vk::DescriptorSetLayoutCreateInfo,
    postprocess_bindings: [vk::DescriptorSetLayoutBinding; 3],
    postprocess_layout: vk::DescriptorSetLayoutCreateInfo,
    assembly: vk::PipelineInputAssemblyStateCreateInfo,
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
    viewport_state: vk::PipelineViewportStateCreateInfo,
    dynamic_state: vk::PipelineDynamicStateCreateInfo,
    object_pipeline_layout: vk::PipelineLayoutCreateInfo,
    object_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    object_vertex_bindings: [vk::VertexInputBindingDescription; 1],
    object_vertex_attributes: [vk::VertexInputAttributeDescription; 2],
    object_vertex: vk::PipelineVertexInputStateCreateInfo,
    object_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    object_multisampling: vk::PipelineMultisampleStateCreateInfo,
    object_blend_attachments: [vk::PipelineColorBlendAttachmentState; 2],
    object_blend: vk::PipelineColorBlendStateCreateInfo,
    object_depth: vk::PipelineDepthStencilStateCreateInfo,
    grass_pipeline_layout: vk::PipelineLayoutCreateInfo,
    grass_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    grass_vertex_bindings: [vk::VertexInputBindingDescription; 2],
    grass_vertex_attributes: [vk::VertexInputAttributeDescription; 8],
    grass_vertex: vk::PipelineVertexInputStateCreateInfo,
    grass_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    grass_multisampling: vk::PipelineMultisampleStateCreateInfo,
    grass_blend_attachments: [vk::PipelineColorBlendAttachmentState; 2],
    grass_blend: vk::PipelineColorBlendStateCreateInfo,
    grass_depth: vk::PipelineDepthStencilStateCreateInfo,
    skybox_pipeline_layout: vk::PipelineLayoutCreateInfo,
    skybox_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    skybox_vertex_bindings: [vk::VertexInputBindingDescription; 1],
    skybox_vertex_attributes: [vk::VertexInputAttributeDescription; 1],
    skybox_vertex: vk::PipelineVertexInputStateCreateInfo,
    skybox_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    skybox_multisampling: vk::PipelineMultisampleStateCreateInfo,
    skybox_blend_attachments: [vk::PipelineColorBlendAttachmentState; 2],
    skybox_blend: vk::PipelineColorBlendStateCreateInfo,
    skybox_depth: vk::PipelineDepthStencilStateCreateInfo,
    atmosphere_pipeline_layout: vk::PipelineLayoutCreateInfo,
    atmosphere_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    atmosphere_vertex_bindings: [vk::VertexInputBindingDescription; 0],
    atmosphere_vertex_attributes: [vk::VertexInputAttributeDescription; 0],
    atmosphere_vertex: vk::PipelineVertexInputStateCreateInfo,
    atmosphere_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    atmosphere_multisampling: vk::PipelineMultisampleStateCreateInfo,
    atmosphere_blend_attachments: [vk::PipelineColorBlendAttachmentState; 1],
    atmosphere_blend: vk::PipelineColorBlendStateCreateInfo,
    atmosphere_depth: vk::PipelineDepthStencilStateCreateInfo,
    gaussian_pipeline_layout: vk::PipelineLayoutCreateInfo,
    gaussian_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    gaussian_vertex_bindings: [vk::VertexInputBindingDescription; 0],
    gaussian_vertex_attributes: [vk::VertexInputAttributeDescription; 0],
    gaussian_vertex: vk::PipelineVertexInputStateCreateInfo,
    gaussian_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    gaussian_multisampling: vk::PipelineMultisampleStateCreateInfo,
    gaussian_blend_attachments: [vk::PipelineColorBlendAttachmentState; 1],
    gaussian_blend: vk::PipelineColorBlendStateCreateInfo,
    gaussian_depth: vk::PipelineDepthStencilStateCreateInfo,
    postprocess_pipeline_layout: vk::PipelineLayoutCreateInfo,
    postprocess_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    postprocess_vertex_bindings: [vk::VertexInputBindingDescription; 0],
    postprocess_vertex_attributes: [vk::VertexInputAttributeDescription; 0],
    postprocess_vertex: vk::PipelineVertexInputStateCreateInfo,
    postprocess_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    postprocess_multisampling: vk::PipelineMultisampleStateCreateInfo,
    postprocess_blend_attachments: [vk::PipelineColorBlendAttachmentState; 1],
    postprocess_blend: vk::PipelineColorBlendStateCreateInfo,
    postprocess_depth: vk::PipelineDepthStencilStateCreateInfo,
    object_pipeline: vk::GraphicsPipelineCreateInfo,
    grass_pipeline: vk::GraphicsPipelineCreateInfo,
    skybox_pipeline: vk::GraphicsPipelineCreateInfo,
    atmosphere_pipeline: vk::GraphicsPipelineCreateInfo,
    gaussian_pipeline: vk::GraphicsPipelineCreateInfo,
    postprocess_pipeline: vk::GraphicsPipelineCreateInfo,
}

#[rustfmt::skip]
static mut SCRATCH: Scratch = Scratch {
    pixel_sampler: vk::SamplerCreateInfo {
        s_type: vk::StructureType::SAMPLER_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::SamplerCreateFlags::empty(),
        mag_filter: vk::Filter::NEAREST,
        min_filter: vk::Filter::NEAREST,
        mipmap_mode: vk::SamplerMipmapMode::NEAREST,
        address_mode_u: vk::SamplerAddressMode::CLAMP_TO_BORDER,
        address_mode_v: vk::SamplerAddressMode::CLAMP_TO_BORDER,
        address_mode_w: vk::SamplerAddressMode::REPEAT,
        mip_lod_bias: 0.,
        anisotropy_enable: 0,
        max_anisotropy: 0.,
        compare_enable: 0,
        compare_op: vk::CompareOp::NEVER,
        min_lod: 0.,
        max_lod: 0.,
        border_color: vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
        unnormalized_coordinates: 1,
    },
    object_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 3,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 4,
            descriptor_type: vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    object_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 5,
        p_bindings: unsafe { SCRATCH.object_bindings.as_ptr() },
    },
    grass_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 3,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 4,
            descriptor_type: vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    grass_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 5,
        p_bindings: unsafe { SCRATCH.grass_bindings.as_ptr() },
    },
    skybox_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    skybox_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 1,
        p_bindings: unsafe { SCRATCH.skybox_bindings.as_ptr() },
    },
    atmosphere_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::INPUT_ATTACHMENT,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::INPUT_ATTACHMENT,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 3,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    atmosphere_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 4,
        p_bindings: unsafe { SCRATCH.atmosphere_bindings.as_ptr() },
    },
    gaussian_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    gaussian_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 2,
        p_bindings: unsafe { SCRATCH.gaussian_bindings.as_ptr() },
    },
    postprocess_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    postprocess_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 3,
        p_bindings: unsafe { SCRATCH.postprocess_bindings.as_ptr() },
    },
    assembly: vk::PipelineInputAssemblyStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        primitive_restart_enable: 0,
    },
    viewport: vk::Viewport {
        x: 0.,
        y: 0.,
        width: 0.,
        height: 0.,
        min_depth: 0.,
        max_depth: 1.,
    },
    scissor: vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D { width: 0, height: 0 },
    },
    viewport_state: vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        p_viewports: unsafe { &SCRATCH.viewport },
        scissor_count: 1,
        p_scissors: unsafe { &SCRATCH.scissor },
    },
    dynamic_state: vk::PipelineDynamicStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDynamicStateCreateFlags::empty(),
        dynamic_state_count: 0,
        p_dynamic_states: std::ptr::null(),
    },
    object_pipeline_layout: vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 1,
        p_set_layouts: std::ptr::null(),
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    },
    object_shader_stages: [
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::VERTEX,
            module: vk::ShaderModule::null(),
            p_name: unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }.as_ptr(),
            p_specialization_info: std::ptr::null(),
        },
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: vk::ShaderModule::null(),
            p_name: unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }.as_ptr(),
            p_specialization_info: std::ptr::null(),
        },
    ],
    object_vertex_bindings: [
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: 24,
            input_rate: vk::VertexInputRate::VERTEX,
        },
    ],
    object_vertex_attributes: [
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 12,
        },
    ],
    object_vertex: vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: 1,
        p_vertex_binding_descriptions: unsafe { SCRATCH.object_vertex_bindings.as_ptr() },
        vertex_attribute_description_count: 2,
        p_vertex_attribute_descriptions: unsafe { SCRATCH.object_vertex_attributes.as_ptr() },
    },
    object_rasterizer: vk::PipelineRasterizationStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable: 0,
        rasterizer_discard_enable: 0,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::BACK,
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        depth_bias_enable: 0,
        depth_bias_constant_factor: 0.,
        depth_bias_clamp: 0.,
        depth_bias_slope_factor: 0.,
        line_width: 1.,
    },
    object_multisampling: vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineMultisampleStateCreateFlags::empty(),
        rasterization_samples: vk::SampleCountFlags::TYPE_2,
        sample_shading_enable: 0,
        min_sample_shading: 0.,
        p_sample_mask: std::ptr::null(),
        alpha_to_coverage_enable: 0,
        alpha_to_one_enable: 0,
    },
    object_blend_attachments: [
        vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::ZERO,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        },
        vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::ZERO,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        },
    ],
    object_blend: vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: 2,
        p_attachments: unsafe { SCRATCH.object_blend_attachments.as_ptr() },
        blend_constants: [0., 0., 0., 0.],
    },
    object_depth: vk::PipelineDepthStencilStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
        depth_test_enable: 1,
        depth_write_enable: 1,
        depth_compare_op: vk::CompareOp::LESS,
        depth_bounds_test_enable: 0,
        stencil_test_enable: 0,
        front: vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::NEVER,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        },
        back: vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::NEVER,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        },
        min_depth_bounds: 0.,
        max_depth_bounds: 1.,
    },
    object_pipeline: vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: 2,
        p_stages: unsafe { SCRATCH.object_shader_stages.as_ptr() },
        p_vertex_input_state: unsafe { &SCRATCH.object_vertex },
        p_input_assembly_state: unsafe { &SCRATCH.assembly },
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: unsafe { &SCRATCH.viewport_state },
        p_rasterization_state: unsafe { &SCRATCH.object_rasterizer },
        p_multisample_state: unsafe { &SCRATCH.object_multisampling },
        p_depth_stencil_state: unsafe { &SCRATCH.object_depth },
        p_color_blend_state: unsafe { &SCRATCH.object_blend },
        p_dynamic_state: unsafe { &SCRATCH.dynamic_state },
        layout: vk::PipelineLayout::null(),
        render_pass: vk::RenderPass::null(),
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    },
    grass_pipeline_layout: vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 1,
        p_set_layouts: std::ptr::null(),
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    },
    grass_shader_stages: [
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::VERTEX,
            module: vk::ShaderModule::null(),
            p_name: unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }.as_ptr(),
            p_specialization_info: std::ptr::null(),
        },
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: vk::ShaderModule::null(),
            p_name: unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }.as_ptr(),
            p_specialization_info: std::ptr::null(),
        },
    ],
    grass_vertex_bindings: [
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: 24,
            input_rate: vk::VertexInputRate::VERTEX,
        },
        vk::VertexInputBindingDescription {
            binding: 1,
            stride: 64,
            input_rate: vk::VertexInputRate::INSTANCE,
        },
    ],
    grass_vertex_attributes: [
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 12,
        },
        vk::VertexInputAttributeDescription {
            binding: 1,
            location: 2,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            binding: 1,
            location: 3,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 12,
        },
        vk::VertexInputAttributeDescription {
            binding: 1,
            location: 4,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 24,
        },
        vk::VertexInputAttributeDescription {
            binding: 1,
            location: 5,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 36,
        },
        vk::VertexInputAttributeDescription {
            binding: 1,
            location: 6,
            format: vk::Format::R32_SFLOAT,
            offset: 48,
        },
        vk::VertexInputAttributeDescription {
            binding: 1,
            location: 7,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 52,
        },
    ],
    grass_vertex: vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: 2,
        p_vertex_binding_descriptions: unsafe { SCRATCH.grass_vertex_bindings.as_ptr() },
        vertex_attribute_description_count: 8,
        p_vertex_attribute_descriptions: unsafe { SCRATCH.grass_vertex_attributes.as_ptr() },
    },
    grass_rasterizer: vk::PipelineRasterizationStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable: 0,
        rasterizer_discard_enable: 0,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::NONE,
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        depth_bias_enable: 0,
        depth_bias_constant_factor: 0.,
        depth_bias_clamp: 0.,
        depth_bias_slope_factor: 0.,
        line_width: 1.,
    },
    grass_multisampling: vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineMultisampleStateCreateFlags::empty(),
        rasterization_samples: vk::SampleCountFlags::TYPE_2,
        sample_shading_enable: 0,
        min_sample_shading: 0.,
        p_sample_mask: std::ptr::null(),
        alpha_to_coverage_enable: 0,
        alpha_to_one_enable: 0,
    },
    grass_blend_attachments: [
        vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::ZERO,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        },
        vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::ZERO,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        },
    ],
    grass_blend: vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: 2,
        p_attachments: unsafe { SCRATCH.grass_blend_attachments.as_ptr() },
        blend_constants: [0., 0., 0., 0.],
    },
    grass_depth: vk::PipelineDepthStencilStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
        depth_test_enable: 1,
        depth_write_enable: 1,
        depth_compare_op: vk::CompareOp::LESS,
        depth_bounds_test_enable: 0,
        stencil_test_enable: 0,
        front: vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::NEVER,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        },
        back: vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::NEVER,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        },
        min_depth_bounds: 0.,
        max_depth_bounds: 1.,
    },
    grass_pipeline: vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: 2,
        p_stages: unsafe { SCRATCH.grass_shader_stages.as_ptr() },
        p_vertex_input_state: unsafe { &SCRATCH.grass_vertex },
        p_input_assembly_state: unsafe { &SCRATCH.assembly },
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: unsafe { &SCRATCH.viewport_state },
        p_rasterization_state: unsafe { &SCRATCH.grass_rasterizer },
        p_multisample_state: unsafe { &SCRATCH.grass_multisampling },
        p_depth_stencil_state: unsafe { &SCRATCH.grass_depth },
        p_color_blend_state: unsafe { &SCRATCH.grass_blend },
        p_dynamic_state: unsafe { &SCRATCH.dynamic_state },
        layout: vk::PipelineLayout::null(),
        render_pass: vk::RenderPass::null(),
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    },
    skybox_pipeline_layout: vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 1,
        p_set_layouts: std::ptr::null(),
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    },
    skybox_shader_stages: [
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::VERTEX,
            module: vk::ShaderModule::null(),
            p_name: unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }.as_ptr(),
            p_specialization_info: std::ptr::null(),
        },
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: vk::ShaderModule::null(),
            p_name: unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }.as_ptr(),
            p_specialization_info: std::ptr::null(),
        },
    ],
    skybox_vertex_bindings: [
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: 24,
            input_rate: vk::VertexInputRate::VERTEX,
        },
    ],
    skybox_vertex_attributes: [
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        },
    ],
    skybox_vertex: vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: 1,
        p_vertex_binding_descriptions: unsafe { SCRATCH.skybox_vertex_bindings.as_ptr() },
        vertex_attribute_description_count: 1,
        p_vertex_attribute_descriptions: unsafe { SCRATCH.skybox_vertex_attributes.as_ptr() },
    },
    skybox_rasterizer: vk::PipelineRasterizationStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable: 0,
        rasterizer_discard_enable: 0,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::FRONT,
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        depth_bias_enable: 0,
        depth_bias_constant_factor: 0.,
        depth_bias_clamp: 0.,
        depth_bias_slope_factor: 0.,
        line_width: 1.,
    },
    skybox_multisampling: vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineMultisampleStateCreateFlags::empty(),
        rasterization_samples: vk::SampleCountFlags::TYPE_2,
        sample_shading_enable: 0,
        min_sample_shading: 0.,
        p_sample_mask: std::ptr::null(),
        alpha_to_coverage_enable: 0,
        alpha_to_one_enable: 0,
    },
    skybox_blend_attachments: [
        vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::ZERO,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        },
        vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::ZERO,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        },
    ],
    skybox_blend: vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: 2,
        p_attachments: unsafe { SCRATCH.skybox_blend_attachments.as_ptr() },
        blend_constants: [0., 0., 0., 0.],
    },
    skybox_depth: vk::PipelineDepthStencilStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
        depth_test_enable: 1,
        depth_write_enable: 1,
        depth_compare_op: vk::CompareOp::LESS,
        depth_bounds_test_enable: 0,
        stencil_test_enable: 0,
        front: vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::NEVER,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        },
        back: vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::NEVER,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        },
        min_depth_bounds: 0.,
        max_depth_bounds: 1.,
    },
    skybox_pipeline: vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: 2,
        p_stages: unsafe { SCRATCH.skybox_shader_stages.as_ptr() },
        p_vertex_input_state: unsafe { &SCRATCH.skybox_vertex },
        p_input_assembly_state: unsafe { &SCRATCH.assembly },
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: unsafe { &SCRATCH.viewport_state },
        p_rasterization_state: unsafe { &SCRATCH.skybox_rasterizer },
        p_multisample_state: unsafe { &SCRATCH.skybox_multisampling },
        p_depth_stencil_state: unsafe { &SCRATCH.skybox_depth },
        p_color_blend_state: unsafe { &SCRATCH.skybox_blend },
        p_dynamic_state: unsafe { &SCRATCH.dynamic_state },
        layout: vk::PipelineLayout::null(),
        render_pass: vk::RenderPass::null(),
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    },
    atmosphere_pipeline_layout: vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 1,
        p_set_layouts: std::ptr::null(),
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    },
    atmosphere_shader_stages: [
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::VERTEX,
            module: vk::ShaderModule::null(),
            p_name: unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }.as_ptr(),
            p_specialization_info: std::ptr::null(),
        },
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: vk::ShaderModule::null(),
            p_name: unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }.as_ptr(),
            p_specialization_info: std::ptr::null(),
        },
    ],
    atmosphere_vertex_bindings: [
    ],
    atmosphere_vertex_attributes: [
    ],
    atmosphere_vertex: vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: 0,
        p_vertex_binding_descriptions: unsafe { SCRATCH.atmosphere_vertex_bindings.as_ptr() },
        vertex_attribute_description_count: 0,
        p_vertex_attribute_descriptions: unsafe { SCRATCH.atmosphere_vertex_attributes.as_ptr() },
    },
    atmosphere_rasterizer: vk::PipelineRasterizationStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable: 0,
        rasterizer_discard_enable: 0,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::BACK,
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        depth_bias_enable: 0,
        depth_bias_constant_factor: 0.,
        depth_bias_clamp: 0.,
        depth_bias_slope_factor: 0.,
        line_width: 1.,
    },
    atmosphere_multisampling: vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineMultisampleStateCreateFlags::empty(),
        rasterization_samples: vk::SampleCountFlags::TYPE_2,
        sample_shading_enable: 0,
        min_sample_shading: 0.,
        p_sample_mask: std::ptr::null(),
        alpha_to_coverage_enable: 0,
        alpha_to_one_enable: 0,
    },
    atmosphere_blend_attachments: [
        vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::ZERO,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        },
    ],
    atmosphere_blend: vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: 1,
        p_attachments: unsafe { SCRATCH.atmosphere_blend_attachments.as_ptr() },
        blend_constants: [0., 0., 0., 0.],
    },
    atmosphere_depth: vk::PipelineDepthStencilStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
        depth_test_enable: 0,
        depth_write_enable: 0,
        depth_compare_op: vk::CompareOp::LESS,
        depth_bounds_test_enable: 0,
        stencil_test_enable: 0,
        front: vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::NEVER,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        },
        back: vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::NEVER,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        },
        min_depth_bounds: 0.,
        max_depth_bounds: 1.,
    },
    atmosphere_pipeline: vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: 2,
        p_stages: unsafe { SCRATCH.atmosphere_shader_stages.as_ptr() },
        p_vertex_input_state: unsafe { &SCRATCH.atmosphere_vertex },
        p_input_assembly_state: unsafe { &SCRATCH.assembly },
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: unsafe { &SCRATCH.viewport_state },
        p_rasterization_state: unsafe { &SCRATCH.atmosphere_rasterizer },
        p_multisample_state: unsafe { &SCRATCH.atmosphere_multisampling },
        p_depth_stencil_state: unsafe { &SCRATCH.atmosphere_depth },
        p_color_blend_state: unsafe { &SCRATCH.atmosphere_blend },
        p_dynamic_state: unsafe { &SCRATCH.dynamic_state },
        layout: vk::PipelineLayout::null(),
        render_pass: vk::RenderPass::null(),
        subpass: 1,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    },
    gaussian_pipeline_layout: vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 1,
        p_set_layouts: std::ptr::null(),
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    },
    gaussian_shader_stages: [
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::VERTEX,
            module: vk::ShaderModule::null(),
            p_name: unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }.as_ptr(),
            p_specialization_info: std::ptr::null(),
        },
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: vk::ShaderModule::null(),
            p_name: unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }.as_ptr(),
            p_specialization_info: std::ptr::null(),
        },
    ],
    gaussian_vertex_bindings: [
    ],
    gaussian_vertex_attributes: [
    ],
    gaussian_vertex: vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: 0,
        p_vertex_binding_descriptions: unsafe { SCRATCH.gaussian_vertex_bindings.as_ptr() },
        vertex_attribute_description_count: 0,
        p_vertex_attribute_descriptions: unsafe { SCRATCH.gaussian_vertex_attributes.as_ptr() },
    },
    gaussian_rasterizer: vk::PipelineRasterizationStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable: 0,
        rasterizer_discard_enable: 0,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::BACK,
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        depth_bias_enable: 0,
        depth_bias_constant_factor: 0.,
        depth_bias_clamp: 0.,
        depth_bias_slope_factor: 0.,
        line_width: 1.,
    },
    gaussian_multisampling: vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineMultisampleStateCreateFlags::empty(),
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        sample_shading_enable: 0,
        min_sample_shading: 0.,
        p_sample_mask: std::ptr::null(),
        alpha_to_coverage_enable: 0,
        alpha_to_one_enable: 0,
    },
    gaussian_blend_attachments: [
        vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::ZERO,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        },
    ],
    gaussian_blend: vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: 1,
        p_attachments: unsafe { SCRATCH.gaussian_blend_attachments.as_ptr() },
        blend_constants: [0., 0., 0., 0.],
    },
    gaussian_depth: vk::PipelineDepthStencilStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
        depth_test_enable: 0,
        depth_write_enable: 0,
        depth_compare_op: vk::CompareOp::LESS,
        depth_bounds_test_enable: 0,
        stencil_test_enable: 0,
        front: vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::NEVER,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        },
        back: vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::NEVER,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        },
        min_depth_bounds: 0.,
        max_depth_bounds: 1.,
    },
    gaussian_pipeline: vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: 2,
        p_stages: unsafe { SCRATCH.gaussian_shader_stages.as_ptr() },
        p_vertex_input_state: unsafe { &SCRATCH.gaussian_vertex },
        p_input_assembly_state: unsafe { &SCRATCH.assembly },
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: unsafe { &SCRATCH.viewport_state },
        p_rasterization_state: unsafe { &SCRATCH.gaussian_rasterizer },
        p_multisample_state: unsafe { &SCRATCH.gaussian_multisampling },
        p_depth_stencil_state: unsafe { &SCRATCH.gaussian_depth },
        p_color_blend_state: unsafe { &SCRATCH.gaussian_blend },
        p_dynamic_state: unsafe { &SCRATCH.dynamic_state },
        layout: vk::PipelineLayout::null(),
        render_pass: vk::RenderPass::null(),
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    },
    postprocess_pipeline_layout: vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 1,
        p_set_layouts: std::ptr::null(),
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    },
    postprocess_shader_stages: [
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::VERTEX,
            module: vk::ShaderModule::null(),
            p_name: unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }.as_ptr(),
            p_specialization_info: std::ptr::null(),
        },
        vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: vk::ShaderModule::null(),
            p_name: unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }.as_ptr(),
            p_specialization_info: std::ptr::null(),
        },
    ],
    postprocess_vertex_bindings: [
    ],
    postprocess_vertex_attributes: [
    ],
    postprocess_vertex: vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: 0,
        p_vertex_binding_descriptions: unsafe { SCRATCH.postprocess_vertex_bindings.as_ptr() },
        vertex_attribute_description_count: 0,
        p_vertex_attribute_descriptions: unsafe { SCRATCH.postprocess_vertex_attributes.as_ptr() },
    },
    postprocess_rasterizer: vk::PipelineRasterizationStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable: 0,
        rasterizer_discard_enable: 0,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::BACK,
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        depth_bias_enable: 0,
        depth_bias_constant_factor: 0.,
        depth_bias_clamp: 0.,
        depth_bias_slope_factor: 0.,
        line_width: 1.,
    },
    postprocess_multisampling: vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineMultisampleStateCreateFlags::empty(),
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        sample_shading_enable: 0,
        min_sample_shading: 0.,
        p_sample_mask: std::ptr::null(),
        alpha_to_coverage_enable: 0,
        alpha_to_one_enable: 0,
    },
    postprocess_blend_attachments: [
        vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::ZERO,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        },
    ],
    postprocess_blend: vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: 1,
        p_attachments: unsafe { SCRATCH.postprocess_blend_attachments.as_ptr() },
        blend_constants: [0., 0., 0., 0.],
    },
    postprocess_depth: vk::PipelineDepthStencilStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
        depth_test_enable: 0,
        depth_write_enable: 0,
        depth_compare_op: vk::CompareOp::LESS,
        depth_bounds_test_enable: 0,
        stencil_test_enable: 0,
        front: vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::NEVER,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        },
        back: vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::NEVER,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        },
        min_depth_bounds: 0.,
        max_depth_bounds: 1.,
    },
    postprocess_pipeline: vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: 2,
        p_stages: unsafe { SCRATCH.postprocess_shader_stages.as_ptr() },
        p_vertex_input_state: unsafe { &SCRATCH.postprocess_vertex },
        p_input_assembly_state: unsafe { &SCRATCH.assembly },
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: unsafe { &SCRATCH.viewport_state },
        p_rasterization_state: unsafe { &SCRATCH.postprocess_rasterizer },
        p_multisample_state: unsafe { &SCRATCH.postprocess_multisampling },
        p_depth_stencil_state: unsafe { &SCRATCH.postprocess_depth },
        p_color_blend_state: unsafe { &SCRATCH.postprocess_blend },
        p_dynamic_state: unsafe { &SCRATCH.dynamic_state },
        layout: vk::PipelineLayout::null(),
        render_pass: vk::RenderPass::null(),
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    },
};

impl Samplers {
    pub fn cleanup(&self, dev: &Dev) {
        unsafe { dev.destroy_sampler(self.pixel, None) };
    }
}

impl DescriptorSetLayouts {
    pub fn cleanup(&self, dev: &Dev) {
        unsafe { dev.destroy_descriptor_set_layout(self.object, None) };
        unsafe { dev.destroy_descriptor_set_layout(self.grass, None) };
        unsafe { dev.destroy_descriptor_set_layout(self.skybox, None) };
        unsafe { dev.destroy_descriptor_set_layout(self.atmosphere, None) };
        unsafe { dev.destroy_descriptor_set_layout(self.gaussian, None) };
        unsafe { dev.destroy_descriptor_set_layout(self.postprocess, None) };
    }
}

impl PipelineLayouts {
    pub fn cleanup(&self, dev: &Dev) {
        unsafe { dev.destroy_pipeline_layout(self.object, None) };
        unsafe { dev.destroy_pipeline_layout(self.grass, None) };
        unsafe { dev.destroy_pipeline_layout(self.skybox, None) };
        unsafe { dev.destroy_pipeline_layout(self.atmosphere, None) };
        unsafe { dev.destroy_pipeline_layout(self.gaussian, None) };
        unsafe { dev.destroy_pipeline_layout(self.postprocess, None) };
    }
}

impl Pipelines {
    pub fn cleanup(&self, dev: &Dev) {
        unsafe { dev.destroy_pipeline(self.object, None) };
        unsafe { dev.destroy_pipeline(self.grass, None) };
        unsafe { dev.destroy_pipeline(self.skybox, None) };
        unsafe { dev.destroy_pipeline(self.atmosphere, None) };
        unsafe { dev.destroy_pipeline(self.gaussian, None) };
        unsafe { dev.destroy_pipeline(self.postprocess, None) };
    }
}

#[rustfmt::skip]
pub fn create_samplers(dev: &Dev) -> Samplers {
    let pixel = unsafe { dev.create_sampler(&SCRATCH.pixel_sampler, None).unwrap_unchecked() };
    Samplers {
        pixel,
    }
}

#[rustfmt::skip]
pub fn create_descriptor_set_layouts(samplers: &Samplers, dev: &Dev) -> DescriptorSetLayouts {
    unsafe { SCRATCH.gaussian_bindings[0].p_immutable_samplers = &samplers.pixel };
    unsafe { SCRATCH.postprocess_bindings[0].p_immutable_samplers = &samplers.pixel };
    unsafe { SCRATCH.postprocess_bindings[1].p_immutable_samplers = &samplers.pixel };
    let object = unsafe { dev.create_descriptor_set_layout(&SCRATCH.object_layout, None).unwrap_unchecked() };
    let grass = unsafe { dev.create_descriptor_set_layout(&SCRATCH.grass_layout, None).unwrap_unchecked() };
    let skybox = unsafe { dev.create_descriptor_set_layout(&SCRATCH.skybox_layout, None).unwrap_unchecked() };
    let atmosphere = unsafe { dev.create_descriptor_set_layout(&SCRATCH.atmosphere_layout, None).unwrap_unchecked() };
    let gaussian = unsafe { dev.create_descriptor_set_layout(&SCRATCH.gaussian_layout, None).unwrap_unchecked() };
    let postprocess = unsafe { dev.create_descriptor_set_layout(&SCRATCH.postprocess_layout, None).unwrap_unchecked() };
    DescriptorSetLayouts {
        object,
        grass,
        skybox,
        atmosphere,
        gaussian,
        postprocess,
    }
}

#[rustfmt::skip]
pub fn create_pipeline_layouts(
    descriptor_set_layouts: &DescriptorSetLayouts,
    dev: &Dev,
) -> PipelineLayouts {
    unsafe { SCRATCH.object_pipeline_layout.p_set_layouts = &descriptor_set_layouts.object };
    unsafe { SCRATCH.grass_pipeline_layout.p_set_layouts = &descriptor_set_layouts.grass };
    unsafe { SCRATCH.skybox_pipeline_layout.p_set_layouts = &descriptor_set_layouts.skybox };
    unsafe { SCRATCH.atmosphere_pipeline_layout.p_set_layouts = &descriptor_set_layouts.atmosphere };
    unsafe { SCRATCH.gaussian_pipeline_layout.p_set_layouts = &descriptor_set_layouts.gaussian };
    unsafe { SCRATCH.postprocess_pipeline_layout.p_set_layouts = &descriptor_set_layouts.postprocess };
    let object = unsafe { dev.create_pipeline_layout(&SCRATCH.object_pipeline_layout, None).unwrap_unchecked() };
    let grass = unsafe { dev.create_pipeline_layout(&SCRATCH.grass_pipeline_layout, None).unwrap_unchecked() };
    let skybox = unsafe { dev.create_pipeline_layout(&SCRATCH.skybox_pipeline_layout, None).unwrap_unchecked() };
    let atmosphere = unsafe { dev.create_pipeline_layout(&SCRATCH.atmosphere_pipeline_layout, None).unwrap_unchecked() };
    let gaussian = unsafe { dev.create_pipeline_layout(&SCRATCH.gaussian_pipeline_layout, None).unwrap_unchecked() };
    let postprocess = unsafe { dev.create_pipeline_layout(&SCRATCH.postprocess_pipeline_layout, None).unwrap_unchecked() };
    PipelineLayouts {
        object,
        grass,
        skybox,
        atmosphere,
        gaussian,
        postprocess,
    }
}

#[rustfmt::skip]
pub fn create_pipelines(
    render: &Pass,
    gaussian: &Pass,
    postprocess: &Pass,
    _msaa_samples: vk::SampleCountFlags,
    swapchain: &Swapchain,
    supports_raytracing: bool,
    layouts: &PipelineLayouts,
    dev: &Dev,
) -> Pipelines {
    unsafe { SCRATCH.viewport.width = swapchain.extent.width as f32 };
    unsafe { SCRATCH.viewport.height = swapchain.extent.height as f32 };
    unsafe { SCRATCH.scissor.extent.width = swapchain.extent.width };
    unsafe { SCRATCH.scissor.extent.height = swapchain.extent.height };
    let vertex_shader = create_shader(
        "shaders/object.vert",
        vk::ShaderStageFlags::VERTEX,
        supports_raytracing,
        dev,
    );
    let fragment_shader = create_shader(
        "shaders/object.frag",
        vk::ShaderStageFlags::FRAGMENT,
        supports_raytracing,
        dev,
    );
    unsafe { SCRATCH.object_shader_stages[0].module = vertex_shader.module };
    unsafe { SCRATCH.object_shader_stages[1].module = fragment_shader.module };
    unsafe { SCRATCH.object_pipeline.layout = layouts.object };
    unsafe { SCRATCH.object_pipeline.render_pass = render.pass };
    let vertex_shader = create_shader(
        "shaders/grass.vert",
        vk::ShaderStageFlags::VERTEX,
        supports_raytracing,
        dev,
    );
    let fragment_shader = create_shader(
        "shaders/grass.frag",
        vk::ShaderStageFlags::FRAGMENT,
        supports_raytracing,
        dev,
    );
    unsafe { SCRATCH.grass_shader_stages[0].module = vertex_shader.module };
    unsafe { SCRATCH.grass_shader_stages[1].module = fragment_shader.module };
    unsafe { SCRATCH.grass_pipeline.layout = layouts.grass };
    unsafe { SCRATCH.grass_pipeline.render_pass = render.pass };
    let vertex_shader = create_shader(
        "shaders/skybox.vert",
        vk::ShaderStageFlags::VERTEX,
        supports_raytracing,
        dev,
    );
    let fragment_shader = create_shader(
        "shaders/skybox.frag",
        vk::ShaderStageFlags::FRAGMENT,
        supports_raytracing,
        dev,
    );
    unsafe { SCRATCH.skybox_shader_stages[0].module = vertex_shader.module };
    unsafe { SCRATCH.skybox_shader_stages[1].module = fragment_shader.module };
    unsafe { SCRATCH.skybox_pipeline.layout = layouts.skybox };
    unsafe { SCRATCH.skybox_pipeline.render_pass = render.pass };
    let vertex_shader = create_shader(
        "shaders/atmosphere.vert",
        vk::ShaderStageFlags::VERTEX,
        supports_raytracing,
        dev,
    );
    let fragment_shader = create_shader(
        "shaders/atmosphere.frag",
        vk::ShaderStageFlags::FRAGMENT,
        supports_raytracing,
        dev,
    );
    unsafe { SCRATCH.atmosphere_shader_stages[0].module = vertex_shader.module };
    unsafe { SCRATCH.atmosphere_shader_stages[1].module = fragment_shader.module };
    unsafe { SCRATCH.atmosphere_pipeline.layout = layouts.atmosphere };
    unsafe { SCRATCH.atmosphere_pipeline.render_pass = render.pass };
    let vertex_shader = create_shader(
        "shaders/gaussian.vert",
        vk::ShaderStageFlags::VERTEX,
        supports_raytracing,
        dev,
    );
    let fragment_shader = create_shader(
        "shaders/gaussian.frag",
        vk::ShaderStageFlags::FRAGMENT,
        supports_raytracing,
        dev,
    );
    unsafe { SCRATCH.gaussian_shader_stages[0].module = vertex_shader.module };
    unsafe { SCRATCH.gaussian_shader_stages[1].module = fragment_shader.module };
    unsafe { SCRATCH.gaussian_pipeline.layout = layouts.gaussian };
    unsafe { SCRATCH.gaussian_pipeline.render_pass = gaussian.pass };
    let vertex_shader = create_shader(
        "shaders/postprocess.vert",
        vk::ShaderStageFlags::VERTEX,
        supports_raytracing,
        dev,
    );
    let fragment_shader = create_shader(
        "shaders/postprocess.frag",
        vk::ShaderStageFlags::FRAGMENT,
        supports_raytracing,
        dev,
    );
    unsafe { SCRATCH.postprocess_shader_stages[0].module = vertex_shader.module };
    unsafe { SCRATCH.postprocess_shader_stages[1].module = fragment_shader.module };
    unsafe { SCRATCH.postprocess_pipeline.layout = layouts.postprocess };
    unsafe { SCRATCH.postprocess_pipeline.render_pass = postprocess.pass };
    let mut pipelines = MaybeUninit::uninit();
    let _ = unsafe { (dev.fp_v1_0().create_graphics_pipelines)(
        dev.handle(),
        vk::PipelineCache::null(),
        6,
        &SCRATCH.object_pipeline,
        std::ptr::null(),
        pipelines.as_mut_ptr() as *mut vk::Pipeline,
    ) };
    unsafe { pipelines.assume_init() }
}
