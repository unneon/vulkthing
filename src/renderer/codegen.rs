// Code generated from renderer.kdl.

use crate::renderer::raytracing::RaytraceResources;
use crate::renderer::shader::compile_glsl;
#[rustfmt::skip]
use crate::renderer::uniform::{Global, Material, ModelViewProjection};
use crate::renderer::debug::set_label;
use crate::renderer::util::{AnyUniformBuffer, Dev, ImageResources, UniformBuffer};
use crate::renderer::{Pass, Swapchain, COLOR_FORMAT, DEPTH_FORMAT, FRAMES_IN_FLIGHT};
use ash::extensions::ext::DebugUtils;
use ash::vk;
use std::ffi::CStr;
use std::mem::MaybeUninit;

pub struct Samplers {
    pub nearest: vk::Sampler,
    pub bilinear: vk::Sampler,
}

pub struct DescriptorSetLayouts {
    pub object: vk::DescriptorSetLayout,
    pub deferred: vk::DescriptorSetLayout,
    pub gaussian_horizontal: vk::DescriptorSetLayout,
    pub gaussian_vertical: vk::DescriptorSetLayout,
    pub postprocess: vk::DescriptorSetLayout,
    pub global: vk::DescriptorSetLayout,
}

pub struct DescriptorPools {
    pub object: vk::DescriptorPool,
    pub object_layout: vk::DescriptorSetLayout,
    pub deferred: vk::DescriptorPool,
    pub deferred_layout: vk::DescriptorSetLayout,
    pub gaussian_horizontal: vk::DescriptorPool,
    pub gaussian_horizontal_layout: vk::DescriptorSetLayout,
    pub gaussian_vertical: vk::DescriptorPool,
    pub gaussian_vertical_layout: vk::DescriptorSetLayout,
    pub postprocess: vk::DescriptorPool,
    pub postprocess_layout: vk::DescriptorSetLayout,
    pub global: vk::DescriptorPool,
    pub global_layout: vk::DescriptorSetLayout,
}

pub struct PipelineLayouts {
    pub object: vk::PipelineLayout,
    pub grass: vk::PipelineLayout,
    pub star: vk::PipelineLayout,
    pub skybox: vk::PipelineLayout,
    pub deferred: vk::PipelineLayout,
    pub gaussian_horizontal: vk::PipelineLayout,
    pub gaussian_vertical: vk::PipelineLayout,
    pub postprocess: vk::PipelineLayout,
}

pub struct Shaders {
    pub object_vertex: Vec<u32>,
    pub object_fragment: Vec<u32>,
    pub grass_vertex: Vec<u32>,
    pub grass_fragment: Vec<u32>,
    pub star_vertex: Vec<u32>,
    pub star_fragment: Vec<u32>,
    pub skybox_vertex: Vec<u32>,
    pub skybox_fragment: Vec<u32>,
    pub deferred_vertex: Vec<u32>,
    pub deferred_fragment: Vec<u32>,
    pub gaussian_horizontal_vertex: Vec<u32>,
    pub gaussian_horizontal_fragment: Vec<u32>,
    pub gaussian_vertical_vertex: Vec<u32>,
    pub gaussian_vertical_fragment: Vec<u32>,
    pub postprocess_vertex: Vec<u32>,
    pub postprocess_fragment: Vec<u32>,
}

pub struct ShaderModules {
    pub object_vertex: vk::ShaderModule,
    pub object_fragment: vk::ShaderModule,
    pub grass_vertex: vk::ShaderModule,
    pub grass_fragment: vk::ShaderModule,
    pub star_vertex: vk::ShaderModule,
    pub star_fragment: vk::ShaderModule,
    pub skybox_vertex: vk::ShaderModule,
    pub skybox_fragment: vk::ShaderModule,
    pub deferred_vertex: vk::ShaderModule,
    pub deferred_fragment: vk::ShaderModule,
    pub gaussian_horizontal_vertex: vk::ShaderModule,
    pub gaussian_horizontal_fragment: vk::ShaderModule,
    pub gaussian_vertical_vertex: vk::ShaderModule,
    pub gaussian_vertical_fragment: vk::ShaderModule,
    pub postprocess_vertex: vk::ShaderModule,
    pub postprocess_fragment: vk::ShaderModule,
}

#[repr(C)]
pub struct Passes {
    pub render: Pass,
    pub gaussian_horizontal: Pass,
    pub gaussian_vertical: Pass,
    pub postprocess: Pass,
}

#[repr(C)]
pub struct Pipelines {
    pub object: vk::Pipeline,
    pub grass: vk::Pipeline,
    pub star: vk::Pipeline,
    pub skybox: vk::Pipeline,
    pub deferred: vk::Pipeline,
    pub gaussian_horizontal: vk::Pipeline,
    pub gaussian_vertical: vk::Pipeline,
    pub postprocess: vk::Pipeline,
}

#[repr(C)]
struct Scratch {
    nearest_sampler: vk::SamplerCreateInfo,
    bilinear_sampler: vk::SamplerCreateInfo,
    render_rasterization_color: [vk::AttachmentReference; 1],
    render_rasterization_depth: vk::AttachmentReference,
    render_deferred_input: [vk::AttachmentReference; 1],
    render_attachments: [vk::AttachmentDescription; 2],
    render_subpasses: [vk::SubpassDescription; 2],
    render_dependencies: [vk::SubpassDependency; 1],
    render_pass: vk::RenderPassCreateInfo,
    gaussian_horizontal_gaussian_color: [vk::AttachmentReference; 1],
    gaussian_horizontal_attachments: [vk::AttachmentDescription; 1],
    gaussian_horizontal_subpasses: [vk::SubpassDescription; 1],
    gaussian_horizontal_dependencies: [vk::SubpassDependency; 0],
    gaussian_horizontal_pass: vk::RenderPassCreateInfo,
    gaussian_vertical_gaussian_color: [vk::AttachmentReference; 1],
    gaussian_vertical_attachments: [vk::AttachmentDescription; 1],
    gaussian_vertical_subpasses: [vk::SubpassDescription; 1],
    gaussian_vertical_dependencies: [vk::SubpassDependency; 0],
    gaussian_vertical_pass: vk::RenderPassCreateInfo,
    postprocess_postprocess_color: [vk::AttachmentReference; 1],
    postprocess_attachments: [vk::AttachmentDescription; 1],
    postprocess_subpasses: [vk::SubpassDescription; 1],
    postprocess_dependencies: [vk::SubpassDependency; 0],
    postprocess_pass: vk::RenderPassCreateInfo,
    object_bindings: [vk::DescriptorSetLayoutBinding; 2],
    object_layout: vk::DescriptorSetLayoutCreateInfo,
    object_pool_sizes: [vk::DescriptorPoolSize; 1],
    object_pool: vk::DescriptorPoolCreateInfo,
    deferred_bindings: [vk::DescriptorSetLayoutBinding; 2],
    deferred_layout: vk::DescriptorSetLayoutCreateInfo,
    deferred_pool_sizes: [vk::DescriptorPoolSize; 2],
    deferred_pool: vk::DescriptorPoolCreateInfo,
    gaussian_horizontal_bindings: [vk::DescriptorSetLayoutBinding; 1],
    gaussian_horizontal_layout: vk::DescriptorSetLayoutCreateInfo,
    gaussian_horizontal_pool_sizes: [vk::DescriptorPoolSize; 1],
    gaussian_horizontal_pool: vk::DescriptorPoolCreateInfo,
    gaussian_vertical_bindings: [vk::DescriptorSetLayoutBinding; 1],
    gaussian_vertical_layout: vk::DescriptorSetLayoutCreateInfo,
    gaussian_vertical_pool_sizes: [vk::DescriptorPoolSize; 1],
    gaussian_vertical_pool: vk::DescriptorPoolCreateInfo,
    postprocess_bindings: [vk::DescriptorSetLayoutBinding; 2],
    postprocess_layout: vk::DescriptorSetLayoutCreateInfo,
    postprocess_pool_sizes: [vk::DescriptorPoolSize; 1],
    postprocess_pool: vk::DescriptorPoolCreateInfo,
    global_bindings: [vk::DescriptorSetLayoutBinding; 2],
    global_layout: vk::DescriptorSetLayoutCreateInfo,
    global_pool_sizes: [vk::DescriptorPoolSize; 2],
    global_pool: vk::DescriptorPoolCreateInfo,
    assembly: vk::PipelineInputAssemblyStateCreateInfo,
    dynamic_state: vk::PipelineDynamicStateCreateInfo,
    pub object_layouts: [vk::DescriptorSetLayout; 2],
    object_pipeline_layout: vk::PipelineLayoutCreateInfo,
    object_shader_vertex: vk::ShaderModuleCreateInfo,
    object_shader_fragment: vk::ShaderModuleCreateInfo,
    object_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    object_vertex_bindings: [vk::VertexInputBindingDescription; 1],
    object_vertex_attributes: [vk::VertexInputAttributeDescription; 2],
    object_vertex: vk::PipelineVertexInputStateCreateInfo,
    object_viewport: vk::Viewport,
    object_scissor: vk::Rect2D,
    object_viewport_state: vk::PipelineViewportStateCreateInfo,
    object_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    object_multisampling: vk::PipelineMultisampleStateCreateInfo,
    object_blend_attachments: [vk::PipelineColorBlendAttachmentState; 1],
    object_blend: vk::PipelineColorBlendStateCreateInfo,
    object_depth: vk::PipelineDepthStencilStateCreateInfo,
    pub grass_layouts: [vk::DescriptorSetLayout; 2],
    grass_pipeline_layout: vk::PipelineLayoutCreateInfo,
    grass_shader_vertex: vk::ShaderModuleCreateInfo,
    grass_shader_fragment: vk::ShaderModuleCreateInfo,
    grass_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    grass_vertex_bindings: [vk::VertexInputBindingDescription; 2],
    grass_vertex_attributes: [vk::VertexInputAttributeDescription; 8],
    grass_vertex: vk::PipelineVertexInputStateCreateInfo,
    grass_viewport: vk::Viewport,
    grass_scissor: vk::Rect2D,
    grass_viewport_state: vk::PipelineViewportStateCreateInfo,
    grass_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    grass_multisampling: vk::PipelineMultisampleStateCreateInfo,
    grass_blend_attachments: [vk::PipelineColorBlendAttachmentState; 1],
    grass_blend: vk::PipelineColorBlendStateCreateInfo,
    grass_depth: vk::PipelineDepthStencilStateCreateInfo,
    pub star_layouts: [vk::DescriptorSetLayout; 2],
    star_pipeline_layout: vk::PipelineLayoutCreateInfo,
    star_shader_vertex: vk::ShaderModuleCreateInfo,
    star_shader_fragment: vk::ShaderModuleCreateInfo,
    star_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    star_vertex_bindings: [vk::VertexInputBindingDescription; 2],
    star_vertex_attributes: [vk::VertexInputAttributeDescription; 6],
    star_vertex: vk::PipelineVertexInputStateCreateInfo,
    star_viewport: vk::Viewport,
    star_scissor: vk::Rect2D,
    star_viewport_state: vk::PipelineViewportStateCreateInfo,
    star_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    star_multisampling: vk::PipelineMultisampleStateCreateInfo,
    star_blend_attachments: [vk::PipelineColorBlendAttachmentState; 1],
    star_blend: vk::PipelineColorBlendStateCreateInfo,
    star_depth: vk::PipelineDepthStencilStateCreateInfo,
    pub skybox_layouts: [vk::DescriptorSetLayout; 2],
    skybox_pipeline_layout: vk::PipelineLayoutCreateInfo,
    skybox_shader_vertex: vk::ShaderModuleCreateInfo,
    skybox_shader_fragment: vk::ShaderModuleCreateInfo,
    skybox_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    skybox_vertex_bindings: [vk::VertexInputBindingDescription; 1],
    skybox_vertex_attributes: [vk::VertexInputAttributeDescription; 1],
    skybox_vertex: vk::PipelineVertexInputStateCreateInfo,
    skybox_viewport: vk::Viewport,
    skybox_scissor: vk::Rect2D,
    skybox_viewport_state: vk::PipelineViewportStateCreateInfo,
    skybox_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    skybox_multisampling: vk::PipelineMultisampleStateCreateInfo,
    skybox_blend_attachments: [vk::PipelineColorBlendAttachmentState; 1],
    skybox_blend: vk::PipelineColorBlendStateCreateInfo,
    skybox_depth: vk::PipelineDepthStencilStateCreateInfo,
    pub deferred_layouts: [vk::DescriptorSetLayout; 2],
    deferred_pipeline_layout: vk::PipelineLayoutCreateInfo,
    deferred_shader_vertex: vk::ShaderModuleCreateInfo,
    deferred_shader_fragment: vk::ShaderModuleCreateInfo,
    deferred_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    deferred_vertex_bindings: [vk::VertexInputBindingDescription; 0],
    deferred_vertex_attributes: [vk::VertexInputAttributeDescription; 0],
    deferred_vertex: vk::PipelineVertexInputStateCreateInfo,
    deferred_viewport: vk::Viewport,
    deferred_scissor: vk::Rect2D,
    deferred_viewport_state: vk::PipelineViewportStateCreateInfo,
    deferred_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    deferred_multisampling: vk::PipelineMultisampleStateCreateInfo,
    deferred_blend_attachments: [vk::PipelineColorBlendAttachmentState; 0],
    deferred_blend: vk::PipelineColorBlendStateCreateInfo,
    deferred_depth: vk::PipelineDepthStencilStateCreateInfo,
    pub gaussian_horizontal_layouts: [vk::DescriptorSetLayout; 2],
    gaussian_horizontal_pipeline_layout: vk::PipelineLayoutCreateInfo,
    gaussian_horizontal_shader_vertex: vk::ShaderModuleCreateInfo,
    gaussian_horizontal_shader_fragment: vk::ShaderModuleCreateInfo,
    gaussian_horizontal_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    gaussian_horizontal_vertex_bindings: [vk::VertexInputBindingDescription; 0],
    gaussian_horizontal_vertex_attributes: [vk::VertexInputAttributeDescription; 0],
    gaussian_horizontal_vertex: vk::PipelineVertexInputStateCreateInfo,
    gaussian_horizontal_viewport: vk::Viewport,
    gaussian_horizontal_scissor: vk::Rect2D,
    gaussian_horizontal_viewport_state: vk::PipelineViewportStateCreateInfo,
    gaussian_horizontal_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    gaussian_horizontal_multisampling: vk::PipelineMultisampleStateCreateInfo,
    gaussian_horizontal_blend_attachments: [vk::PipelineColorBlendAttachmentState; 1],
    gaussian_horizontal_blend: vk::PipelineColorBlendStateCreateInfo,
    gaussian_horizontal_depth: vk::PipelineDepthStencilStateCreateInfo,
    pub gaussian_vertical_layouts: [vk::DescriptorSetLayout; 2],
    gaussian_vertical_pipeline_layout: vk::PipelineLayoutCreateInfo,
    gaussian_vertical_shader_vertex: vk::ShaderModuleCreateInfo,
    gaussian_vertical_shader_fragment: vk::ShaderModuleCreateInfo,
    gaussian_vertical_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    gaussian_vertical_vertex_bindings: [vk::VertexInputBindingDescription; 0],
    gaussian_vertical_vertex_attributes: [vk::VertexInputAttributeDescription; 0],
    gaussian_vertical_vertex: vk::PipelineVertexInputStateCreateInfo,
    gaussian_vertical_viewport: vk::Viewport,
    gaussian_vertical_scissor: vk::Rect2D,
    gaussian_vertical_viewport_state: vk::PipelineViewportStateCreateInfo,
    gaussian_vertical_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    gaussian_vertical_multisampling: vk::PipelineMultisampleStateCreateInfo,
    gaussian_vertical_blend_attachments: [vk::PipelineColorBlendAttachmentState; 1],
    gaussian_vertical_blend: vk::PipelineColorBlendStateCreateInfo,
    gaussian_vertical_depth: vk::PipelineDepthStencilStateCreateInfo,
    pub postprocess_layouts: [vk::DescriptorSetLayout; 2],
    postprocess_pipeline_layout: vk::PipelineLayoutCreateInfo,
    postprocess_shader_vertex: vk::ShaderModuleCreateInfo,
    postprocess_shader_fragment: vk::ShaderModuleCreateInfo,
    postprocess_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    postprocess_vertex_bindings: [vk::VertexInputBindingDescription; 0],
    postprocess_vertex_attributes: [vk::VertexInputAttributeDescription; 0],
    postprocess_vertex: vk::PipelineVertexInputStateCreateInfo,
    postprocess_viewport: vk::Viewport,
    postprocess_scissor: vk::Rect2D,
    postprocess_viewport_state: vk::PipelineViewportStateCreateInfo,
    postprocess_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    postprocess_multisampling: vk::PipelineMultisampleStateCreateInfo,
    postprocess_blend_attachments: [vk::PipelineColorBlendAttachmentState; 1],
    postprocess_blend: vk::PipelineColorBlendStateCreateInfo,
    postprocess_depth: vk::PipelineDepthStencilStateCreateInfo,
    object_pipeline: vk::GraphicsPipelineCreateInfo,
    grass_pipeline: vk::GraphicsPipelineCreateInfo,
    star_pipeline: vk::GraphicsPipelineCreateInfo,
    skybox_pipeline: vk::GraphicsPipelineCreateInfo,
    deferred_pipeline: vk::GraphicsPipelineCreateInfo,
    gaussian_horizontal_pipeline: vk::GraphicsPipelineCreateInfo,
    gaussian_vertical_pipeline: vk::GraphicsPipelineCreateInfo,
    postprocess_pipeline: vk::GraphicsPipelineCreateInfo,
}

#[rustfmt::skip]
static mut SCRATCH: Scratch = Scratch {
    nearest_sampler: vk::SamplerCreateInfo {
        s_type: vk::StructureType::SAMPLER_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::SamplerCreateFlags::empty(),
        mag_filter: vk::Filter::NEAREST,
        min_filter: vk::Filter::NEAREST,
        mipmap_mode: vk::SamplerMipmapMode::NEAREST,
        address_mode_u: vk::SamplerAddressMode::CLAMP_TO_BORDER,
        address_mode_v: vk::SamplerAddressMode::CLAMP_TO_BORDER,
        address_mode_w: vk::SamplerAddressMode::CLAMP_TO_BORDER,
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
    bilinear_sampler: vk::SamplerCreateInfo {
        s_type: vk::StructureType::SAMPLER_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::SamplerCreateFlags::empty(),
        mag_filter: vk::Filter::LINEAR,
        min_filter: vk::Filter::LINEAR,
        mipmap_mode: vk::SamplerMipmapMode::NEAREST,
        address_mode_u: vk::SamplerAddressMode::CLAMP_TO_BORDER,
        address_mode_v: vk::SamplerAddressMode::CLAMP_TO_BORDER,
        address_mode_w: vk::SamplerAddressMode::CLAMP_TO_BORDER,
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
    render_rasterization_color: [
        vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        },
    ],
    render_rasterization_depth: vk::AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    },
    render_deferred_input: [
        vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        },
    ],
    render_attachments: [
        vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: COLOR_FORMAT,
            samples: vk::SampleCountFlags::TYPE_2,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        },
        vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: DEPTH_FORMAT,
            samples: vk::SampleCountFlags::TYPE_2,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        },
    ],
    render_subpasses: [
        vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: std::ptr::null(),
            color_attachment_count: 1,
            p_color_attachments: unsafe { SCRATCH.render_rasterization_color.as_ptr() },
            p_resolve_attachments: std::ptr::null(),
            p_depth_stencil_attachment: unsafe { &SCRATCH.render_rasterization_depth },
            preserve_attachment_count: 0,
            p_preserve_attachments: std::ptr::null(),
        },
        vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 1,
            p_input_attachments: unsafe { SCRATCH.render_deferred_input.as_ptr() },
            color_attachment_count: 0,
            p_color_attachments: std::ptr::null(),
            p_resolve_attachments: std::ptr::null(),
            p_depth_stencil_attachment: std::ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: std::ptr::null(),
        },
    ],
    render_dependencies: [
        vk::SubpassDependency {
            src_subpass: 0,
            dst_subpass: 1,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
            src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_access_mask: vk::AccessFlags::INPUT_ATTACHMENT_READ,
            dependency_flags: vk::DependencyFlags::BY_REGION,
        },
    ],
    render_pass: vk::RenderPassCreateInfo {
        s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::RenderPassCreateFlags::empty(),
        attachment_count: 2,
        p_attachments: unsafe { SCRATCH.render_attachments.as_ptr() },
        subpass_count: 2,
        p_subpasses: unsafe { SCRATCH.render_subpasses.as_ptr() },
        dependency_count: 1,
        p_dependencies: unsafe { SCRATCH.render_dependencies.as_ptr() },
    },
    gaussian_horizontal_gaussian_color: [
        vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        },
    ],
    gaussian_horizontal_attachments: [
        vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: COLOR_FORMAT,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::DONT_CARE,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        },
    ],
    gaussian_horizontal_subpasses: [
        vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: std::ptr::null(),
            color_attachment_count: 1,
            p_color_attachments: unsafe { SCRATCH.gaussian_horizontal_gaussian_color.as_ptr() },
            p_resolve_attachments: std::ptr::null(),
            p_depth_stencil_attachment: std::ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: std::ptr::null(),
        },
    ],
    gaussian_horizontal_dependencies: [
    ],
    gaussian_horizontal_pass: vk::RenderPassCreateInfo {
        s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::RenderPassCreateFlags::empty(),
        attachment_count: 1,
        p_attachments: unsafe { SCRATCH.gaussian_horizontal_attachments.as_ptr() },
        subpass_count: 1,
        p_subpasses: unsafe { SCRATCH.gaussian_horizontal_subpasses.as_ptr() },
        dependency_count: 0,
        p_dependencies: unsafe { SCRATCH.gaussian_horizontal_dependencies.as_ptr() },
    },
    gaussian_vertical_gaussian_color: [
        vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        },
    ],
    gaussian_vertical_attachments: [
        vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: COLOR_FORMAT,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::DONT_CARE,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        },
    ],
    gaussian_vertical_subpasses: [
        vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: std::ptr::null(),
            color_attachment_count: 1,
            p_color_attachments: unsafe { SCRATCH.gaussian_vertical_gaussian_color.as_ptr() },
            p_resolve_attachments: std::ptr::null(),
            p_depth_stencil_attachment: std::ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: std::ptr::null(),
        },
    ],
    gaussian_vertical_dependencies: [
    ],
    gaussian_vertical_pass: vk::RenderPassCreateInfo {
        s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::RenderPassCreateFlags::empty(),
        attachment_count: 1,
        p_attachments: unsafe { SCRATCH.gaussian_vertical_attachments.as_ptr() },
        subpass_count: 1,
        p_subpasses: unsafe { SCRATCH.gaussian_vertical_subpasses.as_ptr() },
        dependency_count: 0,
        p_dependencies: unsafe { SCRATCH.gaussian_vertical_dependencies.as_ptr() },
    },
    postprocess_postprocess_color: [
        vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        },
    ],
    postprocess_attachments: [
        vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: vk::Format::UNDEFINED,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::DONT_CARE,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        },
    ],
    postprocess_subpasses: [
        vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: std::ptr::null(),
            color_attachment_count: 1,
            p_color_attachments: unsafe { SCRATCH.postprocess_postprocess_color.as_ptr() },
            p_resolve_attachments: std::ptr::null(),
            p_depth_stencil_attachment: std::ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: std::ptr::null(),
        },
    ],
    postprocess_dependencies: [
    ],
    postprocess_pass: vk::RenderPassCreateInfo {
        s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::RenderPassCreateFlags::empty(),
        attachment_count: 1,
        p_attachments: unsafe { SCRATCH.postprocess_attachments.as_ptr() },
        subpass_count: 1,
        p_subpasses: unsafe { SCRATCH.postprocess_subpasses.as_ptr() },
        dependency_count: 0,
        p_dependencies: unsafe { SCRATCH.postprocess_dependencies.as_ptr() },
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
    ],
    object_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 2,
        p_bindings: unsafe { SCRATCH.object_bindings.as_ptr() },
    },
    object_pool_sizes: [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 20,
        },
    ],
    object_pool: vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: 10,
        pool_size_count: 1,
        p_pool_sizes: unsafe { SCRATCH.object_pool_sizes.as_ptr() },
    },
    deferred_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::INPUT_ATTACHMENT,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::STORAGE_IMAGE,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    deferred_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 2,
        p_bindings: unsafe { SCRATCH.deferred_bindings.as_ptr() },
    },
    deferred_pool_sizes: [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::INPUT_ATTACHMENT,
            descriptor_count: 2,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::STORAGE_IMAGE,
            descriptor_count: 2,
        },
    ],
    deferred_pool: vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: 2,
        pool_size_count: 2,
        p_pool_sizes: unsafe { SCRATCH.deferred_pool_sizes.as_ptr() },
    },
    gaussian_horizontal_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    gaussian_horizontal_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 1,
        p_bindings: unsafe { SCRATCH.gaussian_horizontal_bindings.as_ptr() },
    },
    gaussian_horizontal_pool_sizes: [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 2,
        },
    ],
    gaussian_horizontal_pool: vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: 2,
        pool_size_count: 1,
        p_pool_sizes: unsafe { SCRATCH.gaussian_horizontal_pool_sizes.as_ptr() },
    },
    gaussian_vertical_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    gaussian_vertical_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 1,
        p_bindings: unsafe { SCRATCH.gaussian_vertical_bindings.as_ptr() },
    },
    gaussian_vertical_pool_sizes: [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 2,
        },
    ],
    gaussian_vertical_pool: vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: 2,
        pool_size_count: 1,
        p_pool_sizes: unsafe { SCRATCH.gaussian_vertical_pool_sizes.as_ptr() },
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
    ],
    postprocess_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 2,
        p_bindings: unsafe { SCRATCH.postprocess_bindings.as_ptr() },
    },
    postprocess_pool_sizes: [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 4,
        },
    ],
    postprocess_pool: vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: 2,
        pool_size_count: 1,
        p_pool_sizes: unsafe { SCRATCH.postprocess_pool_sizes.as_ptr() },
    },
    global_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::ALL,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    global_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 2,
        p_bindings: unsafe { SCRATCH.global_bindings.as_ptr() },
    },
    global_pool_sizes: [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 2,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
            descriptor_count: 2,
        },
    ],
    global_pool: vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: 2,
        pool_size_count: 2,
        p_pool_sizes: unsafe { SCRATCH.global_pool_sizes.as_ptr() },
    },
    assembly: vk::PipelineInputAssemblyStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        primitive_restart_enable: 0,
    },
    dynamic_state: vk::PipelineDynamicStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDynamicStateCreateFlags::empty(),
        dynamic_state_count: 0,
        p_dynamic_states: std::ptr::null(),
    },
    object_layouts: [vk::DescriptorSetLayout::null(); 2],
    object_pipeline_layout: vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 2,
        p_set_layouts: unsafe { SCRATCH.object_layouts.as_ptr() },
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    },
    object_shader_vertex: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    },
    object_shader_fragment: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
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
    object_viewport: vk::Viewport {
        x: 0.,
        y: 0.,
        width: 0.,
        height: 0.,
        min_depth: 0.,
        max_depth: 1.,
    },
    object_scissor: vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D { width: 0, height: 0 },
    },
    object_viewport_state: vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        p_viewports: unsafe { &SCRATCH.object_viewport },
        scissor_count: 1,
        p_scissors: unsafe { &SCRATCH.object_scissor },
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
    ],
    object_blend: vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: 1,
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
        p_viewport_state: unsafe { &SCRATCH.object_viewport_state },
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
    grass_layouts: [vk::DescriptorSetLayout::null(); 2],
    grass_pipeline_layout: vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 2,
        p_set_layouts: unsafe { SCRATCH.grass_layouts.as_ptr() },
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    },
    grass_shader_vertex: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    },
    grass_shader_fragment: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
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
    grass_viewport: vk::Viewport {
        x: 0.,
        y: 0.,
        width: 0.,
        height: 0.,
        min_depth: 0.,
        max_depth: 1.,
    },
    grass_scissor: vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D { width: 0, height: 0 },
    },
    grass_viewport_state: vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        p_viewports: unsafe { &SCRATCH.grass_viewport },
        scissor_count: 1,
        p_scissors: unsafe { &SCRATCH.grass_scissor },
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
    ],
    grass_blend: vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: 1,
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
        p_viewport_state: unsafe { &SCRATCH.grass_viewport_state },
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
    star_layouts: [vk::DescriptorSetLayout::null(); 2],
    star_pipeline_layout: vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 2,
        p_set_layouts: unsafe { SCRATCH.star_layouts.as_ptr() },
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    },
    star_shader_vertex: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    },
    star_shader_fragment: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    },
    star_shader_stages: [
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
    star_vertex_bindings: [
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: 24,
            input_rate: vk::VertexInputRate::VERTEX,
        },
        vk::VertexInputBindingDescription {
            binding: 1,
            stride: 76,
            input_rate: vk::VertexInputRate::INSTANCE,
        },
    ],
    star_vertex_attributes: [
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            binding: 1,
            location: 1,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            binding: 1,
            location: 2,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: 16,
        },
        vk::VertexInputAttributeDescription {
            binding: 1,
            location: 3,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: 32,
        },
        vk::VertexInputAttributeDescription {
            binding: 1,
            location: 4,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: 48,
        },
        vk::VertexInputAttributeDescription {
            binding: 1,
            location: 5,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 64,
        },
    ],
    star_vertex: vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: 2,
        p_vertex_binding_descriptions: unsafe { SCRATCH.star_vertex_bindings.as_ptr() },
        vertex_attribute_description_count: 6,
        p_vertex_attribute_descriptions: unsafe { SCRATCH.star_vertex_attributes.as_ptr() },
    },
    star_viewport: vk::Viewport {
        x: 0.,
        y: 0.,
        width: 0.,
        height: 0.,
        min_depth: 0.,
        max_depth: 1.,
    },
    star_scissor: vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D { width: 0, height: 0 },
    },
    star_viewport_state: vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        p_viewports: unsafe { &SCRATCH.star_viewport },
        scissor_count: 1,
        p_scissors: unsafe { &SCRATCH.star_scissor },
    },
    star_rasterizer: vk::PipelineRasterizationStateCreateInfo {
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
    star_multisampling: vk::PipelineMultisampleStateCreateInfo {
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
    star_blend_attachments: [
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
    star_blend: vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: 1,
        p_attachments: unsafe { SCRATCH.star_blend_attachments.as_ptr() },
        blend_constants: [0., 0., 0., 0.],
    },
    star_depth: vk::PipelineDepthStencilStateCreateInfo {
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
    star_pipeline: vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: 2,
        p_stages: unsafe { SCRATCH.star_shader_stages.as_ptr() },
        p_vertex_input_state: unsafe { &SCRATCH.star_vertex },
        p_input_assembly_state: unsafe { &SCRATCH.assembly },
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: unsafe { &SCRATCH.star_viewport_state },
        p_rasterization_state: unsafe { &SCRATCH.star_rasterizer },
        p_multisample_state: unsafe { &SCRATCH.star_multisampling },
        p_depth_stencil_state: unsafe { &SCRATCH.star_depth },
        p_color_blend_state: unsafe { &SCRATCH.star_blend },
        p_dynamic_state: unsafe { &SCRATCH.dynamic_state },
        layout: vk::PipelineLayout::null(),
        render_pass: vk::RenderPass::null(),
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    },
    skybox_layouts: [vk::DescriptorSetLayout::null(); 2],
    skybox_pipeline_layout: vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 2,
        p_set_layouts: unsafe { SCRATCH.skybox_layouts.as_ptr() },
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    },
    skybox_shader_vertex: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    },
    skybox_shader_fragment: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
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
    skybox_viewport: vk::Viewport {
        x: 0.,
        y: 0.,
        width: 0.,
        height: 0.,
        min_depth: 0.,
        max_depth: 1.,
    },
    skybox_scissor: vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D { width: 0, height: 0 },
    },
    skybox_viewport_state: vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        p_viewports: unsafe { &SCRATCH.skybox_viewport },
        scissor_count: 1,
        p_scissors: unsafe { &SCRATCH.skybox_scissor },
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
    ],
    skybox_blend: vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: 1,
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
        p_viewport_state: unsafe { &SCRATCH.skybox_viewport_state },
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
    deferred_layouts: [vk::DescriptorSetLayout::null(); 2],
    deferred_pipeline_layout: vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 2,
        p_set_layouts: unsafe { SCRATCH.deferred_layouts.as_ptr() },
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    },
    deferred_shader_vertex: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    },
    deferred_shader_fragment: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    },
    deferred_shader_stages: [
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
    deferred_vertex_bindings: [
    ],
    deferred_vertex_attributes: [
    ],
    deferred_vertex: vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: 0,
        p_vertex_binding_descriptions: unsafe { SCRATCH.deferred_vertex_bindings.as_ptr() },
        vertex_attribute_description_count: 0,
        p_vertex_attribute_descriptions: unsafe { SCRATCH.deferred_vertex_attributes.as_ptr() },
    },
    deferred_viewport: vk::Viewport {
        x: 0.,
        y: 0.,
        width: 0.,
        height: 0.,
        min_depth: 0.,
        max_depth: 1.,
    },
    deferred_scissor: vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D { width: 0, height: 0 },
    },
    deferred_viewport_state: vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        p_viewports: unsafe { &SCRATCH.deferred_viewport },
        scissor_count: 1,
        p_scissors: unsafe { &SCRATCH.deferred_scissor },
    },
    deferred_rasterizer: vk::PipelineRasterizationStateCreateInfo {
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
    deferred_multisampling: vk::PipelineMultisampleStateCreateInfo {
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
    deferred_blend_attachments: [
    ],
    deferred_blend: vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: 0,
        p_attachments: unsafe { SCRATCH.deferred_blend_attachments.as_ptr() },
        blend_constants: [0., 0., 0., 0.],
    },
    deferred_depth: vk::PipelineDepthStencilStateCreateInfo {
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
    deferred_pipeline: vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: 2,
        p_stages: unsafe { SCRATCH.deferred_shader_stages.as_ptr() },
        p_vertex_input_state: unsafe { &SCRATCH.deferred_vertex },
        p_input_assembly_state: unsafe { &SCRATCH.assembly },
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: unsafe { &SCRATCH.deferred_viewport_state },
        p_rasterization_state: unsafe { &SCRATCH.deferred_rasterizer },
        p_multisample_state: unsafe { &SCRATCH.deferred_multisampling },
        p_depth_stencil_state: unsafe { &SCRATCH.deferred_depth },
        p_color_blend_state: unsafe { &SCRATCH.deferred_blend },
        p_dynamic_state: unsafe { &SCRATCH.dynamic_state },
        layout: vk::PipelineLayout::null(),
        render_pass: vk::RenderPass::null(),
        subpass: 1,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    },
    gaussian_horizontal_layouts: [vk::DescriptorSetLayout::null(); 2],
    gaussian_horizontal_pipeline_layout: vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 2,
        p_set_layouts: unsafe { SCRATCH.gaussian_horizontal_layouts.as_ptr() },
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    },
    gaussian_horizontal_shader_vertex: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    },
    gaussian_horizontal_shader_fragment: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    },
    gaussian_horizontal_shader_stages: [
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
    gaussian_horizontal_vertex_bindings: [
    ],
    gaussian_horizontal_vertex_attributes: [
    ],
    gaussian_horizontal_vertex: vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: 0,
        p_vertex_binding_descriptions: unsafe { SCRATCH.gaussian_horizontal_vertex_bindings.as_ptr() },
        vertex_attribute_description_count: 0,
        p_vertex_attribute_descriptions: unsafe { SCRATCH.gaussian_horizontal_vertex_attributes.as_ptr() },
    },
    gaussian_horizontal_viewport: vk::Viewport {
        x: 0.,
        y: 0.,
        width: 0.,
        height: 0.,
        min_depth: 0.,
        max_depth: 1.,
    },
    gaussian_horizontal_scissor: vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D { width: 0, height: 0 },
    },
    gaussian_horizontal_viewport_state: vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        p_viewports: unsafe { &SCRATCH.gaussian_horizontal_viewport },
        scissor_count: 1,
        p_scissors: unsafe { &SCRATCH.gaussian_horizontal_scissor },
    },
    gaussian_horizontal_rasterizer: vk::PipelineRasterizationStateCreateInfo {
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
    gaussian_horizontal_multisampling: vk::PipelineMultisampleStateCreateInfo {
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
    gaussian_horizontal_blend_attachments: [
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
    gaussian_horizontal_blend: vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: 1,
        p_attachments: unsafe { SCRATCH.gaussian_horizontal_blend_attachments.as_ptr() },
        blend_constants: [0., 0., 0., 0.],
    },
    gaussian_horizontal_depth: vk::PipelineDepthStencilStateCreateInfo {
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
    gaussian_horizontal_pipeline: vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: 2,
        p_stages: unsafe { SCRATCH.gaussian_horizontal_shader_stages.as_ptr() },
        p_vertex_input_state: unsafe { &SCRATCH.gaussian_horizontal_vertex },
        p_input_assembly_state: unsafe { &SCRATCH.assembly },
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: unsafe { &SCRATCH.gaussian_horizontal_viewport_state },
        p_rasterization_state: unsafe { &SCRATCH.gaussian_horizontal_rasterizer },
        p_multisample_state: unsafe { &SCRATCH.gaussian_horizontal_multisampling },
        p_depth_stencil_state: unsafe { &SCRATCH.gaussian_horizontal_depth },
        p_color_blend_state: unsafe { &SCRATCH.gaussian_horizontal_blend },
        p_dynamic_state: unsafe { &SCRATCH.dynamic_state },
        layout: vk::PipelineLayout::null(),
        render_pass: vk::RenderPass::null(),
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    },
    gaussian_vertical_layouts: [vk::DescriptorSetLayout::null(); 2],
    gaussian_vertical_pipeline_layout: vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 2,
        p_set_layouts: unsafe { SCRATCH.gaussian_vertical_layouts.as_ptr() },
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    },
    gaussian_vertical_shader_vertex: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    },
    gaussian_vertical_shader_fragment: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    },
    gaussian_vertical_shader_stages: [
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
    gaussian_vertical_vertex_bindings: [
    ],
    gaussian_vertical_vertex_attributes: [
    ],
    gaussian_vertical_vertex: vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: 0,
        p_vertex_binding_descriptions: unsafe { SCRATCH.gaussian_vertical_vertex_bindings.as_ptr() },
        vertex_attribute_description_count: 0,
        p_vertex_attribute_descriptions: unsafe { SCRATCH.gaussian_vertical_vertex_attributes.as_ptr() },
    },
    gaussian_vertical_viewport: vk::Viewport {
        x: 0.,
        y: 0.,
        width: 0.,
        height: 0.,
        min_depth: 0.,
        max_depth: 1.,
    },
    gaussian_vertical_scissor: vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D { width: 0, height: 0 },
    },
    gaussian_vertical_viewport_state: vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        p_viewports: unsafe { &SCRATCH.gaussian_vertical_viewport },
        scissor_count: 1,
        p_scissors: unsafe { &SCRATCH.gaussian_vertical_scissor },
    },
    gaussian_vertical_rasterizer: vk::PipelineRasterizationStateCreateInfo {
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
    gaussian_vertical_multisampling: vk::PipelineMultisampleStateCreateInfo {
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
    gaussian_vertical_blend_attachments: [
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
    gaussian_vertical_blend: vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: 1,
        p_attachments: unsafe { SCRATCH.gaussian_vertical_blend_attachments.as_ptr() },
        blend_constants: [0., 0., 0., 0.],
    },
    gaussian_vertical_depth: vk::PipelineDepthStencilStateCreateInfo {
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
    gaussian_vertical_pipeline: vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: 2,
        p_stages: unsafe { SCRATCH.gaussian_vertical_shader_stages.as_ptr() },
        p_vertex_input_state: unsafe { &SCRATCH.gaussian_vertical_vertex },
        p_input_assembly_state: unsafe { &SCRATCH.assembly },
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: unsafe { &SCRATCH.gaussian_vertical_viewport_state },
        p_rasterization_state: unsafe { &SCRATCH.gaussian_vertical_rasterizer },
        p_multisample_state: unsafe { &SCRATCH.gaussian_vertical_multisampling },
        p_depth_stencil_state: unsafe { &SCRATCH.gaussian_vertical_depth },
        p_color_blend_state: unsafe { &SCRATCH.gaussian_vertical_blend },
        p_dynamic_state: unsafe { &SCRATCH.dynamic_state },
        layout: vk::PipelineLayout::null(),
        render_pass: vk::RenderPass::null(),
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    },
    postprocess_layouts: [vk::DescriptorSetLayout::null(); 2],
    postprocess_pipeline_layout: vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 2,
        p_set_layouts: unsafe { SCRATCH.postprocess_layouts.as_ptr() },
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    },
    postprocess_shader_vertex: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    },
    postprocess_shader_fragment: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
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
    postprocess_viewport: vk::Viewport {
        x: 0.,
        y: 0.,
        width: 0.,
        height: 0.,
        min_depth: 0.,
        max_depth: 1.,
    },
    postprocess_scissor: vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D { width: 0, height: 0 },
    },
    postprocess_viewport_state: vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        p_viewports: unsafe { &SCRATCH.postprocess_viewport },
        scissor_count: 1,
        p_scissors: unsafe { &SCRATCH.postprocess_scissor },
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
        p_viewport_state: unsafe { &SCRATCH.postprocess_viewport_state },
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
        unsafe { dev.destroy_sampler(self.nearest, None) };
        unsafe { dev.destroy_sampler(self.bilinear, None) };
    }
}

impl DescriptorSetLayouts {
    pub fn cleanup(&self, dev: &Dev) {
        unsafe { dev.destroy_descriptor_set_layout(self.object, None) };
        unsafe { dev.destroy_descriptor_set_layout(self.deferred, None) };
        unsafe { dev.destroy_descriptor_set_layout(self.gaussian_horizontal, None) };
        unsafe { dev.destroy_descriptor_set_layout(self.gaussian_vertical, None) };
        unsafe { dev.destroy_descriptor_set_layout(self.postprocess, None) };
        unsafe { dev.destroy_descriptor_set_layout(self.global, None) };
    }
}

#[rustfmt::skip]
impl DescriptorPools {
    pub fn alloc_object(
        &self,
        mvp: &UniformBuffer<ModelViewProjection>,
        material: &UniformBuffer<Material>,
        dev: &Dev,
    ) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
        let layouts = [self.object_layout; FRAMES_IN_FLIGHT];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.object)
            .set_layouts(&layouts);
        let descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
            unsafe { dev.allocate_descriptor_sets(&descriptor_set_alloc_info) }
                .unwrap()
                .try_into()
                .unwrap();
        self.update_object(&descriptors, mvp, material, dev);
        descriptors
    }

    pub fn update_object(
        &self,
        descriptors: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],
        mvp: &UniformBuffer<ModelViewProjection>,
        material: &UniformBuffer<Material>,
        dev: &Dev,
    ) {
        for (_flight_index, descriptor) in descriptors.iter().enumerate() {
            let mvp_buffer = mvp.descriptor(_flight_index);
            let mvp = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&mvp_buffer));
            let material_buffer = material.descriptor(_flight_index);
            let material = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&material_buffer));
            let writes = [mvp, material];
            unsafe { dev.update_descriptor_sets(&writes, &[]) };
        }
    }

    pub fn alloc_deferred(
        &self,
        render: vk::ImageView,
        bloom: vk::ImageView,
        dev: &Dev,
    ) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
        let layouts = [self.deferred_layout; FRAMES_IN_FLIGHT];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.deferred)
            .set_layouts(&layouts);
        let descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
            unsafe { dev.allocate_descriptor_sets(&descriptor_set_alloc_info) }
                .unwrap()
                .try_into()
                .unwrap();
        self.update_deferred(&descriptors, render, bloom, dev);
        descriptors
    }

    pub fn update_deferred(
        &self,
        descriptors: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],
        render: vk::ImageView,
        bloom: vk::ImageView,
        dev: &Dev,
    ) {
        for (_flight_index, descriptor) in descriptors.iter().enumerate() {
            let render_image = *vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(render);
            let render = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(std::slice::from_ref(&render_image));
            let bloom_image = *vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::GENERAL)
                .image_view(bloom);
            let bloom = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(std::slice::from_ref(&bloom_image));
            let writes = [render, bloom];
            unsafe { dev.update_descriptor_sets(&writes, &[]) };
        }
    }

    pub fn alloc_gaussian_horizontal(
        &self,
        render: vk::ImageView,
        dev: &Dev,
    ) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
        let layouts = [self.gaussian_horizontal_layout; FRAMES_IN_FLIGHT];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.gaussian_horizontal)
            .set_layouts(&layouts);
        let descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
            unsafe { dev.allocate_descriptor_sets(&descriptor_set_alloc_info) }
                .unwrap()
                .try_into()
                .unwrap();
        self.update_gaussian_horizontal(&descriptors, render, dev);
        descriptors
    }

    pub fn update_gaussian_horizontal(
        &self,
        descriptors: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],
        render: vk::ImageView,
        dev: &Dev,
    ) {
        for (_flight_index, descriptor) in descriptors.iter().enumerate() {
            let render_image = *vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::GENERAL)
                .image_view(render);
            let render = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&render_image));
            let writes = [render];
            unsafe { dev.update_descriptor_sets(&writes, &[]) };
        }
    }

    pub fn alloc_gaussian_vertical(
        &self,
        render: vk::ImageView,
        dev: &Dev,
    ) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
        let layouts = [self.gaussian_vertical_layout; FRAMES_IN_FLIGHT];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.gaussian_vertical)
            .set_layouts(&layouts);
        let descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
            unsafe { dev.allocate_descriptor_sets(&descriptor_set_alloc_info) }
                .unwrap()
                .try_into()
                .unwrap();
        self.update_gaussian_vertical(&descriptors, render, dev);
        descriptors
    }

    pub fn update_gaussian_vertical(
        &self,
        descriptors: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],
        render: vk::ImageView,
        dev: &Dev,
    ) {
        for (_flight_index, descriptor) in descriptors.iter().enumerate() {
            let render_image = *vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(render);
            let render = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&render_image));
            let writes = [render];
            unsafe { dev.update_descriptor_sets(&writes, &[]) };
        }
    }

    pub fn alloc_postprocess(
        &self,
        render: vk::ImageView,
        bloom: vk::ImageView,
        dev: &Dev,
    ) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
        let layouts = [self.postprocess_layout; FRAMES_IN_FLIGHT];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.postprocess)
            .set_layouts(&layouts);
        let descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
            unsafe { dev.allocate_descriptor_sets(&descriptor_set_alloc_info) }
                .unwrap()
                .try_into()
                .unwrap();
        self.update_postprocess(&descriptors, render, bloom, dev);
        descriptors
    }

    pub fn update_postprocess(
        &self,
        descriptors: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],
        render: vk::ImageView,
        bloom: vk::ImageView,
        dev: &Dev,
    ) {
        for (_flight_index, descriptor) in descriptors.iter().enumerate() {
            let render_image = *vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(render);
            let render = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&render_image));
            let bloom_image = *vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(bloom);
            let bloom = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&bloom_image));
            let writes = [render, bloom];
            unsafe { dev.update_descriptor_sets(&writes, &[]) };
        }
    }

    pub fn alloc_global(
        &self,
        global: &UniformBuffer<Global>,
        tlas: &Option<RaytraceResources>,
        dev: &Dev,
    ) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
        let layouts = [self.global_layout; FRAMES_IN_FLIGHT];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.global)
            .set_layouts(&layouts);
        let descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
            unsafe { dev.allocate_descriptor_sets(&descriptor_set_alloc_info) }
                .unwrap()
                .try_into()
                .unwrap();
        self.update_global(&descriptors, global, tlas, dev);
        descriptors
    }

    pub fn update_global(
        &self,
        descriptors: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],
        global: &UniformBuffer<Global>,
        tlas: &Option<RaytraceResources>,
        dev: &Dev,
    ) {
        for (_flight_index, descriptor) in descriptors.iter().enumerate() {
            let global_buffer = global.descriptor(_flight_index);
            let global = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&global_buffer));
            let mut tlas_acceleration_structure = *vk::WriteDescriptorSetAccelerationStructureKHR::builder()
                .acceleration_structures(tlas.as_ref().map(|as_| std::slice::from_ref(&as_.acceleration_structure)).unwrap_or_default());
            let mut tlas = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                .push_next(&mut tlas_acceleration_structure);
            tlas.descriptor_count = 1;
            let writes = [global, tlas];
            unsafe { dev.update_descriptor_sets(&writes, &[]) };
        }
    }

    pub fn cleanup(&self, dev: &Dev) {
        unsafe { dev.destroy_descriptor_pool(self.object, None) };
        unsafe { dev.destroy_descriptor_pool(self.deferred, None) };
        unsafe { dev.destroy_descriptor_pool(self.gaussian_horizontal, None) };
        unsafe { dev.destroy_descriptor_pool(self.gaussian_vertical, None) };
        unsafe { dev.destroy_descriptor_pool(self.postprocess, None) };
        unsafe { dev.destroy_descriptor_pool(self.global, None) };
    }
}

impl PipelineLayouts {
    pub fn cleanup(&self, dev: &Dev) {
        unsafe { dev.destroy_pipeline_layout(self.object, None) };
        unsafe { dev.destroy_pipeline_layout(self.grass, None) };
        unsafe { dev.destroy_pipeline_layout(self.star, None) };
        unsafe { dev.destroy_pipeline_layout(self.skybox, None) };
        unsafe { dev.destroy_pipeline_layout(self.deferred, None) };
        unsafe { dev.destroy_pipeline_layout(self.gaussian_horizontal, None) };
        unsafe { dev.destroy_pipeline_layout(self.gaussian_vertical, None) };
        unsafe { dev.destroy_pipeline_layout(self.postprocess, None) };
    }
}

impl ShaderModules {
    pub fn cleanup(&self, dev: &Dev) {
        unsafe { dev.destroy_shader_module(self.object_vertex, None) };
        unsafe { dev.destroy_shader_module(self.object_fragment, None) };
        unsafe { dev.destroy_shader_module(self.grass_vertex, None) };
        unsafe { dev.destroy_shader_module(self.grass_fragment, None) };
        unsafe { dev.destroy_shader_module(self.star_vertex, None) };
        unsafe { dev.destroy_shader_module(self.star_fragment, None) };
        unsafe { dev.destroy_shader_module(self.skybox_vertex, None) };
        unsafe { dev.destroy_shader_module(self.skybox_fragment, None) };
        unsafe { dev.destroy_shader_module(self.deferred_vertex, None) };
        unsafe { dev.destroy_shader_module(self.deferred_fragment, None) };
        unsafe { dev.destroy_shader_module(self.gaussian_horizontal_vertex, None) };
        unsafe { dev.destroy_shader_module(self.gaussian_horizontal_fragment, None) };
        unsafe { dev.destroy_shader_module(self.gaussian_vertical_vertex, None) };
        unsafe { dev.destroy_shader_module(self.gaussian_vertical_fragment, None) };
        unsafe { dev.destroy_shader_module(self.postprocess_vertex, None) };
        unsafe { dev.destroy_shader_module(self.postprocess_fragment, None) };
    }
}

impl Passes {
    pub fn cleanup(&self, dev: &Dev) {
        self.render.cleanup(dev);
        self.gaussian_horizontal.cleanup(dev);
        self.gaussian_vertical.cleanup(dev);
        self.postprocess.cleanup(dev);
    }
}

impl Pipelines {
    pub fn cleanup(&self, dev: &Dev) {
        unsafe { dev.destroy_pipeline(self.object, None) };
        unsafe { dev.destroy_pipeline(self.grass, None) };
        unsafe { dev.destroy_pipeline(self.star, None) };
        unsafe { dev.destroy_pipeline(self.skybox, None) };
        unsafe { dev.destroy_pipeline(self.deferred, None) };
        unsafe { dev.destroy_pipeline(self.gaussian_horizontal, None) };
        unsafe { dev.destroy_pipeline(self.gaussian_vertical, None) };
        unsafe { dev.destroy_pipeline(self.postprocess, None) };
    }
}

#[rustfmt::skip]
pub fn create_samplers(dev: &Dev) -> Samplers {
    let nearest = unsafe { dev.create_sampler(&SCRATCH.nearest_sampler, None).unwrap_unchecked() };
    let bilinear = unsafe { dev.create_sampler(&SCRATCH.bilinear_sampler, None).unwrap_unchecked() };
    Samplers {
        nearest,
        bilinear,
    }
}

#[rustfmt::skip]
pub fn create_descriptor_set_layouts(samplers: &Samplers, dev: &Dev) -> DescriptorSetLayouts {
    unsafe { SCRATCH.gaussian_horizontal_bindings[0].p_immutable_samplers = &samplers.nearest };
    unsafe { SCRATCH.gaussian_vertical_bindings[0].p_immutable_samplers = &samplers.nearest };
    unsafe { SCRATCH.postprocess_bindings[0].p_immutable_samplers = &samplers.nearest };
    unsafe { SCRATCH.postprocess_bindings[1].p_immutable_samplers = &samplers.bilinear };
    let object = unsafe { dev.create_descriptor_set_layout(&SCRATCH.object_layout, None).unwrap_unchecked() };
    let deferred = unsafe { dev.create_descriptor_set_layout(&SCRATCH.deferred_layout, None).unwrap_unchecked() };
    let gaussian_horizontal = unsafe { dev.create_descriptor_set_layout(&SCRATCH.gaussian_horizontal_layout, None).unwrap_unchecked() };
    let gaussian_vertical = unsafe { dev.create_descriptor_set_layout(&SCRATCH.gaussian_vertical_layout, None).unwrap_unchecked() };
    let postprocess = unsafe { dev.create_descriptor_set_layout(&SCRATCH.postprocess_layout, None).unwrap_unchecked() };
    let global = unsafe { dev.create_descriptor_set_layout(&SCRATCH.global_layout, None).unwrap_unchecked() };
    DescriptorSetLayouts {
        object,
        deferred,
        gaussian_horizontal,
        gaussian_vertical,
        postprocess,
        global,
    }
}

#[rustfmt::skip]
pub fn create_descriptor_pools(layouts: &DescriptorSetLayouts, dev: &Dev) -> DescriptorPools {
    let object = unsafe { dev.create_descriptor_pool(&SCRATCH.object_pool, None).unwrap_unchecked() };
    let deferred = unsafe { dev.create_descriptor_pool(&SCRATCH.deferred_pool, None).unwrap_unchecked() };
    let gaussian_horizontal = unsafe { dev.create_descriptor_pool(&SCRATCH.gaussian_horizontal_pool, None).unwrap_unchecked() };
    let gaussian_vertical = unsafe { dev.create_descriptor_pool(&SCRATCH.gaussian_vertical_pool, None).unwrap_unchecked() };
    let postprocess = unsafe { dev.create_descriptor_pool(&SCRATCH.postprocess_pool, None).unwrap_unchecked() };
    let global = unsafe { dev.create_descriptor_pool(&SCRATCH.global_pool, None).unwrap_unchecked() };
    DescriptorPools {
        object,
        object_layout: layouts.object,
        deferred,
        deferred_layout: layouts.deferred,
        gaussian_horizontal,
        gaussian_horizontal_layout: layouts.gaussian_horizontal,
        gaussian_vertical,
        gaussian_vertical_layout: layouts.gaussian_vertical,
        postprocess,
        postprocess_layout: layouts.postprocess,
        global,
        global_layout: layouts.global,
    }
}

#[allow(unused_mut)]
#[allow(clippy::identity_op)]
#[rustfmt::skip]
pub fn create_render_passes(
    swapchain: &Swapchain,
    dev: &Dev,
    debug_ext: &DebugUtils,
) -> Passes {
    unsafe { SCRATCH.postprocess_attachments[0].format = swapchain.format.format };
    let render = unsafe { dev.create_render_pass(&SCRATCH.render_pass, None).unwrap_unchecked() };
    let gaussian_horizontal = unsafe { dev.create_render_pass(&SCRATCH.gaussian_horizontal_pass, None).unwrap_unchecked() };
    let gaussian_vertical = unsafe { dev.create_render_pass(&SCRATCH.gaussian_vertical_pass, None).unwrap_unchecked() };
    let postprocess = unsafe { dev.create_render_pass(&SCRATCH.postprocess_pass, None).unwrap_unchecked() };
    set_label(render, "RENDER-PASS-render", debug_ext, dev);
    set_label(gaussian_horizontal, "RENDER-PASS-gaussian_horizontal", debug_ext, dev);
    set_label(gaussian_vertical, "RENDER-PASS-gaussian_vertical", debug_ext, dev);
    set_label(postprocess, "RENDER-PASS-postprocess", debug_ext, dev);
    let extent = vk::Extent2D {
        width: swapchain.extent.width / 1,
        height: swapchain.extent.height / 1,
    };
    let mut framebuffer_attachments = Vec::new();
    let mut framebuffers = Vec::new();
    let mut resources = Vec::new();
    let resource = ImageResources::create(
        COLOR_FORMAT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
        vk::ImageAspectFlags::COLOR,
        extent,
        vk::SampleCountFlags::TYPE_2,
        dev,
    );
    framebuffer_attachments.push(resource.view);
    resources.push(resource);
    let resource = ImageResources::create(
        DEPTH_FORMAT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
        vk::ImageAspectFlags::DEPTH,
        extent,
        vk::SampleCountFlags::TYPE_2,
        dev,
    );
    framebuffer_attachments.push(resource.view);
    resources.push(resource);
    let info = *vk::FramebufferCreateInfo::builder()
        .render_pass(render)
        .attachments(&framebuffer_attachments)
        .width(extent.width)
        .height(extent.height)
        .layers(1);
    let framebuffer = unsafe { dev.create_framebuffer(&info, None) }.unwrap();
    framebuffers.push(framebuffer);
    let render = Pass {
        debug_name: "Forward rendering pass",
        debug_color: [160, 167, 161],
        pass: render,
        extent,
        clears: vec![
            vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0] } },
            vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } },
        ],
        resources,
        framebuffers,
        direct_to_swapchain: false,
    };
    let extent = vk::Extent2D {
        width: swapchain.extent.width / 2,
        height: swapchain.extent.height / 2,
    };
    let mut framebuffer_attachments = Vec::new();
    let mut framebuffers = Vec::new();
    let mut resources = Vec::new();
    let resource = ImageResources::create(
        COLOR_FORMAT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
        vk::ImageAspectFlags::COLOR,
        extent,
        vk::SampleCountFlags::TYPE_1,
        dev,
    );
    framebuffer_attachments.push(resource.view);
    resources.push(resource);
    let info = *vk::FramebufferCreateInfo::builder()
        .render_pass(gaussian_horizontal)
        .attachments(&framebuffer_attachments)
        .width(extent.width)
        .height(extent.height)
        .layers(1);
    let framebuffer = unsafe { dev.create_framebuffer(&info, None) }.unwrap();
    framebuffers.push(framebuffer);
    let gaussian_horizontal = Pass {
        debug_name: "Gaussian horizontal pass",
        debug_color: [244, 244, 247],
        pass: gaussian_horizontal,
        extent,
        clears: vec![
        ],
        resources,
        framebuffers,
        direct_to_swapchain: false,
    };
    let extent = vk::Extent2D {
        width: swapchain.extent.width / 2,
        height: swapchain.extent.height / 2,
    };
    let mut framebuffer_attachments = Vec::new();
    let mut framebuffers = Vec::new();
    let mut resources = Vec::new();
    let resource = ImageResources::create(
        COLOR_FORMAT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
        vk::ImageAspectFlags::COLOR,
        extent,
        vk::SampleCountFlags::TYPE_1,
        dev,
    );
    framebuffer_attachments.push(resource.view);
    resources.push(resource);
    let info = *vk::FramebufferCreateInfo::builder()
        .render_pass(gaussian_vertical)
        .attachments(&framebuffer_attachments)
        .width(extent.width)
        .height(extent.height)
        .layers(1);
    let framebuffer = unsafe { dev.create_framebuffer(&info, None) }.unwrap();
    framebuffers.push(framebuffer);
    let gaussian_vertical = Pass {
        debug_name: "Gaussian vertical pass",
        debug_color: [244, 244, 247],
        pass: gaussian_vertical,
        extent,
        clears: vec![
        ],
        resources,
        framebuffers,
        direct_to_swapchain: false,
    };
    let extent = vk::Extent2D {
        width: swapchain.extent.width / 1,
        height: swapchain.extent.height / 1,
    };
    let mut framebuffer_attachments = Vec::new();
    let mut framebuffers = Vec::new();
    let mut resources = Vec::new();
    framebuffer_attachments.push(vk::ImageView::null());
    let info = *vk::FramebufferCreateInfo::builder()
        .render_pass(postprocess)
        .attachments(&framebuffer_attachments)
        .width(extent.width)
        .height(extent.height)
        .layers(1);
    for image in &swapchain.image_views {
        unsafe { *(info.p_attachments.add(0) as *mut vk::ImageView) = *image };
        let framebuffer = unsafe { dev.create_framebuffer(&info, None) }.unwrap();
        framebuffers.push(framebuffer);
    }
    let postprocess = Pass {
        debug_name: "Postprocess pass",
        debug_color: [210, 206, 203],
        pass: postprocess,
        extent,
        clears: vec![
        ],
        resources,
        framebuffers,
        direct_to_swapchain: true,
    };
    Passes {
        render,
        gaussian_horizontal,
        gaussian_vertical,
        postprocess,
    }
}

#[rustfmt::skip]
pub fn create_pipeline_layouts(
    descriptor_set_layouts: &DescriptorSetLayouts,
    dev: &Dev,
) -> PipelineLayouts {
    unsafe { SCRATCH.object_layouts[0] = descriptor_set_layouts.object };
    unsafe { SCRATCH.object_layouts[1] = descriptor_set_layouts.global };
    unsafe { SCRATCH.grass_layouts[0] = descriptor_set_layouts.object };
    unsafe { SCRATCH.grass_layouts[1] = descriptor_set_layouts.global };
    unsafe { SCRATCH.star_layouts[0] = descriptor_set_layouts.object };
    unsafe { SCRATCH.star_layouts[1] = descriptor_set_layouts.global };
    unsafe { SCRATCH.skybox_layouts[0] = descriptor_set_layouts.object };
    unsafe { SCRATCH.skybox_layouts[1] = descriptor_set_layouts.global };
    unsafe { SCRATCH.deferred_layouts[0] = descriptor_set_layouts.deferred };
    unsafe { SCRATCH.deferred_layouts[1] = descriptor_set_layouts.global };
    unsafe { SCRATCH.gaussian_horizontal_layouts[0] = descriptor_set_layouts.gaussian_horizontal };
    unsafe { SCRATCH.gaussian_horizontal_layouts[1] = descriptor_set_layouts.global };
    unsafe { SCRATCH.gaussian_vertical_layouts[0] = descriptor_set_layouts.gaussian_vertical };
    unsafe { SCRATCH.gaussian_vertical_layouts[1] = descriptor_set_layouts.global };
    unsafe { SCRATCH.postprocess_layouts[0] = descriptor_set_layouts.postprocess };
    unsafe { SCRATCH.postprocess_layouts[1] = descriptor_set_layouts.global };
    let object = unsafe { dev.create_pipeline_layout(&SCRATCH.object_pipeline_layout, None).unwrap_unchecked() };
    let grass = unsafe { dev.create_pipeline_layout(&SCRATCH.grass_pipeline_layout, None).unwrap_unchecked() };
    let star = unsafe { dev.create_pipeline_layout(&SCRATCH.star_pipeline_layout, None).unwrap_unchecked() };
    let skybox = unsafe { dev.create_pipeline_layout(&SCRATCH.skybox_pipeline_layout, None).unwrap_unchecked() };
    let deferred = unsafe { dev.create_pipeline_layout(&SCRATCH.deferred_pipeline_layout, None).unwrap_unchecked() };
    let gaussian_horizontal = unsafe { dev.create_pipeline_layout(&SCRATCH.gaussian_horizontal_pipeline_layout, None).unwrap_unchecked() };
    let gaussian_vertical = unsafe { dev.create_pipeline_layout(&SCRATCH.gaussian_vertical_pipeline_layout, None).unwrap_unchecked() };
    let postprocess = unsafe { dev.create_pipeline_layout(&SCRATCH.postprocess_pipeline_layout, None).unwrap_unchecked() };
    PipelineLayouts {
        object,
        grass,
        star,
        skybox,
        deferred,
        gaussian_horizontal,
        gaussian_vertical,
        postprocess,
    }
}

#[rustfmt::skip]
pub fn create_shaders(supports_raytracing: bool) -> Shaders {
    let object_vertex = compile_glsl("shaders/object.vert", shaderc::ShaderKind::Vertex, supports_raytracing);
    let object_fragment = compile_glsl("shaders/object.frag", shaderc::ShaderKind::Fragment, supports_raytracing);
    let grass_vertex = compile_glsl("shaders/grass.vert", shaderc::ShaderKind::Vertex, supports_raytracing);
    let grass_fragment = compile_glsl("shaders/grass.frag", shaderc::ShaderKind::Fragment, supports_raytracing);
    let star_vertex = compile_glsl("shaders/star.vert", shaderc::ShaderKind::Vertex, supports_raytracing);
    let star_fragment = compile_glsl("shaders/star.frag", shaderc::ShaderKind::Fragment, supports_raytracing);
    let skybox_vertex = compile_glsl("shaders/skybox.vert", shaderc::ShaderKind::Vertex, supports_raytracing);
    let skybox_fragment = compile_glsl("shaders/skybox.frag", shaderc::ShaderKind::Fragment, supports_raytracing);
    let deferred_vertex = compile_glsl("shaders/util/quad.vert", shaderc::ShaderKind::Vertex, supports_raytracing);
    let deferred_fragment = compile_glsl("shaders/deferred.frag", shaderc::ShaderKind::Fragment, supports_raytracing);
    let gaussian_horizontal_vertex = compile_glsl("shaders/util/quad.vert", shaderc::ShaderKind::Vertex, supports_raytracing);
    let gaussian_horizontal_fragment = compile_glsl("shaders/gaussian.frag", shaderc::ShaderKind::Fragment, supports_raytracing);
    let gaussian_vertical_vertex = compile_glsl("shaders/util/quad.vert", shaderc::ShaderKind::Vertex, supports_raytracing);
    let gaussian_vertical_fragment = compile_glsl("shaders/gaussian.frag", shaderc::ShaderKind::Fragment, supports_raytracing);
    let postprocess_vertex = compile_glsl("shaders/util/quad.vert", shaderc::ShaderKind::Vertex, supports_raytracing);
    let postprocess_fragment = compile_glsl("shaders/postprocess.frag", shaderc::ShaderKind::Fragment, supports_raytracing);
    Shaders {
        object_vertex,
        object_fragment,
        grass_vertex,
        grass_fragment,
        star_vertex,
        star_fragment,
        skybox_vertex,
        skybox_fragment,
        deferred_vertex,
        deferred_fragment,
        gaussian_horizontal_vertex,
        gaussian_horizontal_fragment,
        gaussian_vertical_vertex,
        gaussian_vertical_fragment,
        postprocess_vertex,
        postprocess_fragment,
    }
}

#[rustfmt::skip]
pub fn create_shader_modules(shaders: &Shaders, dev: &Dev) -> ShaderModules {
    unsafe { SCRATCH.object_shader_vertex.code_size = 4 * shaders.object_vertex.len() };
    unsafe { SCRATCH.object_shader_fragment.code_size = 4 * shaders.object_fragment.len() };
    unsafe { SCRATCH.grass_shader_vertex.code_size = 4 * shaders.grass_vertex.len() };
    unsafe { SCRATCH.grass_shader_fragment.code_size = 4 * shaders.grass_fragment.len() };
    unsafe { SCRATCH.star_shader_vertex.code_size = 4 * shaders.star_vertex.len() };
    unsafe { SCRATCH.star_shader_fragment.code_size = 4 * shaders.star_fragment.len() };
    unsafe { SCRATCH.skybox_shader_vertex.code_size = 4 * shaders.skybox_vertex.len() };
    unsafe { SCRATCH.skybox_shader_fragment.code_size = 4 * shaders.skybox_fragment.len() };
    unsafe { SCRATCH.deferred_shader_vertex.code_size = 4 * shaders.deferred_vertex.len() };
    unsafe { SCRATCH.deferred_shader_fragment.code_size = 4 * shaders.deferred_fragment.len() };
    unsafe { SCRATCH.gaussian_horizontal_shader_vertex.code_size = 4 * shaders.gaussian_horizontal_vertex.len() };
    unsafe { SCRATCH.gaussian_horizontal_shader_fragment.code_size = 4 * shaders.gaussian_horizontal_fragment.len() };
    unsafe { SCRATCH.gaussian_vertical_shader_vertex.code_size = 4 * shaders.gaussian_vertical_vertex.len() };
    unsafe { SCRATCH.gaussian_vertical_shader_fragment.code_size = 4 * shaders.gaussian_vertical_fragment.len() };
    unsafe { SCRATCH.postprocess_shader_vertex.code_size = 4 * shaders.postprocess_vertex.len() };
    unsafe { SCRATCH.postprocess_shader_fragment.code_size = 4 * shaders.postprocess_fragment.len() };
    unsafe { SCRATCH.object_shader_vertex.p_code = shaders.object_vertex.as_ptr() };
    unsafe { SCRATCH.object_shader_fragment.p_code = shaders.object_fragment.as_ptr() };
    unsafe { SCRATCH.grass_shader_vertex.p_code = shaders.grass_vertex.as_ptr() };
    unsafe { SCRATCH.grass_shader_fragment.p_code = shaders.grass_fragment.as_ptr() };
    unsafe { SCRATCH.star_shader_vertex.p_code = shaders.star_vertex.as_ptr() };
    unsafe { SCRATCH.star_shader_fragment.p_code = shaders.star_fragment.as_ptr() };
    unsafe { SCRATCH.skybox_shader_vertex.p_code = shaders.skybox_vertex.as_ptr() };
    unsafe { SCRATCH.skybox_shader_fragment.p_code = shaders.skybox_fragment.as_ptr() };
    unsafe { SCRATCH.deferred_shader_vertex.p_code = shaders.deferred_vertex.as_ptr() };
    unsafe { SCRATCH.deferred_shader_fragment.p_code = shaders.deferred_fragment.as_ptr() };
    unsafe { SCRATCH.gaussian_horizontal_shader_vertex.p_code = shaders.gaussian_horizontal_vertex.as_ptr() };
    unsafe { SCRATCH.gaussian_horizontal_shader_fragment.p_code = shaders.gaussian_horizontal_fragment.as_ptr() };
    unsafe { SCRATCH.gaussian_vertical_shader_vertex.p_code = shaders.gaussian_vertical_vertex.as_ptr() };
    unsafe { SCRATCH.gaussian_vertical_shader_fragment.p_code = shaders.gaussian_vertical_fragment.as_ptr() };
    unsafe { SCRATCH.postprocess_shader_vertex.p_code = shaders.postprocess_vertex.as_ptr() };
    unsafe { SCRATCH.postprocess_shader_fragment.p_code = shaders.postprocess_fragment.as_ptr() };
    let object_vertex = unsafe { dev.create_shader_module(&SCRATCH.object_shader_vertex, None).unwrap_unchecked() };
    let object_fragment = unsafe { dev.create_shader_module(&SCRATCH.object_shader_fragment, None).unwrap_unchecked() };
    let grass_vertex = unsafe { dev.create_shader_module(&SCRATCH.grass_shader_vertex, None).unwrap_unchecked() };
    let grass_fragment = unsafe { dev.create_shader_module(&SCRATCH.grass_shader_fragment, None).unwrap_unchecked() };
    let star_vertex = unsafe { dev.create_shader_module(&SCRATCH.star_shader_vertex, None).unwrap_unchecked() };
    let star_fragment = unsafe { dev.create_shader_module(&SCRATCH.star_shader_fragment, None).unwrap_unchecked() };
    let skybox_vertex = unsafe { dev.create_shader_module(&SCRATCH.skybox_shader_vertex, None).unwrap_unchecked() };
    let skybox_fragment = unsafe { dev.create_shader_module(&SCRATCH.skybox_shader_fragment, None).unwrap_unchecked() };
    let deferred_vertex = unsafe { dev.create_shader_module(&SCRATCH.deferred_shader_vertex, None).unwrap_unchecked() };
    let deferred_fragment = unsafe { dev.create_shader_module(&SCRATCH.deferred_shader_fragment, None).unwrap_unchecked() };
    let gaussian_horizontal_vertex = unsafe { dev.create_shader_module(&SCRATCH.gaussian_horizontal_shader_vertex, None).unwrap_unchecked() };
    let gaussian_horizontal_fragment = unsafe { dev.create_shader_module(&SCRATCH.gaussian_horizontal_shader_fragment, None).unwrap_unchecked() };
    let gaussian_vertical_vertex = unsafe { dev.create_shader_module(&SCRATCH.gaussian_vertical_shader_vertex, None).unwrap_unchecked() };
    let gaussian_vertical_fragment = unsafe { dev.create_shader_module(&SCRATCH.gaussian_vertical_shader_fragment, None).unwrap_unchecked() };
    let postprocess_vertex = unsafe { dev.create_shader_module(&SCRATCH.postprocess_shader_vertex, None).unwrap_unchecked() };
    let postprocess_fragment = unsafe { dev.create_shader_module(&SCRATCH.postprocess_shader_fragment, None).unwrap_unchecked() };
    ShaderModules {
        object_vertex,
        object_fragment,
        grass_vertex,
        grass_fragment,
        star_vertex,
        star_fragment,
        skybox_vertex,
        skybox_fragment,
        deferred_vertex,
        deferred_fragment,
        gaussian_horizontal_vertex,
        gaussian_horizontal_fragment,
        gaussian_vertical_vertex,
        gaussian_vertical_fragment,
        postprocess_vertex,
        postprocess_fragment,
    }
}

#[rustfmt::skip]
#[allow(clippy::identity_op)]
pub fn create_pipelines(
    render: &Pass,
    gaussian_horizontal: &Pass,
    gaussian_vertical: &Pass,
    postprocess: &Pass,
    _msaa_samples: vk::SampleCountFlags,
    swapchain: &Swapchain,
    shader_modules: &ShaderModules,
    layouts: &PipelineLayouts,
    dev: &Dev,
) -> Pipelines {
    unsafe { SCRATCH.object_shader_stages[0].module = shader_modules.object_vertex };
    unsafe { SCRATCH.object_shader_stages[1].module = shader_modules.object_fragment };
    unsafe { SCRATCH.object_viewport.width = (swapchain.extent.width / 1) as f32 };
    unsafe { SCRATCH.object_viewport.height = (swapchain.extent.height / 1) as f32 };
    unsafe { SCRATCH.object_scissor.extent.width = swapchain.extent.width / 1 };
    unsafe { SCRATCH.object_scissor.extent.height = swapchain.extent.height / 1 };
    unsafe { SCRATCH.grass_shader_stages[0].module = shader_modules.grass_vertex };
    unsafe { SCRATCH.grass_shader_stages[1].module = shader_modules.grass_fragment };
    unsafe { SCRATCH.grass_viewport.width = (swapchain.extent.width / 1) as f32 };
    unsafe { SCRATCH.grass_viewport.height = (swapchain.extent.height / 1) as f32 };
    unsafe { SCRATCH.grass_scissor.extent.width = swapchain.extent.width / 1 };
    unsafe { SCRATCH.grass_scissor.extent.height = swapchain.extent.height / 1 };
    unsafe { SCRATCH.star_shader_stages[0].module = shader_modules.star_vertex };
    unsafe { SCRATCH.star_shader_stages[1].module = shader_modules.star_fragment };
    unsafe { SCRATCH.star_viewport.width = (swapchain.extent.width / 1) as f32 };
    unsafe { SCRATCH.star_viewport.height = (swapchain.extent.height / 1) as f32 };
    unsafe { SCRATCH.star_scissor.extent.width = swapchain.extent.width / 1 };
    unsafe { SCRATCH.star_scissor.extent.height = swapchain.extent.height / 1 };
    unsafe { SCRATCH.skybox_shader_stages[0].module = shader_modules.skybox_vertex };
    unsafe { SCRATCH.skybox_shader_stages[1].module = shader_modules.skybox_fragment };
    unsafe { SCRATCH.skybox_viewport.width = (swapchain.extent.width / 1) as f32 };
    unsafe { SCRATCH.skybox_viewport.height = (swapchain.extent.height / 1) as f32 };
    unsafe { SCRATCH.skybox_scissor.extent.width = swapchain.extent.width / 1 };
    unsafe { SCRATCH.skybox_scissor.extent.height = swapchain.extent.height / 1 };
    unsafe { SCRATCH.deferred_shader_stages[0].module = shader_modules.deferred_vertex };
    unsafe { SCRATCH.deferred_shader_stages[1].module = shader_modules.deferred_fragment };
    unsafe { SCRATCH.deferred_viewport.width = (swapchain.extent.width / 1) as f32 };
    unsafe { SCRATCH.deferred_viewport.height = (swapchain.extent.height / 1) as f32 };
    unsafe { SCRATCH.deferred_scissor.extent.width = swapchain.extent.width / 1 };
    unsafe { SCRATCH.deferred_scissor.extent.height = swapchain.extent.height / 1 };
    unsafe { SCRATCH.gaussian_horizontal_shader_stages[0].module = shader_modules.gaussian_horizontal_vertex };
    unsafe { SCRATCH.gaussian_horizontal_shader_stages[1].module = shader_modules.gaussian_horizontal_fragment };
    unsafe { SCRATCH.gaussian_horizontal_viewport.width = (swapchain.extent.width / 2) as f32 };
    unsafe { SCRATCH.gaussian_horizontal_viewport.height = (swapchain.extent.height / 2) as f32 };
    unsafe { SCRATCH.gaussian_horizontal_scissor.extent.width = swapchain.extent.width / 2 };
    unsafe { SCRATCH.gaussian_horizontal_scissor.extent.height = swapchain.extent.height / 2 };
    unsafe { SCRATCH.gaussian_vertical_shader_stages[0].module = shader_modules.gaussian_vertical_vertex };
    unsafe { SCRATCH.gaussian_vertical_shader_stages[1].module = shader_modules.gaussian_vertical_fragment };
    unsafe { SCRATCH.gaussian_vertical_viewport.width = (swapchain.extent.width / 2) as f32 };
    unsafe { SCRATCH.gaussian_vertical_viewport.height = (swapchain.extent.height / 2) as f32 };
    unsafe { SCRATCH.gaussian_vertical_scissor.extent.width = swapchain.extent.width / 2 };
    unsafe { SCRATCH.gaussian_vertical_scissor.extent.height = swapchain.extent.height / 2 };
    unsafe { SCRATCH.postprocess_shader_stages[0].module = shader_modules.postprocess_vertex };
    unsafe { SCRATCH.postprocess_shader_stages[1].module = shader_modules.postprocess_fragment };
    unsafe { SCRATCH.postprocess_viewport.width = (swapchain.extent.width / 1) as f32 };
    unsafe { SCRATCH.postprocess_viewport.height = (swapchain.extent.height / 1) as f32 };
    unsafe { SCRATCH.postprocess_scissor.extent.width = swapchain.extent.width / 1 };
    unsafe { SCRATCH.postprocess_scissor.extent.height = swapchain.extent.height / 1 };
    unsafe { SCRATCH.object_pipeline.layout = layouts.object };
    unsafe { SCRATCH.object_pipeline.render_pass = render.pass };
    unsafe { SCRATCH.grass_pipeline.layout = layouts.grass };
    unsafe { SCRATCH.grass_pipeline.render_pass = render.pass };
    unsafe { SCRATCH.star_pipeline.layout = layouts.star };
    unsafe { SCRATCH.star_pipeline.render_pass = render.pass };
    unsafe { SCRATCH.skybox_pipeline.layout = layouts.skybox };
    unsafe { SCRATCH.skybox_pipeline.render_pass = render.pass };
    unsafe { SCRATCH.deferred_pipeline.layout = layouts.deferred };
    unsafe { SCRATCH.deferred_pipeline.render_pass = render.pass };
    unsafe { SCRATCH.gaussian_horizontal_pipeline.layout = layouts.gaussian_horizontal };
    unsafe { SCRATCH.gaussian_horizontal_pipeline.render_pass = gaussian_horizontal.pass };
    unsafe { SCRATCH.gaussian_vertical_pipeline.layout = layouts.gaussian_vertical };
    unsafe { SCRATCH.gaussian_vertical_pipeline.render_pass = gaussian_vertical.pass };
    unsafe { SCRATCH.postprocess_pipeline.layout = layouts.postprocess };
    unsafe { SCRATCH.postprocess_pipeline.render_pass = postprocess.pass };
    let mut pipelines = MaybeUninit::uninit();
    let _ = unsafe { (dev.fp_v1_0().create_graphics_pipelines)(
        dev.handle(),
        vk::PipelineCache::null(),
        8,
        &SCRATCH.object_pipeline,
        std::ptr::null(),
        pipelines.as_mut_ptr() as *mut vk::Pipeline,
    ) };
    unsafe { pipelines.assume_init() }
}
