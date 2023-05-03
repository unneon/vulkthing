use crate::camera::Camera;
use crate::input::InputState;
use nalgebra::Vector3;

pub struct World {
    pub camera: Camera,
    pub light: Light,
    pub light_pause: bool,
    pub light_time: f32,
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
}

const SUN_RADIUS: f32 = 500.;
const SUN_SCALE: f32 = 5.;
const SUN_Z: f32 = 200.;
const SUN_SPEED: f32 = 0.1;

impl World {
    pub fn new() -> World {
        let sun_color = Vector3::new(1., 1., 1.);
        let camera = Camera {
            position: Vector3::new(-250., 0., 0.),
            velocity: Vector3::new(0., 0., 0.),
            yaw: 0.,
            pitch: 0.,
        };
        let light = Light {
            position: Vector3::new(-SUN_RADIUS, 0., SUN_Z),
            color: sun_color,
            ambient_strength: 0.05,
        };
        let time = 0.;
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
            light_pause: false,
            light_time: time,
            entities,
        }
    }

    pub fn update(&mut self, delta_time: f32, input_state: &InputState) {
        self.camera.apply_input(input_state, delta_time);
        if !self.light_pause {
            self.light_time += delta_time;
            self.light.position.x = -SUN_RADIUS * (SUN_SPEED * self.light_time).cos();
            self.light.position.y = -SUN_RADIUS * (SUN_SPEED * self.light_time).sin();
            self.entities[1].position = self.light.position;
        }
    }
}
