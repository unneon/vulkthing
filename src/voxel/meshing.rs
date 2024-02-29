mod culled_meshing;
mod greedy_meshing;

use crate::interface::EnumInterface;
use crate::voxel::local_mesh::LocalMesh;
use crate::voxel::material::Material;
use crate::voxel::meshing::culled_meshing::CulledMeshing;
use crate::voxel::meshing::greedy_meshing::GreedyMeshing;
use crate::voxel::sparse_octree::SparseOctree;
use crate::voxel::VoxelsConfig;
use std::borrow::Cow;

trait MeshingAlgorithm {
    fn mesh(
        chunk_svo: &SparseOctree,
        neighbour_svos: [&SparseOctree; 6],
        chunk_size: usize,
    ) -> LocalMesh;
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum MeshingAlgorithmKind {
    Culled,
    Greedy,
}

impl EnumInterface for MeshingAlgorithmKind {
    const VALUES: &'static [MeshingAlgorithmKind] =
        &[MeshingAlgorithmKind::Culled, MeshingAlgorithmKind::Greedy];

    fn label(&self) -> Cow<str> {
        match self {
            MeshingAlgorithmKind::Culled => Cow::Borrowed("Culled Meshing"),
            MeshingAlgorithmKind::Greedy => Cow::Borrowed("Greedy Meshing"),
        }
    }
}

pub fn generate_mesh(
    chunk_svo: &SparseOctree,
    neighbour_svos: [&SparseOctree; 6],
    config: &VoxelsConfig,
) -> LocalMesh {
    if let SparseOctree::Uniform {
        kind: chunk_uniform,
    } = chunk_svo
    {
        if *chunk_uniform == Material::Air {
            return LocalMesh::new_empty();
        }
        if neighbour_svos.iter().all(|neighbour_svo| matches!(neighbour_svo, SparseOctree::Uniform { kind } if *kind != Material::Air)) {
            return LocalMesh::new_empty();
        }
    }
    let meshing_algorithm = match config.meshing_algorithm {
        MeshingAlgorithmKind::Culled => CulledMeshing::mesh,
        MeshingAlgorithmKind::Greedy => GreedyMeshing::mesh,
    };
    let mesh = meshing_algorithm(chunk_svo, neighbour_svos, config.chunk_size);
    mesh.remove_duplicate_vertices()
}
