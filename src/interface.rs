use crate::renderer::codegen::{PASS_COUNT, PASS_NAMES};
use crate::renderer::{PostprocessSettings, RendererSettings};
use crate::voxel::VoxelsConfig;
use crate::world::World;
use ash::vk;
use imgui::{Condition, Context, Drag, SliderFlags, TreeNodeFlags, Ui};
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
    pub rebuild_voxels: bool,
}

impl Interface {
    pub fn build(
        &mut self,
        world: &mut World,
        renderer: &mut RendererSettings,
        voxels: &mut VoxelsConfig,
        pass_times: Option<&[Duration; PASS_COUNT]>,
    ) -> InterfaceEvents {
        let ui = self.ctx.frame();
        let mut events = InterfaceEvents {
            planet_changed: false,
            grass_changed: false,
            rebuild_swapchain: false,
            rebuild_pipelines: false,
            rebuild_voxels: false,
        };
        ui.window("Debugging")
            .size([0., 0.], Condition::Always)
            .build(|| {
                if ui.collapsing_header("Voxels", TreeNodeFlags::empty()) {
                    let mut changed = false;
                    let mut chunk_size_log2 = 63 - voxels.chunk_size.leading_zeros();
                    changed |= ui.slider("Chunk size", 0, 10, &mut chunk_size_log2);
                    voxels.chunk_size = 1 << chunk_size_log2;
                    changed |= ui.slider(
                        "Heightmap amplitude",
                        0.,
                        256.,
                        &mut voxels.heightmap_amplitude,
                    );
                    changed |= ui
                        .slider_config("Heightmap frequency", 0.001, 100.)
                        .flags(SliderFlags::LOGARITHMIC)
                        .build(&mut voxels.heightmap_frequency);
                    changed |= ui.slider("Heightmap bias", -1., 1., &mut voxels.heightmap_bias);
                    changed |= ui.slider(
                        "Render distance (horizontal)",
                        1,
                        1024,
                        &mut voxels.render_distance_horizontal,
                    );
                    changed |= ui.slider(
                        "Render distance (vertical)",
                        1,
                        1024,
                        &mut voxels.render_distance_vertical,
                    );
                    changed |= enum_combo(ui, "Meshing algorithm", &mut voxels.meshing_algorithm);
                    events.rebuild_voxels = changed;
                }
                if ui.collapsing_header("Sun", TreeNodeFlags::empty()) {
                    Drag::new("Time of day")
                        .speed(0.01)
                        .build(ui, &mut world.time_of_day);
                    world.time_of_day = world.time_of_day.rem_euclid(2. * PI);
                    ui.slider_config("Intensity", 0.001, 10000000.)
                        .flags(SliderFlags::LOGARITHMIC)
                        .build(&mut world.sun_intensity);
                    ui.slider("Orbit radius", 0., 4000., &mut world.sun_radius);
                    ui.checkbox("Pause movement", &mut world.sun_pause);
                    ui.slider_config("Speed", 0.001, 10.)
                        .flags(SliderFlags::LOGARITHMIC)
                        .build(&mut world.sun_speed);
                }
                if ui.collapsing_header("Renderer", TreeNodeFlags::empty()) {
                    ui.slider_config("Depth near plane", 0.001, 16.)
                        .flags(SliderFlags::LOGARITHMIC)
                        .build(&mut renderer.depth_near);
                    ui.slider_config("Depth far plane", 16., 1048576.)
                        .flags(SliderFlags::LOGARITHMIC)
                        .build(&mut renderer.depth_far);
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
                    ui.slider_config("Planet radius", 10., 4000.)
                        .flags(SliderFlags::LOGARITHMIC)
                        .build(&mut world.atmosphere.planet_radius);
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

fn build_postprocess(ui: &Ui, postprocess: &mut PostprocessSettings) {
    ui.slider_config("Exposure", 0.001, 100.)
        .flags(SliderFlags::LOGARITHMIC)
        .build(&mut postprocess.exposure);
    enum_combo(ui, "Tonemapper", &mut postprocess.tonemapper);
    Drag::new("Gamma")
        .range(0., f32::INFINITY)
        .speed(0.01)
        .build(ui, &mut postprocess.gamma);
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
