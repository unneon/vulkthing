mod benchmark;

use crate::camera::Camera;
use crate::config::{
    DEFAULT_CAMERA, DEFAULT_PLANET_POSITION, DEFAULT_PLANET_SCALE, DEFAULT_STAR_COUNT,
    DEFAULT_STAR_MAX_SCALE, DEFAULT_STAR_MIN_SCALE, DEFAULT_STAR_RADIUS, DEFAULT_SUN_POSITION,
    DEFAULT_SUN_RADIUS, DEFAULT_SUN_SPEED,
};
use crate::input::InputState;
use crate::mesh::MeshData;
use crate::physics::Physics;
use crate::renderer::uniform::Light;
use crate::util::{RandomDirection, RandomRotation};
use nalgebra::{Matrix4, UnitQuaternion, Vector3};
use rand::Rng;
use rapier3d::dynamics::RigidBodyHandle;
use rapier3d::prelude::*;

pub struct World {
    pub camera: Box<dyn Camera>,
    camera_rigid_body_handle: RigidBodyHandle,
    pub entities: Vec<Entity>,
    pub stars: Vec<Star>,
    physics: Physics,
    pub time: f32,
    pub time_of_day: f32,
    pub sun_intensity: f32,
    pub sun_pause: bool,
    pub sun_radius: f32,
    pub sun_speed: f32,
    pub atmosphere: Atmosphere,
}

pub struct Entity {
    pub transform: Transform,
    albedo: Vector3<f32>,
    metallic: f32,
    roughness: f32,
    ao: f32,
    mesh_id: usize,
}

pub struct Transform {
    pub translation: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}

pub struct Star {
    pub transform: Transform,
    pub emit: Vector3<f32>,
}

pub struct Atmosphere {
    pub density_falloff: f32,
    pub scale: f32,
    pub scattering_strength: f32,
    pub henyey_greenstein_g: f32,
}

const AVERAGE_MALE_HEIGHT: f32 = 1.74;
const AVERAGE_MALE_EYE_HEIGHT: f32 = 1.63;
const AVERAGE_MALE_SHOULDER_WIDTH: f32 = 0.465;

impl World {
    pub fn new(planet_model: &MeshData) -> World {
        let camera = Box::new(DEFAULT_CAMERA);
        let mut physics = Physics::new();
        let camera_rigid_body = RigidBodyBuilder::dynamic()
            .ccd_enabled(true)
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
            transform: Transform {
                translation: DEFAULT_PLANET_POSITION,
                rotation: UnitQuaternion::identity(),
                scale: planet_scale,
            },
            albedo: Vector3::new(0.2, 0.8, 0.03).scale(0.7),
            metallic: 0.,
            roughness: 1.,
            ao: 0.,
            mesh_id: 0,
        };
        physics.insert_static(planet_collider);
        let sun = Entity {
            transform: Transform {
                translation: DEFAULT_SUN_POSITION,
                rotation: UnitQuaternion::identity(),
                scale: Vector3::from_element(50.),
            },
            albedo: Vector3::zeros(),
            metallic: 0.,
            roughness: 1.,
            ao: 0.,
            mesh_id: 4,
        };
        let entities = vec![planet, sun];
        let mut stars = Vec::new();
        let mut rng = rand::thread_rng();
        for _ in 0..DEFAULT_STAR_COUNT {
            stars.push(Star {
                transform: Transform {
                    translation: DEFAULT_STAR_RADIUS * rng.sample(RandomDirection),
                    rotation: rng.sample(RandomRotation),
                    scale: Vector3::from_element(
                        rng.gen_range(DEFAULT_STAR_MIN_SCALE..DEFAULT_STAR_MAX_SCALE),
                    ),
                },
                emit: Vector3::from_element(10.),
            });
        }
        World {
            camera,
            camera_rigid_body_handle,
            entities,
            stars,
            physics,
            time: 0.,
            time_of_day: 0.,
            sun_intensity: 4000000.,
            sun_pause: false,
            sun_radius: DEFAULT_SUN_RADIUS,
            sun_speed: DEFAULT_SUN_SPEED,
            atmosphere: Atmosphere {
                density_falloff: 6.,
                scale: 1.3,
                scattering_strength: 0.01,
                henyey_greenstein_g: 0.,
            },
        }
    }

    pub fn update(&mut self, delta_time: f32, input_state: &InputState, benchmark: bool) {
        if !benchmark {
            self.camera.apply_input(input_state, delta_time);
            self.update_player(input_state);
            self.physics.step(delta_time);
            self.camera.set_position(
                self.physics.get_translation(self.camera_rigid_body_handle)
                    + Vector3::new(0., 0., AVERAGE_MALE_EYE_HEIGHT / 2.),
            );
        }
        if !self.sun_pause {
            self.time_of_day += self.sun_speed * delta_time;
        }
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
        let translation = &mut self.entities[1].transform.translation;
        translation.x = self.sun_radius * self.time_of_day.sin();
        translation.z = self.sun_radius * self.time_of_day.cos();
    }

    pub fn light(&self) -> Light {
        Light {
            position: self.sun().transform.translation,
            intensity: self.sun_intensity,
            color: Vector3::new(1., 1., 1.),
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
    pub fn model_matrix(&self) -> Matrix4<f32> {
        self.transform.model_matrix()
    }

    pub fn albedo(&self) -> Vector3<f32> {
        self.albedo
    }

    pub fn metallic(&self) -> f32 {
        self.metallic
    }

    pub fn roughness(&self) -> f32 {
        self.roughness
    }

    pub fn ao(&self) -> f32 {
        self.ao
    }

    pub fn mesh_id(&self) -> usize {
        self.mesh_id
    }
}

impl Transform {
    pub fn model_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_translation(&self.translation).prepend_nonuniform_scaling(&self.scale)
            * self.rotation.to_homogeneous()
    }
}
