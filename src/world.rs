use crate::camera::Camera;
use crate::input::InputState;
use crate::interface::Editable;
use crate::model::Model;
use crate::physics::Physics;
use imgui::Ui;
use nalgebra::{UnitQuaternion, Vector3};
use rapier3d::dynamics::RigidBodyHandle;

pub struct World {
    pub camera: Camera,
    pub light: Light,
    pub entities: [Entity; 2],
    physics: Physics,
}

pub struct Entity {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub emit: Vector3<f32>,
    pub gpu_object: usize,
    pub rigid_body: Option<RigidBodyHandle>,
}

pub struct Light {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
    pub ambient_strength: f32,
    pub diffuse_strength: f32,
    pub use_ray_tracing: bool,
}

impl World {
    pub fn new(planet_model: &Model) -> World {
        let camera = Camera {
            position: Vector3::new(-350., 0., 0.),
            velocity: Vector3::new(0., 0., 0.),
            rotation: UnitQuaternion::from_euler_angles(0., 0., 0.),
            time: 0.,
        };
        let light = Light {
            position: Vector3::new(0.1, 0.1, 200.),
            color: Vector3::new(1., 1., 1.),
            ambient_strength: 0.05,
            diffuse_strength: 4.,
            use_ray_tracing: true,
        };
        let mut physics = Physics::new();
        let planet_collider = physics.trimesh(planet_model);
        let planet = Entity {
            position: Vector3::new(0., 0., 0.),
            rotation: UnitQuaternion::identity(),
            emit: Vector3::new(0., 0., 0.),
            gpu_object: 0,
            rigid_body: None,
        };
        physics.insert_static(planet_collider);
        let sun_collider = physics.cube(2.);
        let sun = Entity {
            position: light.position,
            rotation: UnitQuaternion::identity(),
            emit: light.color,
            gpu_object: 1,
            rigid_body: Some(physics.insert(light.position, sun_collider)),
        };
        let entities = [planet, sun];
        World {
            camera,
            light,
            entities,
            physics,
        }
    }

    pub fn update(&mut self, delta_time: f32, input_state: &InputState, demo: bool) {
        self.physics.step(delta_time);
        self.camera.apply_input(input_state, delta_time, demo);
        for entity in &mut self.entities {
            if let Some(rigid_body) = entity.rigid_body {
                entity.position = self.physics.get_translation(rigid_body);
                entity.rotation = self.physics.get_rotation(rigid_body);
            }
        }
        self.light.position = self.entities[1].position;
    }
}

impl Editable for World {
    fn name(&self) -> &str {
        "World"
    }

    fn widget(&mut self, ui: &Ui) -> bool {
        let mut changed = false;
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
