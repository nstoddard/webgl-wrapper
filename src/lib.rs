//! A stateless wrapper around WebGL 2, to make it easier to use and more type-safe.

#![deny(bare_trait_objects)]

mod context;
mod framebuffer;
mod mesh;
mod program;
mod rect;
mod surface;
mod texture;
pub mod uniforms;

pub use crate::context::*;
pub use crate::framebuffer::*;
pub use crate::mesh::*;
pub use crate::program::*;
pub use crate::rect::*;
pub use crate::surface::*;
pub use crate::texture::*;
pub use uniforms::{GlUniforms, Uniforms};
