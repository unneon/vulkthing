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

impl VertexOps for Vertex {
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
