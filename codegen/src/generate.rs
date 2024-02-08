use crate::config::{
    Compute, DescriptorBinding, DescriptorSet, Pass, Pipeline, Renderer, Sampler, Subpass,
    VertexAttribute,
};
use crate::helper::{to_camelcase, AttachmentType};
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

#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum ShaderType {
    Compute,
    Vertex,
    Fragment,
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

impl ShaderType {
    fn lowercase(&self) -> &'static str {
        match self {
            ShaderType::Compute => "compute",
            ShaderType::Fragment => "fragment",
            ShaderType::Vertex => "vertex",
        }
    }

    fn camelcase(&self) -> &'static str {
        match self {
            ShaderType::Compute => "Compute",
            ShaderType::Fragment => "Fragment",
            ShaderType::Vertex => "Vertex",
        }
    }

    fn extension(&self) -> &'static str {
        match self {
            ShaderType::Compute => "comp",
            ShaderType::Fragment => "frag",
            ShaderType::Vertex => "vert",
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

impl Display for DescriptorSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
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

impl Display for Subpass {
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
use crate::renderer::debug::set_label;
#[allow(unused_imports)]
use crate::renderer::util::{{AsDescriptor, Dev, ImageResources, StorageBuffer, UniformBuffer}};
use crate::renderer::{{Pass, Swapchain, COLOR_FORMAT, DEPTH_FORMAT, FRAMES_IN_FLIGHT}};
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
    for compute in &renderer.computes {
        writeln!(file, "    pub {compute}: vk::PipelineLayout,").unwrap();
    }
    writeln!(
        file,
        r#"}}

pub struct Shaders {{"#
    )
    .unwrap();
    let mut pipeline_vertex_shaders = HashMap::new();
    let mut pipeline_fragment_shaders = HashMap::new();
    let mut shaders = BTreeSet::new();
    for_pipelines(renderer, |_, _, _, pipeline| {
        let vertex_shader = match &pipeline.vertex_shader {
            Some(path) => path.strip_suffix(".vert").unwrap(),
            None => pipeline.name.as_str(),
        };
        let fragment_shader = match &pipeline.fragment_shader {
            Some(path) => path.strip_suffix(".frag").unwrap(),
            None => pipeline.name.as_str(),
        };
        pipeline_vertex_shaders.insert(pipeline.name.as_str(), vertex_shader);
        pipeline_fragment_shaders.insert(pipeline.name.as_str(), fragment_shader);
        shaders.insert((vertex_shader, ShaderType::Vertex));
        shaders.insert((fragment_shader, ShaderType::Fragment));
    });
    for compute in &renderer.computes {
        shaders.insert((compute.name.as_str(), ShaderType::Compute));
    }
    for (name, typ) in &shaders {
        let typ_lowercase = typ.lowercase();
        writeln!(file, "    pub {name}_{typ_lowercase}: Vec<u32>,").unwrap();
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
    for_pipelines(renderer, |_, _, _, pipeline| {
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
    for_pipelines(renderer, |_, _, _, pipeline| {
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
    for pass in &renderer.passes {
        let attachment_count: usize = pass.subpasses.iter().map(Subpass::attachment_count).sum();
        let subpass_count = pass.subpasses.len();
        let dependency_count = pass.dependencies.len();
        for subpass in &pass.subpasses {
            let color_count = subpass.color_attachments.len();
            let input_count = subpass.input_attachment.len();
            if color_count != 0 {
                writeln!(
                    file,
                    "    {pass}_{subpass}_color: [vk::AttachmentReference; {color_count}],"
                )
                .unwrap();
            }
            if subpass.depth_attachment.is_some() {
                writeln!(file, "    {pass}_{subpass}_depth: vk::AttachmentReference,").unwrap();
            }
            if input_count != 0 {
                writeln!(
                    file,
                    "    {pass}_{subpass}_input: [vk::AttachmentReference; {input_count}],"
                )
                .unwrap();
            }
        }
        writeln!(
            file,
            r#"    {pass}_attachments: [vk::AttachmentDescription; {attachment_count}],
    {pass}_subpasses: [vk::SubpassDescription; {subpass_count}],
    {pass}_dependencies: [vk::SubpassDependency; {dependency_count}],
    {pass}_pass: vk::RenderPassCreateInfo,"#
        )
        .unwrap();
    }
    let mut all_pool_sizes: Vec<Vec<(BindingType, usize)>> =
        vec![Vec::new(); renderer.descriptor_sets.len()];
    for (descriptor_set_index, descriptor_set) in renderer.descriptor_sets.iter().enumerate() {
        let pool_sizes = &mut all_pool_sizes[descriptor_set_index];
        for binding in &descriptor_set.bindings {
            let binding_type = binding.descriptor_type();
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
    dynamic_state: vk::PipelineDynamicStateCreateInfo,"#
    )
    .unwrap();
    for (name, typ) in &shaders {
        let typ_lowercase = typ.lowercase();
        writeln!(
            file,
            r#"    {name}_{typ_lowercase}: vk::ShaderModuleCreateInfo,"#
        )
        .unwrap();
    }
    for_pipelines(renderer, |_, _, _, pipeline| {
        let layout_count = pipeline.descriptor_sets.len();
        if layout_count > 1 {
            writeln!(
                file,
                "    {pipeline}_layouts: [vk::DescriptorSetLayout; {layout_count}],"
            )
            .unwrap();
        }
        writeln!(
            file,
            "    {pipeline}_pipeline_layout: vk::PipelineLayoutCreateInfo,"
        )
        .unwrap();
    });
    for compute in &renderer.computes {
        let layout_count = compute.descriptor_sets.len();
        if layout_count > 1 {
            writeln!(
                file,
                "    {compute}_layouts: [vk::DescriptorSetLayout; {layout_count}],"
            )
            .unwrap();
        }
        writeln!(
            file,
            "    {compute}_pipeline_layout: vk::PipelineLayoutCreateInfo,"
        )
        .unwrap();
    }
    for_pipelines(renderer, |_, _, subpass, pipeline| {
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
        writeln!(
            file,
            r#"    {pipeline}_shader_stages: [vk::PipelineShaderStageCreateInfo; 2],
    {pipeline}_vertex_bindings: [vk::VertexInputBindingDescription; {binding_count}],
    {pipeline}_vertex_attributes: [vk::VertexInputAttributeDescription; {attribute_count}],
    {pipeline}_vertex_state: vk::PipelineVertexInputStateCreateInfo,
    {pipeline}_viewport: vk::Viewport,
    {pipeline}_scissor: vk::Rect2D,
    {pipeline}_viewport_state: vk::PipelineViewportStateCreateInfo,
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
    for compute in &renderer.computes {
        writeln!(
            file,
            r#"    {compute}_pipeline: vk::ComputePipelineCreateInfo,"#
        )
        .unwrap();
    }
    let pass_count = renderer.passes.len();
    writeln!(
        file,
        r#"}}

pub const PASS_COUNT: usize = {pass_count};
pub const PASS_NAMES: [&str; PASS_COUNT] = ["#
    )
    .unwrap();
    for pass in &renderer.passes {
        writeln!(file, "    \"{pass}\",").unwrap();
    }
    writeln!(
        file,
        r#"];

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
    for pass in &renderer.passes {
        let mut attachment_indices = HashMap::new();
        let mut attachment_index = 0;
        for subpass in &pass.subpasses {
            if !subpass.color_attachments.is_empty() {
                writeln!(file, "    {pass}_{subpass}_color: [",).unwrap();
                for color in &subpass.color_attachments {
                    let attachment_layout = &color.layout;
                    writeln!(
                        file,
                        r#"        vk::AttachmentReference {{
            attachment: {attachment_index},
            layout: vk::ImageLayout::{attachment_layout},
        }},"#
                    )
                    .unwrap();
                    attachment_indices.insert(color.name.as_str(), attachment_index);
                    attachment_index += 1;
                }
                writeln!(file, "    ],",).unwrap();
            }
            if let Some(depth) = &subpass.depth_attachment {
                let attachment_layout = &depth.layout;
                writeln!(
                    file,
                    r#"    {pass}_{subpass}_depth: vk::AttachmentReference {{
        attachment: {attachment_index},
        layout: vk::ImageLayout::{attachment_layout},
    }},"#
                )
                .unwrap();
                attachment_index += 1;
            }
            if !subpass.input_attachment.is_empty() {
                writeln!(file, "    {pass}_{subpass}_input: [",).unwrap();
                for input in &subpass.input_attachment {
                    let attachment_index = attachment_indices[input.as_str()];
                    writeln!(
                        file,
                        r#"        vk::AttachmentReference {{
            attachment: {attachment_index},
            layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        }},"#
                    )
                    .unwrap();
                }
                writeln!(file, "    ],",).unwrap();
            }
        }
        writeln!(file, "    {pass}_attachments: [").unwrap();
        for subpass in &pass.subpasses {
            let attachments = subpass
                .color_attachments
                .iter()
                .map(|a| (AttachmentType::Color, a))
                .chain(
                    subpass
                        .depth_attachment
                        .iter()
                        .map(|a| (AttachmentType::Depth, a)),
                );
            for (attachment_type, attachment) in attachments {
                let attachment_format = if let Some(format) = &attachment.format {
                    format!("vk::Format::{format}")
                } else if attachment.swapchain {
                    "vk::Format::UNDEFINED".to_owned()
                } else if attachment_type == AttachmentType::Color {
                    "COLOR_FORMAT".to_owned()
                } else {
                    "DEPTH_FORMAT".to_owned()
                };
                let attachment_samples = if subpass.msaa { "empty()" } else { "TYPE_1" };
                let attachment_load = if attachment.clear.is_some() {
                    "CLEAR"
                } else {
                    "DONT_CARE"
                };
                let attachment_store = if attachment.store {
                    "STORE"
                } else {
                    "DONT_CARE"
                };
                let attachment_final = if let Some(layout) = &attachment.layout_final {
                    layout
                } else {
                    &attachment.layout
                };
                writeln!(
                    file,
                    r#"        vk::AttachmentDescription {{
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: {attachment_format},
            samples: vk::SampleCountFlags::{attachment_samples},
            load_op: vk::AttachmentLoadOp::{attachment_load},
            store_op: vk::AttachmentStoreOp::{attachment_store},
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::{attachment_final},
        }},"#
                )
                .unwrap();
            }
        }
        writeln!(file, "    ],").unwrap();

        writeln!(file, "    {pass}_subpasses: [").unwrap();
        for subpass in &pass.subpasses {
            let color_count = subpass.color_attachments.len();
            let color_ptr = if color_count != 0 {
                format!("unsafe {{ SCRATCH.{pass}_{subpass}_color.as_ptr() }}")
            } else {
                "std::ptr::null()".to_owned()
            };
            let depth_ptr = if subpass.depth_attachment.is_some() {
                format!("unsafe {{ &SCRATCH.{pass}_{subpass}_depth }}")
            } else {
                "std::ptr::null()".to_owned()
            };
            let input_count = subpass.input_attachment.len();
            let input_ptr = if input_count != 0 {
                format!("unsafe {{ SCRATCH.{pass}_{subpass}_input.as_ptr() }}")
            } else {
                "std::ptr::null()".to_owned()
            };
            writeln!(
                file,
                r#"        vk::SubpassDescription {{
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: {input_count},
            p_input_attachments: {input_ptr},
            color_attachment_count: {color_count},
            p_color_attachments: {color_ptr},
            p_resolve_attachments: std::ptr::null(),
            p_depth_stencil_attachment: {depth_ptr},
            preserve_attachment_count: 0,
            p_preserve_attachments: std::ptr::null(),
        }},"#
            )
            .unwrap();
        }
        writeln!(file, "    ],").unwrap();

        writeln!(file, "    {pass}_dependencies: [").unwrap();
        for dep in &pass.dependencies {
            let src_index = pass.subpass_index(&dep.src.subpass);
            let dst_index = pass.subpass_index(&dep.dst.subpass);
            let src_stage = &dep.src.stage_mask;
            let dst_stage = &dep.dst.stage_mask;
            let src_access = &dep.src.access_mask;
            let dst_access = &dep.dst.access_mask;
            let flags = if dep.by_region {
                "vk::DependencyFlags::BY_REGION"
            } else {
                "vk::DependencyFlags::empty()"
            };
            writeln!(
                file,
                r#"        vk::SubpassDependency {{
            src_subpass: {src_index},
            dst_subpass: {dst_index},
            src_stage_mask: vk::PipelineStageFlags::{src_stage},
            dst_stage_mask: vk::PipelineStageFlags::{dst_stage},
            src_access_mask: vk::AccessFlags::{src_access},
            dst_access_mask: vk::AccessFlags::{dst_access},
            dependency_flags: {flags},
        }},"#
            )
            .unwrap();
        }
        writeln!(file, "    ],").unwrap();

        let attachment_count: usize = pass.subpasses.iter().map(Subpass::attachment_count).sum();
        let subpass_count = pass.subpasses.len();
        let dependency_count = pass.dependencies.len();
        writeln!(
            file,
            r#"    {pass}_pass: vk::RenderPassCreateInfo {{
        s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::RenderPassCreateFlags::empty(),
        attachment_count: {attachment_count},
        p_attachments: unsafe {{ SCRATCH.{pass}_attachments.as_ptr() }},
        subpass_count: {subpass_count},
        p_subpasses: unsafe {{ SCRATCH.{pass}_subpasses.as_ptr() }},
        dependency_count: {dependency_count},
        p_dependencies: unsafe {{ SCRATCH.{pass}_dependencies.as_ptr() }},
    }},"#
        )
        .unwrap();
    }
    for (descriptor_set_index, descriptor_set) in renderer.descriptor_sets.iter().enumerate() {
        writeln!(file, "    {}_bindings: [", descriptor_set.name).unwrap();
        for (binding_index, binding) in descriptor_set.bindings.iter().enumerate() {
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
    dynamic_state: vk::PipelineDynamicStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineDynamicStateCreateFlags::empty(),
        dynamic_state_count: 0,
        p_dynamic_states: std::ptr::null(),
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
    }},"#
        )
        .unwrap();
    }
    for_pipelines(renderer, |_, subpass_index, subpass, pipeline| {
        let descriptor_count = pipeline.descriptor_sets.len();
        let attribute_count = pipeline
            .vertex_bindings
            .iter()
            .flat_map(|binding| binding.attributes.iter())
            .filter(|attribute| !attribute.unused)
            .count();
        let set_layouts_ptr = if descriptor_count > 1 {
            format!("unsafe {{ SCRATCH.{pipeline}_layouts.as_ptr() }}")
        } else {
            "std::ptr::null()".to_owned()
        };
        if descriptor_count > 1 {
            writeln!(
                file,
                r"    {pipeline}_layouts: [vk::DescriptorSetLayout::null(); {descriptor_count}],"
            )
            .unwrap();
        }
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
        p_map_entries: unsafe {{ SCRATCH.{pipeline}_fragment_specialization_entries.as_ptr() }},
        data_size: {offset},
        p_data: unsafe {{ (&SCRATCH.{pipeline}_fragment_specialization_scratch) as *const _ as *const std::ffi::c_void }},
    }},
    {pipeline}_fragment_specialization_scratch: {pipeline_camelcase}Specialization {{"#
            )
            .unwrap();
            for spec in fragment_specialization {
                let default = renderer.find_specialization(spec).type_default();
                writeln!(file, "        {spec}: {default},").unwrap();
            }
            writeln!(file, "    }},").unwrap();
            format!("unsafe {{ &SCRATCH.{pipeline}_fragment_specialization_info }}")
        } else {
            "std::ptr::null()".to_owned()
        };
        writeln!(
            file,
            r#"    {pipeline}_pipeline_layout: vk::PipelineLayoutCreateInfo {{
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: {descriptor_count},
        p_set_layouts: {set_layouts_ptr},
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
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
            p_specialization_info: {fragment_specialization_info},
        }},
    ],
    {pipeline}_vertex_bindings: ["#
        )
        .unwrap();
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
        let polygon_mode = &pipeline.polygon_mode;
        let cull_mode = &pipeline.cull_mode;
        let rasterization_samples = if subpass.msaa { "empty()" } else { "TYPE_1" };
        writeln!(
            file,
            r#"    ],
    {pipeline}_vertex_state: vk::PipelineVertexInputStateCreateInfo {{
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineVertexInputStateCreateFlags::empty(),
        vertex_binding_description_count: {vertex_binding_count},
        p_vertex_binding_descriptions: unsafe {{ SCRATCH.{pipeline}_vertex_bindings.as_ptr() }},
        vertex_attribute_description_count: {attribute_count},
        p_vertex_attribute_descriptions: unsafe {{ SCRATCH.{pipeline}_vertex_attributes.as_ptr() }},
    }},
    {pipeline}_viewport: vk::Viewport {{
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
        p_viewports: unsafe {{ &SCRATCH.{pipeline}_viewport }},
        scissor_count: 1,
        p_scissors: unsafe {{ &SCRATCH.{pipeline}_scissor }},
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
        p_vertex_input_state: unsafe {{ &SCRATCH.{pipeline}_vertex_state }},
        p_input_assembly_state: unsafe {{ &SCRATCH.assembly }},
        p_tessellation_state: std::ptr::null(),
        p_viewport_state: unsafe {{ &SCRATCH.{pipeline}_viewport_state }},
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
    for compute in &renderer.computes {
        let descriptor_count = compute.descriptor_sets.len();
        let set_layouts_ptr = if descriptor_count > 1 {
            format!("unsafe {{ SCRATCH.{compute}_layouts.as_ptr() }}")
        } else {
            "std::ptr::null()".to_owned()
        };
        writeln!(
            file,
            r#"    {compute}_pipeline_layout: vk::PipelineLayoutCreateInfo {{
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: {descriptor_count},
        p_set_layouts: {set_layouts_ptr},
        push_constant_range_count: 0,
        p_push_constant_ranges: std::ptr::null(),
    }},
    {compute}_pipeline: vk::ComputePipelineCreateInfo {{
        s_type: vk::StructureType::COMPUTE_PIPELINE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineCreateFlags::empty(),
        stage: vk::PipelineShaderStageCreateInfo {{
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::COMPUTE,
            module: vk::ShaderModule::null(),
            p_name: unsafe {{ CStr::from_bytes_with_nul_unchecked(b"main\0") }}.as_ptr(),
            p_specialization_info: std::ptr::null(),
        }},
        layout: vk::PipelineLayout::null(),
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
    }},"#
        )
        .unwrap();
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
            let name = binding.name();
            let typ = binding.value_type();
            writeln!(file, "        {name}: {typ},").unwrap();
        }
        write!(
            file,
            r#"        dev: &Dev,
    ) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {{
        let layouts = [self.{descriptor_set}_layout; FRAMES_IN_FLIGHT];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.{descriptor_set})
            .set_layouts(&layouts);
        let descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
            unsafe {{ dev.allocate_descriptor_sets(&descriptor_set_alloc_info) }}
                .unwrap()
                .try_into()
                .unwrap();
        self.update_{descriptor_set}(&descriptors"#
        )
        .unwrap();
        for binding in &descriptor_set.bindings {
            let name = binding.name();
            write!(file, ", {name}").unwrap();
        }

        writeln!(
            file,
            r#", dev);
        descriptors
    }}

    #[allow(clippy::unused_enumerate_index)]
    pub fn update_{descriptor_set}(
        &self,
        descriptors: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],"#
        )
        .unwrap();
        let mut only_tlas = None;
        for (binding_index, binding) in descriptor_set.bindings.iter().enumerate() {
            let name = binding.name();
            let typ = binding.value_type();
            writeln!(file, "        {name}: {typ},").unwrap();
            if binding.descriptor_type() == BindingType::AccelerationStructure {
                assert!(only_tlas.is_none());
                assert_eq!(binding_index, descriptor_set.bindings.len() - 1);
                only_tlas = Some(name);
            }
        }
        writeln!(
            file,
            r#"        dev: &Dev,
    ) {{"#
        )
        .unwrap();
        if let Some(tlas) = only_tlas.as_ref() {
            writeln!(
                file,
                r#"        let supports_raytracing = {tlas}.is_some();"#
            )
            .unwrap();
        }
        writeln!(
            file,
            r#"        for (_flight_index, descriptor) in descriptors.iter().enumerate() {{"#
        )
        .unwrap();
        for (binding_index, binding) in descriptor_set.bindings.iter().enumerate() {
            let binding_name = binding.name();
            let binding_type = binding.descriptor_type().name();
            let write_mutable = match binding {
                DescriptorBinding::AccelerationStructure(_) => "mut ",
                _ => "",
            };
            match binding {
                DescriptorBinding::AccelerationStructure(_) => writeln!(
                    file,
                    r#"            let mut {binding_name}_acceleration_structure = *vk::WriteDescriptorSetAccelerationStructureKHR::builder()
                .acceleration_structures({binding_name}.as_ref().map(|as_| std::slice::from_ref(&as_.acceleration_structure)).unwrap_or_default());"#
                )
                    .unwrap(),
                DescriptorBinding::Image(image) => {
                    let layout = &image.layout;
                    writeln!(
                        file,
                        r#"            let {binding_name}_image = *vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::{layout})
                .image_view({binding_name});"#
                    )
                        .unwrap()
                },
                DescriptorBinding::InputAttachment(_) => writeln!(
                    file,
                    r#"            let {binding_name}_image = *vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view({binding_name});"#
                )
                    .unwrap(),
                DescriptorBinding::StorageBuffer(_) => writeln!(file, r#"            let {binding_name}_buffer = {binding_name}.descriptor(_flight_index);"#).unwrap(),
                DescriptorBinding::StorageImage(_) => writeln!(file,
                    r#"            let {binding_name}_image = *vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::GENERAL)
                .image_view({binding_name});"#
                ).unwrap(),
                DescriptorBinding::Uniform(_) => writeln!(
                    file,
                    r#"            let {binding_name}_buffer = {binding_name}.descriptor(_flight_index);"#
                )
                .unwrap(),
            }
            writeln!(
                file,
                r#"            let {write_mutable}{binding_name} = *vk::WriteDescriptorSet::builder()
                .dst_set(*descriptor)
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
                DescriptorBinding::Image(_)
                | DescriptorBinding::InputAttachment(_)
                | DescriptorBinding::StorageImage(_) => writeln!(
                    file,
                    r#"                .image_info(std::slice::from_ref(&{binding_name}_image));"#
                )
                .unwrap(),
                DescriptorBinding::StorageBuffer(_) => writeln!(
                    file,
                    r#"                .buffer_info(std::slice::from_ref(&{binding_name}_buffer));"#
                )
                .unwrap(),
                DescriptorBinding::Uniform(_) => writeln!(
                    file,
                    r#"                .buffer_info(std::slice::from_ref(&{binding_name}_buffer));"#
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
        write!(file, r#"            let writes = "#).unwrap();
        write_writes(&mut file, &descriptor_set.bindings);
        writeln!(file, r#";"#).unwrap();
        if only_tlas.is_some() {
            let count_without_raytracing = descriptor_set.bindings.len() - 1;
            writeln!(
                file,
                r#"            let writes = if supports_raytracing {{
                &writes
            }} else {{
                &writes[..{count_without_raytracing}]
            }};"#
            )
            .unwrap();
        } else {
            writeln!(file, "            let writes = &writes;").unwrap();
        }
        writeln!(
            file,
            r#"            unsafe {{ dev.update_descriptor_sets(writes, &[]) }};
        }}
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
    for compute in &renderer.computes {
        writeln!(
            file,
            "        unsafe {{ dev.destroy_pipeline_layout(self.{compute}, None) }};"
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"    }}
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

impl Passes {{
    pub fn cleanup(&self, dev: &Dev) {{"#
    )
    .unwrap();
    for pass in &renderer.passes {
        writeln!(file, "        self.{pass}.cleanup(dev);").unwrap();
    }
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
        for (subpass, attachment_index, _, attachment) in pass.attachments() {
            if attachment.swapchain {
                writeln!(file, "    unsafe {{ SCRATCH.{pass}_attachments[{attachment_index}].format = swapchain.format.format }};").unwrap();
            }
            if subpass.msaa {
                writeln!(file, "    unsafe {{ SCRATCH.{pass}_attachments[{attachment_index}].samples = _msaa_samples }};").unwrap();
            }
        }
    }
    for pass in &renderer.passes {
        writeln!(file, "    let {pass} = unsafe {{ dev.create_render_pass(&SCRATCH.{pass}_pass, None).unwrap_unchecked() }};").unwrap();
    }
    for pass in &renderer.passes {
        writeln!(file, "    set_label({pass}, \"RENDER-PASS-{pass}\", dev);").unwrap();
    }
    for (index, pass) in renderer.passes.iter().enumerate() {
        let downscale = pass
            .resolution
            .as_ref()
            .map_or(1, |resolution| resolution.downscaled);
        writeln!(
            file,
            r#"    let extent = vk::Extent2D {{
        width: swapchain.extent.width / {downscale},
        height: swapchain.extent.height / {downscale},
    }};
    let mut framebuffer_attachments = Vec::new();
    let mut framebuffers = Vec::new();
    let mut resources = Vec::new();"#
        )
        .unwrap();
        for (subpass, _, attachment_type, attachment) in pass.attachments() {
            if !attachment.swapchain {
                let format = if let Some(format) = &attachment.format {
                    format!("vk::Format::{format}")
                } else if attachment_type == AttachmentType::Color {
                    "COLOR_FORMAT".to_owned()
                } else {
                    "DEPTH_FORMAT".to_owned()
                };
                let mut flags = match attachment_type {
                    AttachmentType::Color => "vk::ImageUsageFlags::COLOR_ATTACHMENT",
                    AttachmentType::Depth => "vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT",
                }
                .to_owned();
                if pass.used_as_input(attachment) {
                    flags += " | vk::ImageUsageFlags::INPUT_ATTACHMENT";
                }
                if attachment.sampled {
                    flags += " | vk::ImageUsageFlags::SAMPLED";
                }
                if attachment.transient {
                    flags += " | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT";
                }
                let aspect = match attachment_type {
                    AttachmentType::Color => "COLOR",
                    AttachmentType::Depth => "DEPTH",
                };
                let samples = if subpass.msaa {
                    "_msaa_samples"
                } else {
                    "vk::SampleCountFlags::TYPE_1"
                };
                writeln!(
                    file,
                    r#"    let resource = ImageResources::create(
        {format},
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::ImageTiling::OPTIMAL,
        {flags},
        vk::ImageAspectFlags::{aspect},
        extent,
        {samples},
        dev,
    );
    framebuffer_attachments.push(resource.view);
    resources.push(resource);"#
                )
                .unwrap();
            } else {
                writeln!(
                    file,
                    "    framebuffer_attachments.push(vk::ImageView::null());"
                )
                .unwrap();
            }
        }
        writeln!(
            file,
            r#"    let info = *vk::FramebufferCreateInfo::builder()
        .render_pass({pass})
        .attachments(&framebuffer_attachments)
        .width(extent.width)
        .height(extent.height)
        .layers(1);"#
        )
        .unwrap();
        if pass.writes_to_swapchain() {
            let swapchain_index = pass.swapchain_attachment_index();
            writeln!(
                file,
                r#"    for image in &swapchain.image_views {{
        unsafe {{ *(info.p_attachments.add({swapchain_index}) as *mut vk::ImageView) = *image }};
        let framebuffer = unsafe {{ dev.create_framebuffer(&info, None) }}.unwrap();
        framebuffers.push(framebuffer);
    }}"#
            )
            .unwrap();
        } else {
            writeln!(
                file,
                r#"    let framebuffer = unsafe {{ dev.create_framebuffer(&info, None) }}.unwrap();
    framebuffers.push(framebuffer);"#
            )
            .unwrap();
        }
        let debug_name = &pass.debug_name;
        let debug_r = pass.debug_color.red;
        let debug_g = pass.debug_color.green;
        let debug_b = pass.debug_color.blue;
        let direct_to_swapchain = pass.writes_to_swapchain();
        writeln!(
            file,
            r#"    let {pass} = Pass {{
        debug_name: {debug_name:?},
        debug_color: [{debug_r}, {debug_g}, {debug_b}],
        pass: {pass},
        extent,
        clears: vec!["#
        )
        .unwrap();
        for subpass in &pass.subpasses {
            for color in &subpass.color_attachments {
                if let Some(clear) = &color.clear {
                    let clear: Vec<_> = clear.iter().map(|c| *c as f32).collect();
                    writeln!(file, "            vk::ClearValue {{ color: vk::ClearColorValue {{ float32: {clear:?} }} }},").unwrap();
                }
            }
            if let Some(depth) = &subpass.depth_attachment {
                if let Some(clear) = &depth.clear {
                    let clear = clear[0] as f32;
                    writeln!(file, "            vk::ClearValue {{ depth_stencil: vk::ClearDepthStencilValue {{ depth: {clear:?}, stencil: 0 }} }},").unwrap();
                }
            }
        }
        writeln!(
            file,
            r#"        ],
        resources,
        framebuffers,
        direct_to_swapchain: {direct_to_swapchain:?},
        index: {index},
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
pub fn create_pipeline_layouts(
    descriptor_set_layouts: &DescriptorSetLayouts,
    dev: &Dev,
) -> PipelineLayouts {{"#
    )
    .unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        if pipeline.descriptor_sets.len() > 1 {
            for (descriptor_set_index, descriptor_set) in
                pipeline.descriptor_sets.iter().enumerate()
            {
                writeln!(file, "    unsafe {{ SCRATCH.{pipeline}_layouts[{descriptor_set_index}] = descriptor_set_layouts.{descriptor_set} }};").unwrap();
            }
        } else {
            let descriptor_set = &pipeline.descriptor_sets[0];
            writeln!(file, "    unsafe {{ SCRATCH.{pipeline}_pipeline_layout.p_set_layouts = &descriptor_set_layouts.{descriptor_set} }};").unwrap();
        }
    });
    for compute in &renderer.computes {
        if compute.descriptor_sets.len() > 1 {
            for (descriptor_set_index, descriptor_set) in compute.descriptor_sets.iter().enumerate()
            {
                writeln!(file, "    unsafe {{ SCRATCH.{compute}_layouts[{descriptor_set_index}] = descriptor_set_layouts.{descriptor_set} }};").unwrap();
            }
        } else {
            let descriptor_set = &compute.descriptor_sets[0];
            writeln!(file, "    unsafe {{ SCRATCH.{compute}_pipeline_layout.p_set_layouts = &descriptor_set_layouts.{descriptor_set} }};").unwrap();
        }
    }
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(file, r#"    let {pipeline} = unsafe {{ dev.create_pipeline_layout(&SCRATCH.{pipeline}_pipeline_layout, None).unwrap_unchecked() }};"#).unwrap();
    });
    for compute in &renderer.computes {
        writeln!(file, r#"    let {compute} = unsafe {{ dev.create_pipeline_layout(&SCRATCH.{compute}_pipeline_layout, None).unwrap_unchecked() }};"#).unwrap();
    }
    writeln!(file, "    PipelineLayouts {{").unwrap();
    for_pipelines(renderer, |_, _, _, pipeline| {
        writeln!(file, "        {pipeline},").unwrap();
    });
    for compute in &renderer.computes {
        writeln!(file, "        {compute},").unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}

#[rustfmt::skip]
pub fn create_shaders(supports_raytracing: bool) -> Shaders {{"#
    )
    .unwrap();
    for (name, typ) in &shaders {
        let typ_lowercase = typ.lowercase();
        let typ_camelcase = typ.camelcase();
        let ext = typ.extension();
        writeln!(file, r#"    let {name}_{typ_lowercase} = compile_glsl("shaders/{name}.{ext}", shaderc::ShaderKind::{typ_camelcase}, supports_raytracing);"#).unwrap();
    }
    writeln!(file, "    Shaders {{").unwrap();
    for (name, typ) in &shaders {
        let typ_lowercase = typ.lowercase();
        writeln!(file, r#"        {name}_{typ_lowercase},"#).unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}

#[rustfmt::skip]
pub fn create_shader_modules(shaders: &Shaders, dev: &Dev) -> ShaderModules {{"#
    )
    .unwrap();
    for (name, typ) in &shaders {
        let typ_lowercase = typ.lowercase();
        writeln!(file, r#"    unsafe {{ SCRATCH.{name}_{typ_lowercase}.code_size = 4 * shaders.{name}_{typ_lowercase}.len() }};"#).unwrap();
    }
    for (name, typ) in &shaders {
        let typ_lowercase = typ.lowercase();
        writeln!(file, r#"    unsafe {{ SCRATCH.{name}_{typ_lowercase}.p_code = shaders.{name}_{typ_lowercase}.as_ptr() }};"#).unwrap();
    }
    for (name, typ) in &shaders {
        let typ_lowercase = typ.lowercase();
        writeln!(file, r#"    let {name}_{typ_lowercase} = unsafe {{ dev.create_shader_module(&SCRATCH.{name}_{typ_lowercase}, None).unwrap_unchecked() }};"#).unwrap();
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
    for_pipelines(renderer, |_, _, _, pipeline| {
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
    layouts: &PipelineLayouts,
    dev: &Dev,
) -> Pipelines {{"#
    )
    .unwrap();
    for_pipelines(renderer, |pass, _, subpass, pipeline| {
        let downscale = pass
            .resolution
            .as_ref()
            .map_or(1, |resolution| resolution.downscaled);
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
        let vertex_shader = pipeline_vertex_shaders[pipeline.name.as_str()];
        let fragment_shader = pipeline_fragment_shaders[pipeline.name.as_str()];
        writeln!(
            file,
            r#"    unsafe {{ SCRATCH.{pipeline}_shader_stages[0].module = shader_modules.{vertex_shader}_vertex }};
    unsafe {{ SCRATCH.{pipeline}_shader_stages[1].module = shader_modules.{fragment_shader}_fragment }};
    unsafe {{ SCRATCH.{pipeline}_viewport.width = (swapchain.extent.width / {downscale}) as f32 }};
    unsafe {{ SCRATCH.{pipeline}_viewport.height = (swapchain.extent.height / {downscale}) as f32 }};
    unsafe {{ SCRATCH.{pipeline}_scissor.extent.width = swapchain.extent.width / {downscale} }};
    unsafe {{ SCRATCH.{pipeline}_scissor.extent.height = swapchain.extent.height / {downscale} }};"#
        )
            .unwrap();
        if subpass.msaa {
            writeln!(file, "    unsafe {{ SCRATCH.{pipeline}_multisampling.rasterization_samples = _msaa_samples }};").unwrap();
        }
    });
    for_pipelines(renderer, |pass, _, _, pipeline| {
        let pass = &pass.name;
        writeln!(
            file,
            r#"    unsafe {{ SCRATCH.{pipeline}_pipeline.layout = layouts.{pipeline} }};
    unsafe {{ SCRATCH.{pipeline}_pipeline.render_pass = passes.{pass}.pass }};"#
        )
        .unwrap();
    });
    for compute in &renderer.computes {
        writeln!(
            file,
            r#"    unsafe {{ SCRATCH.{compute}_pipeline.layout = layouts.{compute} }};
    unsafe {{ SCRATCH.{compute}_pipeline.stage.module = shader_modules.{compute}_compute }};"#
        )
        .unwrap();
    }
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
    ) }};"#
    )
    .unwrap();
    if !renderer.computes.is_empty() {
        let compute_pipeline_count = renderer.computes.len();
        let first_compute_pipeline = &renderer.computes[0].name;
        writeln!(
            file,
            r#"    let _ = unsafe {{ (dev.fp_v1_0().create_compute_pipelines)(
        dev.handle(),
        vk::PipelineCache::null(),
        {compute_pipeline_count},
        &SCRATCH.{first_compute_pipeline}_pipeline,
        std::ptr::null(),
        (pipelines.as_mut_ptr() as *mut vk::Pipeline).offset({pipeline_count}),
    ) }};"#
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"    unsafe {{ pipelines.assume_init() }}
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
        "R16_UINT" => 2,
        "R32_SFLOAT" => 4,
        "R32G32B32_SFLOAT" => 12,
        "R32G32B32A32_SFLOAT" => 16,
        _ => todo!("attribute_size({:?})", attribute.format),
    }
}
