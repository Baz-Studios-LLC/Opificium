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

        let (json, boxes, marks) = bake_a_work(&work, &palette, &name, "");
        let out = baked_dir.join(format!("{name}.json"));
        std::fs::write(&out, json).expect("write baked");
        println!("baked {name}: {boxes} boxes, {marks} marks");
    }
}

/// A clock bakes its face as boxes and its size as a mark.
///
/// Brett: "I wonder if we should make it hands free and have the game create and animate
/// the hands?" That is the line the bench draws everywhere - the bake speaks STATIC boxes,
/// so a thing that moves belongs to the game - and it leaves the village one question the
/// boxes cannot answer: how big to draw the hands. So the mark carries it.
#[test]
fn a_clock_bakes_its_face_and_says_how_wide() {
    let palette = crate::look::load_palette_for_bake();
    let clock = Placed {
        part: part_name(&PartKind::Clock(1.25)),
        at: [2.0, 3.0, -1.0],
        yaw: 0.0,
        tilt: 0.0,
        ramp: None,
        shade: 0.5,
        stage: "furnishing".to_string(),
        flip: false,
        loose: false,
        material: String::new(),
        group: None,
    };
    let (boxes, marks) = bake_one_phase(std::slice::from_ref(&clock), &palette, Vec3::ZERO);

    // THE FACE IS BUILT. It does not move, so the village raises it like any other
    // part of the building.
    assert!(
        boxes.len() >= 6,
        "a clock baked {} boxes - its dial did not come through",
        boxes.len()
    );
    // AND THE MARK SAYS HOW WIDE. Without it the game must hardcode a size per word
    // and every clock in the world is the same clock.
    let clock_mark = marks
        .iter()
        .find(|line| line.contains("\"mark\": \"clock\""))
        .unwrap_or_else(|| panic!("no clock mark in {marks:?}"));
    assert!(
        clock_mark.contains("\"wide\": 1.2500"),
        "the clock mark does not say how wide it is: {clock_mark}"
    );
    // At the middle of the FACE, which is where a hand turns - not at the part's foot,
    // where nothing turns at all.
    assert!(
        clock_mark.contains("3.6250"),
        "the clock mark is not at the middle of its own face: {clock_mark}"
    );

    // And a mark with no size to have says nothing about size: a door's routing mark
    // is a place and only a place.
    let door = Placed {
        part: part_name(&PartKind::Widget("door")),
        stage: "widget".to_string(),
        ..clock.clone()
    };
    let (_, plain) = bake_one_phase(std::slice::from_ref(&door), &palette, Vec3::ZERO);
    assert!(
        plain.iter().all(|line| !line.contains("wide")),
        "a mark that is only a place grew a width: {plain:?}"
    );
}
