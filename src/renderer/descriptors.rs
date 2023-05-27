use crate::renderer::util::AnyUniformBuffer;
use crate::renderer::FRAMES_IN_FLIGHT;
use ash::{vk, Device};

pub struct DescriptorMetadata {
    pub pool: vk::DescriptorPool,
    pub set_layout: vk::DescriptorSetLayout,
    config: Vec<Descriptor>,
}

pub struct DescriptorConfig<'a> {
    pub descriptors: Vec<Descriptor>,
    pub set_count: usize,
    pub logical_device: &'a Device,
}

pub struct Descriptor {
    pub kind: DescriptorKind,
    pub stage: vk::ShaderStageFlags,
}

enum DescriptorInfo {
    Buffer(vk::DescriptorBufferInfo),
    Image(vk::DescriptorImageInfo),
}

pub enum DescriptorKind {
    AccelerationStructure,
    ImmutableSampler { sampler: vk::Sampler },
    UniformBuffer,
}

pub enum DescriptorValue<'a> {
    Buffer(&'a dyn AnyUniformBuffer),
    Image(vk::ImageView),
}

impl DescriptorMetadata {
    pub fn create_sets(
        &self,
        values: &[DescriptorValue],
        logical_device: &Device,
    ) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {
        let layouts = [self.set_layout; FRAMES_IN_FLIGHT];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool)
            .set_layouts(&layouts);
        let descriptor_sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
            unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_alloc_info) }
                .unwrap()
                .try_into()
                .unwrap();
        let mut descriptor_infos = [const { Vec::new() }; FRAMES_IN_FLIGHT];
        let mut descriptor_writes: Vec<vk::WriteDescriptorSet> = Vec::new();
        for (flight_index, descriptor_info) in descriptor_infos.iter_mut().enumerate() {
            for value in values {
                let info = match value {
                    DescriptorValue::Buffer(buffer) => {
                        DescriptorInfo::Buffer(buffer.descriptor(flight_index))
                    }
                    DescriptorValue::Image(view) => DescriptorInfo::Image(
                        *vk::DescriptorImageInfo::builder()
                            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                            .image_view(*view),
                    ),
                };
                descriptor_info.push(info);
            }
        }
        for flight_index in 0..FRAMES_IN_FLIGHT {
            for (binding, info) in descriptor_infos[flight_index].iter().enumerate() {
                let write = vk::WriteDescriptorSet::builder()
                    .dst_set(descriptor_sets[flight_index])
                    .dst_binding(binding as u32)
                    .descriptor_type(self.config[binding].kind.ty());
                let write = *match info {
                    DescriptorInfo::Buffer(info) => write.buffer_info(std::slice::from_ref(info)),
                    DescriptorInfo::Image(info) => write.image_info(std::slice::from_ref(info)),
                };
                descriptor_writes.push(write);
            }
        }
        unsafe { logical_device.update_descriptor_sets(&descriptor_writes, &[]) };
        descriptor_sets
    }

    pub fn cleanup(&self, dev: &Device) {
        unsafe {
            dev.destroy_descriptor_pool(self.pool, None);
            dev.destroy_descriptor_set_layout(self.set_layout, None);
        }
    }
}

impl DescriptorKind {
    fn ty(&self) -> vk::DescriptorType {
        match self {
            DescriptorKind::AccelerationStructure => vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
            DescriptorKind::ImmutableSampler { .. } => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            DescriptorKind::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
        }
    }
}

pub fn create_descriptor_metadata(config: DescriptorConfig) -> DescriptorMetadata {
    let mut pool_sizes: Vec<vk::DescriptorPoolSize> = Vec::new();
    for desc in &config.descriptors {
        let ty = desc.kind.ty();
        let pool_size = match pool_sizes.iter_mut().find(|ps| ps.ty == ty) {
            Some(pool_size) => pool_size,
            None => {
                pool_sizes.push(vk::DescriptorPoolSize {
                    ty,
                    descriptor_count: 0,
                });
                pool_sizes.last_mut().unwrap()
            }
        };
        pool_size.descriptor_count += (config.set_count * FRAMES_IN_FLIGHT) as u32;
    }
    let pool_info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(&pool_sizes)
        .max_sets((config.set_count * FRAMES_IN_FLIGHT) as u32)
        .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);
    let pool = unsafe {
        config
            .logical_device
            .create_descriptor_pool(&pool_info, None)
    }
    .unwrap();

    let mut bindings = Vec::new();
    for (index, desc) in config.descriptors.iter().enumerate() {
        let mut binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(index as u32)
            .descriptor_count(1)
            .descriptor_type(desc.kind.ty())
            .stage_flags(desc.stage);
        if let DescriptorKind::ImmutableSampler { sampler } = &desc.kind {
            binding = binding.immutable_samplers(std::slice::from_ref(sampler));
        }
        bindings.push(*binding);
    }
    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);
    let set_layout = unsafe {
        config
            .logical_device
            .create_descriptor_set_layout(&layout_info, None)
    }
    .unwrap();

    DescriptorMetadata {
        pool,
        set_layout,
        config: config.descriptors,
    }
}
