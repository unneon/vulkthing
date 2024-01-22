use crate::config::DEFAULT_PLANET_SCALE;
use crate::grass::GrassParameters;
use crate::planet::Planet;
use crate::renderer::codegen::{PASS_COUNT, PASS_NAMES};
use crate::renderer::{PostprocessSettings, RendererSettings};
use crate::world::World;
use ash::vk;
use imgui::{Condition, Context, Drag, SliderFlags, TreeNodeFlags, Ui};
use nalgebra::Vector3;
use std::borrow::Cow;
use std::f32::consts::PI;
use std::time::Duration;

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
    pub rebuild_swapchain: bool,
    pub rebuild_pipelines: bool,
}

impl Interface {
    pub fn build(
        &mut self,
        world: &mut World,
        planet: &mut Planet,
        grass: &mut GrassParameters,
        renderer: &mut RendererSettings,
        pass_times: Option<&[Duration; PASS_COUNT]>,
    ) -> InterfaceEvents {
        let ui = self.ctx.frame();
        let mut events = InterfaceEvents {
            planet_changed: false,
            grass_changed: false,
            rebuild_swapchain: false,
            rebuild_pipelines: false,
        };
        ui.window("Debugging")
            .size([0., 0.], Condition::Always)
            .build(|| {
                if ui.collapsing_header("Planet", TreeNodeFlags::empty()) {
                    let mut changed = false;
                    changed |= ui.slider("Resolution", 1, 800, &mut planet.resolution);
                    changed |= ui.slider("Noise magnitude", 0., 100., &mut planet.noise_magnitude);
                    changed |= ui.slider("Noise scale", 0., 64., &mut planet.noise_scale);
                    entity(ui, world, world.planet_entity());
                    events.planet_changed = changed;
                }
                if ui.collapsing_header("Sun", TreeNodeFlags::empty()) {
                    Drag::new("Time of day")
                        .speed(0.01)
                        .build(ui, &mut world.time_of_day);
                    world.time_of_day = world.time_of_day.rem_euclid(2. * PI);
                    ui.slider_config("Intensity", 0.001, 10000000.)
                        .flags(SliderFlags::LOGARITHMIC)
                        .build(&mut world.sun_intensity);
                    ui.slider(
                        "Orbit radius",
                        0.,
                        4. * DEFAULT_PLANET_SCALE,
                        &mut world.sun_radius,
                    );
                    ui.slider_config("Scale", 1., 500.)
                        .flags(SliderFlags::LOGARITHMIC)
                        .build(&mut world.sun_scale);
                    ui.checkbox("Pause movement", &mut world.sun_pause);
                    ui.slider_config("Speed", 0.001, 10.)
                        .flags(SliderFlags::LOGARITHMIC)
                        .build(&mut world.sun_speed);
                }
                if ui.collapsing_header("Grass", TreeNodeFlags::empty()) {
                    events.grass_changed |= ui.checkbox("Enabled", &mut grass.enabled);
                    events.grass_changed |= ui.slider(
                        "Blades per planet triangle",
                        1,
                        256,
                        &mut grass.blades_per_triangle,
                    );
                    events.grass_changed |=
                        ui.slider("Height average", 0.01, 3., &mut grass.height_average);
                    events.grass_changed |= ui.slider(
                        "Height max variance",
                        0.,
                        1.,
                        &mut grass.height_max_variance,
                    );
                    events.grass_changed |= ui.slider(
                        "Height noise frequency",
                        0.01,
                        1.,
                        &mut grass.height_noise_frequency,
                    );
                    events.grass_changed |= ui.slider("Width", 0., 0.5, &mut grass.width);
                    events.grass_changed |=
                        ui.slider("Sway frequency", 0.25, 4., &mut grass.sway_frequency);
                    events.grass_changed |=
                        ui.slider("Sway amplitude", 0., 0.5, &mut grass.sway_amplitude);
                    events.grass_changed |=
                        ui.slider("Chunk count", 1, 4095, &mut grass.chunk_count);
                    grass.chunk_count += grass.chunk_count % 2 - 1;
                    events.grass_changed |= ui.slider(
                        "Chunk load distance",
                        0.,
                        2000.,
                        &mut grass.chunk_load_distance,
                    );
                    events.grass_changed |= ui.slider(
                        "Chunk unload distance",
                        0.,
                        2000.,
                        &mut grass.chunk_unload_distance,
                    );
                    events.grass_changed |= events.grass_changed;
                }
                if ui.collapsing_header("Renderer", TreeNodeFlags::empty()) {
                    ui.slider_config("Depth near plane", 0.001, 16.)
                        .flags(SliderFlags::LOGARITHMIC)
                        .build(&mut renderer.depth_near);
                    ui.slider_config("Depth far plane", 16., 1048576.)
                        .flags(SliderFlags::LOGARITHMIC)
                        .build(&mut renderer.depth_far);
                    ui.checkbox("Ray-traced shadows", &mut renderer.enable_ray_tracing);
                }
                if ui.collapsing_header("Atmosphere", TreeNodeFlags::empty()) {
                    ui.checkbox("Enable", &mut renderer.enable_atmosphere);
                    ui.slider(
                        "In scattering samples",
                        1,
                        32,
                        &mut renderer.atmosphere_in_scattering_samples,
                    );
                    ui.slider(
                        "Optical depth samples",
                        1,
                        32,
                        &mut renderer.atmosphere_optical_depth_samples,
                    );
                    ui.slider_config("Density falloff", 0.001, 100.)
                        .flags(SliderFlags::LOGARITHMIC)
                        .build(&mut world.atmosphere.density_falloff);
                    ui.slider("Scale", 1., 3., &mut world.atmosphere.scale);
                    Drag::new("Wavelengths")
                        .build_array(ui, renderer.atmosphere_wavelengths.as_mut_slice());
                    ui.slider_config("Scattering strength", 0.001, 100.)
                        .flags(SliderFlags::LOGARITHMIC)
                        .build(&mut world.atmosphere.scattering_strength);
                    ui.slider(
                        "Henyey-Greenstein g",
                        -1.,
                        0.,
                        &mut world.atmosphere.henyey_greenstein_g,
                    );
                }
                if ui.collapsing_header("Post-processing", TreeNodeFlags::empty()) {
                    build_postprocess(ui, &mut renderer.postprocess);
                }
                if ui.collapsing_header("Performance", TreeNodeFlags::empty()) {
                    if let Some(pass_times) = pass_times {
                        let mut total_time = Duration::ZERO;
                        for (name, time) in PASS_NAMES.iter().zip(pass_times.iter()) {
                            ui.label_text(
                                format!("{name} pass"),
                                format!("{:.2}ms", time.as_secs_f64() * 1000.),
                            );
                            total_time += *time;
                        }
                        ui.label_text(
                            "Total",
                            format!("{:.2}ms", total_time.as_secs_f64() * 1000.),
                        );
                    }
                }
            });
        events
    }
}

impl EnumInterface for vk::SampleCountFlags {
    const VALUES: &'static [Self] = &[
        vk::SampleCountFlags::TYPE_2,
        vk::SampleCountFlags::TYPE_4,
        vk::SampleCountFlags::TYPE_8,
    ];

    fn label(&self) -> Cow<str> {
        if self.contains(vk::SampleCountFlags::TYPE_8) {
            Cow::Borrowed("8x")
        } else if self.contains(vk::SampleCountFlags::TYPE_4) {
            Cow::Borrowed("4x")
        } else if self.contains(vk::SampleCountFlags::TYPE_2) {
            Cow::Borrowed("2x")
        } else {
            Cow::Borrowed("1x")
        }
    }
}

fn entity(ui: &Ui, world: &mut World, entity: usize) {
    Drag::new("Position").build_array(
        ui,
        world.entities[entity].transform.translation.as_mut_slice(),
    );
}

fn build_postprocess(ui: &Ui, postprocess: &mut PostprocessSettings) {
    ui.slider_config("Exposure", 0.001, 100.)
        .flags(SliderFlags::LOGARITHMIC)
        .build(&mut postprocess.exposure);
    ui.slider_config("Bloom exponent coefficient", 0.001, 100.)
        .flags(SliderFlags::LOGARITHMIC)
        .build(&mut postprocess.bloom_exponent_coefficient);
    ui.slider("Bloom radius", 0, 64, &mut postprocess.bloom_radius);
    ui.slider_config("Bloom strength", 0.001, 10.)
        .flags(SliderFlags::LOGARITHMIC)
        .build(&mut postprocess.bloom_strength);
    ui.slider_config("Bloom threshold", 0.001, 12.)
        .flags(SliderFlags::LOGARITHMIC)
        .build(&mut postprocess.bloom_threshold);
    ui.slider("Temperature", -1.67, 1.67, &mut postprocess.temperature);
    ui.slider("Tint", -1.67, 1.67, &mut postprocess.tint);
    Drag::new("Contrast")
        .range(0., f32::INFINITY)
        .speed(0.01)
        .build(ui, &mut postprocess.contrast);
    Drag::new("Brightness")
        .range(0., f32::INFINITY)
        .speed(0.01)
        .build(ui, &mut postprocess.brightness);
    enum_color(ui, "Color filter", &mut postprocess.color_filter);
    Drag::new("Saturation")
        .range(0., f32::INFINITY)
        .speed(0.01)
        .build(ui, &mut postprocess.saturation);
    enum_combo(ui, "Tonemapper", &mut postprocess.tonemapper);
    Drag::new("Gamma")
        .range(0., f32::INFINITY)
        .speed(0.01)
        .build(ui, &mut postprocess.gamma);
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
