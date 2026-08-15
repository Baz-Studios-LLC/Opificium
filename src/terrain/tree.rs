//! Growing a tree, as this bench draws it.
//!
//! A trunk that tapers, limbs that fork off it at their own angles, and leaves
//! massed at the ends — grown from a seed, so twenty seeds are twenty different
//! trees and a wood is not one tree stamped a thousand times.
//!
//! # The growing is in the crate
//!
//! [`terrain_core::tree`] does it, and the games whose ground this bench shapes
//! link the same code. It has to be the same code: a tree is grown from a hashed
//! stream of numbers, and the ORDER the numbers are drawn in is the tree. Two
//! copies with two lines swapped grow two different woods from one seed, with no
//! error and nothing failing.
//!
//! What is left here is the seam: geometry comes out of the crate as plain
//! vertex arrays, because the crate names no engine, and this turns those into
//! Bevy meshes. A dozen lines, and the only engine-shaped thing in the whole
//! arrangement — the game has its own dozen against its own Bevy.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

pub use terrain_core::tree::{grow, VARIETIES};

/// Vertex arrays into a mesh this bench's renderer can draw.
pub fn as_mesh(geometry: &terrain_core::Geometry) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        // Drawn, never read back.
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, geometry.places.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, geometry.normals.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, geometry.uvs.clone())
    .with_inserted_indices(Indices::U32(geometry.indices.clone()))
}
