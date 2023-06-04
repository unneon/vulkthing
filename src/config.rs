use crate::renderer::uniform::FragSettings;

pub const DEFAULT_FRAG_SETTINGS: FragSettings = FragSettings {
    use_ray_tracing: true,
    _pad0: [0; 3],
};
