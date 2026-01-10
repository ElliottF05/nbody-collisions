use bytemuck::{Pod, Zeroable};

use crate::simulation::vec2::Vec2;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Body {
    pub position: Vec2,
    pub velocity: Vec2,
    pub mass: f32,
    pub radius: f32,
}

impl Body {
    pub fn new(position: Vec2, velocity: Vec2, mass: f32, radius: f32) -> Self {
        Body {
            position,
            velocity,
            mass,
            radius,
        }
    }
}