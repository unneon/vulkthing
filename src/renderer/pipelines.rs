use crate::renderer::util::Dev;
use crate::renderer::{Swapchain, DEPTH_FORMAT};
use ash::vk;
use std::ffi::c_void;
use std::mem::MaybeUninit;

#[repr(align(4))]
struct SpvArray<const N: usize>(pub [u8; N]);

include!(concat!(env!("OUT_DIR"), "/pipelines.rs"));
