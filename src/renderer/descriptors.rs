use crate::renderer::samplers::Samplers;
use crate::renderer::util::AsDescriptor;
use crate::renderer::{Dev, StorageBuffer, UniformBuffer, FRAMES_IN_FLIGHT};
use ash::vk;

include!(concat!(env!("OUT_DIR"), "/descriptors.rs"));
