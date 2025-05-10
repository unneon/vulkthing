use crate::mesh::MeshData;
use crate::renderer::vertex::Vertex;
use nalgebra::{Point3, Vector3};
use rapier3d::prelude::*;

pub struct Physics {
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
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
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }

    #[allow(dead_code)]
    pub fn trimesh(&self, model: &MeshData<Vertex>, scale: &Vector3<f32>) -> ColliderBuilder {
        let vertices: Vec<_> = model
            .vertices
            .iter()
            .map(|v| Point3::from(v.position.component_mul(scale)))
            .collect();
        let indices = (0..vertices.len() as u32 / 3)
            .map(|i| [3 * i, 3 * i + 1, 3 * i + 2])
            .collect();
        ColliderBuilder::trimesh(vertices, indices).unwrap()
    }

    #[allow(dead_code)]
    pub fn insert_static(&mut self, collider: ColliderBuilder) {
        self.collider_set.insert(collider);
    }

    pub fn step(&mut self, delta_time: f32) {
        self.integration_parameters.dt = delta_time;
        self.physics_pipeline.step(
            &Vector3::new(0., 0., 0.),
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
}
