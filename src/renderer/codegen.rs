// Code generated from renderer.kdl.

use crate::renderer::raytracing::RaytraceResources;
use crate::renderer::shader::compile_glsl;
#[rustfmt::skip]
use crate::renderer::uniform::{Atmosphere, Camera, FragSettings, Gaussian, GrassUniform, Light, Material, ModelViewProjection, Postprocessing};
use crate::renderer::util::{AnyUniformBuffer, Dev, UniformBuffer};
use crate::renderer::Swapchain;
use crate::renderer::{Pass, FRAMES_IN_FLIGHT};
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

pub struct DescriptorPools {
    pub object: vk::DescriptorPool,
    pub object_layout: vk::DescriptorSetLayout,
    pub grass: vk::DescriptorPool,
    pub grass_layout: vk::DescriptorSetLayout,
    pub skybox: vk::DescriptorPool,
    pub skybox_layout: vk::DescriptorSetLayout,
    pub atmosphere: vk::DescriptorPool,
    pub atmosphere_layout: vk::DescriptorSetLayout,
    pub gaussian: vk::DescriptorPool,
    pub gaussian_layout: vk::DescriptorSetLayout,
    pub postprocess: vk::DescriptorPool,
    pub postprocess_layout: vk::DescriptorSetLayout,
}

pub struct PipelineLayouts {
    pub object: vk::PipelineLayout,
    pub grass: vk::PipelineLayout,
    pub skybox: vk::PipelineLayout,
    pub atmosphere: vk::PipelineLayout,
    pub gaussian: vk::PipelineLayout,
    pub postprocess: vk::PipelineLayout,
}

pub struct Shaders {
    pub object_vertex: Vec<u32>,
    pub object_fragment: Vec<u32>,
    pub grass_vertex: Vec<u32>,
    pub grass_fragment: Vec<u32>,
    pub skybox_vertex: Vec<u32>,
    pub skybox_fragment: Vec<u32>,
    pub atmosphere_vertex: Vec<u32>,
    pub atmosphere_fragment: Vec<u32>,
    pub gaussian_vertex: Vec<u32>,
    pub gaussian_fragment: Vec<u32>,
    pub postprocess_vertex: Vec<u32>,
    pub postprocess_fragment: Vec<u32>,
}

pub struct ShaderModules {
    pub object_vertex: vk::ShaderModule,
    pub object_fragment: vk::ShaderModule,
    pub grass_vertex: vk::ShaderModule,
    pub grass_fragment: vk::ShaderModule,
    pub skybox_vertex: vk::ShaderModule,
    pub skybox_fragment: vk::ShaderModule,
    pub atmosphere_vertex: vk::ShaderModule,
    pub atmosphere_fragment: vk::ShaderModule,
    pub gaussian_vertex: vk::ShaderModule,
    pub gaussian_fragment: vk::ShaderModule,
    pub postprocess_vertex: vk::ShaderModule,
    pub postprocess_fragment: vk::ShaderModule,
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
    object_pool_sizes: [vk::DescriptorPoolSize; 2],
    object_pool: vk::DescriptorPoolCreateInfo,
    grass_bindings: [vk::DescriptorSetLayoutBinding; 5],
    grass_layout: vk::DescriptorSetLayoutCreateInfo,
    grass_pool_sizes: [vk::DescriptorPoolSize; 2],
    grass_pool: vk::DescriptorPoolCreateInfo,
    skybox_bindings: [vk::DescriptorSetLayoutBinding; 1],
    skybox_layout: vk::DescriptorSetLayoutCreateInfo,
    skybox_pool_sizes: [vk::DescriptorPoolSize; 1],
    skybox_pool: vk::DescriptorPoolCreateInfo,
    atmosphere_bindings: [vk::DescriptorSetLayoutBinding; 4],
    atmosphere_layout: vk::DescriptorSetLayoutCreateInfo,
    atmosphere_pool_sizes: [vk::DescriptorPoolSize; 2],
    atmosphere_pool: vk::DescriptorPoolCreateInfo,
    gaussian_bindings: [vk::DescriptorSetLayoutBinding; 2],
    gaussian_layout: vk::DescriptorSetLayoutCreateInfo,
    gaussian_pool_sizes: [vk::DescriptorPoolSize; 2],
    gaussian_pool: vk::DescriptorPoolCreateInfo,
    postprocess_bindings: [vk::DescriptorSetLayoutBinding; 3],
    postprocess_layout: vk::DescriptorSetLayoutCreateInfo,
    postprocess_pool_sizes: [vk::DescriptorPoolSize; 2],
    postprocess_pool: vk::DescriptorPoolCreateInfo,
    assembly: vk::PipelineInputAssemblyStateCreateInfo,
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
    viewport_state: vk::PipelineViewportStateCreateInfo,
    dynamic_state: vk::PipelineDynamicStateCreateInfo,
    object_pipeline_layout: vk::PipelineLayoutCreateInfo,
    object_shader_vertex: vk::ShaderModuleCreateInfo,
    object_shader_fragment: vk::ShaderModuleCreateInfo,
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
    grass_shader_vertex: vk::ShaderModuleCreateInfo,
    grass_shader_fragment: vk::ShaderModuleCreateInfo,
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
    skybox_shader_vertex: vk::ShaderModuleCreateInfo,
    skybox_shader_fragment: vk::ShaderModuleCreateInfo,
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
    atmosphere_shader_vertex: vk::ShaderModuleCreateInfo,
    atmosphere_shader_fragment: vk::ShaderModuleCreateInfo,
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
    gaussian_shader_vertex: vk::ShaderModuleCreateInfo,
    gaussian_shader_fragment: vk::ShaderModuleCreateInfo,
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
    postprocess_shader_vertex: vk::ShaderModuleCreateInfo,
    postprocess_shader_fragment: vk::ShaderModuleCreateInfo,
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
    object_pool_sizes: [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 32768,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
            descriptor_count: 8192,
        },
    ],
    object_pool: vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: 8192,
        pool_size_count: 2,
        p_pool_sizes: unsafe { SCRATCH.object_pool_sizes.as_ptr() },
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
    grass_pool_sizes: [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 8,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
            descriptor_count: 2,
        },
    ],
    grass_pool: vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: 2,
        pool_size_count: 2,
        p_pool_sizes: unsafe { SCRATCH.grass_pool_sizes.as_ptr() },
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
    skybox_pool_sizes: [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 2,
        },
    ],
    skybox_pool: vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: 2,
        pool_size_count: 1,
        p_pool_sizes: unsafe { SCRATCH.skybox_pool_sizes.as_ptr() },
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
    atmosphere_pool_sizes: [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::INPUT_ATTACHMENT,
            descriptor_count: 4,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 4,
        },
    ],
    atmosphere_pool: vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: 2,
        pool_size_count: 2,
        p_pool_sizes: unsafe { SCRATCH.atmosphere_pool_sizes.as_ptr() },
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
    gaussian_pool_sizes: [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 2,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 2,
        },
    ],
    gaussian_pool: vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: 2,
        pool_size_count: 2,
        p_pool_sizes: unsafe { SCRATCH.gaussian_pool_sizes.as_ptr() },
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
    postprocess_pool_sizes: [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 4,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 2,
        },
    ],
    postprocess_pool: vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: 2,
        pool_size_count: 2,
        p_pool_sizes: unsafe { SCRATCH.postprocess_pool_sizes.as_ptr() },
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
    atmosphere_shader_vertex: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    },
    atmosphere_shader_fragment: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
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
    gaussian_shader_vertex: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    },
    gaussian_shader_fragment: vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
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

#[rustfmt::skip]
impl DescriptorPools {
    pub fn alloc_object(
        &self,
        mvp: &UniformBuffer<ModelViewProjection>,
        material: &UniformBuffer<Material>,
        light: &UniformBuffer<Light>,
        settings: &UniformBuffer<FragSettings>,
        tlas: &Option<RaytraceResources>,
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
        self.update_object(&descriptors, mvp, material, light, settings, tlas, dev);
        descriptors
    }

    pub fn update_object(
        &self,
        descriptors: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],
        mvp: &UniformBuffer<ModelViewProjection>,
        material: &UniformBuffer<Material>,
        light: &UniformBuffer<Light>,
        settings: &UniformBuffer<FragSettings>,
        tlas: &Option<RaytraceResources>,
        dev: &Dev,
    ) {
        for (flight_index, descriptor) in descriptors.iter().enumerate() {
            let mvp_buffer = mvp.descriptor(flight_index);
            let mvp = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&mvp_buffer));
            let material_buffer = material.descriptor(flight_index);
            let material = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&material_buffer));
            let light_buffer = light.descriptor(flight_index);
            let light = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(2)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&light_buffer));
            let settings_buffer = settings.descriptor(flight_index);
            let settings = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(3)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&settings_buffer));
            let mut tlas_acceleration_structure = *vk::WriteDescriptorSetAccelerationStructureKHR::builder()
                .acceleration_structures(std::slice::from_ref(&tlas.as_ref().unwrap().acceleration_structure));
            let mut tlas = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(4)
                .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                .push_next(&mut tlas_acceleration_structure);
            tlas.descriptor_count = 1;
            let writes = [mvp, material, light, settings, tlas];
            unsafe { dev.update_descriptor_sets(&writes, &[]) };
        }
    }

    pub fn alloc_grass(
        &self,
        planet_mvp: &UniformBuffer<ModelViewProjection>,
        grass: &UniformBuffer<GrassUniform>,
        light: &UniformBuffer<Light>,
        settings: &UniformBuffer<FragSettings>,
        tlas: &Option<RaytraceResources>,
        dev: &Dev,
    ) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
        let layouts = [self.grass_layout; FRAMES_IN_FLIGHT];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.grass)
            .set_layouts(&layouts);
        let descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
            unsafe { dev.allocate_descriptor_sets(&descriptor_set_alloc_info) }
                .unwrap()
                .try_into()
                .unwrap();
        self.update_grass(&descriptors, planet_mvp, grass, light, settings, tlas, dev);
        descriptors
    }

    pub fn update_grass(
        &self,
        descriptors: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],
        planet_mvp: &UniformBuffer<ModelViewProjection>,
        grass: &UniformBuffer<GrassUniform>,
        light: &UniformBuffer<Light>,
        settings: &UniformBuffer<FragSettings>,
        tlas: &Option<RaytraceResources>,
        dev: &Dev,
    ) {
        for (flight_index, descriptor) in descriptors.iter().enumerate() {
            let planet_mvp_buffer = planet_mvp.descriptor(flight_index);
            let planet_mvp = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&planet_mvp_buffer));
            let grass_buffer = grass.descriptor(flight_index);
            let grass = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&grass_buffer));
            let light_buffer = light.descriptor(flight_index);
            let light = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(2)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&light_buffer));
            let settings_buffer = settings.descriptor(flight_index);
            let settings = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(3)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&settings_buffer));
            let mut tlas_acceleration_structure = *vk::WriteDescriptorSetAccelerationStructureKHR::builder()
                .acceleration_structures(std::slice::from_ref(&tlas.as_ref().unwrap().acceleration_structure));
            let mut tlas = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(4)
                .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                .push_next(&mut tlas_acceleration_structure);
            tlas.descriptor_count = 1;
            let writes = [planet_mvp, grass, light, settings, tlas];
            unsafe { dev.update_descriptor_sets(&writes, &[]) };
        }
    }

    pub fn alloc_skybox(
        &self,
        mvp: &UniformBuffer<ModelViewProjection>,
        dev: &Dev,
    ) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
        let layouts = [self.skybox_layout; FRAMES_IN_FLIGHT];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.skybox)
            .set_layouts(&layouts);
        let descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
            unsafe { dev.allocate_descriptor_sets(&descriptor_set_alloc_info) }
                .unwrap()
                .try_into()
                .unwrap();
        self.update_skybox(&descriptors, mvp, dev);
        descriptors
    }

    pub fn update_skybox(
        &self,
        descriptors: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],
        mvp: &UniformBuffer<ModelViewProjection>,
        dev: &Dev,
    ) {
        for (flight_index, descriptor) in descriptors.iter().enumerate() {
            let mvp_buffer = mvp.descriptor(flight_index);
            let mvp = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&mvp_buffer));
            let writes = [mvp];
            unsafe { dev.update_descriptor_sets(&writes, &[]) };
        }
    }

    pub fn alloc_atmosphere(
        &self,
        render: vk::ImageView,
        position: vk::ImageView,
        atmosphere: &UniformBuffer<Atmosphere>,
        camera: &UniformBuffer<Camera>,
        dev: &Dev,
    ) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
        let layouts = [self.atmosphere_layout; FRAMES_IN_FLIGHT];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.atmosphere)
            .set_layouts(&layouts);
        let descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
            unsafe { dev.allocate_descriptor_sets(&descriptor_set_alloc_info) }
                .unwrap()
                .try_into()
                .unwrap();
        self.update_atmosphere(&descriptors, render, position, atmosphere, camera, dev);
        descriptors
    }

    pub fn update_atmosphere(
        &self,
        descriptors: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],
        render: vk::ImageView,
        position: vk::ImageView,
        atmosphere: &UniformBuffer<Atmosphere>,
        camera: &UniformBuffer<Camera>,
        dev: &Dev,
    ) {
        for (flight_index, descriptor) in descriptors.iter().enumerate() {
            let render_image = *vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(render);
            let render = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(std::slice::from_ref(&render_image));
            let position_image = *vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(position);
            let position = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(std::slice::from_ref(&position_image));
            let atmosphere_buffer = atmosphere.descriptor(flight_index);
            let atmosphere = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(2)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&atmosphere_buffer));
            let camera_buffer = camera.descriptor(flight_index);
            let camera = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(3)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&camera_buffer));
            let writes = [render, position, atmosphere, camera];
            unsafe { dev.update_descriptor_sets(&writes, &[]) };
        }
    }

    pub fn alloc_gaussian(
        &self,
        render: vk::ImageView,
        gaussian: &UniformBuffer<Gaussian>,
        dev: &Dev,
    ) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
        let layouts = [self.gaussian_layout; FRAMES_IN_FLIGHT];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.gaussian)
            .set_layouts(&layouts);
        let descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
            unsafe { dev.allocate_descriptor_sets(&descriptor_set_alloc_info) }
                .unwrap()
                .try_into()
                .unwrap();
        self.update_gaussian(&descriptors, render, gaussian, dev);
        descriptors
    }

    pub fn update_gaussian(
        &self,
        descriptors: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],
        render: vk::ImageView,
        gaussian: &UniformBuffer<Gaussian>,
        dev: &Dev,
    ) {
        for (flight_index, descriptor) in descriptors.iter().enumerate() {
            let render_image = *vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(render);
            let render = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&render_image));
            let gaussian_buffer = gaussian.descriptor(flight_index);
            let gaussian = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&gaussian_buffer));
            let writes = [render, gaussian];
            unsafe { dev.update_descriptor_sets(&writes, &[]) };
        }
    }

    pub fn alloc_postprocess(
        &self,
        render: vk::ImageView,
        bloom: vk::ImageView,
        postprocessing: &UniformBuffer<Postprocessing>,
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
        self.update_postprocess(&descriptors, render, bloom, postprocessing, dev);
        descriptors
    }

    pub fn update_postprocess(
        &self,
        descriptors: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],
        render: vk::ImageView,
        bloom: vk::ImageView,
        postprocessing: &UniformBuffer<Postprocessing>,
        dev: &Dev,
    ) {
        for (flight_index, descriptor) in descriptors.iter().enumerate() {
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
            let postprocessing_buffer = postprocessing.descriptor(flight_index);
            let postprocessing = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
                .dst_binding(2)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&postprocessing_buffer));
            let writes = [render, bloom, postprocessing];
            unsafe { dev.update_descriptor_sets(&writes, &[]) };
        }
    }

    pub fn cleanup(&self, dev: &Dev) {
        unsafe { dev.destroy_descriptor_pool(self.object, None) };
        unsafe { dev.destroy_descriptor_pool(self.grass, None) };
        unsafe { dev.destroy_descriptor_pool(self.skybox, None) };
        unsafe { dev.destroy_descriptor_pool(self.atmosphere, None) };
        unsafe { dev.destroy_descriptor_pool(self.gaussian, None) };
        unsafe { dev.destroy_descriptor_pool(self.postprocess, None) };
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

impl ShaderModules {
    pub fn cleanup(&self, dev: &Dev) {
        unsafe { dev.destroy_shader_module(self.object_vertex, None) };
        unsafe { dev.destroy_shader_module(self.object_fragment, None) };
        unsafe { dev.destroy_shader_module(self.grass_vertex, None) };
        unsafe { dev.destroy_shader_module(self.grass_fragment, None) };
        unsafe { dev.destroy_shader_module(self.skybox_vertex, None) };
        unsafe { dev.destroy_shader_module(self.skybox_fragment, None) };
        unsafe { dev.destroy_shader_module(self.atmosphere_vertex, None) };
        unsafe { dev.destroy_shader_module(self.atmosphere_fragment, None) };
        unsafe { dev.destroy_shader_module(self.gaussian_vertex, None) };
        unsafe { dev.destroy_shader_module(self.gaussian_fragment, None) };
        unsafe { dev.destroy_shader_module(self.postprocess_vertex, None) };
        unsafe { dev.destroy_shader_module(self.postprocess_fragment, None) };
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
pub fn create_descriptor_pools(layouts: &DescriptorSetLayouts, dev: &Dev) -> DescriptorPools {
    let object = unsafe { dev.create_descriptor_pool(&SCRATCH.object_pool, None).unwrap_unchecked() };
    let grass = unsafe { dev.create_descriptor_pool(&SCRATCH.grass_pool, None).unwrap_unchecked() };
    let skybox = unsafe { dev.create_descriptor_pool(&SCRATCH.skybox_pool, None).unwrap_unchecked() };
    let atmosphere = unsafe { dev.create_descriptor_pool(&SCRATCH.atmosphere_pool, None).unwrap_unchecked() };
    let gaussian = unsafe { dev.create_descriptor_pool(&SCRATCH.gaussian_pool, None).unwrap_unchecked() };
    let postprocess = unsafe { dev.create_descriptor_pool(&SCRATCH.postprocess_pool, None).unwrap_unchecked() };
    DescriptorPools {
        object,
        object_layout: layouts.object,
        grass,
        grass_layout: layouts.grass,
        skybox,
        skybox_layout: layouts.skybox,
        atmosphere,
        atmosphere_layout: layouts.atmosphere,
        gaussian,
        gaussian_layout: layouts.gaussian,
        postprocess,
        postprocess_layout: layouts.postprocess,
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
pub fn create_shaders(supports_raytracing: bool) -> Shaders {
    let object_vertex = compile_glsl("shaders/object.vert", shaderc::ShaderKind::Vertex, supports_raytracing);
    let object_fragment = compile_glsl("shaders/object.frag", shaderc::ShaderKind::Fragment, supports_raytracing);
    let grass_vertex = compile_glsl("shaders/grass.vert", shaderc::ShaderKind::Vertex, supports_raytracing);
    let grass_fragment = compile_glsl("shaders/grass.frag", shaderc::ShaderKind::Fragment, supports_raytracing);
    let skybox_vertex = compile_glsl("shaders/skybox.vert", shaderc::ShaderKind::Vertex, supports_raytracing);
    let skybox_fragment = compile_glsl("shaders/skybox.frag", shaderc::ShaderKind::Fragment, supports_raytracing);
    let atmosphere_vertex = compile_glsl("shaders/atmosphere.vert", shaderc::ShaderKind::Vertex, supports_raytracing);
    let atmosphere_fragment = compile_glsl("shaders/atmosphere.frag", shaderc::ShaderKind::Fragment, supports_raytracing);
    let gaussian_vertex = compile_glsl("shaders/gaussian.vert", shaderc::ShaderKind::Vertex, supports_raytracing);
    let gaussian_fragment = compile_glsl("shaders/gaussian.frag", shaderc::ShaderKind::Fragment, supports_raytracing);
    let postprocess_vertex = compile_glsl("shaders/postprocess.vert", shaderc::ShaderKind::Vertex, supports_raytracing);
    let postprocess_fragment = compile_glsl("shaders/postprocess.frag", shaderc::ShaderKind::Fragment, supports_raytracing);
    Shaders {
        object_vertex,
        object_fragment,
        grass_vertex,
        grass_fragment,
        skybox_vertex,
        skybox_fragment,
        atmosphere_vertex,
        atmosphere_fragment,
        gaussian_vertex,
        gaussian_fragment,
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
    unsafe { SCRATCH.skybox_shader_vertex.code_size = 4 * shaders.skybox_vertex.len() };
    unsafe { SCRATCH.skybox_shader_fragment.code_size = 4 * shaders.skybox_fragment.len() };
    unsafe { SCRATCH.atmosphere_shader_vertex.code_size = 4 * shaders.atmosphere_vertex.len() };
    unsafe { SCRATCH.atmosphere_shader_fragment.code_size = 4 * shaders.atmosphere_fragment.len() };
    unsafe { SCRATCH.gaussian_shader_vertex.code_size = 4 * shaders.gaussian_vertex.len() };
    unsafe { SCRATCH.gaussian_shader_fragment.code_size = 4 * shaders.gaussian_fragment.len() };
    unsafe { SCRATCH.postprocess_shader_vertex.code_size = 4 * shaders.postprocess_vertex.len() };
    unsafe { SCRATCH.postprocess_shader_fragment.code_size = 4 * shaders.postprocess_fragment.len() };
    unsafe { SCRATCH.object_shader_vertex.p_code = shaders.object_vertex.as_ptr() };
    unsafe { SCRATCH.object_shader_fragment.p_code = shaders.object_fragment.as_ptr() };
    unsafe { SCRATCH.grass_shader_vertex.p_code = shaders.grass_vertex.as_ptr() };
    unsafe { SCRATCH.grass_shader_fragment.p_code = shaders.grass_fragment.as_ptr() };
    unsafe { SCRATCH.skybox_shader_vertex.p_code = shaders.skybox_vertex.as_ptr() };
    unsafe { SCRATCH.skybox_shader_fragment.p_code = shaders.skybox_fragment.as_ptr() };
    unsafe { SCRATCH.atmosphere_shader_vertex.p_code = shaders.atmosphere_vertex.as_ptr() };
    unsafe { SCRATCH.atmosphere_shader_fragment.p_code = shaders.atmosphere_fragment.as_ptr() };
    unsafe { SCRATCH.gaussian_shader_vertex.p_code = shaders.gaussian_vertex.as_ptr() };
    unsafe { SCRATCH.gaussian_shader_fragment.p_code = shaders.gaussian_fragment.as_ptr() };
    unsafe { SCRATCH.postprocess_shader_vertex.p_code = shaders.postprocess_vertex.as_ptr() };
    unsafe { SCRATCH.postprocess_shader_fragment.p_code = shaders.postprocess_fragment.as_ptr() };
    let object_vertex = unsafe { dev.create_shader_module(&SCRATCH.object_shader_vertex, None).unwrap_unchecked() };
    let object_fragment = unsafe { dev.create_shader_module(&SCRATCH.object_shader_fragment, None).unwrap_unchecked() };
    let grass_vertex = unsafe { dev.create_shader_module(&SCRATCH.grass_shader_vertex, None).unwrap_unchecked() };
    let grass_fragment = unsafe { dev.create_shader_module(&SCRATCH.grass_shader_fragment, None).unwrap_unchecked() };
    let skybox_vertex = unsafe { dev.create_shader_module(&SCRATCH.skybox_shader_vertex, None).unwrap_unchecked() };
    let skybox_fragment = unsafe { dev.create_shader_module(&SCRATCH.skybox_shader_fragment, None).unwrap_unchecked() };
    let atmosphere_vertex = unsafe { dev.create_shader_module(&SCRATCH.atmosphere_shader_vertex, None).unwrap_unchecked() };
    let atmosphere_fragment = unsafe { dev.create_shader_module(&SCRATCH.atmosphere_shader_fragment, None).unwrap_unchecked() };
    let gaussian_vertex = unsafe { dev.create_shader_module(&SCRATCH.gaussian_shader_vertex, None).unwrap_unchecked() };
    let gaussian_fragment = unsafe { dev.create_shader_module(&SCRATCH.gaussian_shader_fragment, None).unwrap_unchecked() };
    let postprocess_vertex = unsafe { dev.create_shader_module(&SCRATCH.postprocess_shader_vertex, None).unwrap_unchecked() };
    let postprocess_fragment = unsafe { dev.create_shader_module(&SCRATCH.postprocess_shader_fragment, None).unwrap_unchecked() };
    ShaderModules {
        object_vertex,
        object_fragment,
        grass_vertex,
        grass_fragment,
        skybox_vertex,
        skybox_fragment,
        atmosphere_vertex,
        atmosphere_fragment,
        gaussian_vertex,
        gaussian_fragment,
        postprocess_vertex,
        postprocess_fragment,
    }
}

#[rustfmt::skip]
pub fn create_pipelines(
    render: &Pass,
    gaussian: &Pass,
    postprocess: &Pass,
    _msaa_samples: vk::SampleCountFlags,
    swapchain: &Swapchain,
    shader_modules: &ShaderModules,
    layouts: &PipelineLayouts,
    dev: &Dev,
) -> Pipelines {
    unsafe { SCRATCH.viewport.width = swapchain.extent.width as f32 };
    unsafe { SCRATCH.viewport.height = swapchain.extent.height as f32 };
    unsafe { SCRATCH.scissor.extent.width = swapchain.extent.width };
    unsafe { SCRATCH.scissor.extent.height = swapchain.extent.height };
    unsafe { SCRATCH.object_shader_stages[0].module = shader_modules.object_vertex };
    unsafe { SCRATCH.object_shader_stages[1].module = shader_modules.object_fragment };
    unsafe { SCRATCH.grass_shader_stages[0].module = shader_modules.grass_vertex };
    unsafe { SCRATCH.grass_shader_stages[1].module = shader_modules.grass_fragment };
    unsafe { SCRATCH.skybox_shader_stages[0].module = shader_modules.skybox_vertex };
    unsafe { SCRATCH.skybox_shader_stages[1].module = shader_modules.skybox_fragment };
    unsafe { SCRATCH.atmosphere_shader_stages[0].module = shader_modules.atmosphere_vertex };
    unsafe { SCRATCH.atmosphere_shader_stages[1].module = shader_modules.atmosphere_fragment };
    unsafe { SCRATCH.gaussian_shader_stages[0].module = shader_modules.gaussian_vertex };
    unsafe { SCRATCH.gaussian_shader_stages[1].module = shader_modules.gaussian_fragment };
    unsafe { SCRATCH.postprocess_shader_stages[0].module = shader_modules.postprocess_vertex };
    unsafe { SCRATCH.postprocess_shader_stages[1].module = shader_modules.postprocess_fragment };
    unsafe { SCRATCH.object_pipeline.layout = layouts.object };
    unsafe { SCRATCH.object_pipeline.render_pass = render.pass };
    unsafe { SCRATCH.grass_pipeline.layout = layouts.grass };
    unsafe { SCRATCH.grass_pipeline.render_pass = render.pass };
    unsafe { SCRATCH.skybox_pipeline.layout = layouts.skybox };
    unsafe { SCRATCH.skybox_pipeline.render_pass = render.pass };
    unsafe { SCRATCH.atmosphere_pipeline.layout = layouts.atmosphere };
    unsafe { SCRATCH.atmosphere_pipeline.render_pass = render.pass };
    unsafe { SCRATCH.gaussian_pipeline.layout = layouts.gaussian };
    unsafe { SCRATCH.gaussian_pipeline.render_pass = gaussian.pass };
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
