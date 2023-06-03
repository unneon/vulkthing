use crate::model::Model;
use nalgebra::{Point3, Vector3};
use rand::random;
use rapier3d::prelude::*;

pub struct Physics {
    gravity: Vector3<f32>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    ball_body_handle: RigidBodyHandle,
}

impl Physics {
    pub fn new(planet_model: &Model) -> Physics {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        let vertices = planet_model
            .vertices
            .iter()
            .map(|v| Point3::from(v.position))
            .collect();
        let indices = planet_model
            .indices
            .iter()
            .copied()
            .array_chunks()
            .collect();
        let collider = ColliderBuilder::trimesh(vertices, indices);
        collider_set.insert(collider);

        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(Vector3::new(-0.01, 0., 150.))
            .build();
        let collider = ColliderBuilder::cuboid(1., 1., 1.).restitution(0.4).build();
        let ball_body_handle = rigid_body_set.insert(rigid_body);
        collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);

        Physics {
            gravity: Vector3::new(0., 0., -9.81),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set,
            collider_set,
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            ball_body_handle,
        }
    }

    pub fn step(&mut self, delta_time: f32) {
        let old_position = *self.get_ball_position();
        self.integration_parameters.dt = delta_time;
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &(),
            &(),
        );
        let new_position = *self.get_ball_position();
        self.gravity = -9.81 * new_position.normalize();
        if (new_position - old_position).norm() < 0.0001 {
            self.rigid_body_set
                .get_mut(self.ball_body_handle)
                .unwrap()
                .apply_impulse(Vector3::new(random(), random(), random()).scale(400.), true);
        }
    }

    pub fn get_ball_position(&self) -> &Vector3<f32> {
        self.rigid_body_set[self.ball_body_handle].translation()
    }
}
