use crate::config::{
    DescriptorBinding, DescriptorSet, Pass, Pipeline, Renderer, Subpass, VertexAttribute,
};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;

#[derive(Clone, PartialEq)]
enum BindingType {
    AccelerationStructure,
    Image,
    InputAttachment,
    Uniform,
}

impl Display for DescriptorSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl Display for Pipeline {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

pub fn generate_code(in_path: &str, renderer: &Renderer, mut file: File) {
    write!(
        file,
        r#"// Code generated from {in_path}.

use crate::renderer::raytracing::RaytraceResources;
use crate::renderer::shader::compile_glsl;
#[rustfmt::skip]
use crate::renderer::uniform::{{"#
    )
    .unwrap();
    let mut uniform_types = BTreeSet::new();
    for descriptor_set in &renderer.descriptor_sets {
        for binding in &descriptor_set.bindings {
            if let DescriptorBinding::Uniform(uniform) = binding {
                uniform_types.insert(uniform.typ.as_str());
            }
        }
    }
    for (i, typ) in uniform_types.iter().enumerate() {
        write!(file, "{typ}").unwrap();
        if i != uniform_types.len() - 1 {
            write!(file, ", ").unwrap();
        }
    }
    writeln!(
        file,
        r#"}};
use crate::renderer::util::{{AnyUniformBuffer, Dev, UniformBuffer}};
use crate::renderer::Swapchain;
use crate::renderer::{{Pass, FRAMES_IN_FLIGHT}};
use ash::vk;
use std::ffi::CStr;
use std::mem::MaybeUninit;

pub struct Samplers {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(file, "    pub {}: vk::Sampler,", sampler.name).unwrap();
    }
    writeln!(
        file,
        r#"}}

pub struct DescriptorSetLayouts {{"#
    )
    .unwrap();
    for descriptor_set in &renderer.descriptor_sets {
        writeln!(
            file,
            "    pub {}: vk::DescriptorSetLayout,",
            descriptor_set.name
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"}}

pub struct DescriptorPools {{"#
    )
    .unwrap();
    for descriptor_set in &renderer.descriptor_sets {
        writeln!(file, "    pub {descriptor_set}: vk::DescriptorPool,").unwrap();
        writeln!(
            file,
            "    pub {descriptor_set}_layout: vk::DescriptorSetLayout,"
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"}}

pub struct PipelineLayouts {{"#
    )
    .unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(file, "    pub {pipeline}: vk::PipelineLayout,").unwrap();
    });
    writeln!(
        file,
        r#"}}

pub struct Shaders {{"#
    )
    .unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(file, "    pub {pipeline}_vertex: Vec<u32>,").unwrap();
        writeln!(file, "    pub {pipeline}_fragment: Vec<u32>,").unwrap();
    });
    writeln!(
        file,
        r#"}}

pub struct ShaderModules {{"#
    )
    .unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(file, "    pub {pipeline}_vertex: vk::ShaderModule,").unwrap();
        writeln!(file, "    pub {pipeline}_fragment: vk::ShaderModule,").unwrap();
    });
    writeln!(
        file,
        r#"}}

#[repr(C)]
pub struct Pipelines {{"#
    )
    .unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(file, "    pub {pipeline}: vk::Pipeline,").unwrap();
    });
    writeln!(
        file,
        r#"}}

#[repr(C)]
struct Scratch {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(file, "    {}_sampler: vk::SamplerCreateInfo,", sampler.name).unwrap();
    }
    let mut all_pool_sizes: Vec<Vec<(BindingType, usize)>> =
        vec![Vec::new(); renderer.descriptor_sets.len()];
    for (descriptor_set_index, descriptor_set) in renderer.descriptor_sets.iter().enumerate() {
        let pool_sizes = &mut all_pool_sizes[descriptor_set_index];
        for binding in &descriptor_set.bindings {
            let binding_type = match binding {
                DescriptorBinding::AccelerationStructure(_) => BindingType::AccelerationStructure,
                DescriptorBinding::Image(_) => BindingType::Image,
                DescriptorBinding::InputAttachment(_) => BindingType::InputAttachment,
                DescriptorBinding::Uniform(_) => BindingType::Uniform,
            };
            let pool_size = match pool_sizes.iter_mut().find(|(ty, _)| *ty == binding_type) {
                Some(pool_size) => pool_size,
                None => {
                    pool_sizes.push((binding_type, 0));
                    pool_sizes.last_mut().unwrap()
                }
            };
            pool_size.1 += descriptor_set.pool_size * 2;
        }
    }
    for (descriptor_set_index, descriptor_set) in renderer.descriptor_sets.iter().enumerate() {
        let binding_count = descriptor_set.bindings.len();
        let pool_size_count = all_pool_sizes[descriptor_set_index].len();
        writeln!(
            file,
            r#"    {descriptor_set}_bindings: [vk::DescriptorSetLayoutBinding; {binding_count}],
    {descriptor_set}_layout: vk::DescriptorSetLayoutCreateInfo,
    {descriptor_set}_pool_sizes: [vk::DescriptorPoolSize; {pool_size_count}],
    {descriptor_set}_pool: vk::DescriptorPoolCreateInfo,"#
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"    assembly: vk::PipelineInputAssemblyStateCreateInfo,
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
    viewport_state: vk::PipelineViewportStateCreateInfo,
    dynamic_state: vk::PipelineDynamicStateCreateInfo,"#
    )
    .unwrap();
    for_pipelines(renderer, |_, _, subpass, pipeline| {
        let binding_count = pipeline.vertex_bindings.len();
        let attribute_count = pipeline
            .vertex_bindings
            .iter()
            .flat_map(|binding| binding.attributes.iter())
            .filter(|attribute| !attribute.unused)
            .count();
        writeln!(
            file,
            r#"    {pipeline}_pipeline_layout: vk::PipelineLayoutCreateInfo,
    {pipeline}_shader_vertex: vk::ShaderModuleCreateInfo,
    {pipeline}_shader_fragment: vk::ShaderModuleCreateInfo,
    {pipeline}_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    {pipeline}_vertex_bindings: [vk::VertexInputBindingDescription; {binding_count}],
    {pipeline}_vertex_attributes: [vk::VertexInputAttributeDescription; {attribute_count}],
    {pipeline}_vertex: vk::PipelineVertexInputStateCreateInfo,
    {pipeline}_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    {pipeline}_multisampling: vk::PipelineMultisampleStateCreateInfo,
    {pipeline}_blend_attachments: [vk::PipelineColorBlendAttachmentState; {}],
    {pipeline}_blend: vk::PipelineColorBlendStateCreateInfo,
    {pipeline}_depth: vk::PipelineDepthStencilStateCreateInfo,"#,
            subpass.color_attachments.len()
        )
        .unwrap();
    });
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(
            file,
            "    {pipeline}_pipeline: vk::GraphicsPipelineCreateInfo,"
        )
        .unwrap();
    });
    writeln!(
        file,
        r#"}}

#[rustfmt::skip]
static mut SCRATCH: Scratch = Scratch {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(
            file,
            r"    {}_sampler: vk::SamplerCreateInfo {{
        s_type: vk::StructureType::SAMPLER_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::SamplerCreateFlags::empty(),
        mag_filter: vk::Filter::NEAREST,
        min_filter: vk::Filter::NEAREST,
        mipmap_mode: vk::SamplerMipmapMode::NEAREST,
        address_mode_u: vk::SamplerAddressMode::{},
        address_mode_v: vk::SamplerAddressMode::{},
        address_mode_w: vk::SamplerAddressMode::REPEAT,
        mip_lod_bias: 0.,
        anisotropy_enable: 0,
        max_anisotropy: 0.,
        compare_enable: 0,
        compare_op: vk::CompareOp::NEVER,
        min_lod: 0.,
        max_lod: 0.,
        border_color: vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
        unnormalized_coordinates: {},
    }},",
            sampler.name,
            sampler.address_mode_u,
            sampler.address_mode_v,
            if sampler.unnormalized_coordinates {
                1
            } else {
                0
            }
        )
        .unwrap();
    }
    for (descriptor_set_index, descriptor_set) in renderer.descriptor_sets.iter().enumerate() {
        writeln!(file, "    {}_bindings: [", descriptor_set.name).unwrap();
        for (binding_index, binding) in descriptor_set.bindings.iter().enumerate() {
            let (typ, stage) = match binding {
                DescriptorBinding::AccelerationStructure(as_) => {
                    ("ACCELERATION_STRUCTURE_KHR", &as_.stage)
                }
                DescriptorBinding::Image(image) => ("COMBINED_IMAGE_SAMPLER", &image.stage),
                DescriptorBinding::InputAttachment(input) => ("INPUT_ATTACHMENT", &input.stage),
                DescriptorBinding::Uniform(uniform) => ("UNIFORM_BUFFER", &uniform.stage),
            };
            writeln!(
                file,
                r#"        vk::DescriptorSetLayoutBinding {{
            binding: {binding_index},
            descriptor_type: vk::DescriptorType::{typ},
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::{stage},
            p_immutable_samplers: std::ptr::null(),
        }},"#,
            )
            .unwrap();
        }
        let binding_count = descriptor_set.bindings.len();
        writeln!(
            file,
            r"    ],
    {descriptor_set}_layout: vk::DescriptorSetLayoutCreateInfo {{
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: {binding_count},
        p_bindings: unsafe {{ SCRATCH.{descriptor_set}_bindings.as_ptr() }},
    }},
    {descriptor_set}_pool_sizes: [",
        )
        .unwrap();
        for (binding_type, size) in &all_pool_sizes[descriptor_set_index] {
            let binding_type_vk = match binding_type {
                BindingType::AccelerationStructure => "ACCELERATION_STRUCTURE_KHR",
                BindingType::Image => "COMBINED_IMAGE_SAMPLER",
                BindingType::InputAttachment => "INPUT_ATTACHMENT",
                BindingType::Uniform => "UNIFORM_BUFFER",
            };
            writeln!(
                file,
                r"        vk::DescriptorPoolSize {{
            ty: vk::DescriptorType::{binding_type_vk},
            descriptor_count: {size},
        }},"
            )
            .unwrap();
        }
        let max_sets = descriptor_set.pool_size * 2;
        let pool_size_count = all_pool_sizes[descriptor_set_index].len();
        writeln!(
            file,
            "    ],
    {descriptor_set}_pool: vk::DescriptorPoolCreateInfo {{
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: {max_sets},
        pool_size_count: {pool_size_count},
        p_pool_sizes: unsafe {{ SCRATCH.{descriptor_set}_pool_sizes.as_ptr() }},
    }},"
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"    assembly: vk::PipelineInputAssemblyStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        primitive_restart_enable: 0,
    }},
    viewport: vk::Viewport {{
        x: 0.,
        y: 0.,
        width: 0.,
        height: 0.,
        min_depth: 0.,
        max_depth: 1.,
    }},
    scissor: vk::Rect2D {{
        offset: vk::Offset2D {{ x: 0, y: 0 }},
        extent: vk::Extent2D {{ width: 0, height: 0 }},
    }},
    viewport_state: vk::PipelineViewportStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        p_viewports: unsafe {{ &SCRATCH.viewport }},
        scissor_count: 1,
        p_scissors: unsafe {{ &SCRATCH.scissor }},
    }},
    dynamic_state: vk::PipelineDynamicStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDynamicStateCreateFlags::empty(),
        dynamic_state_count: 0,
        p_dynamic_states: std::ptr::null(),
    }},"#
    )
    .unwrap();
    for_pipelines(renderer, |pass, subpass_index, subpass, pipeline| {
        let descriptor_count = pipeline.descriptor_sets.len();
        let attribute_count = pipeline
            .vertex_bindings
            .iter()
            .flat_map(|binding| binding.attributes.iter())
            .filter(|attribute| !attribute.unused)
            .count();
        writeln!(
            file,
            r#"    {pipeline}_pipeline_layout: vk::PipelineLayoutCreateInfo {{
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: {descriptor_count},
        p_set_layouts: std::ptr::null(),
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    }},
    {pipeline}_shader_vertex: vk::ShaderModuleCreateInfo {{
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    }},
    {pipeline}_shader_fragment: vk::ShaderModuleCreateInfo {{
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
    }},
    {pipeline}_shader_stages: [
        vk::PipelineShaderStageCreateInfo {{
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::VERTEX,
            module: vk::ShaderModule::null(),
            p_name: unsafe {{ CStr::from_bytes_with_nul_unchecked(b"main\0") }}.as_ptr(),
            p_specialization_info: std::ptr::null(),
        }},
        vk::PipelineShaderStageCreateInfo {{
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: vk::ShaderModule::null(),
            p_name: unsafe {{ CStr::from_bytes_with_nul_unchecked(b"main\0") }}.as_ptr(),
            p_specialization_info: std::ptr::null(),
        }},
    ],
    {pipeline}_vertex_bindings: ["#
        )
        .unwrap();
        for (binding_index, binding) in pipeline.vertex_bindings.iter().enumerate() {
            let stride: usize = binding.attributes.iter().map(attribute_size).sum();
            let rate = &binding.rate;
            writeln!(
                file,
                r#"        vk::VertexInputBindingDescription {{
            binding: {binding_index},
            stride: {stride},
            input_rate: vk::VertexInputRate::{rate},
        }},"#,
            )
            .unwrap();
        }
        writeln!(
            file,
            r#"    ],
    {pipeline}_vertex_attributes: ["#
        )
        .unwrap();
        let mut total_locations = 0;
        for (binding_index, binding) in pipeline.vertex_bindings.iter().enumerate() {
            let mut offset = 0;
            for attribute in &binding.attributes {
                if !attribute.unused {
                    let format = &attribute.format;
                    writeln!(
                        file,
                        r#"        vk::VertexInputAttributeDescription {{
            binding: {binding_index},
            location: {total_locations},
            format: vk::Format::{format},
            offset: {offset},
        }},"#
                    )
                    .unwrap();
                    total_locations += 1;
                }
                offset += attribute_size(attribute);
            }
        }
        let vertex_binding_count = pipeline.vertex_bindings.len();
        let cull_mode = &pipeline.cull_mode;
        let rasterization_samples = if pass.msaa { "TYPE_2" } else { "TYPE_1" };
        writeln!(
            file,
            r#"    ],
    {pipeline}_vertex: vk::PipelineVertexInputStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: {vertex_binding_count},
        p_vertex_binding_descriptions: unsafe {{ SCRATCH.{pipeline}_vertex_bindings.as_ptr() }},
        vertex_attribute_description_count: {attribute_count},
        p_vertex_attribute_descriptions: unsafe {{ SCRATCH.{pipeline}_vertex_attributes.as_ptr() }},
    }},
    {pipeline}_rasterizer: vk::PipelineRasterizationStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable: 0,
        rasterizer_discard_enable: 0,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::{cull_mode},
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        depth_bias_enable: 0,
        depth_bias_constant_factor: 0.,
        depth_bias_clamp: 0.,
        depth_bias_slope_factor: 0.,
        line_width: 1.,
    }},
    {pipeline}_multisampling: vk::PipelineMultisampleStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineMultisampleStateCreateFlags::empty(),
        rasterization_samples: vk::SampleCountFlags::{rasterization_samples},
        sample_shading_enable: 0,
        min_sample_shading: 0.,
        p_sample_mask: std::ptr::null(),
        alpha_to_coverage_enable: 0,
        alpha_to_one_enable: 0,
    }},
    {pipeline}_blend_attachments: ["#
        )
        .unwrap();
        for _ in &subpass.color_attachments {
            writeln!(
                file,
                r#"        vk::PipelineColorBlendAttachmentState {{
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::ZERO,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        }},"#
            )
            .unwrap();
        }
        let depth_bool = if subpass.depth_attachment.is_some() {
            1
        } else {
            0
        };
        let color_attachment_count = subpass.color_attachments.len();
        writeln!(
            file,
            r#"    ],
    {pipeline}_blend: vk::PipelineColorBlendStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: {color_attachment_count},
        p_attachments: unsafe {{ SCRATCH.{pipeline}_blend_attachments.as_ptr() }},
        blend_constants: [0., 0., 0., 0.],
    }},
    {pipeline}_depth: vk::PipelineDepthStencilStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
        depth_test_enable: {depth_bool},
        depth_write_enable: {depth_bool},
        depth_compare_op: vk::CompareOp::LESS,
        depth_bounds_test_enable: 0,
        stencil_test_enable: 0,
        front: vk::StencilOpState {{
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::NEVER,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        }},
        back: vk::StencilOpState {{
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::NEVER,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        }},
        min_depth_bounds: 0.,
        max_depth_bounds: 1.,
    }},
    {pipeline}_pipeline: vk::GraphicsPipelineCreateInfo {{
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: 2,
        p_stages: unsafe {{ SCRATCH.{pipeline}_shader_stages.as_ptr() }},
        p_vertex_input_state: unsafe {{ &SCRATCH.{pipeline}_vertex }},
        p_input_assembly_state: unsafe {{ &SCRATCH.assembly }},
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: unsafe {{ &SCRATCH.viewport_state }},
        p_rasterization_state: unsafe {{ &SCRATCH.{pipeline}_rasterizer }},
        p_multisample_state: unsafe {{ &SCRATCH.{pipeline}_multisampling }},
        p_depth_stencil_state: unsafe {{ &SCRATCH.{pipeline}_depth }},
        p_color_blend_state: unsafe {{ &SCRATCH.{pipeline}_blend }},
        p_dynamic_state: unsafe {{ &SCRATCH.dynamic_state }},
        layout: vk::PipelineLayout::null(),
        render_pass: vk::RenderPass::null(),
        subpass: {subpass_index},
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    }},"#
        )
        .unwrap();
    });
    writeln!(
        file,
        r#"}};

impl Samplers {{
    pub fn cleanup(&self, dev: &Dev) {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(
            file,
            "        unsafe {{ dev.destroy_sampler(self.{}, None) }};",
            sampler.name
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}

impl DescriptorSetLayouts {{
    pub fn cleanup(&self, dev: &Dev) {{"#
    )
    .unwrap();
    for descriptor_set in &renderer.descriptor_sets {
        writeln!(
            file,
            "        unsafe {{ dev.destroy_descriptor_set_layout(self.{}, None) }};",
            descriptor_set.name
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}

#[rustfmt::skip]
impl DescriptorPools {{"#
    )
    .unwrap();
    for descriptor_set in &renderer.descriptor_sets {
        writeln!(
            file,
            r#"    pub fn alloc_{descriptor_set}(
        &self,"#
        )
        .unwrap();
        for binding in &descriptor_set.bindings {
            match binding {
                DescriptorBinding::AccelerationStructure(as_) => {
                    let name = &as_.name;
                    writeln!(file, "        {name}: &Option<RaytraceResources>,").unwrap();
                }
                DescriptorBinding::Image(image) => {
                    let name = &image.name;
                    writeln!(file, "        {name}: vk::ImageView,").unwrap();
                }
                DescriptorBinding::InputAttachment(input) => {
                    let name = &input.name;
                    writeln!(file, "        {name}: vk::ImageView,").unwrap();
                }
                DescriptorBinding::Uniform(uniform) => {
                    let name = &uniform.name;
                    let typ = &uniform.typ;
                    writeln!(file, "        {name}: &UniformBuffer<{typ}>,").unwrap();
                }
            }
        }
        writeln!(
            file,
            r#"        dev: &Dev,
    ) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {{
        let layouts = [self.{descriptor_set}_layout; FRAMES_IN_FLIGHT];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.{descriptor_set})
            .set_layouts(&layouts);
        let descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
            unsafe {{ dev.allocate_descriptor_sets(&descriptor_set_alloc_info) }}
                .unwrap()
                .try_into()
                .unwrap();
        for (flight_index, descriptor_set) in descriptor_sets.iter().enumerate() {{"#
        )
        .unwrap();
        for (binding_index, binding) in descriptor_set.bindings.iter().enumerate() {
            let binding_name = match binding {
                DescriptorBinding::AccelerationStructure(as_) => &as_.name,
                DescriptorBinding::Image(image) => &image.name,
                DescriptorBinding::InputAttachment(input) => &input.name,
                DescriptorBinding::Uniform(uniform) => &uniform.name,
            };
            let binding_type = match binding {
                DescriptorBinding::AccelerationStructure(_) => "ACCELERATION_STRUCTURE_KHR",
                DescriptorBinding::Image(_) => "COMBINED_IMAGE_SAMPLER",
                DescriptorBinding::InputAttachment(_) => "INPUT_ATTACHMENT",
                DescriptorBinding::Uniform(_) => "UNIFORM_BUFFER",
            };
            let write_mutable = match binding {
                DescriptorBinding::AccelerationStructure(_) => "mut ",
                DescriptorBinding::Image(_) => "",
                DescriptorBinding::InputAttachment(_) => "",
                DescriptorBinding::Uniform(_) => "",
            };
            match binding {
                DescriptorBinding::AccelerationStructure(_) => writeln!(
                    file,
                    r#"            let mut {binding_name}_acceleration_structure = *vk::WriteDescriptorSetAccelerationStructureKHR::builder()
                .acceleration_structures(std::slice::from_ref(&{binding_name}.as_ref().unwrap().acceleration_structure));"#
                )
                    .unwrap(),
                DescriptorBinding::Image(_) | DescriptorBinding::InputAttachment(_) => writeln!(
                    file,
                    r#"            let {binding_name}_image = *vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view({binding_name});"#
                )
                .unwrap(),
                DescriptorBinding::Uniform(_) => writeln!(
                    file,
                    r#"            let {binding_name}_buffer = {binding_name}.descriptor(flight_index);"#
                )
                .unwrap(),
            }
            writeln!(
                file,
                r#"            let {write_mutable}{binding_name} = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor_set)
                .dst_binding({binding_index})
                .descriptor_type(vk::DescriptorType::{binding_type})"#
            )
            .unwrap();
            match binding {
                DescriptorBinding::AccelerationStructure(_) => writeln!(
                    file,
                    r#"                .push_next(&mut {binding_name}_acceleration_structure);
            {binding_name}.descriptor_count = 1;"#
                )
                .unwrap(),
                DescriptorBinding::Image(_) => writeln!(
                    file,
                    r#"                .image_info(std::slice::from_ref(&{binding_name}_image));"#
                )
                .unwrap(),
                DescriptorBinding::InputAttachment(_) => writeln!(
                    file,
                    r#"                .image_info(std::slice::from_ref(&{binding_name}_image));"#
                )
                .unwrap(),
                DescriptorBinding::Uniform(_) => writeln!(
                    file,
                    r#"                .buffer_info(std::slice::from_ref(&{binding_name}_buffer));"#
                )
                .unwrap(),
            }
        }
        write!(file, r"            let writes = [").unwrap();
        for (binding_index, binding) in descriptor_set.bindings.iter().enumerate() {
            let binding_name = match binding {
                DescriptorBinding::AccelerationStructure(as_) => &as_.name,
                DescriptorBinding::Image(image) => &image.name,
                DescriptorBinding::InputAttachment(input) => &input.name,
                DescriptorBinding::Uniform(uniform) => &uniform.name,
            };
            write!(file, "{binding_name}").unwrap();
            if binding_index != descriptor_set.bindings.len() - 1 {
                write!(file, ", ").unwrap();
            }
        }
        writeln!(
            file,
            r#"];
            unsafe {{ dev.update_descriptor_sets(&writes, &[]) }};
        }}
        descriptor_sets
    }}
"#
        )
        .unwrap();
    }
    writeln!(file, r#"    pub fn cleanup(&self, dev: &Dev) {{"#).unwrap();
    for descriptor_set in &renderer.descriptor_sets {
        writeln!(
            file,
            "        unsafe {{ dev.destroy_descriptor_pool(self.{descriptor_set}, None) }};"
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}

impl PipelineLayouts {{
    pub fn cleanup(&self, dev: &Dev) {{"#
    )
    .unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(
            file,
            "        unsafe {{ dev.destroy_pipeline_layout(self.{pipeline}, None) }};"
        )
        .unwrap();
    });
    writeln!(
        file,
        r#"    }}
}}

impl ShaderModules {{
    pub fn cleanup(&self, dev: &Dev) {{"#
    )
    .unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(
            file,
            "        unsafe {{ dev.destroy_shader_module(self.{pipeline}_vertex, None) }};"
        )
        .unwrap();
        writeln!(
            file,
            "        unsafe {{ dev.destroy_shader_module(self.{pipeline}_fragment, None) }};"
        )
        .unwrap();
    });
    writeln!(
        file,
        r#"    }}
}}

impl Pipelines {{
    pub fn cleanup(&self, dev: &Dev) {{"#
    )
    .unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(
            file,
            "        unsafe {{ dev.destroy_pipeline(self.{pipeline}, None) }};"
        )
        .unwrap();
    });
    writeln!(
        file,
        r#"    }}
}}

#[rustfmt::skip]
pub fn create_samplers(dev: &Dev) -> Samplers {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(file, "    let {} = unsafe {{ dev.create_sampler(&SCRATCH.{}_sampler, None).unwrap_unchecked() }};", sampler.name, sampler.name).unwrap();
    }
    writeln!(file, "    Samplers {{").unwrap();
    for sampler in &renderer.samplers {
        writeln!(file, "        {},", sampler.name).unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}

#[rustfmt::skip]
pub fn create_descriptor_set_layouts(samplers: &Samplers, dev: &Dev) -> DescriptorSetLayouts {{"#
    )
    .unwrap();
    for descriptor_set in &renderer.descriptor_sets {
        for (binding_index, binding) in descriptor_set.bindings.iter().enumerate() {
            if let DescriptorBinding::Image(image) = binding {
                writeln!(
                    file,
                    "    unsafe {{ SCRATCH.{}_bindings[{binding_index}].p_immutable_samplers = &samplers.{} }};",
                    descriptor_set.name, image.sampler,
                )
                .unwrap();
            }
        }
    }

    for descriptor_set in &renderer.descriptor_sets {
        writeln!(file, "    let {} = unsafe {{ dev.create_descriptor_set_layout(&SCRATCH.{}_layout, None).unwrap_unchecked() }};", descriptor_set.name, descriptor_set.name).unwrap();
    }
    writeln!(file, "    DescriptorSetLayouts {{").unwrap();
    for descriptor_set in &renderer.descriptor_sets {
        writeln!(file, "        {},", descriptor_set.name).unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}

#[rustfmt::skip]
pub fn create_descriptor_pools(layouts: &DescriptorSetLayouts, dev: &Dev) -> DescriptorPools {{"#
    )
    .unwrap();
    for descriptor_set in &renderer.descriptor_sets {
        writeln!(file, "    let {descriptor_set} = unsafe {{ dev.create_descriptor_pool(&SCRATCH.{descriptor_set}_pool, None).unwrap_unchecked() }};").unwrap();
    }
    writeln!(file, "    DescriptorPools {{").unwrap();
    for descriptor_set in &renderer.descriptor_sets {
        writeln!(file, "        {descriptor_set},").unwrap();
        writeln!(
            file,
            "        {descriptor_set}_layout: layouts.{descriptor_set},"
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}

#[rustfmt::skip]
pub fn create_pipeline_layouts(
    descriptor_set_layouts: &DescriptorSetLayouts,
    dev: &Dev,
) -> PipelineLayouts {{"#
    )
    .unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        let descriptor_set = &pipeline.descriptor_sets[0];
        writeln!(file, r#"    unsafe {{ SCRATCH.{pipeline}_pipeline_layout.p_set_layouts = &descriptor_set_layouts.{descriptor_set} }};"#).unwrap();
    });
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(file, r#"    let {pipeline} = unsafe {{ dev.create_pipeline_layout(&SCRATCH.{pipeline}_pipeline_layout, None).unwrap_unchecked() }};"#).unwrap();
    });
    writeln!(file, "    PipelineLayouts {{").unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(file, "        {pipeline},").unwrap();
    });
    writeln!(
        file,
        r#"    }}
}}

#[rustfmt::skip]
pub fn create_shaders(supports_raytracing: bool) -> Shaders {{"#
    )
    .unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(file, r#"    let {pipeline}_vertex = compile_glsl("shaders/{pipeline}.vert", shaderc::ShaderKind::Vertex, supports_raytracing);"#).unwrap();
        writeln!(file, r#"    let {pipeline}_fragment = compile_glsl("shaders/{pipeline}.frag", shaderc::ShaderKind::Fragment, supports_raytracing);"#).unwrap();
    });
    writeln!(file, "    Shaders {{").unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(file, "        {pipeline}_vertex,").unwrap();
        writeln!(file, "        {pipeline}_fragment,").unwrap();
    });
    writeln!(
        file,
        r#"    }}
}}

#[rustfmt::skip]
pub fn create_shader_modules(shaders: &Shaders, dev: &Dev) -> ShaderModules {{"#
    )
    .unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(
            file,
            "    unsafe {{ SCRATCH.{pipeline}_shader_vertex.code_size = 4 * shaders.{pipeline}_vertex.len() }};"
        )
            .unwrap();
        writeln!(
            file,
            "    unsafe {{ SCRATCH.{pipeline}_shader_fragment.code_size = 4 * shaders.{pipeline}_fragment.len() }};"
        )
            .unwrap();
    });
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(
            file,
            "    unsafe {{ SCRATCH.{pipeline}_shader_vertex.p_code = shaders.{pipeline}_vertex.as_ptr() }};"
        )
        .unwrap();
        writeln!(
            file,
            "    unsafe {{ SCRATCH.{pipeline}_shader_fragment.p_code = shaders.{pipeline}_fragment.as_ptr() }};"
        )
        .unwrap();
    });
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(file, "    let {pipeline}_vertex = unsafe {{ dev.create_shader_module(&SCRATCH.{pipeline}_shader_vertex, None).unwrap_unchecked() }};").unwrap();
        writeln!(file, "    let {pipeline}_fragment = unsafe {{ dev.create_shader_module(&SCRATCH.{pipeline}_shader_fragment, None).unwrap_unchecked() }};").unwrap();
    });
    writeln!(file, "    ShaderModules {{").unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(file, "        {pipeline}_vertex,").unwrap();
        writeln!(file, "        {pipeline}_fragment,").unwrap();
    });
    writeln!(
        file,
        r#"    }}
}}

#[rustfmt::skip]
pub fn create_pipelines("#
    )
    .unwrap();
    for pass in &renderer.passes {
        writeln!(file, "    {}: &Pass,", pass.name).unwrap();
    }
    writeln!(
        file,
        r#"    _msaa_samples: vk::SampleCountFlags,
    swapchain: &Swapchain,
    shader_modules: &ShaderModules,
    layouts: &PipelineLayouts,
    dev: &Dev,
) -> Pipelines {{
    unsafe {{ SCRATCH.viewport.width = swapchain.extent.width as f32 }};
    unsafe {{ SCRATCH.viewport.height = swapchain.extent.height as f32 }};
    unsafe {{ SCRATCH.scissor.extent.width = swapchain.extent.width }};
    unsafe {{ SCRATCH.scissor.extent.height = swapchain.extent.height }};"#
    )
    .unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(
            file,
            r#"    unsafe {{ SCRATCH.{pipeline}_shader_stages[0].module = shader_modules.{pipeline}_vertex }};
    unsafe {{ SCRATCH.{pipeline}_shader_stages[1].module = shader_modules.{pipeline}_fragment }};"#
        )
            .unwrap();
    });
    for_pipelines(renderer, |pass, _, _, pipeline| {
        let pass = &pass.name;
        writeln!(
            file,
            r#"    unsafe {{ SCRATCH.{pipeline}_pipeline.layout = layouts.{pipeline} }};
    unsafe {{ SCRATCH.{pipeline}_pipeline.render_pass = {pass}.pass }};"#
        )
        .unwrap();
    });
    let mut pipeline_count = 0;
    let mut first_pipeline = None;
    for_pipelines(renderer, |_, _, _, pipeline| {
        pipeline_count += 1;
        if first_pipeline.is_none() {
            first_pipeline = Some(pipeline);
        }
    });
    let first_pipeline = first_pipeline.unwrap();
    writeln!(
        file,
        r#"    let mut pipelines = MaybeUninit::uninit();
    let _ = unsafe {{ (dev.fp_v1_0().create_graphics_pipelines)(
        dev.handle(),
        vk::PipelineCache::null(),
        {pipeline_count},
        &SCRATCH.{first_pipeline}_pipeline,
        std::ptr::null(),
        pipelines.as_mut_ptr() as *mut vk::Pipeline,
    ) }};
    unsafe {{ pipelines.assume_init() }}
}}"#
    )
    .unwrap();
}

fn for_pipelines<'a>(
    renderer: &'a Renderer,
    mut f: impl FnMut(&'a Pass, usize, &'a Subpass, &'a Pipeline),
) {
    for pass in &renderer.passes {
        for (subpass_index, subpass) in pass.subpasses.iter().enumerate() {
            for pipeline in &subpass.pipelines {
                f(pass, subpass_index, subpass, pipeline);
            }
        }
    }
}

fn attribute_size(attribute: &VertexAttribute) -> usize {
    match attribute.format.as_str() {
        "R32_SFLOAT" => 4,
        "R32G32B32_SFLOAT" => 12,
        _ => todo!("attribute_size({:?})", attribute.format),
    }
}
