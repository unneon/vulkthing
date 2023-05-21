use crate::renderer::{ImageResources, FRAMES_IN_FLIGHT};
use ash::{vk, Device, Instance};
use log::debug;
use noise::{NoiseFn, Perlin};
use std::mem::MaybeUninit;

pub struct UniformBuffer<T> {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    mapping: *mut T,
    aligned_size: usize,
}

impl<T> UniformBuffer<T> {
    pub fn create(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &Device,
    ) -> UniformBuffer<T> {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let data_size = std::mem::size_of::<T>();
        let aligned_size = data_size
            .next_multiple_of(properties.limits.min_uniform_buffer_offset_alignment as usize);
        let buffer_size = aligned_size * FRAMES_IN_FLIGHT;
        let (buffer, memory) = create_buffer(
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            buffer_size,
            instance,
            physical_device,
            logical_device,
        );
        let mapping = unsafe {
            logical_device.map_memory(memory, 0, buffer_size as u64, vk::MemoryMapFlags::empty())
        }
        .unwrap() as *mut T;
        UniformBuffer {
            buffer,
            memory,
            mapping,
            aligned_size,
        }
    }

    pub fn write(&self, flight_index: usize, value: T) {
        unsafe {
            self.mapping
                .byte_add(self.aligned_size * flight_index)
                .write_volatile(value)
        };
    }

    pub fn deref(&mut self, flight_index: usize) -> &mut T {
        unsafe { &mut *self.mapping.byte_add(self.aligned_size * flight_index) }
    }

    pub fn descriptor(&self, flight_index: usize) -> vk::DescriptorBufferInfo {
        *vk::DescriptorBufferInfo::builder()
            .buffer(self.buffer)
            .offset((flight_index * self.aligned_size) as u64)
            .range(std::mem::size_of::<T>() as u64)
    }

    pub fn cleanup(&self, dev: &Device) {
        unsafe { dev.destroy_buffer(self.buffer, None) };
        unsafe { dev.free_memory(self.memory, None) };
    }
}

pub fn create_buffer(
    properties: vk::MemoryPropertyFlags,
    usage: vk::BufferUsageFlags,
    size: usize,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size as u64)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let buffer = unsafe { logical_device.create_buffer(&buffer_info, None) }.unwrap();
    let requirements = unsafe { logical_device.get_buffer_memory_requirements(buffer) };
    let memory_type_index = find_memory_type(
        properties,
        requirements.memory_type_bits,
        instance,
        physical_device,
    );
    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type_index);
    let memory = unsafe { logical_device.allocate_memory(&memory_info, None) }.unwrap();
    unsafe { logical_device.bind_buffer_memory(buffer, memory, 0) }.unwrap();
    (buffer, memory)
}

pub fn create_image(
    format: vk::Format,
    memory: vk::MemoryPropertyFlags,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    width: usize,
    height: usize,
    mip_levels: usize,
    samples: vk::SampleCountFlags,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
) -> (vk::Image, vk::DeviceMemory) {
    let image_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            width: width as u32,
            height: height as u32,
            depth: 1,
        })
        .mip_levels(mip_levels as u32)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(samples);
    let image = unsafe { logical_device.create_image(&image_info, None) }.unwrap();

    let requirements = unsafe { logical_device.get_image_memory_requirements(image) };
    let memory_type = find_memory_type(
        memory,
        requirements.memory_type_bits,
        instance,
        physical_device,
    );
    let alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type);
    let image_memory = unsafe { logical_device.allocate_memory(&alloc_info, None) }.unwrap();
    unsafe { logical_device.bind_image_memory(image, image_memory, 0) }.unwrap();

    (image, image_memory)
}

pub fn create_image_view(
    image: vk::Image,
    format: vk::Format,
    aspect_mask: vk::ImageAspectFlags,
    mip_levels: usize,
    logical_device: &Device,
) -> vk::ImageView {
    let view_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: 0,
            level_count: mip_levels as u32,
            base_array_layer: 0,
            layer_count: 1,
        });
    unsafe { logical_device.create_image_view(&view_info, None) }.unwrap()
}

pub(super) fn generate_perlin_texture(
    resolution: usize,
    density: f64,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
) -> ImageResources {
    let pixel_count = resolution * resolution;
    let image_format = vk::Format::R8_SNORM;

    // Create texture image with the given resolution and prepare it for writing from the host.
    let (image, memory) = create_image(
        image_format,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        resolution,
        resolution,
        1,
        vk::SampleCountFlags::TYPE_1,
        instance,
        physical_device,
        logical_device,
    );
    transition_image_layout(
        image,
        vk::AccessFlags::empty(),
        vk::AccessFlags::TRANSFER_WRITE,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::TRANSFER,
        1,
        logical_device,
        queue,
        command_pool,
    );

    // Rust image library actually lets you specify a custom buffer, and passing in a
    // Vulkan-allocated one seems to work nicely. I kind of wonder whether I should add resizable
    // bar and/or unified memory support. This would just let me generate the noise straight to GPU
    // memory? Device parameters seem to support it, but I have to read up on the performance
    // implications.
    let (staging_buffer, staging_memory) = create_buffer(
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST,
        pixel_count,
        instance,
        physical_device,
        logical_device,
    );
    with_mapped_slice(staging_memory, pixel_count, logical_device, |mapped| {
        let perlin = Perlin::new(907);
        for y in 0..resolution {
            for x in 0..resolution {
                let nx = (x as f64) / resolution as f64 * density;
                let ny = (y as f64) / resolution as f64 * density;
                let pixel = noise(&perlin, nx, ny);
                mapped[y * resolution + x].write(pixel);
            }
        }
    });
    copy_buffer_to_image(
        staging_buffer,
        image,
        resolution,
        resolution,
        logical_device,
        queue,
        command_pool,
    );
    unsafe { logical_device.destroy_buffer(staging_buffer, None) };
    unsafe { logical_device.free_memory(staging_memory, None) };

    transition_image_layout(
        image,
        vk::AccessFlags::TRANSFER_WRITE,
        vk::AccessFlags::SHADER_READ,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        1,
        logical_device,
        queue,
        command_pool,
    );

    let view = create_image_view(
        image,
        image_format,
        vk::ImageAspectFlags::COLOR,
        1,
        logical_device,
    );
    let texture = ImageResources {
        image,
        memory,
        view,
    };
    debug!("perlin noise generated, \x1B[1msize\x1B[0m: {resolution}x{resolution}");
    texture
}

fn noise(perlin: &Perlin, x: f64, y: f64) -> i8 {
    let mut value = 0.;
    let mut bound = 0.;
    for i in 0..10 {
        let factor = (1 << i) as f64;
        value += perlin.get([factor * x, factor * y]) / factor;
        bound += 1. / factor;
    }
    float_to_snorm(value / bound)
}

fn float_to_snorm(value: f64) -> i8 {
    (value * 127.).round() as i8
}

pub fn transition_image_layout(
    image: vk::Image,
    src_access_mask: vk::AccessFlags,
    dst_access_mask: vk::AccessFlags,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    src_stage_mask: vk::PipelineStageFlags,
    dst_stage_mask: vk::PipelineStageFlags,
    mip_levels: usize,
    logical_device: &Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
) {
    onetime_commands(logical_device, queue, command_pool, move |command_buffer| {
        let barrier = vk::ImageMemoryBarrier::builder()
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: mip_levels as u32,
                base_array_layer: 0,
                layer_count: 1,
            });
        unsafe {
            logical_device.cmd_pipeline_barrier(
                command_buffer,
                src_stage_mask,
                dst_stage_mask,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[*barrier],
            )
        };
    });
}

fn copy_buffer_to_image(
    buffer: vk::Buffer,
    image: vk::Image,
    width: usize,
    height: usize,
    logical_device: &Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
) {
    onetime_commands(logical_device, queue, command_pool, move |command_buffer| {
        let region = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            image_extent: vk::Extent3D {
                width: width as u32,
                height: height as u32,
                depth: 1,
            },
        };

        unsafe {
            logical_device.cmd_copy_buffer_to_image(
                command_buffer,
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            )
        };
    });
}

pub fn copy_buffer(
    src: vk::Buffer,
    dst: vk::Buffer,
    len: usize,
    logical_device: &Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
) {
    onetime_commands(logical_device, queue, command_pool, move |command_buffer| {
        let copy_region = vk::BufferCopy::builder()
            .src_offset(0)
            .dst_offset(0)
            .size(len as u64);
        unsafe { logical_device.cmd_copy_buffer(command_buffer, src, dst, &[*copy_region]) };
    });
}

pub fn with_mapped_slice<T, R>(
    memory: vk::DeviceMemory,
    count: usize,
    logical_device: &Device,
    f: impl FnOnce(&mut [MaybeUninit<T>]) -> R,
) -> R {
    let ptr = unsafe {
        logical_device.map_memory(
            memory,
            0,
            (std::mem::size_of::<T>() * count) as u64,
            vk::MemoryMapFlags::empty(),
        )
    }
    .unwrap();
    let result = f(unsafe { std::slice::from_raw_parts_mut(ptr as *mut MaybeUninit<T>, count) });
    unsafe { logical_device.unmap_memory(memory) };
    result
}

fn onetime_commands<R>(
    logical_device: &Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
    f: impl FnOnce(vk::CommandBuffer) -> R,
) -> R {
    let command_info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(command_pool)
        .command_buffer_count(1);
    let command_buffer =
        unsafe { logical_device.allocate_command_buffers(&command_info) }.unwrap()[0];

    let begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { logical_device.begin_command_buffer(command_buffer, &begin_info) }.unwrap();

    let result = f(command_buffer);

    unsafe { logical_device.end_command_buffer(command_buffer) }.unwrap();

    let submit_buffers = [command_buffer];
    let submit_info = vk::SubmitInfo::builder().command_buffers(&submit_buffers);
    unsafe { logical_device.queue_submit(queue, &[*submit_info], vk::Fence::null()) }.unwrap();
    unsafe { logical_device.queue_wait_idle(queue) }.unwrap();
    unsafe { logical_device.free_command_buffers(command_pool, &[command_buffer]) };

    result
}

// TODO: Use 4x or less MSAA samples, as recommended by AMD.
pub fn find_max_msaa_samples(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> vk::SampleCountFlags {
    let best_order = [
        vk::SampleCountFlags::TYPE_64,
        vk::SampleCountFlags::TYPE_32,
        vk::SampleCountFlags::TYPE_16,
        vk::SampleCountFlags::TYPE_8,
        vk::SampleCountFlags::TYPE_4,
        vk::SampleCountFlags::TYPE_2,
    ];
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };
    let counts = properties.limits.framebuffer_color_sample_counts
        & properties.limits.framebuffer_depth_sample_counts;
    for count in best_order {
        if counts.contains(count) {
            return count;
        }
    }
    vk::SampleCountFlags::TYPE_1
}

fn find_memory_type(
    properties: vk::MemoryPropertyFlags,
    type_filter: u32,
    instance: &Instance,
    device: vk::PhysicalDevice,
) -> u32 {
    let memory = unsafe { instance.get_physical_device_memory_properties(device) };
    for i in 0..memory.memory_type_count {
        if type_filter & (1 << i) != 0
            && (memory.memory_types[i as usize].property_flags & properties) == properties
        {
            return i;
        }
    }
    panic!(
        "no good memory type_filter={type_filter} properties={properties:?} {:#?}",
        properties
    );
}

pub fn exists_newer_file(path: &str, reference: &str) -> bool {
    let Ok(meta) = std::fs::metadata(path) else { return false; };
    let path_mtime = meta.modified().unwrap();
    let reference_mtime = std::fs::metadata(reference).unwrap().modified().unwrap();
    path_mtime >= reference_mtime
}
