#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ShaderType {
    Compute,
    Mesh,
    Task,
    Vertex,
    Fragment,
}

impl ShaderType {
    pub fn lowercase(&self) -> &'static str {
        match self {
            ShaderType::Compute => "compute",
            ShaderType::Fragment => "fragment",
            ShaderType::Mesh => "mesh",
            ShaderType::Task => "task",
            ShaderType::Vertex => "vertex",
        }
    }

    pub fn camelcase(&self) -> &'static str {
        match self {
            ShaderType::Compute => "Compute",
            ShaderType::Fragment => "Fragment",
            ShaderType::Mesh => "Mesh",
            ShaderType::Task => "Task",
            ShaderType::Vertex => "Vertex",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ShaderType::Compute => "comp",
            ShaderType::Fragment => "frag",
            ShaderType::Mesh => "mesh",
            ShaderType::Task => "task",
            ShaderType::Vertex => "vert",
        }
    }

    pub fn requires_mesh_shaders(&self) -> bool {
        matches!(self, ShaderType::Mesh | ShaderType::Task)
    }
}
