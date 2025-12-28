use crate::config::{Compute, Pipeline, Renderer, Sampler, VertexAttribute};
use crate::helper::to_camelcase;
use crate::reflect::collect_all_types;
use crate::types::{AshDescriptor, AshEnum, ShaderType};
use spirv_reflect::types::{ReflectDescriptorBinding, ReflectDescriptorType};
use spirv_reflect::ShaderModule;
use std::collections::{BTreeSet, HashMap};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

impl Display for Compute {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl Display for Pipeline {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl Display for Sampler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

pub fn generate_code(renderer: &Renderer, reflection: &ShaderModule) {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    generate_codegen(renderer, reflection, &out_dir);
    generate_uniform(reflection, &out_dir);
}

fn generate_codegen(renderer: &Renderer, reflection: &ShaderModule, out_dir: &Path) {
    let descriptor_sets = reflection.enumerate_descriptor_sets(None).unwrap();
    assert_eq!(descriptor_sets.len(), 1);
    let descriptor_set = &descriptor_sets[0];

    let mut file = File::create(out_dir.join("codegen.rs")).unwrap();
    write!(file, r#"use crate::gpu::{{"#).unwrap();
    for binding in &descriptor_set.bindings {
        if let Some(struct_type) = binding.struct_type() {
            write!(file, "{struct_type},").unwrap();
        }
    }
    write!(
        file,
        r#"}};
use crate::renderer::debug::set_label;
use crate::renderer::shader::SpvArray;
use crate::renderer::util::{{AsDescriptor, Dev, ImageResources, StorageBuffer, UniformBuffer}};
use crate::renderer::{{DeviceSupport, Swapchain, COLOR_FORMAT, DEPTH_FORMAT, FRAMES_IN_FLIGHT}};
use ash::vk;
use std::ffi::{{c_void, CStr}};
use std::mem::MaybeUninit;

pub struct Samplers {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(file, "    pub {}: vk::Sampler,", sampler.name).unwrap();
    }
    writeln!(file, r#"}}"#).unwrap();
    let mut pipeline_vertex_shaders = HashMap::new();
    let mut pipeline_fragment_shaders = HashMap::new();
    let mut pipeline_task_shaders = HashMap::new();
    let mut pipeline_mesh_shaders = HashMap::new();
    let mut shaders = BTreeSet::new();
    for pipeline in &renderer.pipelines {
        if pipeline.task_shaders {
            let task_shader = match &pipeline.task_shader {
                Some(path) => path.strip_suffix(".task").unwrap(),
                None => pipeline.name.as_str(),
            };
            pipeline_task_shaders.insert(pipeline.name.as_str(), task_shader);
            shaders.insert((task_shader, ShaderType::Task));
        }
        if pipeline.mesh_shaders {
            let mesh_shader = match &pipeline.mesh_shader {
                Some(path) => path.strip_suffix(".mesh").unwrap(),
                None => pipeline.name.as_str(),
            };
            pipeline_mesh_shaders.insert(pipeline.name.as_str(), mesh_shader);
            shaders.insert((mesh_shader, ShaderType::Mesh));
        } else {
            let vertex_shader = match &pipeline.vertex_shader {
                Some(path) => path.strip_suffix(".vert").unwrap(),
                None => pipeline.name.as_str(),
            };
            pipeline_vertex_shaders.insert(pipeline.name.as_str(), vertex_shader);
            shaders.insert((vertex_shader, ShaderType::Vertex));
        }
        let fragment_shader = match &pipeline.fragment_shader {
            Some(path) => path.strip_suffix(".frag").unwrap(),
            None => pipeline.name.as_str(),
        };
        pipeline_fragment_shaders.insert(pipeline.name.as_str(), fragment_shader);
        shaders.insert((fragment_shader, ShaderType::Fragment));
    }
    for compute in &renderer.computes {
        shaders.insert((compute.name.as_str(), ShaderType::Compute));
    }
    writeln!(
        file,
        r#"
#[repr(C)]
pub struct Pipelines {{"#
    )
    .unwrap();
    for pipeline in &renderer.pipelines {
        writeln!(file, "    pub {pipeline}: vk::Pipeline,").unwrap();
    }
    for compute in &renderer.computes {
        writeln!(file, "    pub {compute}: vk::Pipeline,").unwrap();
    }
    writeln!(
        file,
        r#"}}

"#
    )
    .unwrap();
    for pipeline in &renderer.pipelines {
        if let Some(specs) = &pipeline.fragment_specialization {
            let pipeline_camelcase = to_camelcase(&pipeline.to_string());
            writeln!(file, "struct {pipeline_camelcase}Specialization {{").unwrap();
            for spec in specs {
                let ty = &renderer.find_specialization(spec).ty;
                writeln!(file, "    {spec}: {ty},").unwrap();
            }
            writeln!(file, "}}").unwrap();
        }
    }
    writeln!(
        file,
        r#"
#[repr(C)]
struct Scratch {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(file, "    {}_sampler: vk::SamplerCreateInfo,", sampler.name).unwrap();
    }
    let mut pool_sizes = Vec::new();
    for binding in &descriptor_set.bindings {
        let pool_size = match pool_sizes
            .iter_mut()
            .find(|(ty, _)| *ty == binding.descriptor_type)
        {
            Some(pool_size) => pool_size,
            None => {
                pool_sizes.push((binding.descriptor_type, 0));
                pool_sizes.last_mut().unwrap()
            }
        };
        pool_size.1 += 2;
    }
    let binding_count = descriptor_set.bindings.len();
    let pool_size_count = pool_sizes.len();
    writeln!(
        file,
        r#"    descriptor_set_bindings: [vk::DescriptorSetLayoutBinding<'static>; {binding_count}],
    descriptor_set_layout: vk::DescriptorSetLayoutCreateInfo<'static>,
    descriptor_pool_sizes: [vk::DescriptorPoolSize; {pool_size_count}],
    descriptor_pool: vk::DescriptorPoolCreateInfo<'static>,
    assembly: vk::PipelineInputAssemblyStateCreateInfo<'static>,
    dynamic_states: [vk::DynamicState; 2],
    dynamic_state: vk::PipelineDynamicStateCreateInfo<'static>,"#
    )
    .unwrap();
    for (name, typ) in &shaders {
        let typ_lowercase = typ.lowercase();
        writeln!(
            file,
            r#"    {name}_{typ_lowercase}: vk::ShaderModuleCreateInfo<'static>,"#
        )
        .unwrap();
    }
    writeln!(
        file,
        "    pipeline_layout: vk::PipelineLayoutCreateInfo<'static>,"
    )
    .unwrap();
    for pipeline in &renderer.pipelines {
        let binding_count = pipeline.vertex_bindings.len();
        let attribute_count = pipeline
            .vertex_bindings
            .iter()
            .flat_map(|binding| binding.attributes.iter())
            .filter(|attribute| !attribute.unused)
            .count();
        if let Some(fragment_specialization) = &pipeline.fragment_specialization {
            let specialization_count = fragment_specialization.len();
            let pipeline_camelcase = to_camelcase(&pipeline.to_string());
            writeln!(
                file,
                r#"    {pipeline}_fragment_specialization_entries: [vk::SpecializationMapEntry; {specialization_count}],
    {pipeline}_fragment_specialization_info: vk::SpecializationInfo,
    {pipeline}_fragment_specialization_scratch: {pipeline_camelcase}Specialization,"#
            )
                .unwrap();
        }
        let shader_stage_count = if pipeline.task_shaders { 3 } else { 2 };
        writeln!(
            file,
            r#"    {pipeline}_shader_stages: [vk::PipelineShaderStageCreateInfo<'static>; {shader_stage_count}],"#
        )
            .unwrap();
        if pipeline.mesh_shaders {
        } else {
            writeln!(file, r#"    {pipeline}_vertex_bindings: [vk::VertexInputBindingDescription; {binding_count}],
    {pipeline}_vertex_attributes: [vk::VertexInputAttributeDescription; {attribute_count}],
    {pipeline}_vertex_state: vk::PipelineVertexInputStateCreateInfo<'static>,"#).unwrap();
        }
        writeln!(
            file,
            r#"    {pipeline}_viewport_state: vk::PipelineViewportStateCreateInfo<'static>,
    {pipeline}_rasterizer: vk::PipelineRasterizationStateCreateInfo<'static>,
    {pipeline}_multisampling: vk::PipelineMultisampleStateCreateInfo<'static>,
    {pipeline}_blend_attachments: [vk::PipelineColorBlendAttachmentState; 1],
    {pipeline}_blend: vk::PipelineColorBlendStateCreateInfo<'static>,
    {pipeline}_depth: vk::PipelineDepthStencilStateCreateInfo<'static>,
    {pipeline}_color_formats: [vk::Format; 1],
    {pipeline}_rendering: vk::PipelineRenderingCreateInfo<'static>,"#,
        )
        .unwrap();
    }
    for pipeline in &renderer.pipelines {
        writeln!(
            file,
            "    {pipeline}_pipeline: vk::GraphicsPipelineCreateInfo<'static>,"
        )
        .unwrap();
    }
    for compute in &renderer.computes {
        writeln!(
            file,
            r#"    {compute}_pipeline: vk::ComputePipelineCreateInfo,"#
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"}}

static mut SCRATCH: Scratch = Scratch {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        let filter = &sampler.filter;
        let address_mode = &sampler.address_mode;
        let unnormalized_coordinates = if sampler.unnormalized_coordinates {
            1
        } else {
            0
        };
        writeln!(
            file,
            r"    {sampler}_sampler: vk::SamplerCreateInfo {{
        s_type: vk::StructureType::SAMPLER_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::SamplerCreateFlags::empty(),
        mag_filter: vk::Filter::{filter},
        min_filter: vk::Filter::{filter},
        mipmap_mode: vk::SamplerMipmapMode::NEAREST,
        address_mode_u: vk::SamplerAddressMode::{address_mode},
        address_mode_v: vk::SamplerAddressMode::{address_mode},
        address_mode_w: vk::SamplerAddressMode::{address_mode},
        mip_lod_bias: 0.,
        anisotropy_enable: 0,
        max_anisotropy: 0.,
        compare_enable: 0,
        compare_op: vk::CompareOp::NEVER,
        min_lod: 0.,
        max_lod: 0.,
        border_color: vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
        unnormalized_coordinates: {unnormalized_coordinates},
    }},"
        )
        .unwrap();
    }

    writeln!(file, "    descriptor_set_bindings: [").unwrap();
    for binding in &descriptor_set.bindings {
        let binding_index = binding.binding;
        let description_type = binding.descriptor_type.ash_variant();
        writeln!(
            file,
            r#"        vk::DescriptorSetLayoutBinding {{
            binding: {binding_index},
            descriptor_type: vk::DescriptorType::{description_type},
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::ALL,
            p_immutable_samplers: std::ptr::null(),
            _marker: std::marker::PhantomData,
        }},"#,
        )
        .unwrap();
    }
    let binding_count = descriptor_set.bindings.len();
    writeln!(
        file,
        r"    ],
    descriptor_set_layout: vk::DescriptorSetLayoutCreateInfo {{
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: {binding_count},
        p_bindings: unsafe {{ &raw const (SCRATCH.descriptor_set_bindings[0]) }},
        _marker: std::marker::PhantomData,
    }},
    descriptor_pool_sizes: [",
    )
    .unwrap();
    for (binding_type, size) in &pool_sizes {
        let binding_type = binding_type.ash_variant();
        writeln!(
            file,
            r"        vk::DescriptorPoolSize {{
            ty: vk::DescriptorType::{binding_type},
            descriptor_count: {size},
        }},"
        )
        .unwrap();
    }
    let max_sets = 2;
    let pool_size_count = pool_sizes.len();
    writeln!(
        file,
        r#"    ],
    descriptor_pool: vk::DescriptorPoolCreateInfo {{
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        max_sets: {max_sets},
        pool_size_count: {pool_size_count},
        p_pool_sizes: unsafe {{ &raw const SCRATCH.descriptor_pool_sizes[0] }},
        _marker: std::marker::PhantomData,
    }},
    pipeline_layout: vk::PipelineLayoutCreateInfo {{
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 1,
        p_set_layouts: std::ptr::null(),
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
        _marker: std::marker::PhantomData,
    }},
    assembly: vk::PipelineInputAssemblyStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        primitive_restart_enable: 0,
        _marker: std::marker::PhantomData,
    }},
    dynamic_states: [
        vk::DynamicState::VIEWPORT,
        vk::DynamicState::SCISSOR,
    ],
    dynamic_state: vk::PipelineDynamicStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDynamicStateCreateFlags::empty(),
        dynamic_state_count: 2,
        p_dynamic_states: unsafe {{ &raw const SCRATCH.dynamic_states[0] }},
        _marker: std::marker::PhantomData,
    }},"#
    )
    .unwrap();
    for (name, typ) in &shaders {
        let name_uppercase = name.to_uppercase();
        let typ_lowercase = typ.lowercase();
        let typ_uppercase = typ_lowercase.to_uppercase();
        writeln!(
            file,
            r#"    {name}_{typ_lowercase}: vk::ShaderModuleCreateInfo {{
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: {name_uppercase}_{typ_uppercase}_SPV.0.len(),
        p_code: {name_uppercase}_{typ_uppercase}_SPV.0.as_ptr() as *const u32,
        _marker: std::marker::PhantomData,
    }},"#
        )
        .unwrap();
    }
    for pipeline in &renderer.pipelines {
        let attribute_count = pipeline
            .vertex_bindings
            .iter()
            .flat_map(|binding| binding.attributes.iter())
            .filter(|attribute| !attribute.unused)
            .count();
        let fragment_specialization_info = if let Some(fragment_specialization) =
            &pipeline.fragment_specialization
        {
            let pipeline_camelcase = to_camelcase(&pipeline.to_string());
            let specialization_count = fragment_specialization.len();
            writeln!(file, r#"    {pipeline}_fragment_specialization_entries: ["#).unwrap();
            let mut offset = 0;
            for (constant_id, spec) in fragment_specialization.iter().enumerate() {
                let size = renderer.find_specialization(spec).type_size();
                writeln!(
                    file,
                    r#"        vk::SpecializationMapEntry {{
            constant_id: {constant_id},
            offset: {offset},
            size: {size},
        }},"#
                )
                .unwrap();
                offset += size;
            }
            writeln!(
                file,
                r#"    ],
    {pipeline}_fragment_specialization_info: vk::SpecializationInfo {{
        map_entry_count: {specialization_count},
        p_map_entries: unsafe {{ &raw const SCRATCH.{pipeline}_fragment_specialization_entries[0] }},
        data_size: {offset},
        p_data: unsafe {{ (&raw const SCRATCH.{pipeline}_fragment_specialization_scratch) as *const std::ffi::c_void }},
    }},
    {pipeline}_fragment_specialization_scratch: {pipeline_camelcase}Specialization {{"#
            )
                .unwrap();
            for spec in fragment_specialization {
                let default = renderer.find_specialization(spec).type_default();
                writeln!(file, "        {spec}: {default},").unwrap();
            }
            writeln!(file, "    }},").unwrap();
            format!("unsafe {{ &raw const SCRATCH.{pipeline}_fragment_specialization_info }}")
        } else {
            "std::ptr::null()".to_owned()
        };
        let vertex_stage_type = if pipeline.mesh_shaders {
            "MESH_EXT"
        } else {
            "VERTEX"
        };
        let vertex_stage_lowercase = if pipeline.mesh_shaders {
            "mesh"
        } else {
            "vertex"
        };
        writeln!(file, r#"    {pipeline}_shader_stages: ["#).unwrap();
        if pipeline.task_shaders {
            writeln!(
                file,
                r#"        vk::PipelineShaderStageCreateInfo {{
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: unsafe {{ &raw const SCRATCH.{pipeline}_task }} as *const c_void,
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::TASK_EXT,
            module: vk::ShaderModule::null(),
            p_name: c"main".as_ptr(),
            p_specialization_info: std::ptr::null(),
            _marker: std::marker::PhantomData,
        }},"#
            )
            .unwrap();
        }
        writeln!(
            file,
            r#"        vk::PipelineShaderStageCreateInfo {{
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: unsafe {{ &raw const SCRATCH.{pipeline}_{vertex_stage_lowercase} }} as *const c_void,
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::{vertex_stage_type},
            module: vk::ShaderModule::null(),
            p_name: c"main".as_ptr(),
            p_specialization_info: std::ptr::null(),
            _marker: std::marker::PhantomData,
        }},
        vk::PipelineShaderStageCreateInfo {{
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: unsafe {{ &raw const SCRATCH.{pipeline}_fragment }} as *const c_void,
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: vk::ShaderModule::null(),
            p_name: c"main".as_ptr(),
            p_specialization_info: {fragment_specialization_info},
            _marker: std::marker::PhantomData,
        }},
    ],"#
        )
        .unwrap();
        if pipeline.mesh_shaders {
        } else {
            writeln!(file, r#"    {pipeline}_vertex_bindings: ["#).unwrap();
            for (binding_index, binding) in pipeline.vertex_bindings.iter().enumerate() {
                let raw_stride: usize = binding.attributes.iter().map(attribute_size).sum();
                let stride = raw_stride.next_multiple_of(4);
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
            let vertex_binding_descriptions = if vertex_binding_count > 0 {
                &format!("unsafe {{ &raw const SCRATCH.{pipeline}_vertex_bindings[0] }}")
            } else {
                "std::ptr::null()"
            };
            let vertex_attribute_descriptions = if attribute_count > 0 {
                &format!("unsafe {{ &raw const SCRATCH.{pipeline}_vertex_attributes[0] }}")
            } else {
                "std::ptr::null()"
            };
            writeln!(
                file,
                r#"    ],
    {pipeline}_vertex_state: vk::PipelineVertexInputStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: {vertex_binding_count},
        p_vertex_binding_descriptions: {vertex_binding_descriptions},
        vertex_attribute_description_count: {attribute_count},
        p_vertex_attribute_descriptions: {vertex_attribute_descriptions},
        _marker: std::marker::PhantomData,
    }},"#
            )
            .unwrap();
        }
        let polygon_mode = &pipeline.polygon_mode;
        let cull_mode = &pipeline.cull_mode;
        writeln!(
            file,
            r#"    {pipeline}_viewport_state: vk::PipelineViewportStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        p_viewports: std::ptr::null(),
        scissor_count: 1,
        p_scissors: std::ptr::null(),
        _marker: std::marker::PhantomData,
    }},
    {pipeline}_rasterizer: vk::PipelineRasterizationStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable: 0,
        rasterizer_discard_enable: 0,
        polygon_mode: vk::PolygonMode::{polygon_mode},
        cull_mode: vk::CullModeFlags::{cull_mode},
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        depth_bias_enable: 0,
        depth_bias_constant_factor: 0.,
        depth_bias_clamp: 0.,
        depth_bias_slope_factor: 0.,
        line_width: 1.,
        _marker: std::marker::PhantomData,
    }},
    {pipeline}_multisampling: vk::PipelineMultisampleStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineMultisampleStateCreateFlags::empty(),
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        sample_shading_enable: 0,
        min_sample_shading: 0.,
        p_sample_mask: std::ptr::null(),
        alpha_to_coverage_enable: 0,
        alpha_to_one_enable: 0,
        _marker: std::marker::PhantomData,
    }},
    {pipeline}_blend_attachments: [
        vk::PipelineColorBlendAttachmentState {{
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
        let vertex_input_state = if pipeline.mesh_shaders {
            "std::ptr::null()".to_owned()
        } else {
            format!("unsafe {{ &raw const SCRATCH.{pipeline}_vertex_state }}")
        };
        let shader_stage_count = if pipeline.task_shaders { 3 } else { 2 };
        writeln!(
            file,
            r#"    ],
    {pipeline}_blend: vk::PipelineColorBlendStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: 0,
        logic_op: vk::LogicOp::CLEAR,
        attachment_count: 1,
        p_attachments: unsafe {{ &raw const SCRATCH.{pipeline}_blend_attachments[0] }},
        blend_constants: [0., 0., 0., 0.],
        _marker: std::marker::PhantomData,
    }},
    {pipeline}_depth: vk::PipelineDepthStencilStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
        depth_test_enable: 1,
        depth_write_enable: 1,
        depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
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
        _marker: std::marker::PhantomData,
    }},
    {pipeline}_color_formats: [vk::Format::UNDEFINED],
    {pipeline}_rendering: vk::PipelineRenderingCreateInfo {{
        s_type: vk::StructureType::PIPELINE_RENDERING_CREATE_INFO,
        p_next: std::ptr::null(),
        view_mask: 0,
        color_attachment_count: 1,
        p_color_attachment_formats: unsafe {{ &raw const SCRATCH.{pipeline}_color_formats[0] }},
        depth_attachment_format: DEPTH_FORMAT,
        stencil_attachment_format: vk::Format::UNDEFINED,
        _marker: std::marker::PhantomData,
    }},
    {pipeline}_pipeline: vk::GraphicsPipelineCreateInfo {{
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: unsafe {{ &raw const SCRATCH.{pipeline}_rendering as *const _ }},
        flags: vk::PipelineCreateFlags::empty(),
        stage_count: {shader_stage_count},
        p_stages: unsafe {{ &raw const SCRATCH.{pipeline}_shader_stages[0] }},
        p_vertex_input_state: {vertex_input_state},
        p_input_assembly_state: unsafe {{ &raw const SCRATCH.assembly }},
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: unsafe {{ &raw const SCRATCH.{pipeline}_viewport_state }},
        p_rasterization_state: unsafe {{ &raw const SCRATCH.{pipeline}_rasterizer }},
        p_multisample_state: unsafe {{ &raw const SCRATCH.{pipeline}_multisampling }},
        p_depth_stencil_state: unsafe {{ &raw const SCRATCH.{pipeline}_depth }},
        p_color_blend_state: unsafe {{ &raw const SCRATCH.{pipeline}_blend }},
        p_dynamic_state: unsafe {{ &raw const SCRATCH.dynamic_state }},
        layout: vk::PipelineLayout::null(),
        render_pass: vk::RenderPass::null(),
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
        _marker: std::marker::PhantomData,
    }},"#
        )
        .unwrap();
    }
    for compute in &renderer.computes {
        writeln!(
            file,
            r#"    {compute}_pipeline: vk::ComputePipelineCreateInfo {{
        s_type: vk::StructureType::COMPUTE_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage: vk::PipelineShaderStageCreateInfo {{
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::COMPUTE,
            module: vk::ShaderModule::null(),
            p_name: c"main".as_ptr(),
            p_specialization_info: std::ptr::null(),
        }},
        layout: vk::PipelineLayout::null(),
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    }},"#
        )
        .unwrap();
    }
    writeln!(file, r#"}};"#).unwrap();
    for (name, typ) in &shaders {
        let name_uppercase = name.to_uppercase();
        let typ_lowercase = typ.lowercase();
        let typ_uppercase = typ_lowercase.to_uppercase();
        let ext = typ.extension();
        let bytes =
            format!(r#"include_bytes!(concat!(env!("OUT_DIR"), "/shaders/{name}.{ext}.spv"))"#);
        writeln!(file, r#"static {name_uppercase}_{typ_uppercase}_SPV: SpvArray<{{ {bytes}.len() }}> = SpvArray(*{bytes});"#).unwrap();
    }
    writeln!(
        file,
        r#"
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

pub fn alloc_descriptor_set("#
    )
    .unwrap();
    for binding in &descriptor_set.bindings {
        let name = &binding.name;
        let typ = binding.ash_value_type();
        writeln!(file, "    {name}: {typ},").unwrap();
    }
    write!(
        file,
        r#"    dev: &Dev,
    layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {{
    let layouts = [layout; FRAMES_IN_FLIGHT];
    let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
        unsafe {{ dev.allocate_descriptor_sets(&descriptor_set_alloc_info) }}
            .unwrap()
            .try_into()
            .unwrap();
    update_descriptor_set(&descriptors"#
    )
    .unwrap();
    for binding in &descriptor_set.bindings {
        let name = &binding.name;
        write!(file, ", {name}").unwrap();
    }

    writeln!(
        file,
        r#", dev);
    descriptors
}}

#[allow(clippy::unused_enumerate_index)]
pub fn update_descriptor_set(
    descriptors: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],"#
    )
    .unwrap();
    let mut only_tlas = None;
    for binding in &descriptor_set.bindings {
        let name = &binding.name;
        let typ = binding.ash_value_type();
        writeln!(file, "        {name}: {typ},").unwrap();
        if binding.descriptor_type == ReflectDescriptorType::AccelerationStructureKHR {
            assert!(only_tlas.is_none());
            assert_eq!(binding.binding as usize, descriptor_set.bindings.len() - 1);
            only_tlas = Some(name);
        }
    }
    writeln!(
        file,
        r#"    dev: &Dev,
    ) {{"#
    )
    .unwrap();
    if let Some(tlas) = only_tlas.as_ref() {
        writeln!(file, r#"    let supports_raytracing = {tlas}.is_some();"#).unwrap();
    }
    writeln!(
        file,
        r#"    for (_flight_index, descriptor) in descriptors.iter().enumerate() {{"#
    )
    .unwrap();
    for binding in &descriptor_set.bindings {
        let binding_index = binding.binding;
        let binding_name = &binding.name;
        let binding_type = binding.descriptor_type.ash_variant();
        let write_mutable = match binding.descriptor_type {
            ReflectDescriptorType::AccelerationStructureKHR => "mut ",
            _ => "",
        };
        match binding.descriptor_type {
            ReflectDescriptorType::AccelerationStructureKHR => writeln!(
                file,
                r#"        let mut {binding_name}_acceleration_structure = *vk::WriteDescriptorSetAccelerationStructureKHR::default()
            .acceleration_structures({binding_name}.as_ref().map(|as_| std::slice::from_ref(&as_.acceleration_structure)).unwrap_or_default());"#
            )
                .unwrap(),
            ReflectDescriptorType::SampledImage => {
                todo!()
            //     let layout = &image.layout;
            //     writeln!(
            //         file,
            //         r#"        let {binding_name}_image = *vk::DescriptorImageInfo::default()
            // .image_layout(vk::ImageLayout::{layout})
            // .image_view({binding_name});"#
            //     )
            //         .unwrap()
            }
            ReflectDescriptorType::InputAttachment => writeln!(
                file,
                r#"        let {binding_name}_image = *vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view({binding_name});"#
            )
                .unwrap(),
            ReflectDescriptorType::StorageBuffer => writeln!(file, r#"        let {binding_name}_buffer = {binding_name}.descriptor(_flight_index);"#).unwrap(),
            ReflectDescriptorType::StorageImage => writeln!(file,
                                                           r#"        let {binding_name}_image = *vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view({binding_name});"#
            ).unwrap(),
            ReflectDescriptorType::UniformBuffer => writeln!(
                file,
                r#"        let {binding_name}_buffer = {binding_name}.descriptor(_flight_index);"#
            )
                .unwrap(),
            _ => unimplemented!(),
        }
        writeln!(
            file,
            r#"        let {write_mutable}{binding_name} = vk::WriteDescriptorSet::default()
            .dst_set(*descriptor)
            .dst_binding({binding_index})
            .descriptor_type(vk::DescriptorType::{binding_type})"#
        )
        .unwrap();
        match binding.descriptor_type {
            ReflectDescriptorType::AccelerationStructureKHR => writeln!(
                file,
                r#"            .push_next(&mut {binding_name}_acceleration_structure);
        {binding_name}.descriptor_count = 1;"#
            )
            .unwrap(),
            ReflectDescriptorType::SampledImage
            | ReflectDescriptorType::InputAttachment
            | ReflectDescriptorType::StorageImage => writeln!(
                file,
                r#"            .image_info(std::slice::from_ref(&{binding_name}_image));"#
            )
            .unwrap(),
            ReflectDescriptorType::StorageBuffer => writeln!(
                file,
                r#"            .buffer_info(std::slice::from_ref(&{binding_name}_buffer));"#
            )
            .unwrap(),
            ReflectDescriptorType::UniformBuffer => writeln!(
                file,
                r#"            .buffer_info(std::slice::from_ref(&{binding_name}_buffer));"#
            )
            .unwrap(),
            _ => unimplemented!(),
        }
    }
    let write_writes = |file: &mut File, bindings: &[ReflectDescriptorBinding]| {
        write!(file, r"[").unwrap();
        for (binding_index, binding) in bindings.iter().enumerate() {
            let binding_name = &binding.name;
            write!(file, "{binding_name}").unwrap();
            if binding_index != bindings.len() - 1 {
                write!(file, ", ").unwrap();
            }
        }
        write!(file, "]").unwrap();
    };
    write!(file, r#"        let writes = "#).unwrap();
    write_writes(&mut file, &descriptor_set.bindings);
    writeln!(file, r#";"#).unwrap();
    if only_tlas.is_some() {
        let count_without_raytracing = descriptor_set.bindings.len() - 1;
        writeln!(
            file,
            r#"        let writes = if supports_raytracing {{
            &writes
        }} else {{
            &writes[..{count_without_raytracing}]
        }};"#
        )
        .unwrap();
    } else {
        writeln!(file, "        let writes = &writes;").unwrap();
    }
    writeln!(
        file,
        r#"        unsafe {{ dev.update_descriptor_sets(writes, &[]) }};
    }}
}}

impl Pipelines {{
    pub fn cleanup(&self, dev: &Dev) {{"#
    )
    .unwrap();
    for pipeline in &renderer.pipelines {
        writeln!(
            file,
            "        unsafe {{ dev.destroy_pipeline(self.{pipeline}, None) }};"
        )
        .unwrap();
    }
    for compute in &renderer.computes {
        writeln!(
            file,
            "        unsafe {{ dev.destroy_pipeline(self.{compute}, None) }};"
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}

pub fn create_samplers(dev: &Dev) -> Samplers {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(file, "    let {} = unsafe {{ dev.create_sampler(&*&raw const SCRATCH.{}_sampler, None).unwrap_unchecked() }};", sampler.name, sampler.name).unwrap();
    }
    writeln!(file, "    Samplers {{").unwrap();
    for sampler in &renderer.samplers {
        writeln!(file, "        {},", sampler.name).unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}

pub fn create_descriptor_set_layout(_samplers: &Samplers, dev: &Dev) -> vk::DescriptorSetLayout {{"#
    )
    .unwrap();
    for binding in &descriptor_set.bindings {
        // let binding_index = binding.binding;
        if binding.descriptor_type == ReflectDescriptorType::SampledImage {
            // writeln!(
            //     file,
            //     "    unsafe {{ SCRATCH.descriptor_set_bindings[{binding_index}].p_immutable_samplers = &_samplers.{} }};",
            //     image.sampler,
            // )
            //     .unwrap();
            todo!()
        }
    }
    writeln!(file, "    unsafe {{ dev.create_descriptor_set_layout(&*&raw const SCRATCH.descriptor_set_layout, None).unwrap_unchecked() }}").unwrap();
    writeln!(
        file,
        r#"}}

pub fn create_descriptor_pool(layout: vk::DescriptorSetLayout, dev: &Dev) -> vk::DescriptorPool {{
    unsafe {{ dev.create_descriptor_pool(&*&raw const SCRATCH.descriptor_pool, None).unwrap_unchecked() }}
}}

#[allow(clippy::identity_op)]
pub fn create_pipelines(
    _msaa_samples: vk::SampleCountFlags,"#
    )
    .unwrap();
    for spec in &renderer.specializations {
        let name = &spec.name;
        let ty = &spec.ty;
        if spec.shared {
            writeln!(file, "    {name}: {ty},").unwrap();
        }
    }
    for pipeline in &renderer.pipelines {
        if let Some(specs) = &pipeline.fragment_specialization {
            for spec in specs {
                let metadata = renderer.find_specialization(spec);
                if !metadata.shared {
                    let ty = &metadata.ty;
                    writeln!(file, "    {pipeline}_{spec}: {ty},").unwrap();
                }
            }
        }
    }
    writeln!(
        file,
        r#"    swapchain: &Swapchain,
    layout: vk::PipelineLayout,
    dev: &Dev,
) -> Pipelines {{"#
    )
    .unwrap();
    for pipeline in &renderer.pipelines {
        if let Some(specs) = &pipeline.fragment_specialization {
            for spec in specs {
                let metadata = renderer.find_specialization(spec);
                let value = if metadata.shared {
                    spec.clone()
                } else {
                    format!("{pipeline}_{spec}")
                };
                writeln!(file, "    unsafe {{ SCRATCH.{pipeline}_fragment_specialization_scratch.{spec} = {value} }};").unwrap();
            }
        }
        writeln!(
            file,
            r#"    unsafe {{ SCRATCH.{pipeline}_color_formats[0] = swapchain.format.format }};"#
        )
        .unwrap();
    }
    for pipeline in &renderer.pipelines {
        writeln!(
            file,
            r#"    unsafe {{ SCRATCH.{pipeline}_pipeline.layout = layout }};"#
        )
        .unwrap();
    }
    for compute in &renderer.computes {
        writeln!(
            file,
            r#"    unsafe {{ SCRATCH.{compute}_pipeline.layout = layout }};"#
        )
        .unwrap();
    }
    let pipeline_count = renderer.pipelines.len();
    writeln!(
        file,
        r#"    let mut pipelines: Pipelines = unsafe {{ MaybeUninit::zeroed().assume_init() }};"#
    )
    .unwrap();
    for pipeline in &renderer.pipelines {
        let tab = if pipeline.mesh_shaders {
            write!(
                file,
                r#"    if dev.support.mesh_shaders {{
    "#
            )
            .unwrap();
            "    "
        } else {
            ""
        };
        writeln!(
            file,
            r#"    let _ = unsafe {{ (dev.fp_v1_0().create_graphics_pipelines)(
{tab}        dev.handle(),
{tab}        vk::PipelineCache::null(),
{tab}        1,
{tab}        &*&raw const SCRATCH.{pipeline}_pipeline,
{tab}        std::ptr::null(),
{tab}        &mut pipelines.{pipeline},
{tab}    ) }};"#
        )
        .unwrap();
        if pipeline.mesh_shaders {
            writeln!(file, "    }}").unwrap();
        }
    }
    if !renderer.computes.is_empty() {
        let compute_pipeline_count = renderer.computes.len();
        let first_compute_pipeline = &renderer.computes[0].name;
        writeln!(
            file,
            r#"    let _ = unsafe {{ (dev.fp_v1_0().create_compute_pipelines)(
        dev.handle(),
        vk::PipelineCache::null(),
        {compute_pipeline_count},
        &*&raw const SCRATCH.{first_compute_pipeline}_pipeline,
        std::ptr::null(),
        (pipelines.as_mut_ptr() as *mut vk::Pipeline).offset({pipeline_count}),
    ) }};"#
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"    pipelines
}}"#
    )
    .unwrap();
}

fn generate_uniform(reflection: &ShaderModule, out_dir: &Path) {
    let descriptor_sets = reflection.enumerate_descriptor_sets(None).unwrap();
    let type_info = collect_all_types(&descriptor_sets);
    let mut file = File::create(out_dir.join("gpu.rs")).unwrap();
    for (struct_name, struct_) in &type_info.structs {
        let alignment = struct_.alignment;
        writeln!(
            file,
            r#"    #[repr(C, align({alignment}))]
    #[derive(Clone, Copy, Debug)]
    pub struct {struct_name} {{"#
        )
        .unwrap();
        for (member_name, member_typ) in &struct_.members {
            let member_typ_rust = member_typ.to_rust();
            writeln!(file, "        pub {member_name}: {member_typ_rust},").unwrap();
        }
        writeln!(
            file,
            r#"    }}
    "#
        )
        .unwrap();
    }
}

fn attribute_size(attribute: &VertexAttribute) -> usize {
    match attribute.format.as_str() {
        "R16_UINT" => 2,
        "R32_SFLOAT" => 4,
        "R32G32B32_SFLOAT" => 12,
        "R32G32B32A32_SFLOAT" => 16,
        _ => todo!("attribute_size({:?})", attribute.format),
    }
}
