use crate::types::ShaderType;
use knuffel::Decode;

#[derive(Debug, Decode)]
pub struct Renderer {
    #[knuffel(children(name = "sampler"))]
    pub samplers: Vec<Sampler>,
    #[knuffel(child)]
    pub descriptor_set: DescriptorSet,
    #[knuffel(children(name = "pass"))]
    pub passes: Vec<Pass>,
    #[knuffel(children(name = "compute"))]
    pub computes: Vec<Compute>,
    #[knuffel(children(name = "specialization"))]
    pub specializations: Vec<Specialization>,
}

#[derive(Debug, Decode)]
pub struct Sampler {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(child, unwrap(argument))]
    pub filter: String,
    #[knuffel(child, unwrap(argument))]
    pub address_mode: String,
    #[knuffel(child)]
    pub unnormalized_coordinates: bool,
}

#[derive(Debug, Decode)]
pub struct DescriptorSet {
    #[knuffel(children)]
    pub bindings: Vec<DescriptorBinding>,
}

#[derive(Debug, Decode)]
pub enum DescriptorBinding {
    AccelerationStructure(AccelerationStructureBinding),
    Image(ImageBinding),
    InputAttachment(InputAttachmentBinding),
    StorageBuffer(StorageBufferBinding),
    StorageImage(StorageImageBinding),
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
    #[knuffel(property, default = "SHADER_READ_ONLY_OPTIMAL".to_owned())]
    pub layout: String,
}

#[derive(Debug, Decode)]
pub struct InputAttachmentBinding {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(argument)]
    pub stage: String,
}

#[derive(Debug, Decode)]
pub struct StorageBufferBinding {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(argument)]
    pub stage: String,
    #[knuffel(argument)]
    pub typ: String,
}

#[derive(Debug, Decode)]
pub struct StorageImageBinding {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(argument)]
    pub stage: String,
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
    #[knuffel(children(name = "pipeline"))]
    pub pipelines: Vec<Pipeline>,
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
pub struct Pipeline {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(property, default = false)]
    pub task_shaders: bool,
    #[knuffel(property, default = false)]
    pub mesh_shaders: bool,
    #[knuffel(child, unwrap(argument))]
    pub vertex_shader: Option<String>,
    #[knuffel(children(name = "vertex-binding"))]
    pub vertex_bindings: Vec<VertexBinding>,
    #[knuffel(child, unwrap(argument))]
    pub task_shader: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub mesh_shader: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub fragment_shader: Option<String>,
    #[knuffel(child, unwrap(arguments))]
    pub fragment_specialization: Option<Vec<String>>,
    #[knuffel(child, unwrap(argument), default = "FILL".into())]
    pub polygon_mode: String,
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
    pub _name: String,
    #[knuffel(argument)]
    pub format: String,
    #[knuffel(property, default)]
    pub unused: bool,
}

#[derive(Debug, Decode)]
pub struct Compute {
    #[knuffel(argument)]
    pub name: String,
}

#[derive(Debug, Decode)]
pub struct Specialization {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(argument)]
    pub ty: String,
    #[knuffel(property, default = true)]
    pub shared: bool,
}

impl Renderer {
    pub fn pipelines(&self) -> impl Iterator<Item = &Pipeline> {
        self.passes.iter().flat_map(|pass| &pass.pipelines)
    }

    pub fn shaders(&self) -> impl Iterator<Item = (&str, ShaderType)> {
        self.pipelines().flat_map(|pipeline| {
            let task_shader = if pipeline.task_shaders {
                let task_shader = match &pipeline.task_shader {
                    Some(path) => path.strip_suffix(".task").unwrap(),
                    None => pipeline.name.as_str(),
                };
                Some((task_shader, ShaderType::Task))
            } else {
                None
            };
            let mesh_shader = if pipeline.mesh_shaders {
                let mesh_shader = match &pipeline.mesh_shader {
                    Some(path) => path.strip_suffix(".mesh").unwrap(),
                    None => pipeline.name.as_str(),
                };
                Some((mesh_shader, ShaderType::Mesh))
            } else {
                None
            };
            let vertex_shader = if !pipeline.mesh_shaders {
                let vertex_shader = match &pipeline.vertex_shader {
                    Some(path) => path.strip_suffix(".vert").unwrap(),
                    None => pipeline.name.as_str(),
                };
                Some((vertex_shader, ShaderType::Vertex))
            } else {
                None
            };
            let fragment_shader = match &pipeline.fragment_shader {
                Some(path) => path.strip_suffix(".frag").unwrap(),
                None => pipeline.name.as_str(),
            };
            let fragment_shader = (fragment_shader, ShaderType::Fragment);
            task_shader
                .into_iter()
                .chain(mesh_shader.into_iter())
                .chain(vertex_shader.into_iter())
                .chain(std::iter::once(fragment_shader))
        })
    }
}
