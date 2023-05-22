use crate::camera::Camera;
use crate::input::InputState;
use crate::interface::Editable;
use imgui::Ui;
use nalgebra::Vector3;
use std::f32::consts::PI;

pub struct World {
    pub camera: Camera,
    pub light: Light,
    pub entities: [Entity; 2],
}

pub struct Entity {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub emit: Vector3<f32>,
    pub gpu_object: usize,
}

pub struct Light {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
    pub ambient_strength: f32,
    pub diffuse_strength: f32,
    pub movement: bool,
    pub speed: f32,
    radius: f32,
    pub argument: f32,
    pub use_ray_tracing: bool,
}

const SUN_SCALE: f32 = 5.;
const SUN_Z: f32 = 200.;

impl World {
    pub fn new() -> World {
        let sun_color = Vector3::new(1., 1., 1.);
        let camera = Camera {
            position: Vector3::new(-250., 0., 0.),
            velocity: Vector3::new(0., 0., 0.),
            yaw: 0.,
            pitch: 0.,
        };
        let light_radius = 500.;
        let light = Light {
            position: Vector3::new(-light_radius, 0., SUN_Z),
            color: sun_color,
            ambient_strength: 0.05,
            diffuse_strength: 4.,
            movement: true,
            speed: 0.2,
            radius: light_radius,
            argument: 0.,
            use_ray_tracing: true,
        };
        let planet = Entity {
            position: Vector3::new(0., 0., 0.),
            scale: Vector3::new(1., 1., 1.),
            emit: Vector3::new(0., 0., 0.),
            gpu_object: 0,
        };
        let sun = Entity {
            position: light.position,
            scale: Vector3::new(SUN_SCALE, SUN_SCALE, SUN_SCALE),
            emit: sun_color,
            gpu_object: 1,
        };
        let entities = [planet, sun];
        World {
            camera,
            light,
            entities,
        }
    }

    pub fn update(&mut self, delta_time: f32, input_state: &InputState) {
        self.camera.apply_input(input_state, delta_time);
        self.light.update(delta_time);
        self.entities[1].position = self.light.position;
    }
}

impl Light {
    fn update(&mut self, delta_time: f32) {
        if self.movement {
            self.argument = (self.argument + self.speed * delta_time).rem_euclid(2. * PI);
            self.position.x = -self.radius * self.argument.cos();
            self.position.y = -self.radius * self.argument.sin();
        }
    }
}

impl Editable for World {
    fn name(&self) -> &str {
        "World"
    }

    fn widget(&mut self, ui: &Ui) -> bool {
        let mut changed = false;
        changed |= ui.checkbox("Light movement", &mut self.light.movement);
        changed |= ui.slider("Light speed", 0., 1., &mut self.light.speed);
        changed |= ui.slider("Light radius", 0., 1000., &mut self.light.radius);
        changed |= ui.slider("Light argument", 0., 2. * PI, &mut self.light.argument);
        changed |= ui.slider(
            "Light ambient strength",
            0.,
            0.3,
            &mut self.light.ambient_strength,
        );
        changed |= ui.slider(
            "Light diffuse strength",
            0.,
            32.,
            &mut self.light.diffuse_strength,
        );
        changed |= ui.checkbox("Use ray tracing", &mut self.light.use_ray_tracing);
        changed
    }
}
