use crate::camera::Camera;
use crate::config::{
    DEFAULT_CAMERA, DEFAULT_STAR_COUNT, DEFAULT_STAR_MAX_SCALE, DEFAULT_STAR_MIN_SCALE,
    DEFAULT_STAR_RADIUS, DEFAULT_SUN_POSITION, DEFAULT_SUN_RADIUS, DEFAULT_SUN_SPEED,
};
use crate::gpu::std140::Light;
use crate::input::InputState;
use crate::physics::Physics;
use crate::util::{RandomDirection, RandomRotation};
use nalgebra::{Matrix4, UnitQuaternion, Vector3};
use rand::Rng;
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
}

pub struct Transform {
    pub translation: Vector3<f32>,
    rotation: UnitQuaternion<f32>,
    scale: Vector3<f32>,
}

pub struct Star {
    pub transform: Transform,
}

pub struct Atmosphere {
    pub density_falloff: f32,
    pub scale: f32,
    pub scattering_strength: f32,
    pub henyey_greenstein_g: f32,
    pub planet_radius: f32,
}

const AVERAGE_MALE_HEIGHT: f32 = 1.74;
const AVERAGE_MALE_EYE_HEIGHT: f32 = 1.63;
const AVERAGE_MALE_SHOULDER_WIDTH: f32 = 0.465;

impl World {
    pub fn new() -> World {
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
        let sun = Entity {
            transform: Transform {
                translation: DEFAULT_SUN_POSITION,
                rotation: UnitQuaternion::identity(),
                scale: Vector3::from_element(50.),
            },
        };
        let entities = vec![sun];
        let mut stars = Vec::new();
        let mut rng = rand::rng();
        for _ in 0..DEFAULT_STAR_COUNT {
            stars.push(Star {
                transform: Transform {
                    translation: DEFAULT_STAR_RADIUS * rng.sample(RandomDirection),
                    rotation: rng.sample(RandomRotation),
                    scale: Vector3::from_element(
                        rng.random_range(DEFAULT_STAR_MIN_SCALE..DEFAULT_STAR_MAX_SCALE),
                    ),
                },
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
            sun_intensity: 1.,
            sun_pause: true,
            sun_radius: DEFAULT_SUN_RADIUS,
            sun_speed: DEFAULT_SUN_SPEED,
            atmosphere: Atmosphere {
                density_falloff: 6.,
                scale: 1.5,
                scattering_strength: 0.03,
                henyey_greenstein_g: 0.,
                planet_radius: 1000.,
            },
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
        rigid_body.set_linear_damping(2.);
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
        let translation = &mut self.entities[0].transform.translation;
        translation.x = self.sun_radius * self.time_of_day.sin();
        translation.z = self.sun_radius * self.time_of_day.cos();
    }

    pub fn light(&self) -> Light {
        Light {
            position: self.sun().transform.translation,
            intensity: self.sun_intensity,
            color: Vector3::new(1., 1., 1.),
            scale: 50.,
        }
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        self.camera.view_matrix()
    }

    pub fn sun(&self) -> &Entity {
        &self.entities[0]
    }
}

impl Transform {
    pub fn model_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_translation(&self.translation).prepend_nonuniform_scaling(&self.scale)
            * self.rotation.to_homogeneous()
    }
}
