use crate::config::{DescriptorBinding, Renderer, VertexAttribute};
use std::fs::File;
use std::io::Write;

pub fn generate_code(in_path: &str, renderer: &Renderer, mut file: File) {
    writeln!(
        file,
        r#"// Code generated from {in_path}.

use crate::renderer::shader::create_shader;
use crate::renderer::util::Dev;
use crate::renderer::Pass;
use crate::renderer::Pipeline;
use crate::renderer::Swapchain;
use ash::vk;
use std::ffi::CStr;

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

pub struct Pipelines {{"#
    )
    .unwrap();
    for pass in &renderer.passes {
        for subpass in &pass.subpasses {
            for pipeline in &subpass.pipelines {
                writeln!(file, "    pub {}: Pipeline,", pipeline.name).unwrap();
            }
        }
    }
    writeln!(
        file,
        r#"}}

struct Scratch {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(file, "    {}_sampler: vk::SamplerCreateInfo,", sampler.name).unwrap();
    }
    for descriptor_set in &renderer.descriptor_sets {
        writeln!(
            file,
            r#"    {}_bindings: [vk::DescriptorSetLayoutBinding; {}],
    {}_layout: vk::DescriptorSetLayoutCreateInfo,"#,
            descriptor_set.name,
            descriptor_set.bindings.len(),
            descriptor_set.name
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
    for pass in &renderer.passes {
        for subpass in &pass.subpasses {
            for pipeline in &subpass.pipelines {
                let name = &pipeline.name;
                let binding_count = pipeline.vertex_bindings.len();
                let attribute_count = pipeline
                    .vertex_bindings
                    .iter()
                    .flat_map(|binding| binding.attributes.iter())
                    .filter(|attribute| !attribute.unused)
                    .count();
                writeln!(
                    file,
                    r#"    {name}_pipeline_layout: vk::PipelineLayoutCreateInfo,
    {name}_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    {name}_vertex_bindings: [vk::VertexInputBindingDescription; {binding_count}],
    {name}_vertex_attributes: [vk::VertexInputAttributeDescription; {attribute_count}],
    {name}_vertex: vk::PipelineVertexInputStateCreateInfo,
    {name}_rasterizer: vk::PipelineRasterizationStateCreateInfo,
    {name}_multisampling: vk::PipelineMultisampleStateCreateInfo,
    {name}_blend_attachments: [vk::PipelineColorBlendAttachmentState; {}],
    {name}_blend: vk::PipelineColorBlendStateCreateInfo,
    {name}_depth: vk::PipelineDepthStencilStateCreateInfo,
    {name}_pipeline: vk::GraphicsPipelineCreateInfo,"#,
                    subpass.color_attachments.len()
                )
                .unwrap();
            }
        }
    }
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
    for descriptor_set in &renderer.descriptor_sets {
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
        writeln!(
            file,
            r"    ],
    {}_layout: vk::DescriptorSetLayoutCreateInfo {{
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: {},
        p_bindings: unsafe {{ SCRATCH.{}_bindings.as_ptr() }},
    }},",
            descriptor_set.name,
            descriptor_set.bindings.len(),
            descriptor_set.name,
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
    for pass in &renderer.passes {
        for (subpass_index, subpass) in pass.subpasses.iter().enumerate() {
            for pipeline in &subpass.pipelines {
                let name = &pipeline.name;
                let attribute_count = pipeline
                    .vertex_bindings
                    .iter()
                    .flat_map(|binding| binding.attributes.iter())
                    .filter(|attribute| !attribute.unused)
                    .count();
                writeln!(
                    file,
                    r#"    {name}_pipeline_layout: vk::PipelineLayoutCreateInfo {{
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: {},
        p_set_layouts: std::ptr::null(),
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    }},
    {name}_shader_stages: [
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
    {name}_vertex_bindings: ["#,
                    pipeline.descriptor_sets.len(),
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
    {name}_vertex_attributes: ["#
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
                writeln!(
                    file,
                    r#"    ],
    {name}_vertex: vk::PipelineVertexInputStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: {},
        p_vertex_binding_descriptions: unsafe {{ SCRATCH.{name}_vertex_bindings.as_ptr() }},
        vertex_attribute_description_count: {attribute_count},
        p_vertex_attribute_descriptions: unsafe {{ SCRATCH.{name}_vertex_attributes.as_ptr() }},
    }},
    {name}_rasterizer: vk::PipelineRasterizationStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable: 0,
        rasterizer_discard_enable: 0,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::{},
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        depth_bias_enable: 0,
        depth_bias_constant_factor: 0.,
        depth_bias_clamp: 0.,
        depth_bias_slope_factor: 0.,
        line_width: 1.,
    }},
    {name}_multisampling: vk::PipelineMultisampleStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineMultisampleStateCreateFlags::empty(),
        rasterization_samples: vk::SampleCountFlags::{},
        sample_shading_enable: 0,
        min_sample_shading: 0.,
        p_sample_mask: std::ptr::null(),
        alpha_to_coverage_enable: 0,
        alpha_to_one_enable: 0,
    }},
    {name}_blend_attachments: ["#,
                    pipeline.vertex_bindings.len(),
                    pipeline.cull_mode,
                    if pass.msaa { "TYPE_2" } else { "TYPE_1" },
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
                writeln!(
                    file,
                    r#"    ],
    {name}_blend: vk::PipelineColorBlendStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: {},
        p_attachments: unsafe {{ SCRATCH.{name}_blend_attachments.as_ptr() }},
        blend_constants: [0., 0., 0., 0.],
    }},
    {name}_depth: vk::PipelineDepthStencilStateCreateInfo {{
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
    {name}_pipeline: vk::GraphicsPipelineCreateInfo {{
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: 2,
        p_stages: unsafe {{ SCRATCH.{name}_shader_stages.as_ptr() }},
        p_vertex_input_state: unsafe {{ &SCRATCH.{name}_vertex }},
        p_input_assembly_state: unsafe {{ &SCRATCH.assembly }},
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: unsafe {{ &SCRATCH.viewport_state }},
        p_rasterization_state: unsafe {{ &SCRATCH.{name}_rasterizer }},
        p_multisample_state: unsafe {{ &SCRATCH.{name}_multisampling }},
        p_depth_stencil_state: unsafe {{ &SCRATCH.{name}_depth }},
        p_color_blend_state: unsafe {{ &SCRATCH.{name}_blend }},
        p_dynamic_state: unsafe {{ &SCRATCH.dynamic_state }},
        layout: vk::PipelineLayout::null(),
        render_pass: vk::RenderPass::null(),
        subpass: {subpass_index},
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    }},"#,
                    subpass.color_attachments.len()
                )
                .unwrap();
            }
        }
    }
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

impl Pipelines {{
    pub fn cleanup(&self, dev: &Dev) {{"#
    )
    .unwrap();
    for pass in &renderer.passes {
        for subpass in &pass.subpasses {
            for pipeline in &subpass.pipelines {
                writeln!(file, "        self.{}.cleanup(dev);", pipeline.name).unwrap();
            }
        }
    }
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
    supports_raytracing: bool,
    descriptor_set_layouts: &DescriptorSetLayouts,
    dev: &Dev,
) -> Pipelines {{
    unsafe {{ SCRATCH.viewport.width = swapchain.extent.width as f32 }};
    unsafe {{ SCRATCH.viewport.height = swapchain.extent.height as f32 }};
    unsafe {{ SCRATCH.scissor.extent.width = swapchain.extent.width }};
    unsafe {{ SCRATCH.scissor.extent.height = swapchain.extent.height }};"#
    )
    .unwrap();
    for pass in &renderer.passes {
        let pass_name = &pass.name;
        for subpass in &pass.subpasses {
            for pipeline in &subpass.pipelines {
                let name = &pipeline.name;
                assert_eq!(pipeline.descriptor_sets.len(), 1);
                let descriptor_set = &pipeline.descriptor_sets[0];
                writeln!(file, r#"    unsafe {{ SCRATCH.{name}_pipeline_layout.p_set_layouts = &descriptor_set_layouts.{descriptor_set} }};
    let layout = unsafe {{ dev.create_pipeline_layout(&SCRATCH.{name}_pipeline_layout, None).unwrap_unchecked() }};
    let vertex_shader = create_shader(
        "shaders/{name}.vert",
        vk::ShaderStageFlags::VERTEX,
        supports_raytracing,
        dev,
    );
    let fragment_shader = create_shader(
        "shaders/{name}.frag",
        vk::ShaderStageFlags::FRAGMENT,
        supports_raytracing,
        dev,
    );
    unsafe {{ SCRATCH.{name}_shader_stages[0].module = vertex_shader.module }};
    unsafe {{ SCRATCH.{name}_shader_stages[1].module = fragment_shader.module }};
    unsafe {{ SCRATCH.{name}_pipeline.layout = layout }};
    unsafe {{ SCRATCH.{name}_pipeline.render_pass = {pass_name}.pass }};
    let pipeline = unsafe {{ dev.create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&SCRATCH.{name}_pipeline), None).unwrap_unchecked()[0] }};
    let {name} = Pipeline {{ layout, pipeline }};"#).unwrap();
            }
        }
    }
    writeln!(file, "    Pipelines {{").unwrap();
    for pass in &renderer.passes {
        for subpass in &pass.subpasses {
            for pipeline in &subpass.pipelines {
                writeln!(file, "        {},", pipeline.name).unwrap();
            }
        }
    }
    writeln!(
        file,
        r#"    }}
}}"#
    )
    .unwrap();
}

fn attribute_size(attribute: &VertexAttribute) -> usize {
    match attribute.format.as_str() {
        "R32_SFLOAT" => 4,
        "R32G32B32_SFLOAT" => 12,
        _ => todo!("attribute_size({:?})", attribute.format),
    }
}
