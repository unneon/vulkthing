use crate::renderer::traits::VertexOps;
use ash::vk;
use nalgebra_glm as glm;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub normal: glm::Vec3,
    pub tex: glm::Vec2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UniformBufferObject {
    pub model: glm::Mat4,
    pub view: glm::Mat4,
    pub proj: glm::Mat4,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Lighting {
    pub color: glm::Vec3,
    pub pos: glm::Vec3,
}

impl VertexOps for Vertex {
    const ATTRIBUTE_COUNT: usize = 3;
    const ATTRIBUTE_FORMATS: &'static [vk::Format] = &[
        vk::Format::R32G32B32_SFLOAT,
        vk::Format::R32G32B32_SFLOAT,
        vk::Format::R32G32_SFLOAT,
    ];
    const ATTRIBUTE_SIZES: &'static [usize] = &[
        std::mem::size_of::<glm::Vec3>(),
        std::mem::size_of::<glm::Vec3>(),
        std::mem::size_of::<glm::Vec2>(),
    ];
}
