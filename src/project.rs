//! A project: one game's own folder of work.
//!
//! Opificium holds no game's content. The bench is the program; the
//! buildings, the palette, the bodies and the templates all belong to
//! whichever game asked for them, and they live in that game's own
//! repository beside its code. A project is simply the folder where they
//! sit, described by an `opificium.json` at its root.
//!
//! That is the whole of what makes this bench serve more than one game.
//! Everything else - the drawing, the baking, the rig - never learns which
//! world it is working for.
//!
//! ```json
//! {
//!   "format": 1,
//!   "name": "Divus Factus",
//!   "install": "../Divus Factus/assets/buildings"
//! }
//! ```
//!
//! Every path but the name has a sensible default, so the manifest above
//! is a complete project. `install` is the one worth setting by hand: it
//! is where baked work is carried so the game itself can read it, and
//! only the game knows where that is.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// The manifest as it is written on disk. Every path is relative to the
/// project root, and every one of them is optional.
#[derive(Serialize, Deserialize, Default)]
struct Manifest {
    #[serde(default)]
    format: u32,
    name: Option<String>,
    palette: Option<String>,
    bodies: Option<String>,
    templates: Option<String>,
    work: Option<String>,
    baked: Option<String>,
    /// Where baked work is carried so the game can read it. Absolute, or
    /// relative to the project root - `../Some Game/assets/buildings`.
    install: Option<String>,
}

/// One game's folder of work, with every path already resolved.
#[derive(Clone, Debug)]
pub struct Project {
    pub name: String,
    pub root: PathBuf,
    pub palette: PathBuf,
    pub bodies: PathBuf,
    pub templates: PathBuf,
    pub work: PathBuf,
    pub baked: PathBuf,
    pub install: Option<PathBuf>,
}

/// The project the bench is working in.
///
/// A global rather than a Bevy resource on purpose: the paths are asked
/// for by plain functions all through the builder, the rig and the
/// palette - places that have no world to borrow from - and threading a
/// resource into every one of them would be a great deal of noise for a
/// value that changes about once a session.
static CURRENT: RwLock<Option<Project>> = RwLock::new(None);

/// The name of the manifest at a project's root.
pub const MANIFEST: &str = "opificium.json";

impl Project {
    /// Reads the project rooted at this folder.
    ///
    /// A folder with no manifest is still a project - it takes every
    /// default - so a maker can point the bench at an empty directory and
    /// start working immediately.
    pub fn read(root: &Path) -> Result<Project, String> {
        let root = root
            .canonicalize()
            .map_err(|why| format!("{}: {why}", root.display()))?;
        let manifest = match std::fs::read_to_string(root.join(MANIFEST)) {
            Ok(text) => serde_json::from_str::<Manifest>(&text)
                .map_err(|why| format!("{MANIFEST}: {why}"))?,
            Err(_) => Manifest::default(),
        };

        let under = |set: Option<String>, fallback: &str| -> PathBuf {
            let said = set.unwrap_or_else(|| fallback.to_string());
            let path = PathBuf::from(&said);
            if path.is_absolute() {
                path
            } else {
                root.join(path)
            }
        };

        Ok(Project {
            name: manifest.name.unwrap_or_else(|| {
                root.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "Untitled".into())
            }),
            palette: under(manifest.palette, "data/palette.json"),
            bodies: under(manifest.bodies, "data/bodies"),
            templates: under(manifest.templates, "templates"),
            work: under(manifest.work, "out/buildings"),
            baked: under(manifest.baked, "out/baked"),
            install: manifest.install.map(|set| under(Some(set), "")),
            root,
        })
    }

    /// Makes sure the folders this project writes into exist.
    pub fn prepare(&self) {
        for dir in [&self.work, &self.baked] {
            let _ = std::fs::create_dir_all(dir);
        }
    }
}

/// Opens a project and remembers it as the most recent.
pub fn open(root: &Path) -> Result<Project, String> {
    let project = Project::read(root)?;
    project.prepare();
    remember(&project.root);
    *CURRENT.write().unwrap() = Some(project.clone());
    Ok(project)
}

/// The project in hand, if one is open.
pub fn current() -> Option<Project> {
    CURRENT.read().unwrap().clone()
}

/// The project root, falling back to the source tree and then to wherever
/// the bench was started - so a bench with no project open still works
/// out of the folder it is standing in rather than refusing to draw.
pub fn root() -> PathBuf {
    if let Some(project) = current() {
        return project.root;
    }
    if let Ok(tree) = std::env::var("CARGO_MANIFEST_DIR") {
        return PathBuf::from(tree);
    }
    PathBuf::from(".")
}

/// The palette this project paints with.
pub fn palette() -> PathBuf {
    current()
        .map(|project| project.palette)
        .unwrap_or_else(|| root().join("data/palette.json"))
}

/// The bodies the rig animates.
pub fn bodies() -> PathBuf {
    current()
        .map(|project| project.bodies)
        .unwrap_or_else(|| root().join("data/bodies"))
}

/// Starting shapes a maker can draw from.
pub fn templates() -> PathBuf {
    current()
        .map(|project| project.templates)
        .unwrap_or_else(|| root().join("templates"))
}

/// Where a maker's own work is saved and reopened.
pub fn work() -> PathBuf {
    current()
        .map(|project| project.work)
        .unwrap_or_else(|| root().join("out/buildings"))
}

/// Where baked work is written.
pub fn baked() -> PathBuf {
    current()
        .map(|project| project.baked)
        .unwrap_or_else(|| root().join("out/baked"))
}

/// Where baked work is carried so the game can read it, if the project
/// says. Without it, baking stops at `baked()` and the carrying is done
/// by hand.
pub fn install() -> Option<PathBuf> {
    current().and_then(|project| project.install)
}

/// Opificium's own corner of the machine - the bench's settings and the
/// list of projects it has opened. Never a game's folder: the bench
/// belongs to nobody's world.
pub fn support() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let base = if cfg!(target_os = "macos") {
        format!("{home}/Library/Application Support/Opificium")
    } else if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(|roaming| format!("{roaming}/Opificium"))
            .unwrap_or_else(|_| ".".into())
    } else {
        format!("{home}/.local/share/opificium")
    };
    PathBuf::from(base)
}

fn recents_file() -> PathBuf {
    support().join("recent.json")
}

/// The projects this bench has opened, most recent first.
///
/// Folders that have since been moved or deleted are dropped on the way
/// out, so the list never offers a project that is not there.
pub fn recent() -> Vec<PathBuf> {
    let Ok(text) = std::fs::read_to_string(recents_file()) else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<PathBuf>>(&text)
        .unwrap_or_default()
        .into_iter()
        .filter(|road| road.is_dir())
        .collect()
}

/// Puts a project at the head of the recent list.
pub fn remember(root: &Path) {
    let mut roads = recent();
    roads.retain(|road| road != root);
    roads.insert(0, root.to_path_buf());
    roads.truncate(12);
    let _ = std::fs::create_dir_all(support());
    if let Ok(text) = serde_json::to_string_pretty(&roads) {
        let _ = std::fs::write(recents_file(), text);
    }
}

/// The project to open at startup: the one named on the command line, or
/// the last one worked in.
///
/// `opificium /path/to/game` is how a second game gets opened without
/// touching the bench's memory of the first.
pub fn opening() -> Option<PathBuf> {
    if let Some(said) = std::env::args().nth(1) {
        let road = PathBuf::from(said);
        if road.is_dir() {
            return Some(road);
        }
    }
    recent().into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A folder with nothing in it is a working project.
    ///
    /// The bench must never refuse to open on a missing manifest: a maker
    /// starting a new game points it at an empty directory, and the
    /// defaults are exactly the layout the first project used.
    #[test]
    fn an_empty_folder_is_a_project() {
        let root = std::env::temp_dir().join("opificium-test-empty");
        let _ = std::fs::create_dir_all(&root);
        let project = Project::read(&root).expect("an empty folder opens");
        assert_eq!(project.name, "opificium-test-empty");
        assert!(project.palette.ends_with("data/palette.json"));
        assert!(project.work.ends_with("out/buildings"));
        assert!(project.install.is_none(), "nothing to install into yet");
    }

    /// What the manifest says wins, and relative paths hang off the root.
    #[test]
    fn a_manifest_places_every_folder() {
        let root = std::env::temp_dir().join("opificium-test-named");
        let _ = std::fs::create_dir_all(&root);
        std::fs::write(
            root.join(MANIFEST),
            r#"{"format":1,"name":"Some Game","work":"art/works",
                "install":"../Some Game/assets/buildings"}"#,
        )
        .expect("write manifest");

        let project = Project::read(&root).expect("reads");
        assert_eq!(project.name, "Some Game");
        assert!(project.work.ends_with("art/works"));
        // Untouched fields still take their defaults.
        assert!(project.baked.ends_with("out/baked"));
        // A relative install is relative to the PROJECT, not to wherever
        // the bench happens to have been started from.
        let install = project.install.expect("an install target");
        assert!(install.starts_with(project.root.parent().unwrap()));
    }

    /// The bench's own settings never live in a game's folder.
    #[test]
    fn the_bench_keeps_its_own_house() {
        let support = support();
        let said = support.to_string_lossy().to_lowercase();
        assert!(said.contains("opificium"), "{said}");
        assert!(
            !said.contains("divus"),
            "the bench belongs to no single game: {said}",
        );
    }
}
