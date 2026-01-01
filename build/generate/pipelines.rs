use crate::pipelines::Pipeline;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn generate_pipelines(pipelines: &[Pipeline], out_dir: &Path) {
    let mut file = File::create(out_dir.join("pipelines.rs")).unwrap();

    writeln!(file, r#"pub struct Pipelines {{"#).unwrap();
    for pipeline in pipelines {
        let pipeline_name = pipeline.name();
        writeln!(file, "    pub {pipeline_name}: vk::Pipeline,").unwrap();
    }
    writeln!(file, "}}").unwrap();

    // for pipeline in pipelines {
    //     if let Some(specs) = &pipeline.fragment_specialization {
    //         let pipeline_camelcase = to_camelcase(&pipeline.to_string());
    //         writeln!(file, "\nstruct {pipeline_camelcase}Specialization {{").unwrap();
    //         for spec in specs {
    //             let ty = &renderer.find_specialization(spec).ty;
    //             writeln!(file, "    {spec}: {ty},").unwrap();
    //         }
    //         writeln!(file, "}}").unwrap();
    //     }
    // }

    writeln!(
        file,
        r#"
static ASSEMBLY: vk::PipelineInputAssemblyStateCreateInfo = vk::PipelineInputAssemblyStateCreateInfo {{
    s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
    p_next: std::ptr::null(),
    flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
    topology: vk::PrimitiveTopology::TRIANGLE_LIST,
    primitive_restart_enable: 0,
    _marker: std::marker::PhantomData,
}};

static DYNAMIC_STATES: [vk::DynamicState; 4] = [
    vk::DynamicState::VIEWPORT,
    vk::DynamicState::SCISSOR,
    vk::DynamicState::CULL_MODE,
    vk::DynamicState::POLYGON_MODE_EXT,
];

static DYNAMIC_STATE: vk::PipelineDynamicStateCreateInfo = vk::PipelineDynamicStateCreateInfo {{
    s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
    p_next: std::ptr::null(),
    flags: vk::PipelineDynamicStateCreateFlags::empty(),
    dynamic_state_count: 4,
    p_dynamic_states: &raw const DYNAMIC_STATES[0],
    _marker: std::marker::PhantomData,
}};"#
    )
    .unwrap();
    for pipeline in pipelines {
        for shader in pipeline.shaders() {
            let pipeline_name = pipeline.name();
            let pipeline_name_uppercase = pipeline.name().to_uppercase();
            let shader_stage_uppercase = shader.stage().to_string().to_uppercase();
            let shader_stage = shader.stage();
            let include_bytes = format!(
                r#"include_bytes!(concat!(env!("OUT_DIR"), "/shaders/{pipeline_name}.{shader_stage}.spv"))"#
            );
            writeln!(
                file,
                r#"
static {pipeline_name_uppercase}_{shader_stage_uppercase}: vk::ShaderModuleCreateInfo = vk::ShaderModuleCreateInfo {{
    s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
    p_next: std::ptr::null(),
    flags: vk::ShaderModuleCreateFlags::empty(),
    code_size: {pipeline_name_uppercase}_{shader_stage_uppercase}_SPIR_V.0.len(),
    p_code: {pipeline_name_uppercase}_{shader_stage_uppercase}_SPIR_V.0.as_ptr() as *const u32,
    _marker: std::marker::PhantomData,
}};

static {pipeline_name_uppercase}_{shader_stage_uppercase}_SPIR_V: SpvArray<{{ {include_bytes}.len() }}> = SpvArray(*{include_bytes});"#
            )
                .unwrap();
        }
    }
    for pipeline in pipelines {
        let pipeline_name_uppercase = pipeline.name().to_uppercase();
        //         let fragment_specialization_info = if let Some(fragment_specialization) =
        //             &pipeline.fragment_specialization
        //         {
        //             let pipeline_camelcase = to_camelcase(&pipeline.to_string());
        //             let specialization_count = fragment_specialization.len();
        //             writeln!(file, r#"
        // static {pipeline_uppercase}_FRAGMENT_SPECIALIZATION_ENTRIES: [vk::SpecializationMapEntry; {specialization_count}] = ["#).unwrap();
        //             let mut offset = 0;
        //             for (constant_id, spec) in fragment_specialization.iter().enumerate() {
        //                 let size = renderer.find_specialization(spec).type_size();
        //                 writeln!(
        //                     file,
        //                     r#"    vk::SpecializationMapEntry {{
        //         constant_id: {constant_id},
        //         offset: {offset},
        //         size: {size},
        //     }},"#
        //                 )
        //                 .unwrap();
        //                 offset += size;
        //             }
        //             writeln!(
        //                 file,
        //                 r#"];
        //
        // static {pipeline_uppercase}_FRAGMENT_SPECIALIZATION_INFO: vk::SpecializationInfo = vk::SpecializationInfo {{
        //     map_entry_count: {specialization_count},
        //     p_map_entries: &raw const {pipeline_uppercase}_FRAGMENT_SPECIALIZATION_ENTRIES[0],
        //     data_size: {offset},
        //     p_data: &raw const {pipeline_uppercase}_FRAGMENT_SPECIALIZATION_SCRATCH) as *const std::ffi::c_void,
        // }};
        //
        // static {pipeline_uppercase}_FRAGMENT_SPECIALIZATION_SCRATCH: {pipeline_camelcase}Specialization = {pipeline_camelcase}Specialization {{"#
        //             )
        //                 .unwrap();
        //             for spec in fragment_specialization {
        //                 let default = renderer.find_specialization(spec).type_default();
        //                 writeln!(file, "        {spec}: {default},").unwrap();
        //             }
        //             writeln!(file, "}};").unwrap();
        //             format!("&raw const {pipeline_uppercase}_FRAGMENT_SPECIALIZATION_INFO")
        //         } else {
        //             "std::ptr::null()".to_owned()
        //         };
        let shader_stage_count = pipeline.shaders().count();
        writeln!(
            file,
            r#"
static {pipeline_name_uppercase}_SHADER_STAGES: [vk::PipelineShaderStageCreateInfo; {shader_stage_count}] = ["#
        )
        .unwrap();
        for shader in pipeline.shaders() {
            let shader_stage_uppercase = shader.stage().to_string().to_uppercase();
            let shader_stage_ash_uppercase = shader.stage().ash_uppercase();
            writeln!(
                file,
                r#"    vk::PipelineShaderStageCreateInfo {{
        s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        p_next: &raw const {pipeline_name_uppercase}_{shader_stage_uppercase} as *const c_void,
        flags: vk::PipelineShaderStageCreateFlags::empty(),
        stage: vk::ShaderStageFlags::{shader_stage_ash_uppercase},
        module: vk::ShaderModule::null(),
        p_name: c"main".as_ptr(),
        p_specialization_info: std::ptr::null(),
        _marker: std::marker::PhantomData,
    }},"#
            )
            .unwrap();
        }
        writeln!(file, "];").unwrap();
        if pipeline.vertex_shader().is_some() {
            writeln!(
                file,
                r#"
static {pipeline_name_uppercase}_VERTEX_STATE: vk::PipelineVertexInputStateCreateInfo = vk::PipelineVertexInputStateCreateInfo {{
    s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
    p_next: std::ptr::null(),
    flags: vk::PipelineVertexInputStateCreateFlags::empty(),
    vertex_binding_description_count: 0,
    p_vertex_binding_descriptions: std::ptr::null(),
    vertex_attribute_description_count: 0,
    p_vertex_attribute_descriptions: std::ptr::null(),
    _marker: std::marker::PhantomData,
}};"#
            )
            .unwrap();
        }
        let vertex_input_state = if pipeline.vertex_shader().is_some() {
            format!("&raw const {pipeline_name_uppercase}_VERTEX_STATE")
        } else {
            "std::ptr::null()".to_owned()
        };
        writeln!(
            file,
            r#"
static {pipeline_name_uppercase}_VIEWPORT_STATE: vk::PipelineViewportStateCreateInfo = vk::PipelineViewportStateCreateInfo {{
    s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
    p_next: std::ptr::null(),
    flags: vk::PipelineViewportStateCreateFlags::empty(),
    viewport_count: 1,
    p_viewports: std::ptr::null(),
    scissor_count: 1,
    p_scissors: std::ptr::null(),
    _marker: std::marker::PhantomData,
}};

static {pipeline_name_uppercase}_RASTERIZER: vk::PipelineRasterizationStateCreateInfo = vk::PipelineRasterizationStateCreateInfo {{
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
    _marker: std::marker::PhantomData,
}};

static {pipeline_name_uppercase}_MULTISAMPLING: vk::PipelineMultisampleStateCreateInfo = vk::PipelineMultisampleStateCreateInfo {{
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
}};

static {pipeline_name_uppercase}_BLEND_ATTACHMENTS: [vk::PipelineColorBlendAttachmentState; 1] = [
    vk::PipelineColorBlendAttachmentState {{
        blend_enable: 0,
        src_color_blend_factor: vk::BlendFactor::ZERO,
        dst_color_blend_factor: vk::BlendFactor::ZERO,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ZERO,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::RGBA,
    }},
];

static {pipeline_name_uppercase}_BLEND: vk::PipelineColorBlendStateCreateInfo = vk::PipelineColorBlendStateCreateInfo {{
    s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
    p_next: std::ptr::null(),
    flags: vk::PipelineColorBlendStateCreateFlags::empty(),
    logic_op_enable: 0,
    logic_op: vk::LogicOp::CLEAR,
    attachment_count: 1,
    p_attachments: &raw const {pipeline_name_uppercase}_BLEND_ATTACHMENTS[0],
    blend_constants: [0., 0., 0., 0.],
    _marker: std::marker::PhantomData,
}};

static {pipeline_name_uppercase}_DEPTH: vk::PipelineDepthStencilStateCreateInfo = vk::PipelineDepthStencilStateCreateInfo {{
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
}};

static mut {pipeline_name_uppercase}_COLOR_FORMATS: [vk::Format; 1] = [vk::Format::UNDEFINED];

static {pipeline_name_uppercase}_RENDERING: vk::PipelineRenderingCreateInfo = vk::PipelineRenderingCreateInfo {{
    s_type: vk::StructureType::PIPELINE_RENDERING_CREATE_INFO,
    p_next: std::ptr::null(),
    view_mask: 0,
    color_attachment_count: 1,
    p_color_attachment_formats: unsafe {{ &raw const {pipeline_name_uppercase}_COLOR_FORMATS[0] }},
    depth_attachment_format: DEPTH_FORMAT,
    stencil_attachment_format: vk::Format::UNDEFINED,
    _marker: std::marker::PhantomData,
}};

static mut {pipeline_name_uppercase}_PIPELINE: vk::GraphicsPipelineCreateInfo = vk::GraphicsPipelineCreateInfo {{
    s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
    p_next: &raw const {pipeline_name_uppercase}_RENDERING as *const _,
    flags: vk::PipelineCreateFlags::empty(),
    stage_count: {shader_stage_count},
    p_stages: &raw const {pipeline_name_uppercase}_SHADER_STAGES[0],
    p_vertex_input_state: {vertex_input_state},
    p_input_assembly_state: &raw const ASSEMBLY,
    p_tessellation_state: std::ptr::null(),
    p_viewport_state: &raw const {pipeline_name_uppercase}_VIEWPORT_STATE,
    p_rasterization_state: &raw const {pipeline_name_uppercase}_RASTERIZER,
    p_multisample_state: &raw const {pipeline_name_uppercase}_MULTISAMPLING,
    p_depth_stencil_state: &raw const {pipeline_name_uppercase}_DEPTH,
    p_color_blend_state: &raw const {pipeline_name_uppercase}_BLEND,
    p_dynamic_state: &raw const DYNAMIC_STATE,
    layout: vk::PipelineLayout::null(),
    render_pass: vk::RenderPass::null(),
    subpass: 0,
    base_pipeline_handle: vk::Pipeline::null(),
    base_pipeline_index: 0,
    _marker: std::marker::PhantomData,
}};"#
        )
        .unwrap();
    }
    //     for compute in &renderer.computes {
    //         writeln!(
    //             file,
    //             r#"
    // static {compute}_PIPELINE: vk::ComputePipelineCreateInfo = vk::ComputePipelineCreateInfo {{
    //     s_type: vk::StructureType::COMPUTE_PIPELINE_CREATE_INFO,
    //     p_next: std::ptr::null(),
    //     flags: vk::PipelineCreateFlags::empty(),
    //     stage: vk::PipelineShaderStageCreateInfo {{
    //         s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
    //         p_next: std::ptr::null(),
    //         flags: vk::PipelineShaderStageCreateFlags::empty(),
    //         stage: vk::ShaderStageFlags::COMPUTE,
    //         module: vk::ShaderModule::null(),
    //         p_name: c"main".as_ptr(),
    //         p_specialization_info: std::ptr::null(),
    //     }},
    //     layout: vk::PipelineLayout::null(),
    //     base_pipeline_handle: vk::Pipeline::null(),
    //     base_pipeline_index: 0,
    // }};"#
    //         )
    //         .unwrap();
    //     }
    writeln!(
        file,
        r#"
impl Pipelines {{
    pub fn cleanup(&self, dev: &Dev) {{"#
    )
    .unwrap();
    for pipeline in pipelines {
        let pipeline_name = pipeline.name();
        writeln!(
            file,
            "        unsafe {{ dev.destroy_pipeline(self.{pipeline_name}, None) }};"
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}

#[allow(clippy::identity_op)]
pub fn create_pipelines(
    _msaa_samples: vk::SampleCountFlags,"#
    )
    .unwrap();
    // for spec in &renderer.specializations {
    //     let name = &spec.name;
    //     let ty = &spec.ty;
    //     if spec.shared {
    //         writeln!(file, "    {name}: {ty},").unwrap();
    //     }
    // }
    // for pipeline in &renderer.pipelines {
    //     if let Some(specs) = &pipeline.fragment_specialization {
    //         for spec in specs {
    //             let metadata = renderer.find_specialization(spec);
    //             if !metadata.shared {
    //                 let ty = &metadata.ty;
    //                 writeln!(file, "    {pipeline}_{spec}: {ty},").unwrap();
    //             }
    //         }
    //     }
    // }
    writeln!(
        file,
        r#"    swapchain: &Swapchain,
    layout: vk::PipelineLayout,
    dev: &Dev,
) -> Pipelines {{"#
    )
    .unwrap();
    for pipeline in pipelines {
        let pipeline_uppercase = pipeline.name().to_uppercase();
        // if let Some(specs) = &pipeline.fragment_specialization {
        //     for spec in specs {
        //         let metadata = renderer.find_specialization(spec);
        //         let value = if metadata.shared {
        //             spec.clone()
        //         } else {
        //             format!("{pipeline}_{spec}")
        //         };
        //         writeln!(
        //             file,
        //             "    unsafe {{ {pipeline_uppercase}_FRAGMENT_SPECIALIZATION_SCRATCH.{spec} = {value} }};"
        //         )
        //         .unwrap();
        //     }
        // }
        writeln!(
            file,
            r#"    unsafe {{ {pipeline_uppercase}_COLOR_FORMATS[0] = swapchain.format.format }};"#
        )
        .unwrap();
    }
    for pipeline in pipelines {
        let pipeline_uppercase = pipeline.name().to_uppercase();
        writeln!(
            file,
            r#"    unsafe {{ {pipeline_uppercase}_PIPELINE.layout = layout }};"#
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"    let mut pipelines: Pipelines = unsafe {{ MaybeUninit::zeroed().assume_init() }};"#
    )
    .unwrap();
    for pipeline in pipelines {
        let pipeline_name = pipeline.name();
        let pipeline_name_uppercase = pipeline.name().to_uppercase();
        let tab = if pipeline.mesh_shader().is_some() {
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
{tab}        &raw const {pipeline_name_uppercase}_PIPELINE,
{tab}        std::ptr::null(),
{tab}        &mut pipelines.{pipeline_name},
{tab}    ) }};"#
        )
        .unwrap();
        if pipeline.mesh_shader().is_some() {
            writeln!(file, "    }}").unwrap();
        }
    }
    // if !renderer.computes.is_empty() {
    //     let compute_pipeline_count = renderer.computes.len();
    //     let first_compute_pipeline = &renderer.computes[0].name;
    //     writeln!(
    //         file,
    //         r#"    let _ = unsafe {{ (dev.fp_v1_0().create_compute_pipelines)(
    //     dev.handle(),
    //     vk::PipelineCache::null(),
    //     {compute_pipeline_count},
    //     &raw const {first_compute_pipeline}_PIPELINE,
    //     std::ptr::null(),
    //     (pipelines.as_mut_ptr() as *mut vk::Pipeline).offset({pipeline_count}),
    // ) }};"#
    //     )
    //     .unwrap();
    // }
    writeln!(
        file,
        r#"    pipelines
}}"#
    )
    .unwrap();
}
