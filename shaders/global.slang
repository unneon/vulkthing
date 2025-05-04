module global;

[[vk::binding(0)]]
public ConstantBuffer<Global> global;

[[vk::binding(1)]]
public StructuredBuffer<Star> stars;

[[vk::binding(2)]]
public StructuredBuffer<VoxelVertex> voxel_vertices;

[[vk::binding(3)]]
public StructuredBuffer<VoxelTriangle> voxel_triangles;

[[vk::binding(4)]]
public StructuredBuffer<VoxelMeshlet> meshlets;

[[vk::binding(5)]]
public StructuredBuffer<SvoNode> svo_nodes;

// Temporarily, read the reflection codegen and make sure the alignment will be handled correctly
// before modifying any struct definitions! This can be fixed by either implementing full std140 and
// std430 alignment rules in codegen, or hoping the scalar_block_layout extension works well.

public struct Global {
    public Voxels voxels;
    public Light light;
    public Atmosphere atmosphere;
    public Camera camera;
    public VoxelMaterial materials[256];
    public Debug debug;
};

public struct Voxels {
    public uint chunk_size;
    public uint meshlet_count;
    public uint root_svo_index;
    public uint root_svo_side;
    public uint3 root_svo_base;
};

public struct Light {
    public float3 color;
    public float intensity;
    public float3 position;
    public float scale;
};

public struct Atmosphere {
    public bool enable;
    public uint scatter_point_count;
    public uint optical_depth_point_count;
    public float density_falloff;
    public float3 planet_position;
    public float planet_radius;
    public float3 sun_position;
    public float scale;
    public float3 wavelengths;
    public float scattering_strength;
    public float henyey_greenstein_g;
};

public struct Camera {
    public float4x4 view_matrix;
    public float4x4 projection_matrix;
    public float4x4 inverse_view_matrix;
    public float4x4 inverse_projection_matrix;
    public float2 resolution;
    public float depth_near;
    public float depth_far;
    public float3 position;
    float _pad0;
    public float3 direction;
};

public struct VoxelMaterial {
    public float3 albedo;
    public float roughness;
    public float3 emit;
    public float metallic;
};

public struct Debug {
    public uint meshlet_id;
};

public struct Star {
    public float4x4 model;
};

public struct VoxelMeshlet {
    public uint vertex_offset;
    public uint vertex_count;
    public uint triangle_offset;
    public uint triangle_count;
    public int16_t3 chunk;
    int16_t _pad0;
    public uint8_t3 bound_base;
    uint8_t _pad1;
    public uint8_t3 bound_size;
    uint8_t _pad2;

    public property world_space_min : float3 {
        get {
            Global global = global;
            return global.voxels.chunk_size * float3(chunk) + bound_base;
        }
    }

    public property world_space_max : float3 {
        get {
            return world_space_min + bound_size;
        }
    }
};

public struct VoxelVertex {
    public uint8_t3 position;
    // Two lowest bits encode an ambient occlusion value (00 => 0, 11 => 3/4), the other bits are unused.
    public uint8_t data;
};

public struct VoxelTriangle {
    public uint8_t3 indices;
    // Three lowest bits encode a normal matching the convention of the DIRECTION array, five highest bits are used for
    // material ID. Subject to heavy changes later.
    public uint8_t data;
};

public struct SvoNode {
    public uint children[8];
    public uint parent;
};
