mod culled_meshing;
mod greedy_meshing;

#[cfg(feature = "dev-menu")]
use crate::interface::EnumInterface;
use crate::voxel::local_mesh::LocalMesh;
use crate::voxel::material::Material;
use crate::voxel::meshing::culled_meshing::CulledMeshing;
use crate::voxel::meshing::greedy_meshing::GreedyMeshing;
use crate::voxel::neighbourhood::Neighbourhood;
use crate::voxel::sparse_octree::SparseOctree;
use crate::voxel::VoxelsConfig;

trait MeshingAlgorithm {
    fn mesh(svos: &Neighbourhood, chunk_size: usize) -> LocalMesh;
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum MeshingAlgorithmKind {
    Culled,
    Greedy,
}

#[cfg(feature = "dev-menu")]
impl EnumInterface for MeshingAlgorithmKind {
    const VALUES: &'static [MeshingAlgorithmKind] =
        &[MeshingAlgorithmKind::Culled, MeshingAlgorithmKind::Greedy];

    fn label(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed(match self {
            MeshingAlgorithmKind::Culled => "Culled Meshing",
            MeshingAlgorithmKind::Greedy => "Greedy Meshing",
        })
    }
}

pub fn generate_mesh(svos: &Neighbourhood, config: &VoxelsConfig) -> LocalMesh {
    if let SparseOctree::Uniform {
        kind: chunk_uniform,
    } = svos.chunk()
    {
        if *chunk_uniform == Material::Air {
            return LocalMesh::new_empty();
        }
        if svos.neighbour_chunks_manhattan().iter().all(|neighbour_svo| matches!(neighbour_svo, SparseOctree::Uniform { kind } if *kind != Material::Air)) {
            return LocalMesh::new_empty();
        }
    }
    let meshing_algorithm = match config.meshing_algorithm {
        MeshingAlgorithmKind::Culled => CulledMeshing::mesh,
        MeshingAlgorithmKind::Greedy => GreedyMeshing::mesh,
    };
    let mesh = meshing_algorithm(svos, config.chunk_size);
    mesh.remove_duplicate_vertices()
}
