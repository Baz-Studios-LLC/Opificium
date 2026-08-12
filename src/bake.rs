//! Baking without a bench.
//!
//! `opificium --bake` resolves every saved work in a project into what a game
//! can eat, and carries it where the manifest's `install` says. It is the same
//! code the BAKE button runs — [`crate::builder::carry_into_the_game`] — because
//! a headless path that resolves parts its own way is a second bench nobody
//! maintains, and it would drift in a week.
//!
//! It exists because a drawing could otherwise only enter a game through a
//! person pressing a button. That makes a floor plan a checked-in artefact
//! rather than a generated one: a build cannot rebuild the houses from their
//! blueprints, and neither can anyone who has the source but not the bench
//! open. The plumbing was already waiting for this — `open_quietly` says in so
//! many words that it is "for the headless paths, a bake run from a script or a
//! build" — there was simply nothing to call it.
//!
//! ```text
//! opificium --bake                      every work in the last project
//! opificium <project> --bake            every work in that one
//! opificium --bake main-house           just that work
//! opificium --bake main-house --kind house
//! OPIFICIUM_PROJECT=... opificium --bake
//! ```

/// What the command line asked to be baked.
pub struct Ask {
    /// Empty means everything.
    only: Vec<String>,
    /// Overrides the guess the name would make.
    kind: Option<String>,
}

/// Reads the command line. `None` means open the bench in the ordinary way.
pub fn asked_for() -> Option<Ask> {
    let mut args = std::env::args().skip(1);
    let mut wanted = false;
    let mut only = Vec::new();
    let mut kind = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bake" => wanted = true,
            "--kind" => kind = args.next(),
            // The project is already named by a bare path — see
            // `project::opening` — so a folder is never a work to bake.
            other if other.starts_with('-') => {}
            other if std::path::Path::new(other).is_dir() => {}
            other => only.push(other.to_string()),
        }
    }

    wanted.then_some(Ask { only, kind })
}

/// The kind a name suggests, the same way the naming card guesses it: the
/// LONGEST kind-word that begins the name, or `longhouse1` opens on "house".
///
/// Out of the PROJECT's own kinds, so a headless bake and a bake by hand reach the
/// same answer for the same drawing. A project with no kinds guesses nothing and
/// writes nothing, and the game reads the name itself - which is the reading every
/// drawing baked before there was a card relies on, so it is a true one and not a
/// shrug.
fn kind_from(name: &str) -> String {
    let known = crate::project::kinds();
    let mut order: Vec<usize> = (0..known.len()).collect();
    order.sort_by_key(|index| std::cmp::Reverse(known[*index].word.len()));
    order
        .into_iter()
        .find(|index| name.starts_with(&known[*index].word))
        .map(|index| known[index].word.clone())
        .unwrap_or_default()
}

/// Bakes, and returns the process's exit status.
pub fn run(ask: &Ask) -> i32 {
    let Some(project) = crate::project::open_quietly() else {
        eprintln!(
            "opificium --bake: no project. Name a folder on the command line, \
             or set OPIFICIUM_PROJECT to one."
        );
        return 2;
    };
    println!("baking {} ({})", project.name, project.root.display());

    let palette = crate::look::load_palette_for_bake();
    let work_dir = crate::project::work();
    let Ok(entries) = std::fs::read_dir(&work_dir) else {
        eprintln!("opificium --bake: nothing to read at {}", work_dir.display());
        return 2;
    };

    let mut works: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| crate::builder::is_a_work(path))
        .collect();
    works.sort();

    let mut baked = 0usize;
    let mut failed = 0usize;
    let mut found: Vec<String> = Vec::new();

    for path in &works {
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_default();
        found.push(name.clone());
        if !ask.only.is_empty() && !ask.only.iter().any(|wanted| *wanted == name) {
            continue;
        }

        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(why) => {
                eprintln!("{name}: {why}");
                failed += 1;
                continue;
            }
        };
        let work = match serde_json::from_str::<crate::builder::Workbench>(&text) {
            Ok(work) => work,
            Err(why) => {
                eprintln!("{name}: not a work this bench understands — {why}");
                failed += 1;
                continue;
            }
        };

        let kind = ask.kind.clone().unwrap_or_else(|| kind_from(&name));
        match crate::builder::carry_into_the_game(&work, &palette, &name, &kind) {
            Ok((boxes, marks)) => {
                // A project with no kinds says so, rather than trailing off
                // after "as a".
                let said = if kind.is_empty() {
                    "to be claimed by its name".to_string()
                } else {
                    format!("as a {kind}")
                };
                println!("  {name}: {boxes} boxes, {marks} marks, carried in {said}");
                baked += 1;
            }
            Err(why) => {
                eprintln!("  {name}: {why}");
                failed += 1;
            }
        }
    }

    // A name that matched nothing is nearly always a typo, and the cure is the
    // list rather than a shrug.
    for wanted in &ask.only {
        if !found.iter().any(|name| name == wanted) {
            eprintln!(
                "opificium --bake: no work called '{wanted}' — {} has: {}",
                work_dir.display(),
                if found.is_empty() {
                    "nothing".to_string()
                } else {
                    found.join(", ")
                }
            );
            failed += 1;
        }
    }

    if baked == 0 && failed == 0 {
        println!("nothing to bake in {}", work_dir.display());
    } else {
        println!(
            "baked {baked} into {}",
            crate::builder::carried_home("buildings").display()
        );
    }
    if failed > 0 { 1 } else { 0 }
}
