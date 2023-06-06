use crate::model::Model;
use nalgebra::{Point3, UnitQuaternion, Vector3};
use rapier3d::prelude::*;

pub struct Physics {
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    pub query_pipeline: QueryPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
}

impl Physics {
    pub fn new() -> Physics {
        Physics {
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            query_pipeline: QueryPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }

    pub fn trimesh(&self, model: &Model) -> ColliderBuilder {
        let vertices: Vec<_> = model
            .vertices
            .iter()
            .map(|v| Point3::from(v.position))
            .collect();
        let indices = (0..vertices.len() as u32).array_chunks().collect();
        ColliderBuilder::trimesh(vertices, indices)
    }

    pub fn cube(&self, side: f32) -> ColliderBuilder {
        ColliderBuilder::cuboid(side / 2., side / 2., side / 2.)
    }

    pub fn insert_static(&mut self, collider: ColliderBuilder) {
        self.collider_set.insert(collider);
    }

    pub fn insert(
        &mut self,
        translation: Vector3<f32>,
        collider: ColliderBuilder,
    ) -> RigidBodyHandle {
        let rigid_body = RigidBodyBuilder::dynamic().translation(translation);
        let handle = self.rigid_body_set.insert(rigid_body);
        self.collider_set
            .insert_with_parent(collider, handle, &mut self.rigid_body_set);
        handle
    }

    pub fn step(&mut self, delta_time: f32) {
        self.integration_parameters.dt = delta_time;
        self.physics_pipeline.step(
            &Vector3::new(0., 0., -9.81),
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
    }

    pub fn get_translation(&self, rigid_body: RigidBodyHandle) -> Vector3<f32> {
        *self.rigid_body_set[rigid_body].translation()
    }

    pub fn get_rotation(&self, rigid_body: RigidBodyHandle) -> UnitQuaternion<f32> {
        *self.rigid_body_set[rigid_body].rotation()
    }
}
