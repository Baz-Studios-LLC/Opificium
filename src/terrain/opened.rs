//! Which world the bench has open, and how it is chosen.
//!
//! **A world is not a project.** The other benches work on one game's authored
//! content and are pointed at it once, when the bench opens. This one is a TOOL:
//! you bring it a world and shape it, the way you bring an image to the kiln.
//! Any folder holding a `heightmap.png` is a world, whoever's it is and wherever
//! it lives, and choosing one has nothing to do with which game the bench opened.
//!
//! The last one is remembered, so a maker who spent yesterday on a coastline is
//! back on it today without going looking. Remembered in the BENCH's own corner
//! of the machine, never in the world's folder — which world was last open is
//! the bench's business and not that world's.

use std::path::{Path, PathBuf};

use bevy::log::warn;
use serde::{Deserialize, Serialize};

use crate::terrain::ground::HEIGHTMAP;

#[derive(Serialize, Deserialize, Default)]
struct Remembered {
    #[serde(skip_serializing_if = "Option::is_none")]
    world: Option<PathBuf>,
}

fn settings() -> PathBuf {
    crate::project::support().join("terrain.json")
}

/// Which world to open, without being asked.
///
/// Whatever was open last, or failing that whatever the open game says is its
/// own — so a maker who opens their game and walks to this bench is standing on
/// its world, which is the thing they came here for. Neither is required: with
/// no memory and no game, the bench waits to be handed one.
pub fn expected() -> Option<PathBuf> {
    remembered().or_else(|| crate::project::world().filter(|road| looks_like_a_world(road)))
}

/// The world the bench had open last, if it is still there.
fn remembered() -> Option<PathBuf> {
    let text = std::fs::read_to_string(settings()).ok()?;
    let kept: Remembered = serde_json::from_str(&text).ok()?;
    // A folder that has since moved or been deleted is not offered, the same way
    // the opening screen drops projects that are no longer there.
    kept.world.filter(|road| looks_like_a_world(road))
}

/// Puts a world at the front of the bench's memory.
pub fn remember(folder: &Path) {
    let road = settings();
    if let Some(above) = road.parent()
        && let Err(why) = std::fs::create_dir_all(above)
    {
        warn!("{}: {why}", above.display());
        return;
    }
    let kept = Remembered {
        world: Some(folder.to_path_buf()),
    };
    match serde_json::to_string_pretty(&kept) {
        Ok(text) => {
            if let Err(why) = std::fs::write(&road, text) {
                warn!("{}: {why}", road.display());
            }
        }
        Err(why) => warn!("could not write {}: {why}", road.display()),
    }
}

/// Whether a folder holds a world at all.
pub fn looks_like_a_world(folder: &Path) -> bool {
    folder.join(HEIGHTMAP).is_file()
}

/// Asks for a world.
///
/// The map image is picked rather than the folder, because that is the thing a
/// maker can actually see and recognise - a folder of assets all look alike in a
/// dialog, and a picture does not. The folder it sits in is the world.
pub fn ask(from: Option<&Path>) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new()
        .set_title("Open a world - pick its map image")
        .add_filter("Map image", &["png", "jpg", "jpeg", "webp"]);
    // Start where the maker most likely means: the world they have open, or the
    // one the open game says is its own.
    let start = from
        .map(Path::to_path_buf)
        .or_else(crate::project::world);
    if let Some(from) = start {
        dialog = dialog.set_directory(from);
    }
    let picked = dialog.pick_file()?;
    let folder = picked.parent()?.to_path_buf();

    // Picked something that is not the map itself - a texture in the same
    // folder, say. The folder is still the world if the map is in it.
    if !looks_like_a_world(&folder) {
        warn!(
            "{} holds no {HEIGHTMAP}, so there is no world in it",
            folder.display()
        );
        return None;
    }
    Some(folder)
}

/// What to call an open world on the shelf: the folder it sits in, and the one
/// above, which is usually the game's name.
pub fn called(folder: &Path) -> String {
    let leaf = folder
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| folder.display().to_string());
    match folder.parent().and_then(|above| above.file_name()) {
        Some(above) => format!("{}/{leaf}", above.to_string_lossy()),
        None => leaf,
    }
}
