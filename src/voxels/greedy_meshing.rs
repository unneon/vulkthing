use crate::mesh::MeshData;
use crate::renderer::vertex::Vertex;
use crate::voxels::sparse_octree::SparseOctree;
use crate::voxels::MeshingAlgorithm;

pub struct GreedyMeshing;

struct State<'a> {
    chunk_size: usize,
    chunk_svo: &'a SparseOctree,
    neighbour_svos: [&'a SparseOctree; 6],
    vertices: Vec<Vertex>,
}

impl MeshingAlgorithm for GreedyMeshing {
    fn mesh(
        chunk_svo: &SparseOctree,
        neighbour_svos: [&SparseOctree; 6],
        chunk_size: usize,
    ) -> MeshData {
        let mut state = State {
            chunk_size,
            chunk_svo,
            neighbour_svos,
            vertices: Vec::new(),
        };
        MeshData {
            vertices: state.vertices,
        }
    }
}
