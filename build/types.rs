use spirv_reflect::types::{ReflectDescriptorBinding, ReflectDescriptorType};

pub trait AshDescriptor {
    fn ash_value_type(&self) -> String;

    fn struct_type(&self) -> Option<&str>;
}

pub trait AshEnum {
    fn ash_variant(&self) -> &'static str;
}

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

    pub fn extension(&self) -> &'static str {
        match self {
            ShaderType::Compute => "comp",
            ShaderType::Fragment => "frag",
            ShaderType::Mesh => "mesh",
            ShaderType::Task => "task",
            ShaderType::Vertex => "vert",
        }
    }
}

impl AshDescriptor for ReflectDescriptorBinding {
    fn ash_value_type(&self) -> String {
        match self.descriptor_type {
            ReflectDescriptorType::AccelerationStructureKHR => "&Option<RaytraceResources>".into(),
            ReflectDescriptorType::SampledImage
            | ReflectDescriptorType::InputAttachment
            | ReflectDescriptorType::StorageImage => "vk::ImageView".into(),
            ReflectDescriptorType::StorageBuffer => {
                let typ = self.struct_type().unwrap();
                format!("&StorageBuffer<[{typ}]>")
            }
            ReflectDescriptorType::UniformBuffer => {
                let typ = self.struct_type().unwrap();
                format!("&UniformBuffer<{typ}>")
            }
            _ => todo!(),
        }
    }

    fn struct_type(&self) -> Option<&str> {
        let type_description = self.type_description.as_ref()?;
        if self.descriptor_type == ReflectDescriptorType::UniformBuffer {
            Some(type_description.type_name.strip_suffix("_std140").unwrap())
        } else if self.descriptor_type == ReflectDescriptorType::StorageBuffer {
            Some(
                type_description.members[0]
                    .type_name
                    .strip_suffix("_std430")
                    .unwrap(),
            )
        } else {
            None
        }
    }
}

impl AshEnum for ReflectDescriptorType {
    fn ash_variant(&self) -> &'static str {
        match self {
            ReflectDescriptorType::UniformBuffer => "UNIFORM_BUFFER",
            ReflectDescriptorType::StorageBuffer => "STORAGE_BUFFER",
            _ => todo!(),
        }
    }
}
