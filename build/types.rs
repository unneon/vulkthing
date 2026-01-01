use spirv_reflect::types::{ReflectDescriptorBinding, ReflectDescriptorType};

pub trait AshDescriptor {
    fn name(&self) -> &str;

    fn ash_value_type(&self) -> String;

    fn struct_type(&self) -> Option<&str>;
}

pub trait AshEnum {
    fn ash_variant(&self) -> &'static str;
}

impl AshDescriptor for ReflectDescriptorBinding {
    fn name(&self) -> &str {
        &self.type_description.as_ref().unwrap().members[0].struct_member_name
    }

    fn ash_value_type(&self) -> String {
        match self.descriptor_type {
            ReflectDescriptorType::AccelerationStructureKHR => "&Option<RaytraceResources>".into(),
            ReflectDescriptorType::SampledImage
            | ReflectDescriptorType::InputAttachment
            | ReflectDescriptorType::StorageImage => "vk::ImageView".into(),
            ReflectDescriptorType::StorageBuffer => {
                let typ = self.struct_type().unwrap();
                format!("&StorageBuffer<[crate::gpu::{typ}]>")
            }
            ReflectDescriptorType::UniformBuffer => {
                let typ = self.struct_type().unwrap();
                format!("&UniformBuffer<crate::gpu::{typ}>")
            }
            _ => todo!(),
        }
    }

    fn struct_type(&self) -> Option<&str> {
        let type_description = self.type_description.as_ref()?;
        if self.descriptor_type == ReflectDescriptorType::UniformBuffer {
            Some(&type_description.type_name)
        } else if self.descriptor_type == ReflectDescriptorType::StorageBuffer {
            Some(&type_description.members[0].type_name)
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
