use crate::config::{
    Compute, DescriptorBinding, Pass, Pipeline, Renderer, Sampler, VertexAttribute,
};
use crate::helper::to_camelcase;
use crate::types::ShaderType;
use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;

#[derive(Clone, PartialEq)]
enum BindingType {
    AccelerationStructure,
    Image,
    InputAttachment,
    StorageBuffer,
    StorageImage,
    Uniform,
}

impl BindingType {
    fn name(&self) -> &'static str {
        match self {
            BindingType::AccelerationStructure => "ACCELERATION_STRUCTURE_KHR",
            BindingType::Image => "COMBINED_IMAGE_SAMPLER",
            BindingType::InputAttachment => "INPUT_ATTACHMENT",
            BindingType::StorageBuffer => "STORAGE_BUFFER",
            BindingType::StorageImage => "STORAGE_IMAGE",
            BindingType::Uniform => "UNIFORM_BUFFER",
        }
    }
}

impl DescriptorBinding {
    fn descriptor_type(&self) -> BindingType {
        match self {
            DescriptorBinding::AccelerationStructure(_) => BindingType::AccelerationStructure,
            DescriptorBinding::Image(_) => BindingType::Image,
            DescriptorBinding::InputAttachment(_) => BindingType::InputAttachment,
            DescriptorBinding::StorageBuffer(_) => BindingType::StorageBuffer,
            DescriptorBinding::StorageImage(_) => BindingType::StorageImage,
            DescriptorBinding::Uniform(_) => BindingType::Uniform,
        }
    }

    fn name(&self) -> &str {
        match self {
            DescriptorBinding::AccelerationStructure(as_) => &as_.name,
            DescriptorBinding::Image(image) => &image.name,
            DescriptorBinding::InputAttachment(input) => &input.name,
            DescriptorBinding::StorageBuffer(storage) => &storage.name,
            DescriptorBinding::StorageImage(image) => &image.name,
            DescriptorBinding::Uniform(uniform) => &uniform.name,
        }
    }

    fn stage(&self) -> &str {
        match self {
            DescriptorBinding::AccelerationStructure(as_) => &as_.stage,
            DescriptorBinding::Image(image) => &image.stage,
            DescriptorBinding::InputAttachment(input) => &input.stage,
            DescriptorBinding::StorageBuffer(storage) => &storage.stage,
            DescriptorBinding::StorageImage(image) => &image.stage,
            DescriptorBinding::Uniform(uniform) => &uniform.stage,
        }
    }

    fn value_type(&self) -> Cow<'static, str> {
        match self {
            DescriptorBinding::AccelerationStructure(_) => "&Option<RaytraceResources>".into(),
            DescriptorBinding::Image(_)
            | DescriptorBinding::InputAttachment(_)
            | DescriptorBinding::StorageImage(_) => "vk::ImageView".into(),
            DescriptorBinding::StorageBuffer(storage) => {
                let typ = &storage.typ;
                format!("&StorageBuffer<{typ}>").into()
            }
            DescriptorBinding::Uniform(uniform) => {
                let typ = &uniform.typ;
                format!("&UniformBuffer<{typ}>").into()
            }
        }
    }
}

impl Display for Compute {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl Display for Pass {
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

pub fn generate_code(in_path: &str, renderer: &Renderer, mut file: File) {
    // clippy::deref_addrof has false positives for *&raw const expressions.
    write!(
        file,
        r#"// Code generated from {in_path}.

#![allow(unused, clippy::deref_addrof)]

#[rustfmt::skip]
use crate::renderer::uniform::{{"#
    )
    .unwrap();
    let mut uniform_types = BTreeSet::new();
    for binding in &renderer.descriptor_set.bindings {
        if let DescriptorBinding::Uniform(uniform) = binding {
            uniform_types.insert(uniform.typ.as_str());
        } else if let DescriptorBinding::StorageBuffer(storage) = binding {
            uniform_types.insert(
                storage
                    .typ
                    .strip_prefix("[")
                    .unwrap()
                    .strip_suffix("]")
                    .unwrap(),
            );
        }
    }
    for typ in &uniform_types {
        if *typ == "u8" || *typ == "u32" || typ.starts_with("crate::") {
            continue;
        }
        write!(file, "{typ},").unwrap();
    }
    writeln!(
        file,
        r#"}};
use crate::renderer::debug::set_label;
use crate::renderer::shader::SpvArray;
use crate::renderer::util::{{AsDescriptor, Dev, ImageResources, StorageBuffer, UniformBuffer}};
use crate::renderer::{{DeviceSupport, Pass, Swapchain, COLOR_FORMAT, DEPTH_FORMAT, FRAMES_IN_FLIGHT}};
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

pub struct Shaders {{"#
    )
    .unwrap();
    let mut pipeline_vertex_shaders = HashMap::new();
    let mut pipeline_fragment_shaders = HashMap::new();
    let mut pipeline_task_shaders = HashMap::new();
    let mut pipeline_mesh_shaders = HashMap::new();
    let mut shaders = BTreeSet::new();
    for_pipelines(renderer, |_, pipeline| {
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
    });
    for compute in &renderer.computes {
        shaders.insert((compute.name.as_str(), ShaderType::Compute));
    }
    for (name, typ) in &shaders {
        let typ_lowercase = typ.lowercase();
        writeln!(file, "    pub {name}_{typ_lowercase}: &'static [u8],").unwrap();
    }
    writeln!(
        file,
        r#"}}

pub struct ShaderModules {{"#
    )
    .unwrap();
    for (name, typ) in &shaders {
        let typ_lowercase = typ.lowercase();
        writeln!(file, "    pub {name}_{typ_lowercase}: vk::ShaderModule,").unwrap();
    }
    writeln!(
        file,
        r#"}}

#[repr(C)]
pub struct Passes {{"#
    )
    .unwrap();
    for pass in &renderer.passes {
        writeln!(file, "    pub {pass}: Pass,").unwrap();
    }
    writeln!(
        file,
        r#"}}

#[repr(C)]
pub struct Pipelines {{"#
    )
    .unwrap();
    for_pipelines(renderer, |_, pipeline| {
        writeln!(file, "    pub {pipeline}: vk::Pipeline,").unwrap();
    });
    for compute in &renderer.computes {
        writeln!(file, "    pub {compute}: vk::Pipeline,").unwrap();
    }
    writeln!(
        file,
        r#"}}

"#
    )
    .unwrap();
    for_pipelines(renderer, |_, pipeline| {
        if let Some(specs) = &pipeline.fragment_specialization {
            let pipeline_camelcase = to_camelcase(&pipeline.to_string());
            writeln!(file, "struct {pipeline_camelcase}Specialization {{").unwrap();
            for spec in specs {
                let ty = &renderer.find_specialization(spec).ty;
                writeln!(file, "    {spec}: {ty},").unwrap();
            }
            writeln!(file, "}}").unwrap();
        }
    });
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
    for binding in &renderer.descriptor_set.bindings {
        let binding_type = binding.descriptor_type();
        let pool_size = match pool_sizes.iter_mut().find(|(ty, _)| *ty == binding_type) {
            Some(pool_size) => pool_size,
            None => {
                pool_sizes.push((binding_type, 0));
                pool_sizes.last_mut().unwrap()
            }
        };
        pool_size.1 += 2;
    }
    let binding_count = renderer.descriptor_set.bindings.len();
    let pool_size_count = pool_sizes.len();
    writeln!(
        file,
        r#"    descriptor_set_bindings: [vk::DescriptorSetLayoutBinding<'static>; {binding_count}],
    descriptor_set_layout: vk::DescriptorSetLayoutCreateInfo<'static>,
    descriptor_pool_sizes: [vk::DescriptorPoolSize; {pool_size_count}],
    descriptor_pool: vk::DescriptorPoolCreateInfo<'static>,
    assembly: vk::PipelineInputAssemblyStateCreateInfo<'static>,
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
    for_pipelines(renderer, |_, pipeline| {
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
            r#"    {pipeline}_viewport: vk::Viewport,
    {pipeline}_scissor: vk::Rect2D,
    {pipeline}_viewport_state: vk::PipelineViewportStateCreateInfo<'static>,
    {pipeline}_rasterizer: vk::PipelineRasterizationStateCreateInfo<'static>,
    {pipeline}_multisampling: vk::PipelineMultisampleStateCreateInfo<'static>,
    {pipeline}_blend_attachments: [vk::PipelineColorBlendAttachmentState; 1],
    {pipeline}_blend: vk::PipelineColorBlendStateCreateInfo<'static>,
    {pipeline}_depth: vk::PipelineDepthStencilStateCreateInfo<'static>,
    {pipeline}_color_formats: [vk::Format; 1],
    {pipeline}_rendering: vk::PipelineRenderingCreateInfo<'static>,"#,
        )
        .unwrap();
    });
    for_pipelines(renderer, |_, pipeline| {
        writeln!(
            file,
            "    {pipeline}_pipeline: vk::GraphicsPipelineCreateInfo<'static>,"
        )
        .unwrap();
    });
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

#[rustfmt::skip]
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
    for (binding_index, binding) in renderer.descriptor_set.bindings.iter().enumerate() {
        let typ = binding.descriptor_type().name();
        let stage = binding.stage();
        writeln!(
            file,
            r#"        vk::DescriptorSetLayoutBinding {{
            binding: {binding_index},
            descriptor_type: vk::DescriptorType::{typ},
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::{stage},
            p_immutable_samplers: std::ptr::null(),
            _marker: std::marker::PhantomData,
        }},"#,
        )
        .unwrap();
    }
    let binding_count = renderer.descriptor_set.bindings.len();
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
        let binding_type_name = binding_type.name();
        writeln!(
            file,
            r"        vk::DescriptorPoolSize {{
            ty: vk::DescriptorType::{binding_type_name},
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
    dynamic_state: vk::PipelineDynamicStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDynamicStateCreateFlags::empty(),
        dynamic_state_count: 0,
        p_dynamic_states: std::ptr::null(),
        _marker: std::marker::PhantomData,
    }},"#
    )
    .unwrap();
    for (name, typ) in &shaders {
        let typ_lowercase = typ.lowercase();
        writeln!(
            file,
            r#"    {name}_{typ_lowercase}: vk::ShaderModuleCreateInfo {{
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: 0,
        p_code: std::ptr::null(),
        _marker: std::marker::PhantomData,
    }},"#
        )
        .unwrap();
    }
    for_pipelines(renderer, |pass, pipeline| {
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
        writeln!(file, r#"    {pipeline}_shader_stages: ["#).unwrap();
        if pipeline.task_shaders {
            writeln!(
                file,
                r#"        vk::PipelineShaderStageCreateInfo {{
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
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
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::{vertex_stage_type},
            module: vk::ShaderModule::null(),
            p_name: c"main".as_ptr(),
            p_specialization_info: std::ptr::null(),
            _marker: std::marker::PhantomData,
        }},
        vk::PipelineShaderStageCreateInfo {{
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
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
        let rasterization_samples = if pass.msaa { "empty()" } else { "TYPE_1" };
        writeln!(
            file,
            r#"    {pipeline}_viewport: vk::Viewport {{
        x: 0.,
        y: 0.,
        width: 0.,
        height: 0.,
        min_depth: 0.,
        max_depth: 1.,
    }},
    {pipeline}_scissor: vk::Rect2D {{
        offset: vk::Offset2D {{ x: 0, y: 0 }},
        extent: vk::Extent2D {{ width: 0, height: 0 }},
    }},
    {pipeline}_viewport_state: vk::PipelineViewportStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count: 1,
        p_viewports: unsafe {{ &raw const SCRATCH.{pipeline}_viewport }},
        scissor_count: 1,
        p_scissors: unsafe {{ &raw const SCRATCH.{pipeline}_scissor }},
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
        rasterization_samples: vk::SampleCountFlags::{rasterization_samples},
        sample_shading_enable: 0,
        min_sample_shading: 0.,
        p_sample_mask: std::ptr::null(),
        alpha_to_coverage_enable: 0,
        alpha_to_one_enable: 0,
        _marker: std::marker::PhantomData,
    }},
    {pipeline}_blend_attachments: ["#
        )
        .unwrap();
        for _ in [()] {
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
        let depth_bool = if true { 1 } else { 0 };
        let color_attachment_count = 1;
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
        attachment_count: {color_attachment_count},
        p_attachments: unsafe {{ &raw const SCRATCH.{pipeline}_blend_attachments[0] }},
        blend_constants: [0., 0., 0., 0.],
        _marker: std::marker::PhantomData,
    }},
    {pipeline}_depth: vk::PipelineDepthStencilStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
        depth_test_enable: {depth_bool},
        depth_write_enable: {depth_bool},
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
    });
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
        let bytes = format!(r#"include_bytes!(concat!(env!("OUT_DIR"), "/{name}.{ext}.spv"))"#);
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

#[rustfmt::skip]
pub fn alloc_descriptor_set("#
    )
    .unwrap();
    for binding in &renderer.descriptor_set.bindings {
        let name = binding.name();
        let typ = binding.value_type();
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
    for binding in &renderer.descriptor_set.bindings {
        let name = binding.name();
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
    for (binding_index, binding) in renderer.descriptor_set.bindings.iter().enumerate() {
        let name = binding.name();
        let typ = binding.value_type();
        writeln!(file, "        {name}: {typ},").unwrap();
        if binding.descriptor_type() == BindingType::AccelerationStructure {
            assert!(only_tlas.is_none());
            assert_eq!(binding_index, renderer.descriptor_set.bindings.len() - 1);
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
    for (binding_index, binding) in renderer.descriptor_set.bindings.iter().enumerate() {
        let binding_name = binding.name();
        let binding_type = binding.descriptor_type().name();
        let write_mutable = match binding {
            DescriptorBinding::AccelerationStructure(_) => "mut ",
            _ => "",
        };
        match binding {
            DescriptorBinding::AccelerationStructure(_) => writeln!(
                file,
                r#"        let mut {binding_name}_acceleration_structure = *vk::WriteDescriptorSetAccelerationStructureKHR::default()
            .acceleration_structures({binding_name}.as_ref().map(|as_| std::slice::from_ref(&as_.acceleration_structure)).unwrap_or_default());"#
            )
                .unwrap(),
            DescriptorBinding::Image(image) => {
                let layout = &image.layout;
                writeln!(
                    file,
                    r#"        let {binding_name}_image = *vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::{layout})
            .image_view({binding_name});"#
                )
                    .unwrap()
            }
            DescriptorBinding::InputAttachment(_) => writeln!(
                file,
                r#"        let {binding_name}_image = *vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view({binding_name});"#
            )
                .unwrap(),
            DescriptorBinding::StorageBuffer(_) => writeln!(file, r#"        let {binding_name}_buffer = {binding_name}.descriptor(_flight_index);"#).unwrap(),
            DescriptorBinding::StorageImage(_) => writeln!(file,
                                                           r#"        let {binding_name}_image = *vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view({binding_name});"#
            ).unwrap(),
            DescriptorBinding::Uniform(_) => writeln!(
                file,
                r#"        let {binding_name}_buffer = {binding_name}.descriptor(_flight_index);"#
            )
                .unwrap(),
        }
        writeln!(
            file,
            r#"        let {write_mutable}{binding_name} = vk::WriteDescriptorSet::default()
            .dst_set(*descriptor)
            .dst_binding({binding_index})
            .descriptor_type(vk::DescriptorType::{binding_type})"#
        )
        .unwrap();
        match binding {
            DescriptorBinding::AccelerationStructure(_) => writeln!(
                file,
                r#"            .push_next(&mut {binding_name}_acceleration_structure);
        {binding_name}.descriptor_count = 1;"#
            )
            .unwrap(),
            DescriptorBinding::Image(_)
            | DescriptorBinding::InputAttachment(_)
            | DescriptorBinding::StorageImage(_) => writeln!(
                file,
                r#"            .image_info(std::slice::from_ref(&{binding_name}_image));"#
            )
            .unwrap(),
            DescriptorBinding::StorageBuffer(_) => writeln!(
                file,
                r#"            .buffer_info(std::slice::from_ref(&{binding_name}_buffer));"#
            )
            .unwrap(),
            DescriptorBinding::Uniform(_) => writeln!(
                file,
                r#"            .buffer_info(std::slice::from_ref(&{binding_name}_buffer));"#
            )
            .unwrap(),
        }
    }
    let write_writes = |file: &mut File, bindings: &[DescriptorBinding]| {
        write!(file, r"[").unwrap();
        for (binding_index, binding) in bindings.iter().enumerate() {
            let binding_name = binding.name();
            write!(file, "{binding_name}").unwrap();
            if binding_index != bindings.len() - 1 {
                write!(file, ", ").unwrap();
            }
        }
        write!(file, "]").unwrap();
    };
    write!(file, r#"        let writes = "#).unwrap();
    write_writes(&mut file, &renderer.descriptor_set.bindings);
    writeln!(file, r#";"#).unwrap();
    if only_tlas.is_some() {
        let count_without_raytracing = renderer.descriptor_set.bindings.len() - 1;
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

impl ShaderModules {{
    pub fn cleanup(&self, dev: &Dev) {{"#
    )
    .unwrap();
    for (name, typ) in &shaders {
        let typ_lowercase = typ.lowercase();
        writeln!(
            file,
            r#"        unsafe {{ dev.destroy_shader_module(self.{name}_{typ_lowercase}, None) }};"#
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
    for_pipelines(renderer, |_, pipeline| {
        writeln!(
            file,
            "        unsafe {{ dev.destroy_pipeline(self.{pipeline}, None) }};"
        )
        .unwrap();
    });
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

#[rustfmt::skip]
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

#[rustfmt::skip]
pub fn create_descriptor_set_layout(_samplers: &Samplers, dev: &Dev) -> vk::DescriptorSetLayout {{"#
    )
    .unwrap();
    for (binding_index, binding) in renderer.descriptor_set.bindings.iter().enumerate() {
        if let DescriptorBinding::Image(image) = binding {
            writeln!(
                file,
                "    unsafe {{ SCRATCH.descriptor_set_bindings[{binding_index}].p_immutable_samplers = &_samplers.{} }};",
                image.sampler,
            )
                .unwrap();
        }
    }
    writeln!(file, "    unsafe {{ dev.create_descriptor_set_layout(&*&raw const SCRATCH.descriptor_set_layout, None).unwrap_unchecked() }}").unwrap();
    writeln!(
        file,
        r#"}}

#[rustfmt::skip]
pub fn create_descriptor_pool(layout: vk::DescriptorSetLayout, dev: &Dev) -> vk::DescriptorPool {{
    unsafe {{ dev.create_descriptor_pool(&*&raw const SCRATCH.descriptor_pool, None).unwrap_unchecked() }}
}}

#[allow(unused_mut)]
#[allow(clippy::identity_op)]
#[rustfmt::skip]
pub fn create_render_passes(
    swapchain: &Swapchain,
    _msaa_samples: vk::SampleCountFlags,
    dev: &Dev,
) -> Passes {{"#
    )
    .unwrap();
    for pass in &renderer.passes {
        let debug_name = &pass.debug_name;
        let debug_r = pass.debug_color.red;
        let debug_g = pass.debug_color.green;
        let debug_b = pass.debug_color.blue;
        writeln!(
            file,
            r#"    let {pass} = Pass {{
        debug_name: {debug_name:?},
        debug_color: [{debug_r}, {debug_g}, {debug_b}],
    }};"#
        )
        .unwrap();
    }
    writeln!(file, "    Passes {{").unwrap();
    for pass in &renderer.passes {
        writeln!(file, "        {pass},").unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}

#[rustfmt::skip]
pub fn create_shader_modules(dev: &Dev) -> ShaderModules {{"#
    )
    .unwrap();
    for (name, typ) in &shaders {
        let name_uppercase = name.to_uppercase();
        let typ_lowercase = typ.lowercase();
        let typ_uppercase = typ_lowercase.to_uppercase();
        writeln!(file, r#"    unsafe {{ SCRATCH.{name}_{typ_lowercase}.code_size = {name_uppercase}_{typ_uppercase}_SPV.0.len() }};"#).unwrap();
        writeln!(file, r#"    unsafe {{ SCRATCH.{name}_{typ_lowercase}.p_code = {name_uppercase}_{typ_uppercase}_SPV.0.as_ptr() as *const u32 }};"#).unwrap();
    }
    for (name, typ) in &shaders {
        let typ_lowercase = typ.lowercase();
        // TODO: Fragment shaders which rely on mesh shader functionality (such
        // as perprimitiveEXT) are still built on devices without mesh shaders
        // support. This would need some dependency tracking, but if I add full
        // non-mesh shader codepath I will be able to use conditional
        // compilation in GLSL instead.
        if !typ.requires_mesh_shaders() {
            writeln!(file, r#"    let {name}_{typ_lowercase} = unsafe {{ dev.create_shader_module(&*&raw const SCRATCH.{name}_{typ_lowercase}, None).unwrap_unchecked() }};"#).unwrap();
        } else {
            writeln!(file, r#"    let {name}_{typ_lowercase} = if dev.support.mesh_shaders {{
        unsafe {{ dev.create_shader_module(&*&raw const SCRATCH.{name}_{typ_lowercase}, None).unwrap_unchecked() }}
    }} else {{
        vk::ShaderModule::null()
    }};"#).unwrap();
        }
    }
    writeln!(file, "    ShaderModules {{").unwrap();
    for (name, typ) in &shaders {
        let typ_lowercase = typ.lowercase();
        writeln!(file, "        {name}_{typ_lowercase},").unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}

#[rustfmt::skip]
#[allow(clippy::identity_op)]
pub fn create_pipelines(
    _msaa_samples: vk::SampleCountFlags,
    passes: &Passes,"#
    )
    .unwrap();
    for spec in &renderer.specializations {
        let name = &spec.name;
        let ty = &spec.ty;
        if spec.shared {
            writeln!(file, "    {name}: {ty},").unwrap();
        }
    }
    for_pipelines(renderer, |_, pipeline| {
        if let Some(specs) = &pipeline.fragment_specialization {
            for spec in specs {
                let metadata = renderer.find_specialization(spec);
                if !metadata.shared {
                    let ty = &metadata.ty;
                    writeln!(file, "    {pipeline}_{spec}: {ty},").unwrap();
                }
            }
        }
    });
    writeln!(
        file,
        r#"    swapchain: &Swapchain,
    shader_modules: &ShaderModules,
    layout: vk::PipelineLayout,
    dev: &Dev,
) -> Pipelines {{"#
    )
    .unwrap();
    for_pipelines(renderer, |pass, pipeline| {
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
        if pipeline.task_shaders {
            let task_shader = pipeline_task_shaders[pipeline.name.as_str()];
            writeln!(
                file,
                r#"    unsafe {{ SCRATCH.{pipeline}_shader_stages[0].module = shader_modules.{task_shader}_task }};"#
            ).unwrap();
        }
        if pipeline.mesh_shaders {
            let mesh_stage_index = if pipeline.task_shaders { 1 } else { 0 };
            let mesh_shader = pipeline_mesh_shaders[pipeline.name.as_str()];
            writeln!(
                file,
                r#"    unsafe {{ SCRATCH.{pipeline}_shader_stages[{mesh_stage_index}].module = shader_modules.{mesh_shader}_mesh }};"#
            ).unwrap();
        } else {
            let vertex_stage_index = if pipeline.task_shaders { 1 } else { 0 };
            let vertex_shader = pipeline_vertex_shaders[pipeline.name.as_str()];
            writeln!(
                file,
                r#"    unsafe {{ SCRATCH.{pipeline}_shader_stages[{vertex_stage_index}].module = shader_modules.{vertex_shader}_vertex }};"#
            ).unwrap();
        }
        let fragment_stage_index = if pipeline.task_shaders { 2 } else { 1 };
        let fragment_shader = pipeline_fragment_shaders[pipeline.name.as_str()];
        writeln!(
            file,
            r#"    unsafe {{ SCRATCH.{pipeline}_shader_stages[{fragment_stage_index}].module = shader_modules.{fragment_shader}_fragment }};
    unsafe {{ SCRATCH.{pipeline}_viewport.width = swapchain.extent.width as f32 }};
    unsafe {{ SCRATCH.{pipeline}_viewport.height = swapchain.extent.height as f32 }};
    unsafe {{ SCRATCH.{pipeline}_scissor.extent.width = swapchain.extent.width }};
    unsafe {{ SCRATCH.{pipeline}_scissor.extent.height = swapchain.extent.height }};"#
        )
            .unwrap();
        if pass.msaa {
            writeln!(file, "    unsafe {{ SCRATCH.{pipeline}_multisampling.rasterization_samples = _msaa_samples }};").unwrap();
        }
        writeln!(
            file,
            r#"    unsafe {{ SCRATCH.{pipeline}_color_formats[0] = swapchain.format.format }};"#
        )
        .unwrap();
    });
    for_pipelines(renderer, |_, pipeline| {
        writeln!(
            file,
            r#"    unsafe {{ SCRATCH.{pipeline}_pipeline.layout = layout }};"#
        )
        .unwrap();
    });
    for compute in &renderer.computes {
        writeln!(
            file,
            r#"    unsafe {{ SCRATCH.{compute}_pipeline.layout = layout }};
    unsafe {{ SCRATCH.{compute}_pipeline.stage.module = shader_modules.{compute}_compute }};"#
        )
        .unwrap();
    }
    let mut pipeline_count = 0;
    let mut first_pipeline = None;
    for_pipelines(renderer, |_, pipeline| {
        pipeline_count += 1;
        if first_pipeline.is_none() {
            first_pipeline = Some(pipeline);
        }
    });
    writeln!(
        file,
        r#"    let mut pipelines: Pipelines = unsafe {{ MaybeUninit::zeroed().assume_init() }};"#
    )
    .unwrap();
    for_pipelines(renderer, |_, pipeline| {
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
    });
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

fn for_pipelines<'a>(renderer: &'a Renderer, mut f: impl FnMut(&'a Pass, &'a Pipeline)) {
    for pass in &renderer.passes {
        for pipeline in &pass.pipelines {
            f(pass, pipeline);
        }
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
