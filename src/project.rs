//! A project: one game's own folder of work.
//!
//! Opificium holds no game's content. The bench is the program; the
//! buildings, the palette and the templates all belong to
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
//!   "name": "Divus Factus"
//! }
//! ```
//!
//! EVERY path has a sensible default, `install` included, so the manifest
//! above is a complete project and an empty folder is a working one. A game
//! that keeps its assets somewhere unusual says so; a game that does what
//! every other game does says nothing at all.

use bevy::log::warn;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// The manifest as it is written on disk. Every path is relative to the
/// project root, and every one of them is optional.
///
/// Every field is skipped when it is unset, because the bench WRITES one of
/// these now as well as reading it - see [`start_a_project`] - and a manifest
/// full of `null`s is a poor first thing for a maker to open and edit.
#[derive(Serialize, Deserialize, Default)]
struct Manifest {
    #[serde(default)]
    format: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    palette: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    templates: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kinds: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    widgets: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    work: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    baked: Option<String>,
    /// Where baked work is carried so the game can read it. Absolute, or
    /// relative to the project root. Defaults to [`INSTALL`]; an EMPTY string
    /// means "carry it nowhere", and the bake stops in `baked`.
    #[serde(skip_serializing_if = "Option::is_none")]
    install: Option<String>,
    /// Where this game keeps its world, if it has one - see [`world`].
    #[serde(skip_serializing_if = "Option::is_none")]
    world: Option<String>,
}

/// One game's folder of work, with every path already resolved.
#[derive(Clone, Debug)]
pub struct Project {
    pub name: String,
    pub root: PathBuf,
    pub palette: PathBuf,
    pub templates: PathBuf,
    /// What this game raises a baked drawing AS - see [`kinds`].
    pub kinds: PathBuf,
    /// The marks this game understands - see [`widgets`].
    pub widgets: PathBuf,
    pub work: PathBuf,
    pub baked: PathBuf,
    pub install: Option<PathBuf>,
    /// Where this game keeps its world, if it says - see [`world`].
    pub world: Option<PathBuf>,
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

/// The same path with its `.` and `..` walked out of it.
///
/// `install` points OUT of the project - `../assets/buildings` - so without this
/// every log line and every error about it reads
/// `…/game/opificium/../assets/buildings`, and two paths that mean one folder
/// compare as different. Walked lexically, which is safe here because what it is
/// joined onto is already canonical: popping a component off a real path cannot
/// land somewhere a symlink would have gone.
fn tidied(path: PathBuf) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::ParentDir => {
                // Nothing to pop is not an error: a relative path may genuinely
                // start above wherever it is being read from.
                if !out.pop() {
                    out.push("..");
                }
            }
            std::path::Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

/// Where baked work goes when a game has not said otherwise.
///
/// Relative to the project root, which is `<game>/opificium` - so this is
/// `<game>/assets/buildings`, and it is a CONVENTION rather than a guess. Brett:
/// "Shouldnt the folder that the baked files go into be universal so that other
/// games don't need to dictate the specific file?"
///
/// It used to be the one path in the manifest with no default, on the reasoning
/// that only a game knows where its own assets live and a wrong path looks exactly
/// like success. That was the wrong call twice over: every OTHER path here has a
/// default, and the bench already dictates that its work lives in `opificium/`
/// inside the game - so having no opinion about this one folder was not caution,
/// it was an inconsistency a new game had to read the manual to get past.
///
/// A game that keeps its assets elsewhere names them; anything unusual is one
/// line, and the ordinary case is none.
pub const INSTALL: &str = "../assets/buildings";

/// What the bench's own folder is called inside a game's repository.
///
/// One name, everywhere, so a maker points at a GAME and never has to know what
/// the bench keeps its work in. Both of the studio's games already use it.
pub const BENCH_FOLDER: &str = "opificium";

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
                tidied(path)
            } else {
                tidied(root.join(path))
            }
        };

        Ok(Project {
            name: manifest.name.unwrap_or_else(|| {
                root.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "Untitled".into())
            }),
            palette: under(manifest.palette, "data/palette.json"),
            templates: under(manifest.templates, "templates"),
            kinds: under(manifest.kinds, "data/kinds.json"),
            widgets: under(manifest.widgets, "data/widgets.json"),
            work: under(manifest.work, "out/buildings"),
            baked: under(manifest.baked, "out/baked"),
            world: manifest.world.as_deref().map(|said| {
                let path = PathBuf::from(said);
                if path.is_absolute() {
                    tidied(path)
                } else {
                    tidied(root.join(path))
                }
            }),
            install: match manifest.install.as_deref() {
                // Said empty on purpose: keep the bakes in this project and let a
                // hand carry them. Worth being able to say, and impossible to say
                // at all while a missing field meant the same thing.
                Some("") => None,
                said => Some(under(said.map(str::to_string), INSTALL)),
            },
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

/// The bench's own folder for whatever folder a maker pointed at, making one if
/// it is not there yet.
///
/// A maker picks the GAME - "I would like to be able to select a game root folder
/// and it creates an opificium folder in that games root and works from there" -
/// so this takes the three things a picked folder can be and answers all of them:
///
/// - a project already, having a manifest, or being the bench's own folder with
///   none (a folder with no manifest is still a project);
/// - a game whose bench folder exists, which is opened;
/// - a game with no bench folder at all, which gets one, with a manifest.
///
/// The last is the interesting case, and it is the ordinary one: a new game is an
/// empty repository, and the bench should be the thing that furnishes it rather
/// than something a maker has to prepare a folder for by hand.
pub fn start_a_project(picked: &Path) -> Result<PathBuf, String> {
    if picked.join(MANIFEST).is_file()
        || picked.file_name().is_some_and(|name| name == BENCH_FOLDER)
    {
        return Ok(picked.to_path_buf());
    }
    let inside = picked.join(BENCH_FOLDER);
    if inside.is_dir() {
        return Ok(inside);
    }
    std::fs::create_dir_all(&inside).map_err(|why| format!("{}: {why}", inside.display()))?;
    write_a_manifest(&inside, picked)?;
    // And the word that explains the folder to whoever finds it next.
    let _ = write_the_word(&inside);
    Ok(inside)
}

/// Writes the note that explains this folder to whoever opens it next.
///
/// A folder full of JSON in somebody else's repository is a puzzle, and the person
/// solving it is often not a person: Brett's reason for wanting this is "that way
/// the AI can read that and understand everything". So it says what every file is,
/// what the bench writes into the game, what shape a baked building has, and what
/// not to commit.
///
/// GENERIC, deliberately - "that file should be generic and not specific to any
/// game". Nothing here knows what kind of game it is standing in, and it must not
/// pretend to: a note that talked about villages would be wrong in the first game
/// that had none.
///
/// Only ever written when the folder is MADE. A maker's own edits to it are theirs,
/// and a bench that rewrote this on every open would throw them away.
fn write_the_word(root: &Path) -> Result<(), String> {
    let word = format!(
        r#"# Opificium

This folder is a **project**: one game's own authored work for
[Opificium](https://github.com/Baz-Studios-LLC/Opificium), a maker's bench for
buildings and models. Opificium made this folder and this note.

The bench and the game **share no code**. Everything that passes between them is a
file described here.

## What is in here

| path                | who writes it | what it is                                        |
| ------------------- | ------------- | ------------------------------------------------- |
| `{MANIFEST}`   | you           | which folders this project uses. Every path has a default, so it may be nearly empty |
| `data/palette.json` | the game      | the colour ramps the bench paints with            |
| `data/kinds.json`   | either        | what a finished drawing may be baked AS           |
| `data/widgets.json` | either        | the marks the bench may place, and their colours  |
| `templates/`        | you           | starting shapes to draw from                      |
| `out/buildings/`    | the bench     | saved drawings, `.baz` - **the source of truth**  |
| `out/baked/`        | the bench     | baked output, only if `install` is set empty      |
| `out/models/`       | the bench     | models the kiln made, `.glb` - load these whole   |

A `.baz` is JSON. It is the editable drawing and the thing worth keeping.

## The palette, the kinds and the marks

`data/palette.json` is the one file the game really must provide, or the bench
paints in its own colours instead of the game's:

```json
{{ "ramps": [ {{ "name": "wood", "steps": [[28,19,16], "...5 RGB steps..."] }} ] }}
```

`kinds.json` and `widgets.json` are **vocabulary the game understands**. The bench
offers only what is listed, and a word it is given is passed through untouched:

```json
{{ "format": 1, "kinds": [ {{ "word": "house" }}, {{ "word": "townhall", "label": "TOWN HALL" }} ] }}
{{ "format": 1, "marks": [ {{ "mark": "door", "ramp": "cloth-green", "shade": 0.6 }} ] }}
```

**These are contracts, not data.** The game matches these words against its own
code. A word the game does not understand costs whatever it was attached to, and
nothing in the bench can catch that - it cannot see the game's source. Keep them
true, or have the game generate these two files the way it generates the palette.

## What the bench hands the game

Baking resolves a drawing into plain boxes with colours already looked up, and
writes it to **`../assets/buildings`** - the game's own assets folder, one step out
of this one. That is the default and needs no setting; a game that keeps its assets
elsewhere sets `install` in the manifest, and a game that wants nothing carried
anywhere sets it to `""`, which keeps bakes in `out/baked/`. That output is
**generated**: bake it again rather than editing it.

```json
{{
  "format": 2,
  "name": "...", "kind": "...",
  "half_w": 3.6, "half_d": 4.2, "high": 7.6,
  "boxes": [ {{ "at": [0,1.25,0], "size": [4,2.5,0.25], "turn": [0,0,0,1],
               "form": "box", "rgb": [110,92,70], "alpha": 1.0, "stage": "walls" }} ],
  "marks": [ {{ "mark": "door", "at": [3.6,0.4,0.0], "yaw": 0.0 }} ],
  "levels": [ {{ "name": "", "half_w": 3.6, "half_d": 4.2, "high": 7.6,
                "phases": [ {{ "boxes": ["..."] }} ], "marks": ["..."] }} ]
}}
```

- `boxes` and `marks` are the **base building, finished**. A reader that wants
  nothing else can read only these and ignore the rest.
- `levels` is the building's whole life: the original, then each upgrade. Every
  level carries its own `phases` - one COMPLETE set of boxes per step of raising
  it - and its own footprint and marks, all measured from one shared origin, so an
  upgrade lands on the building it upgrades.
- `stage` on a box says what it IS - `footing`, `frame`, `walls`, `roof`,
  `furnishing` - which is useful for cutaways and for raising a level without
  reading its phases.
- `form` is the box's shape: `box`, `wedge`, `ridge`, `hip:<x>x<z>`, or
  `cut:<low>x<high>`. Both programs build each shape from their own code, so a new
  form must be written twice.
- Local space is +Y up, metres, and every measurement is a whole number of
  sixteenths of a metre.

Re-bake without opening a window:

```sh
opificium <this folder> --bake
```

## What to commit, and what to ignore

Commit everything a person authored:

```
{MANIFEST}
data/
templates/
out/buildings/
```

Ignore what is generated, since it is rebuilt from the drawings on demand:

```gitignore
out/baked/
out/buildings/workbench.baz
```

`workbench.baz` is the bench's scratch pad - whatever was standing when it last
had to keep something. Renaming a drawing is how it becomes worth committing.

The baked output under the game's own assets folder is generated too. Whether to
commit it is the game's call: ignoring it means the game cannot be built without
running the bake first, and committing it means reviewing generated JSON.
"#
    );
    let path = root.join("README.md");
    std::fs::write(&path, word).map_err(|why| format!("{}: {why}", path.display()))
}

/// Writes the manifest that starts a game's project off.
fn write_a_manifest(root: &Path, game: &Path) -> Result<(), String> {
    let name = game
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Untitled".into());
    // No paths at all: every one of them has a default now, `install` included,
    // and a manifest that spelled out the defaults would be a second copy of them
    // to keep in step. The name is the only thing here the bench cannot work out,
    // and even that is only a nicety.
    let manifest = Manifest {
        format: 1,
        name: Some(name),
        ..Manifest::default()
    };
    let text = serde_json::to_string_pretty(&manifest)
        .map_err(|why| format!("could not write a manifest: {why}"))?;
    let path = root.join(MANIFEST);
    std::fs::write(&path, format!("{text}\n")).map_err(|why| format!("{}: {why}", path.display()))
}

/// What to call a project on a button, worked out from its path alone.
///
/// The manifest's own `name` would be better and is not worth a dozen file reads
/// to draw a list. A project's folder is named after the game in every case the
/// bench has met, and the bench's own folder INSIDE it is always called
/// `opificium` - so where the last component is that, the game's name is one step
/// up, and a rail full of buttons reading OPIFICIUM is avoided.
pub fn called(root: &Path) -> String {
    let last = root
        .file_name()
        .map(|name| name.to_string_lossy().into_owned());
    if last.as_deref() == Some(BENCH_FOLDER)
        && let Some(parent) = root.parent().and_then(|up| up.file_name())
    {
        return parent.to_string_lossy().into_owned();
    }
    last.unwrap_or_else(|| root.display().to_string())
}

/// Leaves this bench and opens another project in a fresh one.
///
/// A RESTART rather than a swap, and deliberately. The palette, the kinds, the
/// shelf of saved works and the window's own title are each read once and kept in
/// half a dozen places, so switching in place means remembering to refresh every
/// one of them - and the one that gets forgotten does not fail, it goes quietly
/// stale. That is not a hypothetical: a palette read for one game and left
/// standing for another is exactly how a bench full of magenta walls happened.
/// A new process cannot have a stale corner.
///
/// The environment carries over untouched, which is what makes this work from a
/// source tree: `cargo run` puts `CARGO_MANIFEST_DIR` in it, and that is how Bevy
/// finds `assets/`. The same binary run without it comes up with no fonts at all.
/// A bundle keeps its assets beside the binary and needs none of this.
/// Where the bench writes what it is doing.
pub fn log_file() -> PathBuf {
    support().join("opificium.log")
}

/// Opens it for appending, or `None` if it cannot be - in which case the bench keeps
/// whatever it was given and says nothing, since failing to launch over a log file would
/// be the tail wagging the dog.
fn the_log_file() -> Option<std::fs::File> {
    let road = log_file();
    let _ = std::fs::create_dir_all(road.parent()?);
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&road)
        .ok()
}

pub fn relaunch(root: &Path) -> Result<std::process::Child, String> {
    // Remembered FIRST, so a relaunch that fails to spawn still leaves the bench
    // pointed at the project the maker asked for: they open it again by hand and
    // arrive where they meant to go.
    remember(root);
    let program = std::env::current_exe().map_err(|why| format!("no path to this bench: {why}"))?;
    let mut opening = std::process::Command::new(&program);
    opening.arg(root);
    // EVERYTHING IT SAYS, INTO A FILE. The bench that a maker actually uses is this child,
    // and its stdout and stderr were the Terminal's pipe - a Terminal that closes as soon
    // as the launching process exits, which is immediately. So every word it said after
    // that went nowhere, a panic included, and a write to that dead pipe took a firing
    // down with it.
    //
    // Appended rather than replaced, and one file rather than one per run: a maker asking
    // what went wrong has one place to look, and the file is theirs to delete whenever it
    // stops being interesting.
    if let Some(log) = the_log_file() {
        opening
            .stdout(std::process::Stdio::from(
                log.try_clone().map_err(|why| format!("{why}"))?,
            ))
            .stderr(std::process::Stdio::from(log));
    }
    opening
        .spawn()
        .map_err(|why| format!("{}: {why}", program.display()))
}

/// One kind of thing this game raises, and what the bench calls it on the card.
#[derive(Serialize, Deserialize, Clone)]
pub struct Kind {
    /// The word written into a baked file, and it is the GAME'S word.
    ///
    /// The game matches this against its own vocabulary and takes no other
    /// reading of it, so a word the game does not know is not a new kind of
    /// building - it is a drawing the game raises as nothing at all. See
    /// [`kinds`] for whose job that is.
    pub word: String,
    /// What the card says, when the word alone would not do: `townhall` is TOWN
    /// HALL. Left out, the word is simply spoken aloud.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl Kind {
    /// What to write on a button.
    pub fn said(&self) -> String {
        self.label
            .clone()
            .unwrap_or_else(|| self.word.to_uppercase().replace('-', " "))
    }
}

#[derive(Serialize, Deserialize, Default)]
struct KindsFile {
    #[serde(default)]
    format: u32,
    #[serde(default)]
    kinds: Vec<Kind>,
}

/// What this game raises a baked drawing AS.
///
/// The bench used to hold this list itself: eighteen kinds of medieval village
/// building, in the bench's own source, offered to every game that ever opened
/// it. Which made a housefly's living room a `sawmill`, because a card that
/// insists on an answer will get a wrong one.
///
/// So it belongs to the project, and a game with no list is offered nothing -
/// colours are universal and a sawmill is not. A drawing baked with no kind
/// carries none, and the game falls back to reading the drawing's NAME, which is
/// what every drawing baked before the card existed relies on.
///
/// # Whose word it is
///
/// The GAME's. It matches the word against its own vocabulary - in Divus Factus
/// that is a `match` over a `BuildingKind` enum - and a word it does not know
/// wins nothing and falls back to nothing: the drawing is claimed by no kind and
/// the village raises it as nothing. Nothing here can check that, because the
/// vocabulary lives in the other program's source. It is the developer's to keep
/// true, which is Brett's own call: "it's on the developer to make sure that the
/// kind that he adds to this app are understood by the game that they're
/// developing."
///
/// Hand-written or game-written, either way - the file is the whole contract, and
/// a game that would rather not rely on a hand can export it the way it already
/// exports its palette.
pub fn kinds() -> Vec<Kind> {
    let road = kinds_file();
    let Ok(text) = std::fs::read_to_string(&road) else {
        return Vec::new();
    };
    match serde_json::from_str::<KindsFile>(&text) {
        Ok(file) => file.kinds,
        Err(why) => {
            // Said out loud rather than swallowed: an empty card and a card whose
            // file has a comma out of place look identical from the outside.
            warn!("{}: {why}", road.display());
            Vec::new()
        }
    }
}

/// Adds a kind to this project's list, and keeps it there.
///
/// The list grows by USE: a kind named once at the bake is a kind the project
/// knows from then on, so nobody edits JSON to add the second building they ever
/// draw. A word already known is left exactly as it stands, label and all.
pub fn add_a_kind(word: &str) -> Result<(), String> {
    let word = word.trim().to_lowercase();
    if word.is_empty() {
        return Err("a kind needs a word".to_string());
    }
    let mut known = kinds();
    if known.iter().any(|kind| kind.word == word) {
        return Ok(());
    }
    known.push(Kind { word, label: None });
    let road = kinds_file();
    if let Some(under) = road.parent() {
        std::fs::create_dir_all(under).map_err(|why| format!("{}: {why}", under.display()))?;
    }
    let text = serde_json::to_string_pretty(&KindsFile {
        format: 1,
        kinds: known,
    })
    .map_err(|why| format!("could not write the kinds: {why}"))?;
    std::fs::write(&road, format!("{text}\n")).map_err(|why| format!("{}: {why}", road.display()))
}

/// A mark this game understands, and the colour its block wears on the bench.
pub struct Widget {
    /// The word written into a baked file's `marks`, and it is the GAME'S word -
    /// the same contract, and the same warning, as [`Kind`].
    pub word: &'static str,
    pub ramp: String,
    pub shade: f32,
}

/// One as it is written on disk.
#[derive(Deserialize)]
struct WidgetDef {
    mark: String,
    #[serde(default = "bone")]
    ramp: String,
    #[serde(default = "half")]
    shade: f32,
}

fn bone() -> String {
    "bone".to_string()
}

fn half() -> f32 {
    0.5
}

#[derive(Deserialize, Default)]
struct WidgetsFile {
    #[serde(default)]
    marks: Vec<WidgetDef>,
}

/// The marks this game understands: what a place is FOR.
///
/// `sleep`, `sit`, `fire`, `smoke`, `door`, `work`, `store`, `light` were the
/// bench's own list, which is a village of people with beds and hearths - and a
/// game about a housefly wants somewhere to perch, not somewhere to be wed. The
/// same reasoning as [`kinds`], and the same contract: the game reads these words
/// and the developer keeps them true.
///
/// Read ONCE and kept, because a widget's word has to outlive the read: a
/// `PartKind` is `Copy` and spells its widget as a `&'static str`, so the words are
/// leaked deliberately - a handful of short strings, once, for the life of the
/// program. There is precedent a few lines from where the shelf uses them.
///
/// Kept for the process rather than re-read means a project's marks are fixed once
/// the bench is running, which is exactly true: switching projects RELAUNCHES the
/// bench, so a fresh process reads a fresh list. See [`relaunch`].
pub fn widgets() -> &'static [Widget] {
    static MARKS: std::sync::OnceLock<Vec<Widget>> = std::sync::OnceLock::new();
    MARKS.get_or_init(|| {
        let road = widgets_file();
        let Ok(text) = std::fs::read_to_string(&road) else {
            return Vec::new();
        };
        match serde_json::from_str::<WidgetsFile>(&text) {
            Ok(file) => file
                .marks
                .into_iter()
                .map(|mark| Widget {
                    word: Box::leak(mark.mark.into_boxed_str()),
                    ramp: mark.ramp,
                    shade: mark.shade,
                })
                .collect(),
            Err(why) => {
                warn!("{}: {why}", road.display());
                Vec::new()
            }
        }
    })
}

/// Keeps a word for the life of the program, so a `PartKind` can hold it.
///
/// A saved work names its marks in TEXT, and what reads that text has to hand back
/// a `&'static str` - a `PartKind` is `Copy` and spells a widget as one. So a word
/// read off disk is kept here, once each, however many parts wear it.
///
/// This is what stops a project's list from being able to LOSE anything. Declaring
/// a mark says what the shelf offers and what colour its block wears; it does not
/// say what may be read. Without this, a work drawn in one game and opened in
/// another came up with its marks silently missing - and saving it again would
/// have made that permanent, which is the one kind of bug a maker cannot undo.
pub fn a_kept_word(word: &str) -> &'static str {
    static KEPT: RwLock<Option<std::collections::HashSet<&'static str>>> = RwLock::new(None);
    if let Some(known) = KEPT.read().unwrap().as_ref()
        && let Some(had) = known.get(word)
    {
        return had;
    }
    let mut writing = KEPT.write().unwrap();
    let known = writing.get_or_insert_with(std::collections::HashSet::new);
    // Checked again under the write lock: two threads asking at once would
    // otherwise keep the same word twice.
    if let Some(had) = known.get(word) {
        return had;
    }
    let kept: &'static str = Box::leak(word.to_string().into_boxed_str());
    known.insert(kept);
    kept
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

/// Where the colour sets a maker saved from this game's buildings live.
///
/// In the project because the colours name the GAME's own ramps: a set saved for one game
/// paints nothing in another. Beside the kinds, which is where everything game-shaped that
/// the bench itself authors belongs.
///
/// Not in the manifest. A game has no reason to name this file - it never reads it - and
/// every path a manifest carries is one more thing for a game to get wrong about a bench
/// it does not share code with.
pub fn saved_palettes() -> PathBuf {
    root().join("data/saved-palettes.json")
}

/// Where this game's list of building kinds lives.
pub fn kinds_file() -> PathBuf {
    current()
        .map(|project| project.kinds)
        .unwrap_or_else(|| root().join("data/kinds.json"))
}

/// Where this game's list of marks lives.
pub fn widgets_file() -> PathBuf {
    current()
        .map(|project| project.widgets)
        .unwrap_or_else(|| root().join("data/widgets.json"))
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

/// Where this game keeps its world, if it says so.
///
/// **A HINT, and never a requirement.** A world is not a project: the terrain
/// bench is a tool you bring ground to, and it opens one from its own shelf,
/// from any folder holding a heightmap, whoever's it is. This only spares a
/// maker the walk across the disk when the game they already have open happens
/// to know where its own world lives.
///
/// There is deliberately NO default. Every other path here has one, because
/// every game has buildings; not every game has a world, and guessing a folder
/// that is not there would have the bench announce a missing world to games that
/// never wanted one.
pub fn world() -> Option<PathBuf> {
    current().and_then(|project| project.world)
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

/// A project named OUTRIGHT: on the command line, or in the environment.
///
/// Split from [`opening`] because the two are different questions. A path on argv
/// is an INSTRUCTION - from a script, from the launcher, or from the bench
/// reopening itself after a maker chose a game - and it is obeyed without asking.
/// The last project worked in is only a GUESS at what somebody wants, and a guess
/// is the kind of thing worth asking about. See [`crate::opening::ask`].
///
/// This is also what stops the asking from repeating: choosing a game relaunches
/// the bench with that game on argv, so the new process is told rather than asked.
pub fn named_outright() -> Option<PathBuf> {
    if let Some(said) = std::env::args().nth(1) {
        let road = PathBuf::from(said);
        if road.is_dir() {
            return Some(road);
        }
    }
    // The same thing without a command line: scripts, CI, and the headless bake,
    // none of which own argv.
    if let Ok(said) = std::env::var("OPIFICIUM_PROJECT") {
        let road = PathBuf::from(said);
        if road.is_dir() {
            return Some(road);
        }
    }
    None
}

/// The project to open at startup: the one named on the command line, or
/// the last one worked in.
///
/// `opificium /path/to/game` is how a second game gets opened without
/// touching the bench's memory of the first.
pub fn opening() -> Option<PathBuf> {
    named_outright().or_else(|| recent().into_iter().next())
}

/// Opens whatever `opening()` names, without disturbing the recent list.
///
/// For the headless paths - a bake run from a script or a build - where
/// being remembered as "the project you were last working in" would be a
/// lie about a person's intent.
pub fn open_quietly() -> Option<Project> {
    let road = opening()?;
    let project = Project::read(&road).ok()?;
    *CURRENT.write().unwrap() = Some(project.clone());
    Some(project)
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
        // Even the install path: an empty folder is a COMPLETE project, and a
        // maker who bakes in one should find the result where a game would read
        // it rather than have to go looking for a manifest field.
        let install = project.install.expect("the convention, unasked for");
        assert!(
            install.ends_with("assets/buildings"),
            "{}",
            install.display()
        );
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

    /// A fresh game root gets a bench folder and a manifest to start it off.
    ///
    /// The whole of what a maker does to add a game: point at the game. Brett: "I
    /// would like to be able to select a game root folder and it creates an
    /// opificium folder in that games root and works from there."
    #[test]
    fn a_game_folder_gets_its_own_bench_folder() {
        let game = fresh("opificium-test-game");
        // A game with somewhere to put its assets, which is the signal that says
        // where baked work belongs.
        std::fs::create_dir_all(game.join("assets")).expect("assets");

        let root = start_a_project(&game).expect("a bench folder");
        assert_eq!(root, game.join(BENCH_FOLDER));
        assert!(root.is_dir(), "the folder was not made");

        let project = Project::read(&root).expect("the manifest it just wrote");
        assert_eq!(project.name, "opificium-test-game");
        let install = project.install.expect("an install target");
        assert!(
            install.ends_with("assets/buildings"),
            "baked work would land at {}",
            install.display()
        );
        // And the paths it did not write still take their defaults.
        assert!(project.work.ends_with("out/buildings"));
    }

    /// A game says nothing and gets the convention; only an unusual game speaks.
    ///
    /// And "nowhere" has to be sayable too, or a project that wants its bakes kept
    /// to itself has no way to ask - which is what a missing field used to mean,
    /// and why the two could not be told apart.
    #[test]
    fn where_a_bake_lands_needs_no_asking() {
        // Said nothing at all - not even an assets folder to be inferred from.
        let game = fresh("opificium-test-bare");
        let root = start_a_project(&game).expect("a bench folder");
        let quiet = Project::read(&root).expect("reads");
        let install = quiet.install.expect("the convention");
        assert!(
            install.ends_with("assets/buildings"),
            "{}",
            install.display()
        );
        // And it is the GAME's assets, one step out of the bench's own folder,
        // rather than a folder buried inside the project.
        // Canonical on both sides: the project root is canonicalised when it is
        // read, and on this machine that turns /var into /private/var.
        let game = game.canonicalize().expect("the game folder");
        assert_eq!(install, game.join("assets/buildings"));
        // The manifest it wrote does not spell the default out: a written default
        // is a second copy of it to keep in step.
        let said = std::fs::read_to_string(root.join(MANIFEST)).expect("a manifest");
        assert!(
            !said.contains("install"),
            "the default was written down: {said}"
        );

        // Said empty on purpose: carry it nowhere.
        let held = fresh("opificium-test-held");
        std::fs::write(held.join(MANIFEST), r#"{"format":1,"install":""}"#).expect("write");
        let project = Project::read(&held).expect("reads");
        assert!(
            project.install.is_none(),
            "an empty install still carried work somewhere: {:?}",
            project.install
        );

        // And a game that keeps its assets somewhere odd is still obeyed.
        let odd = fresh("opificium-test-odd");
        std::fs::write(
            odd.join(MANIFEST),
            r#"{"format":1,"install":"../Content/huts"}"#,
        )
        .expect("write");
        let project = Project::read(&odd).expect("reads");
        assert!(
            project.install.expect("named").ends_with("Content/huts"),
            "a named install was overruled by the convention"
        );
    }

    /// A project folder handed in directly is taken as it is.
    ///
    /// Both ways of being one: a folder with a manifest, and the bench's own
    /// folder without one. Otherwise pointing at the place the work already lives
    /// would bury a second bench folder inside it - `opificium/opificium` - and
    /// every colour and saved work would go missing at once.
    #[test]
    fn a_project_folder_is_taken_as_it_stands() {
        let manifested = fresh("opificium-test-has-manifest");
        std::fs::write(manifested.join(MANIFEST), r#"{"format":1}"#).expect("manifest");
        assert_eq!(
            start_a_project(&manifested).expect("taken as it is"),
            manifested
        );

        // Named `opificium`, no manifest - which is a project, and must not gain
        // one of its own inside it.
        let bare = fresh("opificium-test-bare-bench").join(BENCH_FOLDER);
        std::fs::create_dir_all(&bare).expect("the bench folder");
        assert_eq!(start_a_project(&bare).expect("taken as it is"), bare);
        assert!(
            !bare.join(BENCH_FOLDER).exists(),
            "a bench folder was buried inside a bench folder"
        );
    }

    /// Opening the same game twice finds the folder rather than making another,
    /// and leaves the manifest exactly as the maker last edited it.
    #[test]
    fn opening_a_game_again_keeps_what_is_there() {
        let game = fresh("opificium-test-again");
        let first = start_a_project(&game).expect("a bench folder");
        // A maker's own edit to the manifest, which a second opening must not
        // walk over.
        std::fs::write(
            first.join(MANIFEST),
            r#"{"format":1,"name":"By Hand","work":"drawings"}"#,
        )
        .expect("edit");

        let second = start_a_project(&game).expect("the same folder");
        assert_eq!(first, second);
        let project = Project::read(&second).expect("reads");
        assert_eq!(project.name, "By Hand", "the manifest was overwritten");
        assert!(project.work.ends_with("drawings"));
    }

    /// A project is called after the GAME, not after the folder the bench keeps
    /// its work in - or a rail of recent projects reads OPIFICIUM all the way
    /// down.
    #[test]
    fn a_project_is_called_after_its_game() {
        assert_eq!(
            called(Path::new("/games/Fly on the Wall/opificium")),
            "Fly on the Wall"
        );
        // A project folder that is not the bench's own keeps its own name.
        assert_eq!(called(Path::new("/games/Divus Factus/art")), "art");
    }

    /// A project with no kinds file offers nothing, and adding one keeps it.
    ///
    /// The whole point of the list being the project's: a game that raises no
    /// sawmills is never offered one, and the list grows by use rather than by
    /// somebody editing the bench's source.
    #[test]
    fn a_project_learns_its_kinds() {
        let root = fresh("opificium-test-kinds");
        // Stood in, NOT opened. `open` remembers a project as the most recent, and
        // a test that did that would put a folder in the machine's temp directory
        // at the head of the maker's own list of games.
        *CURRENT.write().unwrap() = Some(Project::read(&root).expect("reads"));
        assert!(kinds().is_empty(), "a new project knows no kinds");

        add_a_kind("sawmill").expect("adds");
        add_a_kind("townhall").expect("adds");
        let known = kinds();
        assert_eq!(known.len(), 2);
        assert_eq!(known[0].word, "sawmill");
        // No label written, so the word speaks itself.
        assert_eq!(known[0].said(), "SAWMILL");

        // The same word twice is the same one kind.
        add_a_kind("sawmill").expect("adds");
        assert_eq!(kinds().len(), 2, "a kind was added twice");

        // Case and stray space are the maker's, not the game's.
        add_a_kind("  Blacksmith ").expect("adds");
        assert!(
            kinds().iter().any(|kind| kind.word == "blacksmith"),
            "a kind was kept with its capitals: {:?}",
            kinds().iter().map(|k| k.word.clone()).collect::<Vec<_>>()
        );
        // And a word that is no word at all is refused rather than written.
        assert!(add_a_kind("   ").is_err());
    }

    /// A label is only written when the word alone will not do, and it wins.
    #[test]
    fn a_kind_can_be_called_something_else() {
        let plain = Kind {
            word: "townhall".to_string(),
            label: None,
        };
        assert_eq!(plain.said(), "TOWNHALL");
        let said = Kind {
            word: "townhall".to_string(),
            label: Some("TOWN HALL".to_string()),
        };
        assert_eq!(said.said(), "TOWN HALL");
        // A hyphen is how the card's own text entry writes a space.
        let hyphened = Kind {
            word: "watch-tower".to_string(),
            label: None,
        };
        assert_eq!(hyphened.said(), "WATCH TOWER");
    }

    /// An empty folder to work in, wiped first so the test says the same thing
    /// twice running.
    fn fresh(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a folder to test in");
        root
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

#[cfg(test)]
mod the_word {
    use super::*;

    /// A new project gets a note explaining itself, and it says the true things.
    ///
    /// Checked rather than eyeballed because it is a formatted string with braces
    /// escaped throughout - the kind of text that goes subtly wrong and is never
    /// read again by the person who wrote it.
    #[test]
    fn a_new_project_explains_itself() {
        let game = std::env::temp_dir().join("opificium-test-word");
        let _ = std::fs::remove_dir_all(&game);
        std::fs::create_dir_all(&game).expect("a folder");
        let root = start_a_project(&game).expect("a bench folder");

        let word = std::fs::read_to_string(root.join("README.md")).expect("a note");
        // The manifest's real name reached the table rather than a stray brace.
        assert!(word.contains(MANIFEST), "the note never names {MANIFEST}");
        assert!(
            !word.contains("{MANIFEST}"),
            "a placeholder was left unfilled"
        );
        assert!(
            !word.contains("{{"),
            "an escaped brace survived into the note"
        );
        // The things a reader most needs: where drawings live, what is generated,
        // and that the vocabulary files are a contract.
        for said in [
            "out/buildings/",
            "out/baked/",
            "workbench.baz",
            "data/palette.json",
            "\"levels\"",
            "contracts, not data",
            "--bake",
        ] {
            assert!(word.contains(said), "the note never mentions {said}");
        }
        // And it is generic: no game's own words in it.
        let said = word.to_lowercase();
        for game_word in ["village", "villager", "sawmill", "divus", "housefly"] {
            assert!(
                !said.contains(game_word),
                "the note talks about {game_word}"
            );
        }
    }
}
