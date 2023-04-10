use nalgebra_glm as glm;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ModelViewProjection {
    pub model: glm::Mat4,
    pub view: glm::Mat4,
    pub proj: glm::Mat4,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Light {
    pub color: glm::Vec3,
    pub ambient_strength: f32,
    pub position: glm::Vec3,
}
