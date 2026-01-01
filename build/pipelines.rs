use std::fmt;
use std::path::{Path, PathBuf};

pub struct Pipeline {
    name: String,
    pre_rasterization: PreRasterization,
}

pub enum PreRasterization {
    Vertex,
    Mesh { has_task_shader: bool },
}

pub struct Shader<'a> {
    stage: ShaderStage,
    pipeline_name: &'a str,
}

#[derive(Clone, Copy)]
pub enum ShaderStage {
    Vertex,
    Task,
    Mesh,
    Fragment,
}

impl Pipeline {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn shaders(&self) -> impl Iterator<Item = Shader<'_>> {
        self.vertex_shader()
            .into_iter()
            .chain(self.task_shader())
            .chain(self.mesh_shader())
            .chain(std::iter::once(self.fragment_shader()))
    }

    pub fn vertex_shader(&self) -> Option<Shader<'_>> {
        match self.pre_rasterization {
            PreRasterization::Vertex => Some(Shader {
                stage: ShaderStage::Vertex,
                pipeline_name: &self.name,
            }),
            _ => None,
        }
    }

    pub fn task_shader(&self) -> Option<Shader<'_>> {
        match self.pre_rasterization {
            PreRasterization::Mesh {
                has_task_shader: true,
            } => Some(Shader {
                stage: ShaderStage::Task,
                pipeline_name: &self.name,
            }),
            _ => None,
        }
    }

    pub fn mesh_shader(&self) -> Option<Shader<'_>> {
        match self.pre_rasterization {
            PreRasterization::Mesh { .. } => Some(Shader {
                stage: ShaderStage::Mesh,
                pipeline_name: &self.name,
            }),
            PreRasterization::Vertex => None,
        }
    }

    pub fn fragment_shader(&self) -> Shader<'_> {
        Shader {
            stage: ShaderStage::Fragment,
            pipeline_name: &self.name,
        }
    }
}

impl Shader<'_> {
    pub fn pipeline_name(&self) -> &str {
        self.pipeline_name
    }

    pub fn stage(&self) -> ShaderStage {
        self.stage
    }

    pub fn glsl_path(&self) -> PathBuf {
        let file_name = match self.stage {
            ShaderStage::Vertex => "vertex.glsl",
            ShaderStage::Task => "task.glsl",
            ShaderStage::Mesh => "mesh.glsl",
            ShaderStage::Fragment => "fragment.glsl",
        };
        Path::new("shaders")
            .join(self.pipeline_name)
            .join(file_name)
    }

    pub fn glslang_validator_stage(&self) -> &str {
        match self.stage {
            ShaderStage::Vertex => "vert",
            ShaderStage::Task => "task",
            ShaderStage::Mesh => "mesh",
            ShaderStage::Fragment => "frag",
        }
    }
}

impl ShaderStage {
    pub fn ash_uppercase(&self) -> &'static str {
        match self {
            ShaderStage::Vertex => "VERTEX",
            ShaderStage::Task => "TASK_EXT",
            ShaderStage::Mesh => "MESH_EXT",
            ShaderStage::Fragment => "FRAGMENT",
        }
    }
}

impl fmt::Display for ShaderStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ShaderStage::Vertex => "vertex",
            ShaderStage::Task => "task",
            ShaderStage::Mesh => "mesh",
            ShaderStage::Fragment => "fragment",
        })
    }
}

pub fn collect_pipelines() -> Vec<Pipeline> {
    let mut pipelines = Vec::new();
    for entry in std::fs::read_dir("shaders").unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            let name = entry.file_name().into_string().unwrap();
            let mut has_vertex_shader = false;
            let mut has_task_shader = false;
            let mut has_mesh_shader = false;
            let mut has_fragment_shader = false;
            for entry in std::fs::read_dir(entry.path()).unwrap() {
                let entry = entry.unwrap();
                let entry = entry.file_name();
                if entry == "vertex.glsl" {
                    has_vertex_shader = true;
                } else if entry == "task.glsl" {
                    has_task_shader = true;
                } else if entry == "mesh.glsl" {
                    has_mesh_shader = true;
                } else if entry == "fragment.glsl" {
                    has_fragment_shader = true;
                } else {
                    unimplemented!()
                }
            }
            let pre_rasterization = match (has_vertex_shader, has_task_shader, has_mesh_shader) {
                (true, false, false) => PreRasterization::Vertex,
                (false, _, true) => PreRasterization::Mesh { has_task_shader },
                _ => unimplemented!(),
            };
            assert!(has_fragment_shader);
            pipelines.push(Pipeline {
                name,
                pre_rasterization,
            });
        }
    }
    pipelines
}
