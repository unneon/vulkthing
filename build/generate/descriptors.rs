use crate::types::{AshDescriptor, AshEnum};
use spirv_reflect::types::{ReflectDescriptorBinding, ReflectDescriptorType};
use spirv_reflect::ShaderModule;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn generate_descriptors(reflection: &ShaderModule, out_dir: &Path) {
    let descriptor_sets = reflection.enumerate_descriptor_sets(None).unwrap();
    assert_eq!(descriptor_sets.len(), 1);
    let descriptor_set = &descriptor_sets[0];
    let mut pool_sizes = Vec::new();
    for binding in &descriptor_set.bindings {
        let pool_size = match pool_sizes
            .iter_mut()
            .find(|(ty, _)| *ty == binding.descriptor_type)
        {
            Some(pool_size) => pool_size,
            None => {
                pool_sizes.push((binding.descriptor_type, 0));
                pool_sizes.last_mut().unwrap()
            }
        };
        pool_size.1 += 2;
    }
    let binding_count = descriptor_set.bindings.len();
    let pool_size_count = pool_sizes.len();

    let mut file = File::create(out_dir.join("descriptors.rs")).unwrap();

    writeln!(file, "static DESCRIPTOR_SET_BINDINGS: [vk::DescriptorSetLayoutBinding<'static>; {binding_count}] = [").unwrap();
    for binding in &descriptor_set.bindings {
        let binding_index = binding.binding;
        let description_type = binding.descriptor_type.ash_variant();
        writeln!(
            file,
            r#"    vk::DescriptorSetLayoutBinding {{
        binding: {binding_index},
        descriptor_type: vk::DescriptorType::{description_type},
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::ALL,
        p_immutable_samplers: std::ptr::null(),
        _marker: std::marker::PhantomData,
    }},"#,
        )
        .unwrap();
    }
    writeln!(
        file,
        r"];

static DESCRIPTOR_SET_LAYOUT: vk::DescriptorSetLayoutCreateInfo = vk::DescriptorSetLayoutCreateInfo {{
    s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
    p_next: std::ptr::null(),
    flags: vk::DescriptorSetLayoutCreateFlags::empty(),
    binding_count: {binding_count},
    p_bindings: &raw const DESCRIPTOR_SET_BINDINGS[0],
    _marker: std::marker::PhantomData,
}};

static DESCRIPTOR_POOL_SIZES: [vk::DescriptorPoolSize; {pool_size_count}] = [",
    )
    .unwrap();
    for (binding_type, size) in &pool_sizes {
        let binding_type = binding_type.ash_variant();
        writeln!(
            file,
            r"    vk::DescriptorPoolSize {{
        ty: vk::DescriptorType::{binding_type},
        descriptor_count: {size},
    }},"
        )
        .unwrap();
    }
    let max_sets = 2;
    let pool_size_count = pool_sizes.len();
    writeln!(
        file,
        r#"];

static DESCRIPTOR_POOL: vk::DescriptorPoolCreateInfo = vk::DescriptorPoolCreateInfo {{
    s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
    p_next: std::ptr::null(),
    flags: vk::DescriptorPoolCreateFlags::empty(),
    max_sets: {max_sets},
    pool_size_count: {pool_size_count},
    p_pool_sizes: &raw const DESCRIPTOR_POOL_SIZES[0],
    _marker: std::marker::PhantomData,
}};

pub fn alloc_descriptor_set("#
    )
    .unwrap();
    for binding in &descriptor_set.bindings {
        let name = &binding.name;
        let typ = binding.ash_value_type();
        writeln!(file, "    {name}: {typ},").unwrap();
    }
    write!(
        file,
        r#"    dev: &Dev,
    layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
) -> [vk::DescriptorSet; FRAMES_IN_FLIGHT] {{
    let layouts = [layout; FRAMES_IN_FLIGHT];
    let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let descriptors: [vk::DescriptorSet; FRAMES_IN_FLIGHT] =
        unsafe {{ dev.allocate_descriptor_sets(&descriptor_set_alloc_info) }}
            .unwrap()
            .try_into()
            .unwrap();
    update_descriptor_set(&descriptors"#
    )
    .unwrap();
    for binding in &descriptor_set.bindings {
        let name = &binding.name;
        write!(file, ", {name}").unwrap();
    }

    writeln!(
        file,
        r#", dev);
    descriptors
}}

#[allow(clippy::unused_enumerate_index)]
pub fn update_descriptor_set(
    descriptors: &[vk::DescriptorSet; FRAMES_IN_FLIGHT],"#
    )
    .unwrap();
    let mut only_tlas = None;
    for binding in &descriptor_set.bindings {
        let name = &binding.name;
        let typ = binding.ash_value_type();
        writeln!(file, "        {name}: {typ},").unwrap();
        if binding.descriptor_type == ReflectDescriptorType::AccelerationStructureKHR {
            assert!(only_tlas.is_none());
            assert_eq!(binding.binding as usize, descriptor_set.bindings.len() - 1);
            only_tlas = Some(name);
        }
    }
    writeln!(
        file,
        r#"    dev: &Dev,
    ) {{"#
    )
    .unwrap();
    if let Some(tlas) = only_tlas.as_ref() {
        writeln!(file, r#"    let supports_raytracing = {tlas}.is_some();"#).unwrap();
    }
    writeln!(
        file,
        r#"    for (_flight_index, descriptor) in descriptors.iter().enumerate() {{"#
    )
    .unwrap();
    for binding in &descriptor_set.bindings {
        let binding_index = binding.binding;
        let binding_name = &binding.name;
        let binding_type = binding.descriptor_type.ash_variant();
        let write_mutable = match binding.descriptor_type {
            ReflectDescriptorType::AccelerationStructureKHR => "mut ",
            _ => "",
        };
        match binding.descriptor_type {
            ReflectDescriptorType::AccelerationStructureKHR => writeln!(
                file,
                r#"        let mut {binding_name}_acceleration_structure = *vk::WriteDescriptorSetAccelerationStructureKHR::default()
            .acceleration_structures({binding_name}.as_ref().map(|as_| std::slice::from_ref(&as_.acceleration_structure)).unwrap_or_default());"#
            )
                .unwrap(),
            ReflectDescriptorType::SampledImage => {
                todo!()
                //     let layout = &image.layout;
                //     writeln!(
                //         file,
                //         r#"        let {binding_name}_image = *vk::DescriptorImageInfo::default()
                // .image_layout(vk::ImageLayout::{layout})
                // .image_view({binding_name});"#
                //     )
                //         .unwrap()
            }
            ReflectDescriptorType::InputAttachment => writeln!(
                file,
                r#"        let {binding_name}_image = *vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view({binding_name});"#
            )
                .unwrap(),
            ReflectDescriptorType::StorageBuffer => writeln!(file, r#"        let {binding_name}_buffer = {binding_name}.descriptor(_flight_index);"#).unwrap(),
            ReflectDescriptorType::StorageImage => writeln!(file,
                                                            r#"        let {binding_name}_image = *vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view({binding_name});"#
            ).unwrap(),
            ReflectDescriptorType::UniformBuffer => writeln!(
                file,
                r#"        let {binding_name}_buffer = {binding_name}.descriptor(_flight_index);"#
            )
                .unwrap(),
            _ => unimplemented!(),
        }
        writeln!(
            file,
            r#"        let {write_mutable}{binding_name} = vk::WriteDescriptorSet::default()
            .dst_set(*descriptor)
            .dst_binding({binding_index})
            .descriptor_type(vk::DescriptorType::{binding_type})"#
        )
        .unwrap();
        match binding.descriptor_type {
            ReflectDescriptorType::AccelerationStructureKHR => writeln!(
                file,
                r#"            .push_next(&mut {binding_name}_acceleration_structure);
        {binding_name}.descriptor_count = 1;"#
            )
            .unwrap(),
            ReflectDescriptorType::SampledImage
            | ReflectDescriptorType::InputAttachment
            | ReflectDescriptorType::StorageImage => writeln!(
                file,
                r#"            .image_info(std::slice::from_ref(&{binding_name}_image));"#
            )
            .unwrap(),
            ReflectDescriptorType::StorageBuffer => writeln!(
                file,
                r#"            .buffer_info(std::slice::from_ref(&{binding_name}_buffer));"#
            )
            .unwrap(),
            ReflectDescriptorType::UniformBuffer => writeln!(
                file,
                r#"            .buffer_info(std::slice::from_ref(&{binding_name}_buffer));"#
            )
            .unwrap(),
            _ => unimplemented!(),
        }
    }
    let write_writes = |file: &mut File, bindings: &[ReflectDescriptorBinding]| {
        write!(file, r"[").unwrap();
        for (binding_index, binding) in bindings.iter().enumerate() {
            let binding_name = &binding.name;
            write!(file, "{binding_name}").unwrap();
            if binding_index != bindings.len() - 1 {
                write!(file, ", ").unwrap();
            }
        }
        write!(file, "]").unwrap();
    };
    write!(file, r#"        let writes = "#).unwrap();
    write_writes(&mut file, &descriptor_set.bindings);
    writeln!(file, r#";"#).unwrap();
    if only_tlas.is_some() {
        let count_without_raytracing = descriptor_set.bindings.len() - 1;
        writeln!(
            file,
            r#"        let writes = if supports_raytracing {{
            &writes
        }} else {{
            &writes[..{count_without_raytracing}]
        }};"#
        )
        .unwrap();
    } else {
        writeln!(file, "        let writes = &writes;").unwrap();
    }
    writeln!(
        file,
        r#"        unsafe {{ dev.update_descriptor_sets(writes, &[]) }};
    }}
}}

pub fn create_descriptor_set_layout(_samplers: &Samplers, dev: &Dev) -> vk::DescriptorSetLayout {{"#
    )
    .unwrap();
    for binding in &descriptor_set.bindings {
        // let binding_index = binding.binding;
        if binding.descriptor_type == ReflectDescriptorType::SampledImage {
            // writeln!(
            //     file,
            //     "    unsafe {{ SCRATCH.descriptor_set_bindings[{binding_index}].p_immutable_samplers = &_samplers.{} }};",
            //     image.sampler,
            // )
            //     .unwrap();
            todo!()
        }
    }
    writeln!(file, "    unsafe {{ dev.create_descriptor_set_layout(&DESCRIPTOR_SET_LAYOUT, None).unwrap_unchecked() }}").unwrap();
    writeln!(
        file,
        r#"}}

pub fn create_descriptor_pool(dev: &Dev) -> vk::DescriptorPool {{
    unsafe {{ dev.create_descriptor_pool(&DESCRIPTOR_POOL, None).unwrap_unchecked() }}
}}"#
    )
    .unwrap();
}
