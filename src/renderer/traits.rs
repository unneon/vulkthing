use ash::vk;

pub trait VertexOps {
    const ATTRIBUTE_COUNT: usize;
    const ATTRIBUTE_FORMATS: &'static [vk::Format];
    const ATTRIBUTE_SIZES: &'static [usize];

    fn attribute_descriptions(
        binding: u32,
    ) -> [vk::VertexInputAttributeDescription; Self::ATTRIBUTE_COUNT] {
        assert_eq!(Self::ATTRIBUTE_COUNT, Self::ATTRIBUTE_FORMATS.len());
        assert_eq!(Self::ATTRIBUTE_COUNT, Self::ATTRIBUTE_SIZES.len());
        let mut offset = 0;
        let mut descriptions = [Default::default(); Self::ATTRIBUTE_COUNT];
        for i in 0..Self::ATTRIBUTE_COUNT {
            descriptions[i] = vk::VertexInputAttributeDescription {
                binding,
                location: i as u32,
                format: Self::ATTRIBUTE_FORMATS[i],
                offset: offset as u32,
            };
            offset += Self::ATTRIBUTE_SIZES[i];
        }
        descriptions
    }
}
