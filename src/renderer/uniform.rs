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
    pub gamma: f32,
}

impl Editable for Filters {
    fn name(&self) -> &str {
        "Postprocessing"
    }

    fn widget(&mut self, ui: &Ui) {
        let mut color_filter = [
            self.color_filter.x,
            self.color_filter.y,
            self.color_filter.z,
        ];

        Drag::new("Exposure")
            .range(0., f32::INFINITY)
            .speed(0.01)
            .build(ui, &mut self.exposure);
        ui.slider("Temperature", -1.67, 1.67, &mut self.temperature);
        ui.slider("Tint", -1.67, 1.67, &mut self.tint);
        Drag::new("Contrast")
            .range(0., f32::INFINITY)
            .speed(0.01)
            .build(ui, &mut self.contrast);
        Drag::new("Brightness")
            .range(0., f32::INFINITY)
            .speed(0.01)
            .build(ui, &mut self.brightness);
        ui.color_edit3("Color filter", &mut color_filter);
        Drag::new("Saturation")
            .range(0., f32::INFINITY)
            .speed(0.01)
            .build(ui, &mut self.saturation);
        Drag::new("Gamma")
            .range(0., f32::INFINITY)
            .speed(0.01)
            .build(ui, &mut self.gamma);

        self.color_filter = Vector3::new(color_filter[0], color_filter[1], color_filter[2]);
    }
}

impl Default for Filters {
    fn default() -> Self {
        Filters {
            exposure: 1.,
            temperature: -0.4,
            tint: 0.,
            contrast: 1.,
            brightness: 0.,
            color_filter: Vector3::new(1., 1., 1.),
            saturation: 1.,
            gamma: 1.,
        }
    }
}
