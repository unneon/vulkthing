use crate::camera::Camera;
use crate::config::{DEFAULT_CAMERA, DEFAULT_PLANET_SCALE, DEFAULT_SUN_POSITION};
use crate::input::InputState;
use crate::model::Model;
use crate::physics::Physics;
use crate::renderer::uniform::Light;
use nalgebra::{Matrix4, UnitQuaternion, Vector3};
use rapier3d::dynamics::RigidBodyHandle;
use rapier3d::prelude::*;
use std::f32::consts::FRAC_PI_2;

pub struct World {
    pub camera: Box<dyn Camera>,
    camera_rigid_body_handle: RigidBodyHandle,
    pub entities: [Entity; 2],
    physics: Physics,
    pub time: f32,
    pub time_of_day: f32,
    pub ambient_strength: f32,
    pub diffuse_strength: f32,
    pub sun_radius: f32,
}

pub struct Entity {
    transform: Transform,
    diffuse: Vector3<f32>,
    emit: Vector3<f32>,
    gpu_object: usize,
}

enum Transform {
    Static {
        translation: Vector3<f32>,
        rotation: UnitQuaternion<f32>,
        scale: Vector3<f32>,
    },
    #[allow(dead_code)]
    Dynamic {
        rigid_body: RigidBodyHandle,
        scale: Vector3<f32>,
    },
}

const AVERAGE_MALE_HEIGHT: f32 = 1.74;
const AVERAGE_MALE_EYE_HEIGHT: f32 = 1.63;
const AVERAGE_MALE_SHOULDER_WIDTH: f32 = 0.465;

impl World {
    pub fn new(planet_model: &Model) -> World {
        let camera = Box::new(DEFAULT_CAMERA);
        let mut physics = Physics::new();
        let camera_rigid_body = RigidBodyBuilder::dynamic()
            .translation(camera.position)
            .lock_rotations();
        let camera_rigid_body_handle = physics.rigid_body_set.insert(camera_rigid_body);
        let camera_collider =
            ColliderBuilder::capsule_z(AVERAGE_MALE_HEIGHT / 2., AVERAGE_MALE_SHOULDER_WIDTH / 2.)
                .friction(1.)
                .friction_combine_rule(CoefficientCombineRule::Max)
                .build();
        physics.collider_set.insert_with_parent(
            camera_collider,
            camera_rigid_body_handle,
            &mut physics.rigid_body_set,
        );
        let planet_scale = Vector3::from_element(DEFAULT_PLANET_SCALE);
        let planet_collider = physics.trimesh(planet_model, &planet_scale).friction(0.);
        let planet = Entity {
            transform: Transform::Static {
                translation: Vector3::zeros(),
                rotation: UnitQuaternion::identity(),
                scale: planet_scale,
            },
            diffuse: Vector3::new(0.2, 0.8, 0.03).scale(0.7),
            emit: Vector3::zeros(),
            gpu_object: 0,
        };
        physics.insert_static(planet_collider);
        let sun = Entity {
            transform: Transform::Static {
                translation: DEFAULT_SUN_POSITION,
                rotation: UnitQuaternion::identity(),
                scale: Vector3::from_element(50.),
            },
            diffuse: Vector3::zeros(),
            emit: Vector3::from_element(1.),
            gpu_object: 1,
        };
        let entities = [planet, sun];
        World {
            camera,
            camera_rigid_body_handle,
            entities,
            physics,
            time: 0.,
            time_of_day: FRAC_PI_2,
            ambient_strength: 0.01,
            diffuse_strength: 1.,
            sun_radius: 2000.,
        }
    }

    pub fn update(&mut self, delta_time: f32, input_state: &InputState) {
        self.camera.apply_input(input_state, delta_time);
        self.update_player(input_state);
        self.physics.step(delta_time);
        self.camera.set_position(
            self.physics.get_translation(self.camera_rigid_body_handle)
                + Vector3::new(0., 0., AVERAGE_MALE_EYE_HEIGHT / 2.),
        );
        self.update_sun();
        self.time += delta_time;
    }

    pub fn update_player(&mut self, input_state: &InputState) {
        let rigid_body = self
            .physics
            .rigid_body_set
            .get_mut(self.camera_rigid_body_handle)
            .unwrap();
        rigid_body.reset_forces(true);
        let can_accelerate =
            rigid_body.linvel().dot(&self.camera.walk_direction()) <= 16. * 1.42 * 1.42;
        if can_accelerate {
            rigid_body.add_force(16. * self.camera.walk_direction(), true);
        }
        if input_state.movement_jumps() > 0 {
            rigid_body.apply_impulse(Vector3::new(0., 0., 4.), true);
        }
    }

    pub fn update_sun(&mut self) {
        let Transform::Static { translation, .. } = &mut self.entities[1].transform else { unreachable!() };
        translation.y = self.sun_radius * self.time_of_day.cos();
        translation.z = self.sun_radius * self.time_of_day.sin();
    }

    pub fn light(&self) -> Light {
        Light {
            position: self.sun().translation(self),
            color: self.sun().emit,
            ambient_strength: self.ambient_strength,
            diffuse_strength: self.diffuse_strength,
        }
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        self.camera.view_matrix()
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    pub fn planet(&self) -> &Entity {
        &self.entities[0]
    }

    pub fn planet_entity(&self) -> usize {
        0
    }

    pub fn sun(&self) -> &Entity {
        &self.entities[1]
    }
}

impl Entity {
    pub fn translation(&self, world: &World) -> Vector3<f32> {
        match self.transform {
            Transform::Static { translation, .. } => translation,
            Transform::Dynamic { rigid_body, .. } => world.physics.get_translation(rigid_body),
        }
    }

    pub fn static_translation_mut(&mut self) -> &mut Vector3<f32> {
        match &mut self.transform {
            Transform::Static { translation, .. } => translation,
            _ => unreachable!(),
        }
    }

    pub fn rotation(&self, world: &World) -> UnitQuaternion<f32> {
        match self.transform {
            Transform::Static { rotation, .. } => rotation,
            Transform::Dynamic { rigid_body, .. } => world.physics.get_rotation(rigid_body),
        }
    }

    pub fn scale(&self) -> Vector3<f32> {
        match self.transform {
            Transform::Static { scale, .. } => scale,
            Transform::Dynamic { scale, .. } => scale,
        }
    }

    pub fn model_matrix(&self, world: &World) -> Matrix4<f32> {
        Matrix4::new_translation(&self.translation(world)).prepend_nonuniform_scaling(&self.scale())
            * self.rotation(world).to_homogeneous()
    }

    pub fn diffuse(&self) -> Vector3<f32> {
        self.diffuse
    }

    pub fn emit(&self) -> Vector3<f32> {
        self.emit
    }

    pub fn gpu_object(&self) -> usize {
        self.gpu_object
    }
}
