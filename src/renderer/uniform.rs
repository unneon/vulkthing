use crate::interface::Editable;
use imgui::{Drag, Ui};
use nalgebra::{Matrix4, Vector3};

#[repr(C)]
pub struct ModelViewProjection {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

#[repr(C)]
pub struct Material {
    pub emit: Vector3<f32>,
}

#[repr(C)]
pub struct Light {
    pub color: Vector3<f32>,
    pub ambient_strength: f32,
    pub position: Vector3<f32>,
    pub diffuse_strength: f32,
}

#[repr(C)]
pub struct Filters {
    pub color_filter: Vector3<f32>,
    pub exposure: f32,
    pub temperature: f32,
    pub tint: f32,
    pub contrast: f32,
    pub brightness: f32,
    pub saturation: f32,
    pub tonemapper: Tonemapper,
    pub gamma: f32,
}

#[repr(u32)]
#[derive(Clone, Copy, Eq, PartialEq)]
#[allow(dead_code)]
pub enum Tonemapper {
    RgbClamping = 0,
    TumblinRushmeier = 1,
    Schlick = 2,
    Ward = 3,
    Reinhard = 4,
    ReinhardExtended = 5,
    Hable = 6,
    Uchimura = 7,
    NarkowiczAces = 8,
    HillAces = 9,
}

const TONEMAPPERS: &[Tonemapper] = &[
    Tonemapper::RgbClamping,
    Tonemapper::Reinhard,
    Tonemapper::NarkowiczAces,
];

impl Tonemapper {
    fn name(&self) -> &'static str {
        match self {
            Tonemapper::RgbClamping => "RGB Clamping",
            Tonemapper::TumblinRushmeier => "Tumblin Rushmeier",
            Tonemapper::Schlick => "Schlick",
            Tonemapper::Ward => "Ward",
            Tonemapper::Reinhard => "Reinhard",
            Tonemapper::ReinhardExtended => "Reinhard extended",
            Tonemapper::Hable => "Hable",
            Tonemapper::Uchimura => "Uchimura",
            Tonemapper::NarkowiczAces => "Narkowicz ACES",
            Tonemapper::HillAces => "Hill ACES",
        }
    }
}

impl Editable for Filters {
    fn name(&self) -> &str {
        "Postprocessing"
    }

    fn widget(&mut self, ui: &Ui) -> bool {
        let mut color_filter = [
            self.color_filter.x,
            self.color_filter.y,
            self.color_filter.z,
        ];
        let mut tonemapper = TONEMAPPERS
            .iter()
            .enumerate()
            .find(|(_, tm)| **tm == self.tonemapper)
            .unwrap()
            .0;
        let mut changed = false;

        changed |= Drag::new("Exposure")
            .range(0., f32::INFINITY)
            .speed(0.01)
            .build(ui, &mut self.exposure);
        changed |= ui.slider("Temperature", -1.67, 1.67, &mut self.temperature);
        changed |= ui.slider("Tint", -1.67, 1.67, &mut self.tint);
        changed |= Drag::new("Contrast")
            .range(0., f32::INFINITY)
            .speed(0.01)
            .build(ui, &mut self.contrast);
        changed |= Drag::new("Brightness")
            .range(0., f32::INFINITY)
            .speed(0.01)
            .build(ui, &mut self.brightness);
        changed |= ui.color_edit3("Color filter", &mut color_filter);
        changed |= Drag::new("Saturation")
            .range(0., f32::INFINITY)
            .speed(0.01)
            .build(ui, &mut self.saturation);
        changed |= ui.combo("Tonemapper", &mut tonemapper, &TONEMAPPERS, |tm| {
            tm.name().into()
        });
        changed |= Drag::new("Gamma")
            .range(0., f32::INFINITY)
            .speed(0.01)
            .build(ui, &mut self.gamma);

        self.color_filter = Vector3::new(color_filter[0], color_filter[1], color_filter[2]);
        self.tonemapper = TONEMAPPERS[tonemapper];
        changed
    }
}

impl Default for Filters {
    fn default() -> Self {
        Filters {
            exposure: 1.,
            temperature: -0.7,
            tint: 0.,
            contrast: 1.,
            brightness: 0.,
            color_filter: Vector3::new(1., 1., 1.),
            saturation: 1.,
            tonemapper: Tonemapper::NarkowiczAces,
            gamma: 1.,
        }
    }
}
