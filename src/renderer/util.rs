use crate::renderer::{DeviceSupport, FRAMES_IN_FLIGHT};
use ash::ext::{debug_utils, mesh_shader};
use ash::khr::{buffer_device_address, surface, swapchain};
use ash::{vk, Device, Instance};
use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::time::Duration;

pub trait AsDescriptor {
    fn descriptor(&self, flight_index: usize) -> vk::DescriptorBufferInfo;
}

pub struct Ctx<'a> {
    pub dev: &'a Dev,
    pub queue: vk::Queue,
    pub command_pool: vk::CommandPool,
}

#[derive(Clone)]
pub struct Dev {
    pub logical: Device,
    pub physical: vk::PhysicalDevice,
    pub instance: Instance,
    pub debug_ext: debug_utils::Device,
    pub debug_ext_instance: debug_utils::Instance,
    pub surface_ext: surface::Instance,
    pub swapchain_ext: swapchain::Device,
    pub mesh_ext: mesh_shader::Device,
    pub support: DeviceSupport,
}

pub struct Buffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: usize,
}

pub struct ImageResources {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
}

pub struct UniformBuffer<T> {
    buffer: Buffer,
    mapping: *mut T,
    aligned_size: usize,
}

pub struct StorageBuffer<T: ?Sized> {
    buffer: Buffer,
    mapping: *mut T,
}

impl Buffer {
    pub fn create(
        properties: vk::MemoryPropertyFlags,
        usage: vk::BufferUsageFlags,
        size: usize,
        dev: &Dev,
    ) -> Buffer {
        let create_info = vk::BufferCreateInfo::default()
            .size(size as u64)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { dev.create_buffer(&create_info, None) }.unwrap();
        let requirements = unsafe { dev.get_buffer_memory_requirements(buffer) };
        let memory_type_index = find_memory_type(properties, requirements.memory_type_bits, dev);
        let mut memory_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);

        let mut allocate_flags;
        if usage.contains(vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS) {
            allocate_flags = vk::MemoryAllocateFlagsInfoKHR::default()
                .flags(vk::MemoryAllocateFlags::DEVICE_ADDRESS);
            memory_info = memory_info.push_next(&mut allocate_flags);
        }

        let memory = unsafe { dev.allocate_memory(&memory_info, None) }.unwrap();
        unsafe { dev.bind_buffer_memory(buffer, memory, 0) }.unwrap();
        Buffer {
            buffer,
            memory,
            size,
        }
    }

    pub fn fill_from_slice_host_visible<T: Copy>(&mut self, data: &[T], dev: &Dev) {
        assert_eq!(std::mem::size_of_val(data), self.size);
        self.with_mapped(dev, |mapped: *mut [T]| {
            unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), mapped as *mut T, data.len()) };
        });
    }

    pub fn with_mapped<T, R>(&mut self, dev: &Dev, f: impl FnOnce(*mut [T]) -> R) -> R {
        let memory = self.map_memory(dev);
        let r = f(memory);
        unsafe { dev.unmap_memory(self.memory) };
        r
    }

    pub fn map_memory<T>(&mut self, dev: &Dev) -> *mut [T] {
        let count = self.size / std::mem::size_of::<T>();
        let flags = vk::MemoryMapFlags::empty();
        let ptr = unsafe { dev.map_memory(self.memory, 0, self.size as u64, flags) }.unwrap();
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr as *mut MaybeUninit<T>, count) };
        slice as *mut [MaybeUninit<T>] as *mut [T]
    }

    #[allow(dead_code)]
    pub fn device_address(&self, buffer_device_address_ext: &buffer_device_address::Device) -> u64 {
        let info = vk::BufferDeviceAddressInfoKHR::default().buffer(self.buffer);
        unsafe { buffer_device_address_ext.get_buffer_device_address(&info) }
    }

    pub fn cleanup(&self, dev: &Device) {
        unsafe { dev.destroy_buffer(self.buffer, None) };
        unsafe { dev.free_memory(self.memory, None) };
    }
}

impl Ctx<'_> {
    #[allow(dead_code)]
    pub fn execute<R>(&self, f: impl FnOnce(vk::CommandBuffer) -> R) -> R {
        let command_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(self.command_pool)
            .command_buffer_count(1);
        let buf = unsafe { self.dev.allocate_command_buffers(&command_info) }.unwrap()[0];

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { self.dev.begin_command_buffer(buf, &begin_info) }.unwrap();

        let result = f(buf);

        unsafe { self.dev.end_command_buffer(buf) }.unwrap();

        let submit_buffers = [buf];
        let submit_info = vk::SubmitInfo::default().command_buffers(&submit_buffers);
        unsafe {
            self.dev
                .queue_submit(self.queue, &[submit_info], vk::Fence::null())
        }
        .unwrap();
        unsafe { self.dev.queue_wait_idle(self.queue) }.unwrap();
        unsafe { self.dev.free_command_buffers(self.command_pool, &[buf]) };

        result
    }
}

impl ImageResources {
    pub fn create(
        format: vk::Format,
        memory: vk::MemoryPropertyFlags,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        aspect: vk::ImageAspectFlags,
        extent: vk::Extent2D,
        samples: vk::SampleCountFlags,
        dev: &Dev,
    ) -> ImageResources {
        let (image, memory) = create_image(format, memory, tiling, usage, extent, samples, dev);
        let view = create_image_view(image, format, aspect, dev);
        ImageResources {
            image,
            memory,
            view,
        }
    }

    pub fn cleanup(&self, dev: &Device) {
        unsafe {
            dev.destroy_image_view(self.view, None);
            dev.destroy_image(self.image, None);
            dev.free_memory(self.memory, None);
        }
    }
}

impl<T: Copy> UniformBuffer<T> {
    pub fn create(dev: &Dev) -> UniformBuffer<T> {
        let properties = unsafe { dev.instance.get_physical_device_properties(dev.physical) };
        let data_size = std::mem::size_of::<T>();
        let aligned_size = data_size
            .next_multiple_of(properties.limits.min_uniform_buffer_offset_alignment as usize);
        let size = aligned_size * FRAMES_IN_FLIGHT;
        let buffer = Buffer::create(
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            size,
            dev,
        );
        let flags = vk::MemoryMapFlags::empty();
        let mapping =
            unsafe { dev.map_memory(buffer.memory, 0, size as u64, flags) }.unwrap() as *mut T;
        UniformBuffer {
            buffer,
            mapping,
            aligned_size,
        }
    }

    pub fn write(&self, flight_index: usize, value: &T) {
        unsafe {
            self.mapping
                .byte_add(self.aligned_size * flight_index)
                .write_volatile(*value)
        };
    }

    pub fn cleanup(&self, dev: &Device) {
        self.buffer.cleanup(dev);
    }
}

impl<T: ?Sized> StorageBuffer<T> {
    pub fn cleanup(&self, dev: &Device) {
        self.buffer.cleanup(dev);
    }
}

impl<T: Copy> StorageBuffer<T> {
    pub fn new(flags: vk::MemoryPropertyFlags, dev: &Dev) -> StorageBuffer<T> {
        let size = std::mem::size_of::<T>();
        let buffer = Buffer::create(flags, vk::BufferUsageFlags::STORAGE_BUFFER, size, dev);
        let flags = vk::MemoryMapFlags::empty();
        let mapping =
            unsafe { dev.map_memory(buffer.memory, 0, size as u64, flags) }.unwrap() as *mut T;
        StorageBuffer { buffer, mapping }
    }
}

impl<T: Copy> StorageBuffer<[T]> {
    pub fn new_array(
        flags: vk::MemoryPropertyFlags,
        count: usize,
        dev: &Dev,
    ) -> StorageBuffer<[T]> {
        let size = std::mem::size_of::<T>() * count;
        let buffer = Buffer::create(flags, vk::BufferUsageFlags::STORAGE_BUFFER, size, dev);
        let flags = vk::MemoryMapFlags::empty();
        let raw_mapping = unsafe { dev.map_memory(buffer.memory, 0, size as u64, flags) }.unwrap();
        let mapping = unsafe { std::slice::from_raw_parts_mut(raw_mapping as *mut T, count) };
        StorageBuffer { buffer, mapping }
    }

    pub fn generate(&mut self, mut f: impl FnMut(usize) -> T) {
        for (index, element) in self.mapped().iter_mut().enumerate() {
            element.write(f(index));
        }
    }
}

impl<T> StorageBuffer<[T]> {
    pub fn mapped(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe { &mut *(self.mapping as *mut [MaybeUninit<T>]) }
    }
}

impl Deref for Dev {
    type Target = Device;

    fn deref(&self) -> &Device {
        &self.logical
    }
}

impl<T> AsDescriptor for UniformBuffer<T> {
    fn descriptor(&self, flight_index: usize) -> vk::DescriptorBufferInfo {
        vk::DescriptorBufferInfo::default()
            .buffer(self.buffer.buffer)
            .offset((flight_index * self.aligned_size) as u64)
            .range(std::mem::size_of::<T>() as u64)
    }
}

impl<T: ?Sized> AsDescriptor for StorageBuffer<T> {
    fn descriptor(&self, _flight_index: usize) -> vk::DescriptorBufferInfo {
        vk::DescriptorBufferInfo::default()
            .buffer(self.buffer.buffer)
            .range(unsafe { std::mem::size_of_val(&*self.mapping) } as u64)
    }
}

// TODO: This probably isn't safe because of Buffer methods.
unsafe impl<T: Send + ?Sized> Send for StorageBuffer<T> {}

unsafe impl<T: Sync + ?Sized> Sync for StorageBuffer<T> {}

pub fn create_image(
    format: vk::Format,
    memory: vk::MemoryPropertyFlags,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    extent: vk::Extent2D,
    samples: vk::SampleCountFlags,
    dev: &Dev,
) -> (vk::Image, vk::DeviceMemory) {
    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(samples);
    let image = unsafe { dev.create_image(&image_info, None) }.unwrap();

    let requirements = unsafe { dev.get_image_memory_requirements(image) };
    let memory_type = find_memory_type(memory, requirements.memory_type_bits, dev);
    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(requirements.size)
        .memory_type_index(memory_type);
    let image_memory = unsafe { dev.allocate_memory(&alloc_info, None) }.unwrap();
    unsafe { dev.bind_image_memory(image, image_memory, 0) }.unwrap();

    (image, image_memory)
}

pub fn create_image_view(
    image: vk::Image,
    format: vk::Format,
    aspect_mask: vk::ImageAspectFlags,
    dev: &Dev,
) -> vk::ImageView {
    let view_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    unsafe { dev.create_image_view(&view_info, None) }.unwrap()
}

// TODO: Use 4x or less MSAA samples, as recommended by AMD.
#[allow(dead_code)]
pub fn find_max_msaa_samples(dev: &Dev) -> vk::SampleCountFlags {
    let best_order = [
        vk::SampleCountFlags::TYPE_64,
        vk::SampleCountFlags::TYPE_32,
        vk::SampleCountFlags::TYPE_16,
        vk::SampleCountFlags::TYPE_8,
        vk::SampleCountFlags::TYPE_4,
        vk::SampleCountFlags::TYPE_2,
    ];
    let properties = unsafe { dev.instance.get_physical_device_properties(dev.physical) };
    let counts = properties.limits.framebuffer_color_sample_counts
        & properties.limits.framebuffer_depth_sample_counts;
    for count in best_order {
        if counts.contains(count) {
            return count;
        }
    }
    vk::SampleCountFlags::TYPE_1
}

fn find_memory_type(properties: vk::MemoryPropertyFlags, type_filter: u32, dev: &Dev) -> u32 {
    let memory = unsafe {
        dev.instance
            .get_physical_device_memory_properties(dev.physical)
    };
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

pub fn vulkan_str(slice: &[i8]) -> &str {
    unsafe { CStr::from_ptr(slice.as_ptr()) }.to_str().unwrap()
}

pub fn timestamp_difference_to_duration(
    timestamp: u64,
    properties: &vk::PhysicalDeviceProperties,
) -> Duration {
    Duration::from_secs_f64(
        timestamp as f64 * properties.limits.timestamp_period as f64 / 1_000_000_000.,
    )
}
