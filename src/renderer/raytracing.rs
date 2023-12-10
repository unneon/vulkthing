use crate::renderer::util::{Buffer, Ctx, Dev};
use crate::renderer::vertex::Vertex;
use crate::renderer::{MeshObject, UNIFIED_MEMORY};
use ash::extensions::khr::{AccelerationStructure, BufferDeviceAddress};
use ash::vk;
use ash::vk::Packed24_8;
use nalgebra::Matrix4;

pub struct RaytraceResources {
    pub acceleration_structure: vk::AccelerationStructureKHR,
    buffer: Buffer,
    primitive_count: usize,
}

impl RaytraceResources {
    pub fn cleanup(&self, dev: &Dev) {
        unsafe {
            dev.acceleration_structure_ext
                .as_ref()
                .unwrap()
                .destroy_acceleration_structure(self.acceleration_structure, None)
        };
        self.buffer.cleanup(dev);
    }
}

pub fn create_blas(mesh: &MeshObject, ctx: &Ctx) -> RaytraceResources {
    let as_ext = ctx.dev.acceleration_structure_ext.as_ref().unwrap();
    let bda_ext = ctx.dev.buffer_device_address_ext.as_ref().unwrap();

    let vertex_address = mesh.vertex.device_address(bda_ext);
    let triangles = *vk::AccelerationStructureGeometryTrianglesDataKHR::builder()
        .vertex_format(vk::Format::R32G32B32_SFLOAT)
        .vertex_data(vk::DeviceOrHostAddressConstKHR {
            device_address: vertex_address,
        })
        .vertex_stride(std::mem::size_of::<Vertex>() as u64)
        .index_type(vk::IndexType::NONE_KHR)
        .transform_data(vk::DeviceOrHostAddressConstKHR::default())
        .max_vertex(3 * mesh.triangle_count as u32);
    let geometry = *vk::AccelerationStructureGeometryKHR::builder()
        .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
        .flags(vk::GeometryFlagsKHR::OPAQUE)
        .geometry(vk::AccelerationStructureGeometryDataKHR { triangles });
    let range_info = *vk::AccelerationStructureBuildRangeInfoKHR::builder()
        .first_vertex(0)
        .primitive_count(mesh.triangle_count as u32)
        .primitive_offset(0)
        .transform_offset(0);

    let geometries = [geometry];
    let mut blas_info = *vk::AccelerationStructureBuildGeometryInfoKHR::builder()
        .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
        .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
        .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
        .geometries(&geometries);

    let size_info = unsafe {
        as_ext.get_acceleration_structure_build_sizes(
            vk::AccelerationStructureBuildTypeKHR::DEVICE,
            &blas_info,
            &[range_info.primitive_count],
        )
    };

    let scratch = Buffer::create(
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::STORAGE_BUFFER,
        size_info.build_scratch_size as usize,
        ctx.dev,
    );

    let blas_buffer = Buffer::create(
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
        size_info.acceleration_structure_size as usize,
        ctx.dev,
    );
    let blas_create_info = *vk::AccelerationStructureCreateInfoKHR::builder()
        .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
        .size(size_info.acceleration_structure_size)
        .buffer(blas_buffer.buffer);
    let blas = unsafe { as_ext.create_acceleration_structure(&blas_create_info, None) }.unwrap();

    blas_info.dst_acceleration_structure = blas;
    blas_info.scratch_data.device_address = scratch.device_address(bda_ext);

    let blas_range_infos = [range_info];
    let all_blas_build_infos = [blas_info];
    let all_blas_range_infos = [blas_range_infos.as_slice()];
    ctx.execute(|buf| {
        unsafe {
            as_ext.cmd_build_acceleration_structures(
                buf,
                &all_blas_build_infos,
                &all_blas_range_infos,
            )
        };
    });

    scratch.cleanup(ctx.dev);

    RaytraceResources {
        acceleration_structure: blas,
        buffer: blas_buffer,
        primitive_count: mesh.triangle_count,
    }
}

pub fn create_tlas(model: &Matrix4<f32>, blas: &RaytraceResources, ctx: &Ctx) -> RaytraceResources {
    let as_ext = AccelerationStructure::new(&ctx.dev.instance, ctx.dev);
    let bda_ext = BufferDeviceAddress::new(&ctx.dev.instance, ctx.dev);

    let instanced = vk::AccelerationStructureInstanceKHR {
        transform: to_vulkan_matrix(model),
        instance_custom_index_and_mask: Packed24_8::new(0, 0xff),
        instance_shader_binding_table_record_offset_and_flags: Packed24_8::new(
            0,
            vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE.as_raw() as u8,
        ),
        acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
            device_handle: unsafe {
                as_ext.get_acceleration_structure_device_address(
                    &vk::AccelerationStructureDeviceAddressInfoKHR::builder()
                        .acceleration_structure(blas.acceleration_structure),
                )
            },
        },
    };

    let instances_buffer = Buffer::create(
        UNIFIED_MEMORY,
        vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR,
        std::mem::size_of::<vk::AccelerationStructureInstanceKHR>(),
        ctx.dev,
    );
    instances_buffer.fill_from_slice_host_visible(&[instanced], ctx.dev);
    let instances_address = instances_buffer.device_address(&bda_ext);

    let instances_vk = *vk::AccelerationStructureGeometryInstancesDataKHR::builder().data(
        vk::DeviceOrHostAddressConstKHR {
            device_address: instances_address,
        },
    );

    let tlas_geometry = *vk::AccelerationStructureGeometryKHR::builder()
        .geometry_type(vk::GeometryTypeKHR::INSTANCES)
        .geometry(vk::AccelerationStructureGeometryDataKHR {
            instances: instances_vk,
        });
    let tlas_geometries = [tlas_geometry];
    let mut tlas_info = *vk::AccelerationStructureBuildGeometryInfoKHR::builder()
        .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
        .geometries(&tlas_geometries)
        .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
        .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL);

    let tlas_size_info = unsafe {
        as_ext.get_acceleration_structure_build_sizes(
            vk::AccelerationStructureBuildTypeKHR::DEVICE,
            &tlas_info,
            &[blas.primitive_count as u32],
        )
    };

    let tlas_buffer = Buffer::create(
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR,
        tlas_size_info.acceleration_structure_size as usize,
        ctx.dev,
    );
    let tlas_create_info = *vk::AccelerationStructureCreateInfoKHR::builder()
        .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
        .size(tlas_size_info.acceleration_structure_size)
        .buffer(tlas_buffer.buffer);
    let tlas = unsafe { as_ext.create_acceleration_structure(&tlas_create_info, None) }.unwrap();

    let tlas_scratch = Buffer::create(
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::STORAGE_BUFFER,
        tlas_size_info.build_scratch_size as usize,
        ctx.dev,
    );

    tlas_info.dst_acceleration_structure = tlas;
    tlas_info.scratch_data.device_address = tlas_scratch.device_address(&bda_ext);

    let tlas_range_info = *vk::AccelerationStructureBuildRangeInfoKHR::builder()
        .first_vertex(0)
        .primitive_count(1)
        .primitive_offset(0)
        .transform_offset(0);
    let tlas_range_infos = [tlas_range_info];
    let all_tlas_build_infos = [tlas_info];
    let all_tlas_range_infos = [tlas_range_infos.as_slice()];
    ctx.execute(|buf| {
        unsafe {
            as_ext.cmd_build_acceleration_structures(
                buf,
                &all_tlas_build_infos,
                &all_tlas_range_infos,
            )
        };
    });

    tlas_scratch.cleanup(ctx.dev);
    instances_buffer.cleanup(ctx.dev);

    RaytraceResources {
        acceleration_structure: tlas,
        buffer: tlas_buffer,
        primitive_count: 1,
    }
}

fn to_vulkan_matrix(model: &Matrix4<f32>) -> vk::TransformMatrixKHR {
    // Nalgebra matrices are column-major, while VkTransformMatrixKHR is row-major.
    let mut matrix = [0.; 12];
    for (i, row) in model.row_iter().take(3).enumerate() {
        for (j, cell) in row.iter().enumerate() {
            matrix[4 * i + j] = *cell;
        }
    }
    vk::TransformMatrixKHR { matrix }
}
