use knuffel::Decode;

#[derive(Debug, Decode)]
pub struct Renderer {
    #[knuffel(children(name = "mesh"))]
    pub meshes: Vec<Mesh>,
    #[knuffel(children(name = "sampler"))]
    pub samplers: Vec<Sampler>,
    #[knuffel(children(name = "descriptor-set"))]
    pub descriptor_sets: Vec<DescriptorSet>,
    #[knuffel(children(name = "pass"))]
    pub passes: Vec<Pass>,
}

#[derive(Debug, Decode)]
pub struct Mesh {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(property, default)]
    pub dynamic: bool,
}

#[derive(Debug, Decode)]
pub struct Sampler {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(child, unwrap(argument))]
    pub address_mode_u: String,
    #[knuffel(child, unwrap(argument))]
    pub address_mode_v: String,
    #[knuffel(child)]
    pub unnormalized_coordinates: bool,
}

#[derive(Debug, Decode)]
pub struct DescriptorSet {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(argument)]
    pub pool_size: usize,
    #[knuffel(children)]
    pub bindings: Vec<DescriptorBinding>,
}

#[derive(Debug, Decode)]
pub enum DescriptorBinding {
    AccelerationStructure(AccelerationStructureBinding),
    Image(ImageBinding),
    InputAttachment(InputAttachmentBinding),
    Uniform(UniformBinding),
}

#[derive(Debug, Decode)]
pub struct AccelerationStructureBinding {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(argument)]
    pub stage: String,
}

#[derive(Debug, Decode)]
pub struct ImageBinding {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(argument)]
    pub stage: String,
    #[knuffel(argument)]
    pub sampler: String,
    #[knuffel(property, default)]
    pub msaa: bool,
}

#[derive(Debug, Decode)]
pub struct InputAttachmentBinding {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(argument)]
    pub stage: String,
    #[knuffel(property, default)]
    pub msaa: bool,
}

#[derive(Debug, Decode)]
pub struct UniformBinding {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(argument)]
    pub stage: String,
    #[knuffel(argument)]
    pub typ: String,
}

#[derive(Debug, Decode)]
pub struct Pass {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(child, unwrap(argument))]
    pub debug_name: String,
    #[knuffel(child)]
    pub debug_color: SdrColor,
    #[knuffel(child)]
    pub msaa: bool,
    #[knuffel(children(name = "subpass"))]
    pub subpasses: Vec<Subpass>,
    #[knuffel(children(name = "dependency"))]
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Decode)]
pub struct SdrColor {
    #[knuffel(argument)]
    pub red: u8,
    #[knuffel(argument)]
    pub green: u8,
    #[knuffel(argument)]
    pub blue: u8,
}

#[derive(Debug, Decode)]
pub struct Subpass {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(children(name = "input-attachment"), unwrap(argument))]
    pub input_attachment: Vec<String>,
    #[knuffel(children(name = "color-attachment"))]
    pub color_attachments: Vec<Attachment>,
    #[knuffel(child)]
    pub depth_attachment: Option<Attachment>,
    #[knuffel(children(name = "pipeline"))]
    pub pipelines: Vec<Pipeline>,
}

#[derive(Debug, Decode)]
pub struct Attachment {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(child, unwrap(argument))]
    pub layout: String,
    #[knuffel(child, unwrap(argument))]
    pub layout_final: Option<String>,
    #[knuffel(child, unwrap(arguments))]
    pub clear: Option<Vec<i32>>,
    #[knuffel(child)]
    pub input_attachment: bool,
    #[knuffel(child)]
    pub sampled: bool,
    #[knuffel(child)]
    pub store: bool,
    #[knuffel(child)]
    pub swapchain: bool,
    #[knuffel(child)]
    pub transient: bool,
}

#[derive(Debug, Decode)]
pub struct Pipeline {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(children(name = "vertex-binding"))]
    pub vertex_bindings: Vec<VertexBinding>,
    #[knuffel(children(name = "specialization"))]
    pub specializations: Vec<Specialization>,
    #[knuffel(child, unwrap(arguments))]
    pub descriptor_sets: Vec<String>,
    #[knuffel(child, unwrap(argument), default = "BACK".into())]
    pub cull_mode: String,
}

#[derive(Debug, Decode)]
pub struct VertexBinding {
    #[knuffel(property, default = "VERTEX".into())]
    pub rate: String,
    #[knuffel(children(name = "attribute"))]
    pub attributes: Vec<VertexAttribute>,
}

#[derive(Debug, Decode)]
pub struct VertexAttribute {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(argument)]
    pub format: String,
    #[knuffel(property, default)]
    pub unused: bool,
}

#[derive(Debug, Decode)]
pub struct Specialization {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(argument)]
    pub stage: String,
}

#[derive(Debug, Decode)]
pub struct Dependency {
    #[knuffel(child)]
    pub src: DependencyTarget,
    #[knuffel(child)]
    pub dst: DependencyTarget,
    #[knuffel(child)]
    pub by_region: bool,
}

#[derive(Debug, Decode)]
pub struct DependencyTarget {
    #[knuffel(argument)]
    pub subpass: String,
    #[knuffel(argument)]
    pub stage_mask: String,
    #[knuffel(argument)]
    pub access_mask: String,
}
