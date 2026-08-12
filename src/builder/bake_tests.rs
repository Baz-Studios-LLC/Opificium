//! Lifted whole out of `builder.rs`. See that module for what these check.

use super::*;

/// Bakes every saved work into what the game can eat: plain boxes
/// with resolved colours, and the marks that say what the place is
/// FOR. Run by hand when a building is ready to be carried in:
/// `cargo test bake_the_works -- --ignored --nocapture`
///
/// The game shares no code with the bench, so the bench resolves its
/// own catalogue and palette here and hands over the result.
#[test]
#[ignore = "a hand-run export, not a check"]
fn bake_the_works() {
    // Which game's works. Named by OPIFICIUM_PROJECT or on the
    // command line; without one there is nothing here to bake, and
    // saying so beats walking a stranger's folders.
    let project = crate::project::open_quietly()
        .expect("no project - set OPIFICIUM_PROJECT to a game's opificium folder");
    println!("baking {} ({})", project.name, project.root.display());

    let palette = crate::look::load_palette_for_bake();
    let dir = crate::project::work();
    let baked_dir = crate::project::baked();
    std::fs::create_dir_all(&baked_dir).expect("baked dir");

    for entry in std::fs::read_dir(&dir).expect("the project's work folder") {
        let path = entry.expect("entry").path();
        if !is_a_work(&path) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(work) = serde_json::from_str::<Workbench>(&text) else {
            continue;
        };
        let name = path.file_stem().unwrap().to_string_lossy().to_string();

        let (json, boxes, marks) = bake_a_work(&work, &palette, &name);
        let out = baked_dir.join(format!("{name}.json"));
        std::fs::write(&out, json).expect("write baked");
        println!("baked {name}: {boxes} boxes, {marks} marks");
    }
}
