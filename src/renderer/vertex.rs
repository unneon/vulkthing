use crate::renderer::traits::VertexOps;
use ash::vk;
use nalgebra::{Vector2, Vector3};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex: Vector2<f32>,
}

impl VertexOps for Vertex {
    const ATTRIBUTE_FORMATS: &'static [vk::Format] = &[
        vk::Format::R32G32B32_SFLOAT,
        vk::Format::R32G32B32_SFLOAT,
        vk::Format::R32G32_SFLOAT,
    ];
    const ATTRIBUTE_SIZES: &'static [usize] = &[
        std::mem::size_of::<Vector3<f32>>(),
        std::mem::size_of::<Vector3<f32>>(),
        std::mem::size_of::<Vector2<f32>>(),
    ];
}
