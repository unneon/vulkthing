use crate::camera::Camera;
use crate::input::InputState;
use crate::model::Model;
use crate::physics::Physics;
use crate::renderer::uniform::Light;
use nalgebra::{Matrix4, UnitQuaternion, Vector3};
use rapier3d::dynamics::RigidBodyHandle;

pub struct World {
    camera: Camera,
    entities: [Entity; 2],
    physics: Physics,
}

pub struct Entity {
    transform: Transform,
    emit: Vector3<f32>,
    gpu_object: usize,
}

enum Transform {
    Static {
        translation: Vector3<f32>,
        rotation: UnitQuaternion<f32>,
    },
    Dynamic {
        rigid_body: RigidBodyHandle,
    },
}

impl World {
    pub fn new(planet_model: &Model) -> World {
        let camera = Camera {
            position: Vector3::new(-350., 0., 0.),
            velocity: Vector3::new(0., 0., 0.),
            rotation: UnitQuaternion::from_euler_angles(0., 0., 0.),
            time: 0.,
        };
        let mut physics = Physics::new();
        let planet_collider = physics.trimesh(planet_model);
        let planet = Entity {
            transform: Transform::Static {
                translation: Vector3::new(0., 0., 0.),
                rotation: UnitQuaternion::identity(),
            },
            emit: Vector3::new(0., 0., 0.),
            gpu_object: 0,
        };
        physics.insert_static(planet_collider.friction(0.));
        let sun_collider = physics.cube(2.).restitution(0.7);
        let sun = Entity {
            transform: Transform::Dynamic {
                rigid_body: physics.insert(Vector3::new(0.1, 0.1, 200.), sun_collider),
            },
            emit: Vector3::new(1., 1., 1.),
            gpu_object: 1,
        };
        let entities = [planet, sun];
        World {
            camera,
            entities,
            physics,
        }
    }

    pub fn update(&mut self, delta_time: f32, input_state: &InputState, demo: bool) {
        self.physics.step(delta_time);
        self.camera.apply_input(input_state, delta_time, demo);
    }

    pub fn light(&self) -> Light {
        Light {
            position: self.entities[1].translation(&self),
            color: self.entities[1].emit,
            ambient_strength: 0.05,
            diffuse_strength: 4.,
        }
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        self.camera.view_matrix()
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }
}

impl Entity {
    pub fn translation(&self, world: &World) -> Vector3<f32> {
        match self.transform {
            Transform::Static { translation, .. } => translation,
            Transform::Dynamic { rigid_body } => world.physics.get_translation(rigid_body),
        }
    }

    pub fn rotation(&self, world: &World) -> UnitQuaternion<f32> {
        match self.transform {
            Transform::Static { rotation, .. } => rotation,
            Transform::Dynamic { rigid_body } => world.physics.get_rotation(rigid_body),
        }
    }

    pub fn model_matrix(&self, world: &World) -> Matrix4<f32> {
        Matrix4::new_translation(&self.translation(world)) * self.rotation(world).to_homogeneous()
    }

    pub fn emit(&self) -> Vector3<f32> {
        self.emit
    }

    pub fn gpu_object(&self) -> usize {
        self.gpu_object
    }
}
