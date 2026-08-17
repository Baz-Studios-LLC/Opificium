//! Lifted whole out of `builder.rs`. See that module for what these check.

use super::*;

/// Bakes every saved work into what the game can eat: plain boxes
/// with resolved colors, and the marks that say what the place is
/// FOR. Run by hand when a building is ready to be carried in:
/// `cargo test bake_the_works -- --ignored --nocapture`
///
/// The game shares no code with the bench, so the bench resolves its
/// own catalog and palette here and hands over the result.
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
        plain
            .iter()
            .all(|line| !line.contains("wide") && !line.contains("size")),
        "a mark that is only a place grew a size: {plain:?}"
    );
}

/// A WIDTH AND A ROOM ARE DIFFERENT THINGS, and no mark says both.
///
/// This is the test I owed after tidying one into the other. Divus Factus asked
/// whether `wide` wanted retiring now that `size` existed; I agreed, folded the
/// clock into `size: [wide, wide, depth]`, and moved its anchor to the foot to
/// match a pallet. Then they read the clock and put it better than the question:
/// "a disc is not a box."
///
/// A dial's size is ONE number, so two of that volume's three were noise, and the
/// hands turn about the dial's MIDDLE where a stack grows up off its FLOOR. The
/// two anchor differently because they measure different kinds of thing, and no
/// amount of tidiness makes them one. So the rule is written down here where a
/// future tidy-up has to argue with it.
#[test]
fn a_width_and_a_room_are_not_the_same_field() {
    let palette = crate::look::load_palette_for_bake();
    let stand = |part: String| Placed {
        part,
        at: [0.0, 0.0, 0.0],
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
    // A DIAL says a width and no room, at the middle of its face.
    let dial = stand(part_name(&PartKind::Clock(2.0)));
    let (_, said) = bake_one_phase(std::slice::from_ref(&dial), &palette, Vec3::ZERO);
    let dial_mark = said.join("\n");
    assert!(
        dial_mark.contains("\"wide\"") && !dial_mark.contains("\"size\""),
        "a dial should say a width and no room: {dial_mark}"
    );
    assert!(
        dial_mark.contains("\"at\": [0.0000, 1.0000, 0.0000]"),
        "a dial's mark is not at the middle of its face: {dial_mark}"
    );
    // A ROOM says a room and no width, at the middle of its foot.
    let pallet = stand(part_name(&PartKind::Area {
        word: "pallet",
        long: 2.0,
        deep: 1.0,
        high: 1.0,
    }));
    let (_, said) = bake_one_phase(std::slice::from_ref(&pallet), &palette, Vec3::ZERO);
    let room_mark = said.join("\n");
    assert!(
        room_mark.contains("\"size\"") && !room_mark.contains("\"wide\""),
        "a room should say a room and no width: {room_mark}"
    );
    assert!(
        room_mark.contains("\"at\": [0.0000, 0.0000, 0.0000]"),
        "a room's mark is not at the middle of its foot: {room_mark}"
    );
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
