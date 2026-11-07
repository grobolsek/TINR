//! What things look like. Only data: the platform crate does the drawing.

use bevy_ecs::prelude::*;

use crate::Vec2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Shape {
    #[default]
    Rect,
    Circle,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Sprite {
    pub shape: Shape,
    pub size: Vec2,
    pub radius: f32,
    /// Red, green, blue, 0-255.
    pub color: [u8; 3],
    /// Higher layers are drawn on top.
    pub layer: i32,
    /// Part of the atlas image to draw, in pixels: [x, y, w, h]. None = draw `shape` in `color`.
    pub region: Option<[f32; 4]>,
    /// Drawn this far from the entity's position, in screen pixels (+y is down).
    pub offset: Vec2,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            shape: Shape::Rect,
            size: Vec2::new(32.0, 32.0),
            radius: 16.0,
            color: [255, 0, 0],
            layer: 0,
            region: None,
            offset: Vec2::ZERO,
        }
    }
}

impl Sprite {
    pub fn rect(w: f32, h: f32, color: [u8; 3]) -> Self {
        Self { shape: Shape::Rect, size: Vec2::new(w, h), color, ..Default::default() }
    }

    /// A picture cut from the atlas at pixel rect (x, y, w, h), drawn at its own size.
    /// Change `size` afterwards to scale it.
    pub fn atlas(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { size: Vec2::new(w, h), region: Some([x, y, w, h]), ..Default::default() }
    }

    pub fn circle(radius: f32, color: [u8; 3]) -> Self {
        Self { shape: Shape::Circle, radius, color, ..Default::default() }
    }
}
