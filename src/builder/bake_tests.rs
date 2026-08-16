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
fn a_clock_bakes_its_face_and_says_how_big() {
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
    // AND THE MARK SAYS HOW BIG THE DIAL IS. Without it the game must hardcode a
    // size per word and every clock in the world is the same clock.
    //
    // Through `size`, which is the same field a pallet says its room with. It was
    // a `wide` of its own, and Divus Factus asked the right question about that:
    // one concept beats two, and a reader that had to know which parts said which
    // was a reader with two rules for one idea.
    let clock_mark = marks
        .iter()
        .find(|line| line.contains("\"mark\": \"clock\""))
        .unwrap_or_else(|| panic!("no clock mark in {marks:?}"));
    assert!(
        !clock_mark.contains("wide"),
        "the clock still says `wide` beside its size: {clock_mark}"
    );
    assert!(
        clock_mark.contains("\"size\": [1.2500, 1.2500, 0.2500]"),
        "the clock mark does not say how big its dial is: {clock_mark}"
    );
    // AT THE MIDDLE OF ITS FOOT, like every other sized mark - so one rule finds any
    // of them in the world. A village wanting the pivot the hands turn on adds half
    // the height, which is the arithmetic it already does for anything standing on
    // something. The dial's foot is the part's own seat, and its case stands forward
    // of the wall by half its depth.
    assert!(
        clock_mark.contains("\"at\": [2.0000, 3.0000, -0.8750]"),
        "the clock mark is not at the middle of its own foot: {clock_mark}"
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
        plain.iter().all(|line| !line.contains("size")),
        "a mark that is only a place grew a size: {plain:?}"
    );
}

/// AND THE DIAL IS AS DEEP AS THE MARK SAYS IT IS.
///
/// The mark's third number is a constant, and the case it describes is built out of
/// atoms somewhere else entirely. A case rebuilt thicker would go on claiming the
/// old depth, and the village would hang its hands a little inside the wood.
#[test]
fn a_clock_is_as_deep_as_it_says() {
    for wide in [1.0, 1.25, 2.0] {
        let body = body_of(&PartKind::Clock(wide), None);
        let front = body
            .iter()
            .map(|piece| piece.at.z + piece.size.z * 0.5)
            .fold(f32::NEG_INFINITY, f32::max);
        let back = body
            .iter()
            .map(|piece| piece.at.z - piece.size.z * 0.5)
            .fold(f32::INFINITY, f32::min);
        assert!(
            (back).abs() < 1e-4 && (front - CLOCK_DEEP).abs() < 1e-4,
            "a {wide} m clock's case stands {back}..{front}, not 0..{CLOCK_DEEP}"
        );
        // And as tall and wide as it says, so all three numbers are true.
        let top = body
            .iter()
            .map(|piece| piece.at.y + piece.size.y * 0.5)
            .fold(f32::NEG_INFINITY, f32::max);
        let side = body
            .iter()
            .map(|piece| piece.at.x.abs() + piece.size.x * 0.5)
            .fold(0.0f32, f32::max);
        assert!(
            (top - wide).abs() < 1e-4 && (side - wide * 0.5).abs() < 1e-4,
            "a {wide} m clock measures {side} across and {top} up"
        );
    }
}

/// A MARKED VOLUME BAKES ITS ROOM, at the foot the stack grows off.
///
/// The whole point of the thing: a game filling a space has to be told how big the
/// space is. Brett wanted "a pallet widget that I could put in the building that
/// the food stacks into", and everything the game needs to do that is these four
/// numbers - where the floor of it is, which way it lies, and how much room.
///
/// It is drawn as boxes on the bench so a maker can see it, and it must bake as
/// NONE, because it is not there: a pallet is where goods will be, and a crate
/// baked into the building would be a crate standing inside the pile.
#[test]
fn a_marked_volume_bakes_its_room_and_no_boxes() {
    let palette = crate::look::load_palette_for_bake();
    let pallet = Placed {
        part: part_name(&PartKind::Area {
            word: "pallet",
            long: 2.0,
            deep: 1.5,
            high: 1.25,
        }),
        at: [2.0, 0.0, -1.0],
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
    let (boxes, marks) = bake_one_phase(std::slice::from_ref(&pallet), &palette, Vec3::ZERO);
    assert!(
        boxes.is_empty(),
        "a marked volume was built into the building: {} boxes",
        boxes.len()
    );
    let said = marks.join("\n");
    assert!(
        said.contains("\"mark\": \"pallet\""),
        "the pallet does not say what it is: {said}"
    );
    // Long, HIGH, deep - the order a box is measured in everywhere else in this
    // file, so a reader that already parses `boxes` parses this the same way.
    assert!(
        said.contains("\"size\": [2.0000, 1.2500, 1.5000]"),
        "the pallet does not say how much room it has: {said}"
    );
    // At the middle of its FOOT: a stack grows upward off a floor, so the game is
    // given the floor rather than a point in the middle of the air.
    assert!(
        said.contains("\"at\": [2.0000, 0.0000, -1.0000]"),
        "the pallet does not stand on its own foot: {said}"
    );
    // AND A POINT MARK IS UNCHANGED. Declaring sizes is how a game opts a mark in;
    // every mark that has not is spelled exactly the way it always was.
    let door = Placed {
        part: part_name(&PartKind::Widget("door")),
        ..pallet.clone()
    };
    let (_, plain) = bake_one_phase(std::slice::from_ref(&door), &palette, Vec3::ZERO);
    assert!(
        plain.iter().all(|line| !line.contains("size")),
        "a mark that is only a place now claims to have room in it: {plain:?}"
    );
}
