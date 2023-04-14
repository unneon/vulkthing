use ash::vk;

pub trait VertexOps {
    const ATTRIBUTE_FORMATS: &'static [vk::Format];
    const ATTRIBUTE_SIZES: &'static [usize];

    fn attribute_descriptions(binding: u32) -> Vec<vk::VertexInputAttributeDescription> {
        assert_eq!(Self::ATTRIBUTE_FORMATS.len(), Self::ATTRIBUTE_SIZES.len());
        let attribute_count = Self::ATTRIBUTE_FORMATS.len();
        let mut offset = 0;
        let mut descriptions = Vec::new();
        for i in 0..attribute_count {
            descriptions.push(vk::VertexInputAttributeDescription {
                binding,
                location: i as u32,
                format: Self::ATTRIBUTE_FORMATS[i],
                offset: offset as u32,
            });
            offset += Self::ATTRIBUTE_SIZES[i];
        }
        descriptions
    }
}
