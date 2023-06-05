use crate::grass::Grass;
use crate::planet::Planet;
use crate::renderer::uniform::{FragSettings, Postprocessing};
use imgui::{Condition, Context, Drag, TreeNodeFlags, Ui};
use nalgebra::Vector3;
use std::borrow::Cow;

pub mod integration;

pub trait EnumInterface: Sized + 'static {
    const VALUES: &'static [Self];

    fn label(&self) -> Cow<str>;
}

pub struct Interface {
    pub ctx: Context,
    cursor_visible: bool,
}

pub struct InterfaceEvents {
    pub planet_changed: bool,
    pub grass_changed: bool,
}

impl Interface {
    pub fn build(
        &mut self,
        planet: &mut Planet,
        grass: &mut Grass,
        frag_settings: &mut FragSettings,
        postprocessing: &mut Postprocessing,
    ) -> InterfaceEvents {
        let ui = self.ctx.frame();
        let mut events = InterfaceEvents {
            planet_changed: false,
            grass_changed: false,
        };
        ui.window("Debugging")
            .size([0., 0.], Condition::Always)
            .build(|| {
                if ui.collapsing_header("Planet generation", TreeNodeFlags::empty()) {
                    let mut changed = false;
                    changed |= ui.slider("Resolution", 1, 800, &mut planet.resolution);
                    changed |= ui.slider("Radius", 10., 200., &mut planet.radius);
                    changed |= enum_combo(ui, "Noise type", &mut planet.noise_type);
                    changed |= ui.slider("Noise magnitude", 0., 100., &mut planet.noise_magnitude);
                    changed |= ui.slider("Noise scale", 0., 64., &mut planet.noise_scale);
                    changed |= ui.slider("Noise layers", 0, 16, &mut planet.noise_layers);
                    changed |= ui.slider("Chunk count", 1, 255, &mut planet.chunk_count);
                    planet.chunk_count += planet.chunk_count % 2 - 1;
                    events.planet_changed = changed;
                }
                if ui.collapsing_header("Grass", TreeNodeFlags::DEFAULT_OPEN) {
                    let mut changed = false;
                    changed |= ui.slider(
                        "Blades per planet triangle",
                        1,
                        256,
                        &mut grass.blades_per_triangle,
                    );
                    ui.slider("Height average", 0.01, 3., &mut grass.height_average);
                    ui.slider(
                        "Height max variance",
                        0.,
                        1.,
                        &mut grass.height_max_variance,
                    );
                    changed |= ui.slider(
                        "Height noise frequency",
                        0.01,
                        1.,
                        &mut grass.height_noise_frequency,
                    );
                    ui.slider("Width", 0.01, 100., &mut grass.width);
                    events.grass_changed = changed;
                }
                ui.checkbox("Ray-traced shadows", &mut frag_settings.use_ray_tracing);
                if ui.collapsing_header("Postprocessing", TreeNodeFlags::empty()) {
                    build_postprocessing(ui, postprocessing);
                }
            });
        events
    }
}

fn build_postprocessing(ui: &Ui, postprocessing: &mut Postprocessing) {
    Drag::new("Exposure")
        .range(0., f32::INFINITY)
        .speed(0.01)
        .build(ui, &mut postprocessing.exposure);
    ui.slider("Temperature", -1.67, 1.67, &mut postprocessing.temperature);
    ui.slider("Tint", -1.67, 1.67, &mut postprocessing.tint);
    Drag::new("Contrast")
        .range(0., f32::INFINITY)
        .speed(0.01)
        .build(ui, &mut postprocessing.contrast);
    Drag::new("Brightness")
        .range(0., f32::INFINITY)
        .speed(0.01)
        .build(ui, &mut postprocessing.brightness);
    enum_color(ui, "Color filter", &mut postprocessing.color_filter);
    Drag::new("Saturation")
        .range(0., f32::INFINITY)
        .speed(0.01)
        .build(ui, &mut postprocessing.saturation);
    enum_combo(ui, "Tonemapper", &mut postprocessing.tonemapper);
    Drag::new("Gamma")
        .range(0., f32::INFINITY)
        .speed(0.01)
        .build(ui, &mut postprocessing.gamma);
}

fn enum_color(ui: &Ui, label: &str, value: &mut Vector3<f32>) {
    let mut array = [value.x, value.y, value.z];
    ui.color_edit3(label, &mut array);
    *value = Vector3::from_column_slice(&array);
}

fn enum_combo<T: Copy + EnumInterface + PartialEq>(ui: &Ui, label: &str, value: &mut T) -> bool {
    let mut index = T::VALUES
        .iter()
        .enumerate()
        .find(|(_, x)| *x == value)
        .unwrap()
        .0;
    let changed = ui.combo(label, &mut index, T::VALUES, T::label);
    *value = T::VALUES[index];
    changed
}
