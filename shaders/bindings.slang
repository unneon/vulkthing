module bindings;

[[vk::binding(0)]]
public ConstantBuffer<Global> global;

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

[[vk::binding(1)]]
public StructuredBuffer<Star> stars;

public struct Star {
    public float4x4 model;
};

[[vk::binding(2)]]
public StructuredBuffer<VoxelVertex> voxel_vertices;

public struct VoxelVertex {
    public uint8_t3 position;
    public uint8_t data;

    public property ambient_occlusion : float {
        get {
            return 0.75 * bitfieldExtract(data, 0, 2) / 3;
        }
    }
};

[[vk::binding(3)]]
public StructuredBuffer<VoxelTriangle> voxel_triangles;

public struct VoxelTriangle {
    public uint8_t3 indices;
    public VoxelTriangleData data;
};

public struct VoxelTriangleData {
    public uint8_t data;

    public property cubenormal : uint8_t {
        get {
            return bitfieldExtract(data, 0, 3);
        }
    }

    public property material_id : uint8_t {
        get {
            return bitfieldExtract(data, 3, 5);
        }
    }
}

[[vk::binding(4)]]
public StructuredBuffer<VoxelMeshlet> voxel_meshlets;

public struct VoxelMeshlet {
    public uint vertex_offset;
    public uint vertex_count;
    public uint triangle_offset;
    public uint triangle_count;
    public int16_t3 chunk;
    public uint8_t3 bound_base;
    public uint8_t3 bound_size;

    public property world_space_min : float3 {
        get {
            return global.voxels.chunk_size * float3(chunk) + bound_base;
        }
    }

    public property world_space_max : float3 {
        get {
            return world_space_min + bound_size;
        }
    }
};

[[vk::binding(5)]]
public StructuredBuffer<SvoNode> svo_nodes;

public struct SvoNode {
    public uint children[8];
    public uint parent;
};

[[vk::binding(6)]]
public StructuredBuffer<ClassicVertex> classic_vertices;

public struct ClassicVertex {
    public float3 position;
    public float3 normal;
};
