use bytemuck::{Pod, Zeroable};

/// Uniforms struct for passing data to shaders.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Uniforms {
    pub view_port: [u32; 2],
    pub cam_center: [f32; 2],
    pub cam_half_size: [f32; 2],
    pub num_bodies: u32,
    pub _pad: u32,
}