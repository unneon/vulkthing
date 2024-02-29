#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Material {
    Air = 0,
    Stone = 1,
    Dirt = 2,
    Grass = 3,
}

impl Material {
    pub fn is_air(&self) -> bool {
        matches!(self, Material::Air)
    }
}
