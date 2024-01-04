#![no_std]
#![warn(missing_docs)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

extern crate alloc;

mod tileset;
pub use tileset::*;
mod tilemap;
pub use tilemap::*;

pub use rgb;
#[doc(no_inline)]
pub use rgb::RGBA8 as Color;
#[doc(no_inline)]
pub use simple_blit::{Buffer, BufferMut, BlitOptions};
