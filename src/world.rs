use crate::camera::Camera;
use crate::input::InputState;
use nalgebra_glm::{vec3, Vec3};

pub struct World {
    time: f32,
    pub camera: Camera,
    pub light: Light,
    pub entities: [Entity; 2],
}

pub struct Entity {
    pub position: Vec3,
    pub scale: Vec3,
    pub emit: Vec3,
    pub gpu_object: usize,
}

pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
    pub ambient_strength: f32,
}

impl World {
    pub fn new() -> World {
        let sun_color = vec3(1., 0.12, 68.);
        let camera = Camera {
            position: vec3(-10., 0., 0.),
            velocity: vec3(0., 0., 0.),
            yaw: 0.,
            pitch: 0.,
        };
        let light = Light {
            position: vec3(-8., 0., 0.),
            color: sun_color,
            ambient_strength: 0.004,
        };
        let time = 0.;
        let building = Entity {
            position: vec3(0., 0., 0.),
            scale: vec3(1., 1., 1.),
            emit: vec3(0., 0., 0.),
            gpu_object: 0,
        };
        let sun = Entity {
            position: light.position,
            scale: vec3(0.2, 0.2, 0.2),
            emit: sun_color,
            gpu_object: 1,
        };
        let entities = [building, sun];
        World {
            camera,
            light,
            time,
            entities,
        }
    }

    pub fn update(&mut self, delta_time: f32, input_state: &InputState) {
        self.time += delta_time;
        self.camera.apply_input(input_state, delta_time);
        // self.light.position.x = -4. * self.time.cos();
        // self.light.position.y = -4. * self.time.sin();
        self.entities[1].position = self.light.position;
    }
}
