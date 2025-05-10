use crate::config::{Renderer, Specialization};

impl Renderer {
    pub fn find_specialization(&self, name: &str) -> &Specialization {
        self.specializations
            .iter()
            .find(|spec| spec.name == name)
            .unwrap()
    }
}

impl Specialization {
    pub fn type_default(&self) -> &str {
        match self.ty.as_str() {
            "f32" => "0.",
            "i32" => "0",
            "u32" => "0",
            _ => todo!("{}", self.ty),
        }
    }

    pub fn type_size(&self) -> usize {
        match self.ty.as_str() {
            "f32" => 4,
            "i32" => 4,
            "u32" => 4,
            _ => todo!("{}", self.ty),
        }
    }
}

pub fn to_camelcase(name: &str) -> String {
    let mut result = String::new();
    for word in name.split('_') {
        result += &word[..1].to_uppercase();
        result += &word[1..];
    }
    result
}
