//! The ground a maker put there, as this bench keeps it.
//!
//! Generated terrain gets you a plausible landscape; it does not get you THIS
//! hill, HERE. Authored geography is a grid of signed height offsets in metres,
//! laid over whatever the map and the noise produced, and written beside the
//! world it belongs to as `edits.bin`.
//!
//! # The brush is not here
//!
//! It is in [`terrain_core::sculpt`], which this bench and the games whose
//! ground it shapes **both** link. Ground shaped here and ground shaped there is
//! shaped the same way, by construction rather than by agreement.
//!
//! It was here once, and the game had a reader for what it wrote. Then the game
//! grew a sculpting mode of its own — an editor built on top of its runtime, the
//! way the studios do it — and there were two brushes. Two of anything is two
//! things to keep in step by hand, and the world generation had already taught
//! us how that ends: no error, nothing failing, just a bench with one world and
//! a game with another.
//!
//! What is left here is what the crate deliberately does not do: knowing that a
//! world's ground lives in that world's folder, and saying so when something is
//! wrong with it. The crate reads and writes bytes; naming the file is the
//! bench's business.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use bevy::log::{info, warn};
use bevy::prelude::*;

pub use terrain_core::sculpt::{Brushing, Patch, Sculpt, Stamp, CELL};
pub use terrain_core::smoothstep;

/// What the sculpted file is called, inside the world's folder.
pub const EDITS: &str = "edits.bin";

/// Where the sculpted ground is kept, beside the world it belongs to.
pub fn road(folder: &Path) -> PathBuf {
    folder.join(EDITS)
}

pub fn load(folder: &Path, half: Vec2, seed: u32) -> Sculpt {
    load_from(&road(folder), half, seed)
}

/// Reads sculpted ground, or an empty layer if there is none.
///
/// Every way this can go wrong ends the same — the world exactly as generated —
/// so the only real work is saying WHICH went wrong.
pub fn load_from(road: &Path, half: Vec2, seed: u32) -> Sculpt {
    if !road.exists() {
        // The ordinary case for a world nobody has sculpted. Not news.
        return Sculpt::empty(half, seed);
    }

    match fs::read(road) {
        Ok(bytes) => match Sculpt::read(&bytes, half, seed) {
            Ok(sculpt) => {
                info!(
                    "sculpted ground: {} cells from {}",
                    sculpt.sculpted_cells(),
                    road.display()
                );
                sculpt
            }
            // Refused rather than stretched: a maker would otherwise see their
            // afternoon's work smeared across the map, with nothing to undo it.
            Err(why) => {
                warn!("{}: {why} - opening with no edits", road.display());
                Sculpt::empty(half, seed)
            }
        },
        Err(why) => {
            warn!("{}: {why} - opening with no edits", road.display());
            Sculpt::empty(half, seed)
        }
    }
}

pub fn save(folder: &Path, sculpt: &mut Sculpt) -> io::Result<PathBuf> {
    let road = road(folder);
    save_to(&road, sculpt)?;
    Ok(road)
}

pub fn save_to(road: &Path, sculpt: &mut Sculpt) -> io::Result<()> {
    if let Some(above) = road.parent() {
        fs::create_dir_all(above)?;
    }
    fs::write(road, sculpt.to_bytes())?;
    // Only once the bytes have actually landed — the crate has no way to know
    // whether they did, so it doesn't guess.
    sculpt.mark_saved();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const HALF: Vec2 = Vec2::new(400.0, 300.0);
    const SEED: u32 = 7;

    fn flat(_: Vec2) -> f32 {
        0.0
    }

    #[test]
    fn sculpting_survives_the_bench_being_shut() {
        // The woods once had a writer, a passing round-trip test, and nothing
        // calling it, so an afternoon's planting went away on restart. This is
        // that path end to end: sculpt, save through the bench's own writer, and
        // read it back through the bench's own reader.
        let folder = std::env::temp_dir().join("opificium-sculpt-roundtrip");
        let _ = fs::remove_file(road(&folder));

        let mut kept = Sculpt::empty(HALF, SEED);
        kept.apply(&Stamp {
            centre: Vec2::new(-100.0, 50.0),
            radius: 70.0,
            how: Brushing::Raise,
            amount: 18.0,
            target: 0.0,
            under: &flat,
        });
        save(&folder, &mut kept).expect("it should write");
        assert!(!kept.unsaved, "writing clears the unsaved mark");

        let read = load(&folder, HALF, SEED);
        assert_eq!(read.sculpted_cells(), kept.sculpted_cells());
        assert!(
            (read.at(-100.0, 50.0) - kept.at(-100.0, 50.0)).abs() < 1.0e-5,
            "the hill should come back where it was put"
        );

        // Ground sculpted for a different world must be refused, not stretched.
        assert_eq!(load(&folder, HALF * 2.0, SEED).sculpted_cells(), 0);

        let _ = fs::remove_file(road(&folder));
    }

    #[test]
    fn an_absent_or_foreign_file_opens_with_no_edits() {
        assert_eq!(
            load_from(Path::new("no/such/edits.bin"), HALF, SEED).sculpted_cells(),
            0
        );

        let road = std::env::temp_dir().join("opificium-edits-foreign.bin");
        fs::write(&road, b"this is not sculpted ground at all").unwrap();
        assert_eq!(load_from(&road, HALF, SEED).sculpted_cells(), 0);
        let _ = fs::remove_file(&road);
    }
}
