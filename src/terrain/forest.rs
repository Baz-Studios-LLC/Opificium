//! Where the trees are.
//!
//! Two answers, added together.
//!
//! The first is worked out from the ground itself: trees want moisture, ground
//! below the treeline, a slope they can hold on to, and somewhere that is not a
//! beach, a road, or the levelled ground under a town. Nobody hand-plants
//! sixteen square kilometres, so this does the whole map and does it the same
//! way in both programs.
//!
//! The second is what a maker painted on top — a grid of signed bias, saved
//! beside the world's other files. **Zero means leave the automatic answer
//! alone**, which is the same choice the sculpting layer makes and for the same
//! reason: re-tune the automatic placement afterwards and hand-planted woods
//! stay exactly where they were put.
//!
//! # No list of trees is ever written down
//!
//! Trees are scattered from a hash of position. Given the same ground and the
//! same painted layer, this program and the game plant the identical forest
//! without a single tree passing between them — the same trick the towns use.
//! A list would be megabytes, would go stale the moment the ground moved, and
//! would have to be merged when two people edited it.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use bevy::log::{info, warn};
use bevy::prelude::*;

use crate::terrain::edit::smoothstep;

/// Names the file, so a stale or unrelated one is refused.
const MAGIC: &[u8; 8] = b"RNGRFST1";

/// What the painted layer is called, beside the world's other files.
pub const FOREST: &str = "forest.bin";

/// Metres per cell of the painted layer. Coarser than the sculpting grid: a
/// wood is a region, not a contour, and this keeps a world's worth under a
/// megabyte.
pub const CELL: f32 = 16.0;

/// Below this, a cell is untouched and the automatic answer stands.
const PAINTED_EPSILON: f32 = 0.01;


/// One tree, ready to plant.
pub struct Planted {
    pub at: Vec3,
    /// Which of the grown pool this is.
    pub variety: usize,
    /// Turned about its own trunk, so neighbours of one variety do not line up.
    pub turn: f32,
    /// Scaled, so a stand has young trees and old ones in it.
    pub scale: f32,
}

/// The painted layer.
pub struct Painted {
    wide: usize,
    deep: usize,
    half: Vec2,
    /// Signed bias in -1..=1. Positive plants, negative clears, zero defers to
    /// the automatic answer.
    bias: Vec<f32>,
    planted: usize,
    pub unsaved: bool,
}

impl Painted {
    pub fn empty(half: Vec2) -> Self {
        let wide = (half.x * 2.0 / CELL).ceil() as usize + 1;
        let deep = (half.y * 2.0 / CELL).ceil() as usize + 1;
        Self {
            wide,
            deep,
            half,
            bias: vec![0.0; wide * deep],
            planted: 0,
            unsaved: false,
        }
    }

    pub fn load(folder: &Path, half: Vec2) -> Self {
        Self::load_from(&folder.join(FOREST), half)
    }

    pub fn load_from(path: &Path, half: Vec2) -> Self {
        let mut empty = Self::empty(half);
        if !path.exists() {
            return empty;
        }
        let Ok(bytes) = fs::read(path) else {
            warn!("{}: unreadable - taking the ground's own answer", path.display());
            return empty;
        };

        let header = 8 + 4 * 4;
        if bytes.len() < header || &bytes[..8] != MAGIC {
            warn!("{} is not a painted forest - ignoring it", path.display());
            return empty;
        }
        let word = |at: usize| {
            u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]) as usize
        };
        let real =
            |at: usize| f32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
        let (wide, deep) = (word(8), word(12));
        let saved_half = Vec2::new(real(16), real(20));

        // Refused rather than stretched, for the same reason the sculpting is:
        // woods landing in the wrong places is worse than none, and nothing on
        // screen would say why.
        if wide != empty.wide || deep != empty.deep || saved_half.distance(half) > 1.0 {
            warn!(
                "{} was painted for a {:.0}x{:.0} m world, not this {:.0}x{:.0} m one - ignoring it",
                path.display(),
                saved_half.x * 2.0,
                saved_half.y * 2.0,
                half.x * 2.0,
                half.y * 2.0
            );
            return empty;
        }
        if bytes.len() < header + wide * deep * 4 {
            warn!("{} is truncated - ignoring it", path.display());
            return empty;
        }

        empty.bias = (0..wide * deep).map(|i| real(header + i * 4)).collect();
        empty.planted = empty
            .bias
            .iter()
            .filter(|v| v.abs() > PAINTED_EPSILON)
            .count();
        info!(
            "painted forest: {} cells from {}",
            empty.planted,
            path.display()
        );
        empty
    }

    pub fn save(&mut self, folder: &Path) -> io::Result<PathBuf> {
        let path = folder.join(FOREST);
        if let Some(above) = path.parent() {
            fs::create_dir_all(above)?;
        }
        let mut file = fs::File::create(&path)?;
        file.write_all(MAGIC)?;
        file.write_all(&(self.wide as u32).to_le_bytes())?;
        file.write_all(&(self.deep as u32).to_le_bytes())?;
        file.write_all(&self.half.x.to_le_bytes())?;
        file.write_all(&self.half.y.to_le_bytes())?;
        for value in &self.bias {
            file.write_all(&value.to_le_bytes())?;
        }
        file.flush()?;
        self.unsaved = false;
        Ok(path)
    }

    pub fn painted_cells(&self) -> usize {
        self.planted
    }

    /// The bias at a world position, read between cells.
    pub fn at(&self, x: f32, z: f32) -> f32 {
        let fx = (x + self.half.x) / CELL;
        let fz = (z + self.half.y) / CELL;
        if fx < 0.0 || fz < 0.0 || fx > (self.wide - 1) as f32 || fz > (self.deep - 1) as f32 {
            return 0.0;
        }
        let x0 = fx.floor() as usize;
        let z0 = fz.floor() as usize;
        let x1 = (x0 + 1).min(self.wide - 1);
        let z1 = (z0 + 1).min(self.deep - 1);
        let tx = fx - x0 as f32;
        let tz = fz - z0 as f32;
        let at = |x: usize, z: usize| self.bias[z * self.wide + x];
        let near = at(x0, z0) * (1.0 - tx) + at(x1, z0) * tx;
        let far = at(x0, z1) * (1.0 - tx) + at(x1, z1) * tx;
        near * (1.0 - tz) + far * tz
    }

    /// Paints, positive to plant and negative to clear. Returns the ground it
    /// changed, so the trees standing there can be grown again.
    pub fn paint(&mut self, centre: Vec2, radius: f32, amount: f32) -> Rect {
        let to_cell = |v: f32, half: f32, count: usize| {
            (((v + half) / CELL).floor() as isize).clamp(0, count as isize - 1) as usize
        };
        let x0 = to_cell(centre.x - radius, self.half.x, self.wide);
        let x1 = to_cell(centre.x + radius + CELL, self.half.x, self.wide);
        let z0 = to_cell(centre.y - radius, self.half.y, self.deep);
        let z1 = to_cell(centre.y + radius + CELL, self.half.y, self.deep);

        for z in z0..=z1 {
            for x in x0..=x1 {
                let at = Vec2::new(
                    x as f32 * CELL - self.half.x,
                    z as f32 * CELL - self.half.y,
                );
                let away = at.distance(centre);
                if away > radius {
                    continue;
                }
                let falloff = smoothstep(radius, 0.0, away);
                let cell = z * self.wide + x;
                let was = self.bias[cell].abs() > PAINTED_EPSILON;
                let now = (self.bias[cell] + amount * falloff).clamp(-1.0, 1.0);
                let is = now.abs() > PAINTED_EPSILON;
                match (was, is) {
                    (false, true) => self.planted += 1,
                    (true, false) => self.planted -= 1,
                    _ => {}
                }
                self.bias[cell] = now;
            }
        }
        self.unsaved = true;
        Rect::from_corners(centre - (radius + CELL), centre + (radius + CELL))
    }
}

/// What the ground alone says about trees here, 0 to 1.
///
/// Every one of these is a reason a wood would or would not be standing:
/// too dry, too high, too steep, too close to the sea, or ground somebody has
/// already levelled to build on.
#[allow(clippy::too_many_arguments)]
pub fn natural_density(
    moisture: f32,
    height: f32,
    slope: f32,
    shore: f32,
    levelled: f32,
    treeline: f32,
) -> f32 {
    // Not in the sea, and not on the beach where the sand is.
    if shore < 25.0 {
        return 0.0;
    }
    // Moisture is the main say: dry plains, then scrub, then closed woodland.
    let wet = smoothstep(0.34, 0.62, moisture);
    // Thinning out toward the treeline and gone above it.
    let low = 1.0 - smoothstep(treeline * 0.72, treeline, height);
    // Trees hold a slope, but not a cliff.
    let standable = 1.0 - smoothstep(0.42, 0.72, slope);
    // Ground levelled for a town or a road is ground somebody cleared.
    let clear = 1.0 - levelled;

    wet * low * standable * clear
}

/// Combines the ground's answer with what was painted over it.
///
/// Zero bias leaves the ground's answer untouched; the ends of the range force
/// the question either way, so a maker can put a wood on a hilltop or clear one
/// off a plain without arguing with the generator.
pub fn density(natural: f32, painted: f32) -> f32 {
    if painted >= 0.0 {
        natural + (1.0 - natural) * painted
    } else {
        natural * (1.0 + painted)
    }
}

/// A repeatable 0..1 from a place and a purpose.
///
/// Position-hashed rather than drawn in sequence, so a chunk can be planted on
/// its own, in any order, on any thread, in either program, and get the same
/// trees.
pub fn chance(x: i32, z: i32, salt: u32) -> f32 {
    let mut h = (x as u32)
        .wrapping_mul(0x8da6_b343)
        .wrapping_add((z as u32).wrapping_mul(0xd8163841))
        .wrapping_add(salt.wrapping_mul(0xcb1a_b31f));
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
    h as f32 / u32::MAX as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    const HALF: Vec2 = Vec2::new(800.0, 600.0);

    #[test]
    fn nothing_grows_in_the_sea_or_on_the_beach() {
        assert_eq!(natural_density(1.0, 2.0, 0.0, -50.0, 0.0, 200.0), 0.0);
        assert_eq!(natural_density(1.0, 2.0, 0.0, 5.0, 0.0, 200.0), 0.0);
    }

    #[test]
    fn woods_want_moisture_and_gentle_ground_below_the_treeline() {
        let good = natural_density(0.9, 40.0, 0.1, 500.0, 0.0, 200.0);
        assert!(good > 0.6, "a wet gentle lowland should be wooded: {good}");

        for (why, thin) in [
            ("dry", natural_density(0.1, 40.0, 0.1, 500.0, 0.0, 200.0)),
            ("high", natural_density(0.9, 205.0, 0.1, 500.0, 0.0, 200.0)),
            ("steep", natural_density(0.9, 40.0, 0.9, 500.0, 0.0, 200.0)),
            ("levelled", natural_density(0.9, 40.0, 0.1, 500.0, 1.0, 200.0)),
        ] {
            assert!(thin < good * 0.35, "{why} ground should be far barer: {thin}");
        }
    }

    #[test]
    fn painting_forces_the_question_either_way() {
        // The point of the layer: a maker overrules the generator rather than
        // nudging it, so a wood can go on a hilltop and a plain can be cleared.
        assert!(density(0.0, 1.0) > 0.99, "painting should plant bare ground");
        assert!(density(1.0, -1.0) < 0.01, "clearing should empty a wood");
        // And zero is untouched, which is the whole reason it is a bias.
        for natural in [0.0, 0.25, 0.5, 0.75, 1.0] {
            assert_eq!(density(natural, 0.0), natural);
        }
    }

    #[test]
    fn painting_reads_back_where_it_was_put_and_survives_saving() {
        let folder = std::env::temp_dir().join("opificium-forest-test");
        let _ = fs::create_dir_all(&folder);
        let _ = fs::remove_file(folder.join(FOREST));

        let mut painted = Painted::empty(HALF);
        let patch = painted.paint(Vec2::new(100.0, -50.0), 80.0, 1.0);
        assert!(patch.width() > 0.0);
        assert!(painted.painted_cells() > 0);
        assert!(painted.at(100.0, -50.0) > 0.9, "the middle should be planted");
        assert_eq!(painted.at(600.0, 400.0), 0.0, "far ground untouched");

        painted.save(&folder).expect("should save");
        assert!(!painted.unsaved);

        let read = Painted::load(&folder, HALF);
        assert_eq!(read.painted_cells(), painted.painted_cells());
        assert!((read.at(100.0, -50.0) - painted.at(100.0, -50.0)).abs() < 1.0e-5);

        // A layer painted for another world is refused, not stretched.
        assert_eq!(Painted::load(&folder, HALF * 2.0).painted_cells(), 0);
        let _ = fs::remove_file(folder.join(FOREST));
    }

    #[test]
    fn clearing_takes_back_what_planting_put_down() {
        let mut painted = Painted::empty(HALF);
        painted.paint(Vec2::ZERO, 60.0, 1.0);
        painted.paint(Vec2::ZERO, 60.0, -1.0);
        assert!(painted.at(0.0, 0.0).abs() < PAINTED_EPSILON);
        assert_eq!(painted.painted_cells(), 0, "no cell left counted as painted");
    }

    #[test]
    fn the_same_place_always_gets_the_same_answer() {
        // Both programs plant from this and never exchange a list of trees.
        for (x, z) in [(0, 0), (17, -400), (-2000, 900)] {
            assert_eq!(chance(x, z, 1), chance(x, z, 1));
            assert_ne!(chance(x, z, 1), chance(x, z, 2), "salt should matter");
        }
        // And neighbouring slots must not march in step, or the forest comes out
        // in rows.
        let row: Vec<f32> = (0..12).map(|x| chance(x, 0, 1)).collect();
        let rising = row.windows(2).filter(|w| w[1] > w[0]).count();
        assert!(
            (2..=10).contains(&rising),
            "the scatter is marching in order: {row:?}"
        );
    }
}
