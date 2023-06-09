use crate::grass::Grass;
use crate::planet::Planet;
use crate::renderer::uniform::{FragSettings, Postprocessing};
use crate::renderer::{Renderer, RendererSettings};
use imgui::{Condition, Context, Drag, SliderFlags, TreeNodeFlags, Ui};
use nalgebra::Vector3;
use std::borrow::Cow;
use std::sync::atomic::Ordering;

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
        renderer_settings: &mut RendererSettings,
        frag_settings: &mut FragSettings,
        postprocessing: &mut Postprocessing,
        renderer: &Renderer,
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

                    events.planet_changed = changed;
                }
                if ui.collapsing_header("Grass", TreeNodeFlags::DEFAULT_OPEN) {
                    let mut changed = false;
                    ui.label_text(
                        "Total blades",
                        renderer
                            .grass_blades_total
                            .load(Ordering::Relaxed)
                            .to_string(),
                    );
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
                    ui.slider("Width", 0., 0.5, &mut grass.width);
                    ui.slider("Sway frequency", 0.25, 4., &mut grass.sway_frequency);
                    ui.slider("Sway amplitude", 0., 0.5, &mut grass.sway_amplitude);
                    changed |= ui.slider("Chunk count", 1, 4095, &mut grass.chunk_count);
                    grass.chunk_count += grass.chunk_count % 2 - 1;
                    ui.slider(
                        "Chunk load distance",
                        0.,
                        planet.radius * 2.,
                        &mut grass.chunk_load_distance,
                    );
                    ui.slider(
                        "Chunk unload distance",
                        0.,
                        planet.radius * 2.,
                        &mut grass.chunk_unload_distance,
                    );
                    events.grass_changed = changed;
                }
                ui.slider_config("Depth near plane", 0.001, 10.)
                    .flags(SliderFlags::LOGARITHMIC)
                    .build(&mut renderer_settings.depth_near);
                ui.slider_config("Depth far plane", 10., 10000.)
                    .flags(SliderFlags::LOGARITHMIC)
                    .build(&mut renderer_settings.depth_far);
                ui.checkbox("Ray-traced shadows", &mut frag_settings.use_ray_tracing);
                if ui.collapsing_header("Atmosphere", TreeNodeFlags::empty()) {
                    ui.checkbox("Enable", &mut postprocessing.atmosphere);
                    ui.slider(
                        "Scatter points",
                        1,
                        32,
                        &mut postprocessing.atmosphere_scatter_point_count,
                    );
                    ui.slider(
                        "Optical depth points",
                        1,
                        4,
                        &mut postprocessing.atmosphere_optical_depth_point_count,
                    );
                    ui.slider_config("Density falloff", 0.001, 100.)
                        .flags(SliderFlags::LOGARITHMIC)
                        .build(&mut postprocessing.atmosphere_density_falloff);
                    ui.slider("Radius", 0., 4000., &mut postprocessing.atmosphere_radius);
                    ui.slider_config("Scatter coefficient", 0.001, 100.)
                        .flags(SliderFlags::LOGARITHMIC)
                        .build(&mut postprocessing.atmosphere_scatter_coefficient);
                    ui.slider(
                        "Planet radius",
                        0.,
                        2000.,
                        &mut postprocessing.planet_radius,
                    );
                }
                if ui.collapsing_header("Postprocessing", TreeNodeFlags::empty()) {
                    build_postprocessing(ui, postprocessing);
                }
            });
        events
    }
}

fn build_postprocessing(ui: &Ui, postprocessing: &mut Postprocessing) {
    ui.slider_config("Exposure", 0.001, 100.)
        .flags(SliderFlags::LOGARITHMIC)
        .build(&mut postprocessing.exposure);
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
