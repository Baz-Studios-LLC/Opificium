//! A work on disk: its levels, its phases, and where the bench keeps them.

use super::*;

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct Workbench {
    pub(crate) format: u32,
    pub(crate) name: String,
    /// A work from before stages: one flat list, which is the finished
    /// building. Kept readable forever - a maker's work is not something to
    /// lose to a format change - and turned into stages on the way in.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) parts: Vec<Placed>,
    /// One COMPLETE drawing per step of the build.
    ///
    /// Complete, and not a set of additions: raising step two clears step one
    /// off the ground and puts step two there instead. Brett's call, and the
    /// reason is authoring rather than rendering — "replacing the building each
    /// stage allows me to be more creative during the stages". A frame drawn at
    /// step one is a PICTURE of a frame, and by the time the walls are up it
    /// should be gone, because the walls are solid boxes that never needed it.
    /// Stages that accumulate would make that the awkward case; stages that
    /// replace make it the ordinary one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) stages: Vec<Vec<Placed>>,
    /// One LEVEL per form the building takes over its life: the original, and
    /// then each upgrade. Format 2.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) levels: Vec<Level>,
}

/// One form a building takes, and the phases that raise it.
///
/// A building is not one thing for ever. Brett: "I am planning for buildings to
/// have upgrades too. So once a building is finished later on they may want to
/// upgrade that building. That upgrade may need stages too while its being
/// built." So a work holds LEVELS - the original, then each upgrade - and every
/// level is itself a build, with its own phases.
///
/// The two axes were one word before, and only one of them ever left the bench:
/// a maker authored phases that the bake threw away, while the game re-derived
/// its own from the nature tag on each box. Which meant the freedom the phases
/// were for - "replacing the building each stage allows me to be more creative
/// during the stages" - stopped at the door. `longhouse1` already goes 34 boxes
/// then 32: a deviation nothing downstream could see.
#[derive(Clone, Serialize, Deserialize, Default)]
pub(crate) struct Level {
    /// What a maker calls it. The ORDER is what a game reads; this is for the
    /// person.
    #[serde(default)]
    pub name: String,
    /// One COMPLETE drawing per phase of raising this level, the last being the
    /// finished building at it.
    pub phases: Vec<Vec<Placed>>,
}

/// Every level of the work, every phase of each, and where the bench is standing.
///
/// Only the shown phase exists as entities; everything else is records waiting
/// its turn. Switching gathers the standing parts back into their phase and sets
/// the next one out, which is why a phase IS the bench rather than a filter over
/// it - and a level is the same trick one story up.
#[derive(Resource)]
pub struct Stages {
    pub(crate) levels: Vec<Level>,
    /// Which level is being worked on.
    pub(crate) level: usize,
    /// Which of that level's phases is on the bench.
    pub(crate) showing: usize,
}

impl Default for Stages {
    fn default() -> Self {
        Stages {
            levels: vec![Level {
                name: String::new(),
                phases: vec![Vec::new()],
            }],
            level: 0,
            showing: 0,
        }
    }
}

impl Stages {
    /// A work of these levels, showing the first phase of the first.
    ///
    /// For the tests, which need a work well under way without a world to build it
    /// in. The level bar will want it too, and can drop the attribute then.
    #[cfg(test)]
    pub(crate) fn of(levels: Vec<Level>) -> Stages {
        Stages {
            levels: if levels.is_empty() {
                vec![Level {
                    name: String::new(),
                    phases: vec![Vec::new()],
                }]
            } else {
                levels
            },
            level: 0,
            showing: 0,
        }
    }

    /// How many phases the level being worked on has.
    pub fn count(&self) -> usize {
        self.phases().len()
    }

    pub fn showing(&self) -> usize {
        self.showing
    }

    /// Which level is being worked on.
    pub fn level(&self) -> usize {
        self.level.min(self.levels.len().saturating_sub(1))
    }

    /// Every level of the work.
    ///
    /// For the tests alone today - what a sweep leaves behind is checked through it -
    /// so it is marked as such rather than carried as dead weight. The level bar will
    /// want it, and can drop the attribute when it does.
    #[cfg(test)]
    pub(crate) fn all(&self) -> &[Level] {
        &self.levels
    }

    /// The phases of the level being worked on.
    ///
    /// Clamped rather than indexed raw: a level count that has just shrunk would
    /// otherwise take the bench down with it.
    pub(crate) fn phases(&self) -> &Vec<Vec<Placed>> {
        &self.levels[self.level()].phases
    }

    pub(crate) fn phases_mut(&mut self) -> &mut Vec<Vec<Placed>> {
        let at = self.level();
        &mut self.levels[at].phases
    }
}

/// The whole work as it stands, ready to be written.
///
/// The shown step is standing as ENTITIES; every other step is already records.
/// Gathering the shown one back in is what keeps a maker's last few minutes from
/// going missing from whichever step they happened to be on - and it was written
/// out three times, once per thing that writes a file, which is three chances for
/// one of them to be the copy that forgot.
pub(crate) fn gather_the_work<'a>(
    name: &str,
    stages: &Stages,
    standing: impl Iterator<Item = &'a Placed>,
) -> Workbench {
    let mut levels = stages.levels.clone();
    let at = stages.level();
    let showing = stages.showing.min(stages.count().saturating_sub(1));
    if let Some(slot) = levels[at].phases.get_mut(showing) {
        *slot = standing.cloned().collect();
    }
    Workbench {
        format: 2,
        name: name.to_string(),
        parts: Vec::new(),
        stages: Vec::new(),
        levels,
    }
}

/// Keeps whatever is standing, in the project's own `workbench.baz`.
///
/// For the moment the bench LEAVES one project for another. Nothing on the bench
/// is written automatically otherwise, so without this a maker who has drawn for
/// an hour and then opens another game loses the hour to a button press - and the
/// button would not even look dangerous.
///
/// It returns where it wrote, so the thing that called it can say so. An empty
/// bench writes nothing at all: a file saying nothing would overwrite one from
/// last time that said something.
pub(crate) fn keep_the_bench<'a>(
    stages: &Stages,
    standing: impl Iterator<Item = &'a Placed>,
    called: Option<&str>,
) -> Option<std::path::PathBuf> {
    let work = gather_the_work(called.unwrap_or("workbench"), stages, standing);
    if work.stages.iter().all(Vec::is_empty) {
        return None;
    }
    let path = bench_path();
    let _ = std::fs::create_dir_all(path.parent()?);
    let json = serde_json::to_string_pretty(&work).ok()?;
    std::fs::write(&path, json).ok()?;
    Some(path)
}

/// Turns a work from before stages into stages, by the rule the game used to
/// infer them with.
///
/// Which makes the change invisible where it should be: an old building comes
/// back with exactly the steps the village was already raising it in. The rule
/// is `step_of`, and the last stage is the whole finished work.
pub(crate) fn stages_from_flat(parts: &[Placed]) -> Vec<Vec<Placed>> {
    if parts.is_empty() {
        return vec![Vec::new()];
    }
    let framed = parts.iter().any(|record| record.stage == "frame");
    let steps = if framed { 4 } else { 3 };
    (0..steps)
        .map(|step| {
            parts
                .iter()
                .filter(|record| {
                    // The maker's own marks belong to the finished work, and
                    // stand throughout it besides.
                    record.stage == "widget" || step_of(&record.stage, framed) <= step as u8
                })
                .cloned()
                .collect()
        })
        .collect()
}

/// What a saved building is called on disk.
///
/// `.baz`, for the studio whose bench this is. `.json` said only what these are
/// written IN, which is true of a great many files and tells a maker looking at
/// a folder nothing at all; this says whose they are and what made them.
///
/// The contents are still JSON and always were - this is a name, not a format.
pub const WORK_KIND: &str = "baz";

/// Whether a file is a saved work: the name it wears now, or the one it wore
/// before the name changed.
///
/// Both, forever. A maker's buildings are not something to lose to a rename, and
/// the ones already on disk are the only two that exist.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn is_a_work(path: &std::path::Path) -> bool {
    path.extension()
        .is_some_and(|kind| kind == WORK_KIND || kind == "json")
}

/// Where the bench keeps everything: its own folder in a source tree, and the
/// maker's own Application Support beside the game's saves otherwise.
///
/// It used to be `CARGO_MANIFEST_DIR` or, failing that, the working directory —
/// which was right while the only way to run the bench was `cargo run` from its
/// own crate. It is opened from the game's title screen now, and a bundled bench
/// would have taken "the working directory" to mean INSIDE the `.app`: a place
/// that is read-only where it is installed properly, and that breaks the
/// signature where it is not.
///
/// The open project's root: where this game's work is saved, reopened and
/// committed alongside its own code.
pub(crate) fn bench_home() -> std::path::PathBuf {
    crate::project::root()
}

pub(crate) fn bench_path() -> std::path::PathBuf {
    crate::project::work().join(format!("workbench.{WORK_KIND}"))
}

/// Where the works are kept: the folder the bench saves into and loads from.
pub(crate) fn works_home() -> std::path::PathBuf {
    crate::project::work()
}

/// Where the Open dialog should start.
///
/// The project's own works, unless there are none yet - a project on its
/// first day has an empty work folder, and dropping the maker into it
/// with nothing to open is a poor way to begin. The starting shapes are
/// better company.
pub(crate) fn opening_home() -> std::path::PathBuf {
    let works = works_home();
    let empty = std::fs::read_dir(&works)
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(true);
    let templates = crate::project::templates();
    if empty && templates.is_dir() {
        return templates;
    }
    works
}

/// The save button asks the work its name; the writing happens when the
/// name is given, in [`take_the_name`].
pub(crate) fn save_workbench(
    mut commands: Commands,
    bench: Res<Bench>,
    fonts: Res<Fonts>,
    palette: Res<Palette>,
    work_name: Res<WorkName>,
    mut naming: ResMut<Naming>,
    saves: Query<&Interaction, (Changed<Interaction>, With<SaveButton>)>,
) {
    // The glyphs are one row for both benches; what they save is not.
    if *bench != Bench::Builder {
        return;
    }
    let pressed = saves
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if pressed && naming.0.is_none() {
        naming.0 = Some(work_name.0.clone().unwrap_or_default());
        naming.1 = NamingFor::Keeping;
        raise_naming_card(&mut commands, &fonts, &palette, NamingFor::Keeping, 0);
    }
}
