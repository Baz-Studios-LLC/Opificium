//! Where the trees are, as this bench keeps them.
//!
//! Two answers, added together.
//!
//! The first is worked out from the ground itself: trees want moisture, ground
//! below the treeline, a slope they can hold on to, and somewhere that is not a
//! beach, a road, or the levelled ground under a town. Nobody hand-plants
//! sixteen square kilometres, so this does the whole map.
//!
//! The second is what a maker painted on top — a grid of signed bias, saved
//! beside the world's other files as `forest.bin`. **Zero means leave the
//! automatic answer alone**, which is the same choice the sculpting layer makes
//! and for the same reason: re-tune the automatic placement afterwards and
//! hand-planted woods stay exactly where they were put.
//!
//! # Both answers live in the crate
//!
//! [`terrain_core::forest`] has them, and the games whose ground this bench
//! shapes link it too. No list of trees is ever written down — they scatter from
//! a hash of position, so a bench and a game plant the identical forest without
//! a single tree passing between them.
//!
//! That used to be two copies of one algorithm, held together by tests pinning
//! literal numbers copied from one program into the other. Every salt, every
//! multiplier and every rejection rule had to match exactly, and a digit out of
//! place moved every wood in the world with nothing to say it had.
//!
//! What is left here is knowing that a world's woods live in that world's
//! folder, and saying so when something is wrong with them.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use bevy::log::{info, warn};
use bevy::prelude::*;

pub use terrain_core::forest::{chance, density, natural_density, Painted, Planted};

/// What the painted layer is called, beside the world's other files.
pub const FOREST: &str = "forest.bin";

/// Where the woods are kept, beside the world they belong to.
pub fn road(folder: &Path) -> PathBuf {
    folder.join(FOREST)
}

pub fn load(folder: &Path, half: Vec2) -> Painted {
    load_from(&road(folder), half)
}

/// Reads the painted woods, or an empty layer if there are none.
pub fn load_from(road: &Path, half: Vec2) -> Painted {
    if !road.exists() {
        // The ordinary case for a world nobody has planted. Not news.
        return Painted::empty(half);
    }

    match fs::read(road) {
        Ok(bytes) => match Painted::read(&bytes, half) {
            Ok(painted) => {
                info!(
                    "planted woods: {} cells from {}",
                    painted.painted_cells(),
                    road.display()
                );
                painted
            }
            // Refused rather than stretched: woods landing in the wrong places
            // is worse than none, and nothing on screen would say why.
            Err(why) => {
                warn!("{}: {why} - opening with nothing planted", road.display());
                Painted::empty(half)
            }
        },
        Err(why) => {
            warn!("{}: {why} - opening with nothing planted", road.display());
            Painted::empty(half)
        }
    }
}

pub fn save(folder: &Path, painted: &Painted) -> io::Result<PathBuf> {
    let road = road(folder);
    if let Some(above) = road.parent() {
        fs::create_dir_all(above)?;
    }
    fs::write(&road, painted.to_bytes())?;
    Ok(road)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HALF: Vec2 = Vec2::new(800.0, 600.0);

    #[test]
    fn planting_survives_the_bench_being_shut() {
        // This had a writer, a passing round-trip test, and nothing calling it,
        // so an afternoon's planting went away on restart. The test that would
        // have caught it goes through the bench's own reader and writer.
        let folder = std::env::temp_dir().join("opificium-forest-roundtrip");
        let _ = fs::remove_file(road(&folder));

        let mut painted = Painted::empty(HALF);
        painted.paint(Vec2::new(100.0, -50.0), 80.0, 1.0);
        save(&folder, &painted).expect("it should write");

        let read = load(&folder, HALF);
        assert_eq!(read.painted_cells(), painted.painted_cells());
        assert!(
            (read.at(100.0, -50.0) - painted.at(100.0, -50.0)).abs() < 1.0e-5,
            "the wood should come back where it was planted"
        );

        // Woods painted for a different world must be refused, not stretched.
        assert_eq!(load(&folder, HALF * 2.0).painted_cells(), 0);

        let _ = fs::remove_file(road(&folder));
    }
}
