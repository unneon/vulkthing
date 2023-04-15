use crate::camera::Camera;
use crate::input::InputState;
use nalgebra::Vector3;

pub struct World {
    time: f32,
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
}

impl World {
    pub fn new() -> World {
        let sun_color = Vector3::new(1., 1., 1.);
        let camera = Camera {
            position: Vector3::new(-10., 0., 0.),
            velocity: Vector3::new(0., 0., 0.),
            yaw: 0.,
            pitch: 0.,
        };
        let light = Light {
            position: Vector3::new(-4., 0., 2.),
            color: sun_color,
            ambient_strength: 0.05,
        };
        let time = 0.;
        let building = Entity {
            position: Vector3::new(0., 0., 0.),
            scale: Vector3::new(1., 1., 1.),
            emit: Vector3::new(0., 0., 0.),
            gpu_object: 0,
        };
        let sun = Entity {
            position: light.position,
            scale: Vector3::new(0.2, 0.2, 0.2),
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
        self.light.position.x = -4. * self.time.cos();
        self.light.position.y = -4. * self.time.sin();
        self.entities[1].position = self.light.position;
    }
}
