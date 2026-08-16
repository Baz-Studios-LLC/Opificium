//! Lifted whole out of `builder.rs`. See that module for what these check.

use super::*;

/// Every ramp the bench NAMES, the bench can also PAINT.
///
/// A ramp name is a string literal, so nothing in the compiler is watching:
/// `shade` answers a ramp it does not know with the classic missing-colour,
/// which means a name with no ramp behind it is never an error anywhere. It
/// is a magenta wall. The bench's own palette held two ramps for a while and
/// `body_of` named fourteen, so a project with no `palette.json` came up with
/// a whole shelf of parts drawing in magenta and nothing on the screen to say
/// why.
///
/// So the shelf is walked instead of trusted, and the answer is a list rather
/// than a yes: the point of failing is knowing WHICH colour went missing.
#[test]
fn the_bench_can_paint_everything_it_names() {
    let palette = crate::look::bench_palette();
    let mut wanted: std::collections::BTreeSet<String> = Default::default();

    for entry in STRUCTURE.iter().chain(FURNITURE).chain(DECOR) {
        // A stretch is never drawn as itself - what it PLACES is what it
        // becomes at the drawn size, so that is the thing with a body.
        let kind = match entry.kind.run_axes() {
            Some(_) => entry.kind.run_made(2.0, 2.0),
            None => entry.kind,
        };
        for Slab { ramp, .. } in body_of(&kind, None) {
            wanted.insert(ramp);
        }
    }
    // The marks are NOT walked. A mark's ramp is named in the project's own
    // `widgets.json` now, so the bench cannot promise a colour for a word it
    // never chose - what it can promise is the `bone` it falls back to when a
    // mark names nothing, and that is asserted with the dress below.
    // And the bench's own dress, which is named in the other modules rather
    // than in any part's body: the floor grid and the door sill in `stage`,
    // the panels and the accent in `look::theme`, the three handle shafts in
    // `gizmo`.
    for dressing in ["bone", "cloth-gold", "stone", "cloth-red", "cloth-blue"] {
        wanted.insert(dressing.to_string());
    }

    // A walk that found nothing would report no missing colours either, which
    // is the one way this test could pass while saying nothing at all.
    assert!(
        wanted.len() >= 10,
        "only {} ramps found across the whole shelf - the walk is broken, not \
             the palette",
        wanted.len()
    );

    let missing: Vec<&str> = wanted
        .iter()
        .filter(|name| palette.ramp(name).is_none())
        .map(String::as_str)
        .collect();
    assert!(
        missing.is_empty(),
        "the bench draws in {missing:?}, which its own palette does not hold - \
             every one of those comes out magenta in a project that has not \
             exported a palette of its own"
    );
}

/// A saved work never loses a mark to a project that has not declared it.
///
/// The marks a project declares are what the SHELF offers and what colour a
/// block wears. They are not a list of what may be read: a work drawn in one
/// game and opened in another came back with its marks silently missing, and a
/// save would have made that permanent. The one kind of bug a maker cannot
/// undo, so it is checked rather than remembered.
#[test]
fn a_mark_reads_back_whether_the_project_knows_it_or_not() {
    // No project is open in a test, so nothing at all is declared - which is
    // the very case that used to lose them.
    for word in ["sleep", "door", "perch", "a-word-no-game-ever-had"] {
        let name = format!("widget:{word}");
        let read = kind_from_name(&name);
        assert!(
            matches!(read, Some(PartKind::Widget(had)) if had == word),
            "{name} read back as nothing"
        );
        // And it writes back out under the same name, or a work would change
        // shape every time it was opened and saved.
        assert_eq!(part_name(&read.unwrap()), name);
    }
    // An undeclared mark still has a body to draw, so it can be seen and
    // picked up rather than being an invisible part of the work.
    assert!(
        !body_of(&PartKind::Widget("perch"), None).is_empty(),
        "an undeclared mark draws nothing at all"
    );
}

/// A ramp runs shadow to bright, and the shelf leans on it.
///
/// `shade` is handed a 0..1 and reads the step nearest it, so a ramp whose
/// middle is darker than its foot would make a part's own shading read
/// backwards - the lit face darker than the one in shadow. Cheap to check and
/// impossible to see by eye across twenty-four of them.
#[test]
fn every_ramp_climbs() {
    for (name, steps) in crate::look::BENCH_RAMPS {
        let light = |[r, g, b]: [u8; 3]| r as u32 + g as u32 + b as u32;
        for pair in steps.windows(2) {
            assert!(
                light(pair[1]) > light(pair[0]),
                "{name} does not climb: {:?} then {:?}",
                pair[0],
                pair[1]
            );
        }
    }
}

/// Snapping a picked step onto a swatch changes no colour.
///
/// The dropper takes the step it FINDS - an authored 0.65, a 0.4 - and moves the
/// brush onto the nearest of the palette's five, so the armed square can wear the
/// gold ring. That is only honest if the two render identically, which they do
/// because `shade` reads the nearest of five steps anyway. Asserted rather than
/// assumed, since the whole trick rests on it.
#[test]
fn a_dropped_colour_snaps_without_changing() {
    let palette = crate::look::bench_palette();
    for name in ["wood", "stone", "bone", "earth", "cloth-gold"] {
        // Every step that appears in a part's own body, and the awkward middles.
        for shade in [
            0.0, 0.3, 0.35, 0.4, 0.45, 0.5, 0.65, 0.7, 0.75, 0.85, 0.95, 1.0,
        ] {
            let snapped = super::brush::nearest_swatch(shade);
            assert!(
                SWATCHES.contains(&snapped),
                "{shade} snapped to {snapped}, which is not a swatch"
            );
            assert_eq!(
                palette.shade(name, shade),
                palette.shade(name, snapped),
                "{name} at {shade} is a different colour from {name} at {snapped}, \
                 so the dropper would change what it copied"
            );
        }
    }
}

/// Every part the bench MAKES for itself survives the round trip through its name.
///
/// A punch hangs a leaf in an opening a framed wall has framed. Those leaves are on
/// no shelf, and `kind_from_name` resolves a prop by searching the shelves - so they
/// drew once and were unreadable ever after: gone at the next phase change, gone on
/// reopening, and absent from every bake, because a name that reads back as nothing
/// is skipped. Nothing failed and nothing was logged.
#[test]
fn what_the_bench_makes_it_can_also_read() {
    for made in PUNCHED {
        let name = format!("prop:{made}");
        let read = kind_from_name(&name);
        assert!(
            matches!(read, Some(PartKind::Prop(word)) if word == made),
            "{name} reads back as nothing, so it would vanish the first time the \
             bench rebuilt from records"
        );
        // And it writes back out under the same name, or a work would change shape
        // every time it was opened and saved.
        assert_eq!(part_name(&read.unwrap()), name);
        // And it has something to draw. A readable name with an empty body is the
        // same hole seen from the other side.
        assert!(
            !body_of(&PartKind::Prop(made), None).is_empty(),
            "{name} draws nothing at all"
        );
    }
}

/// An opening lands on the grid G sets, not on whole atoms regardless.
///
/// Placing a door was the one thing that ignored the grid - so a maker laying a
/// wall out in quarter metres had to nudge its door in sixteenths. Brett: "when
/// placing a door it should respect the grid settings when you press g".
#[test]
fn an_opening_lands_on_the_grid() {
    let along = Vec3::X;
    let wall = Vec3::ZERO;
    // A wall four metres long, a door 1.25 wide, aimed at an awkward spot.
    let aim = Vec3::new(0.31, 0.0, 0.0);
    for grid in [1, 2, 4, 8, 16] {
        let step = snap_step(false, grid);
        let seat = opening_seat(wall, along, 4.0, 1.25, aim, step);
        let strides = seat * step;
        assert!(
            (strides - strides.round()).abs() < 1e-4,
            "at grid {grid} the door seated at {seat}, which is not a whole stride"
        );
    }
    // Shift is the fine hand, always whole atoms, whatever the grid says.
    let fine = opening_seat(wall, along, 4.0, 1.25, aim, snap_step(true, 4));
    let atoms = fine / ATOM;
    assert!(
        (atoms - atoms.round()).abs() < 1e-4,
        "shift left {fine} off the lattice"
    );
    // And a coarse grid really is coarser: the same aim seats differently.
    assert_ne!(
        opening_seat(wall, along, 4.0, 1.25, aim, snap_step(false, 16)),
        opening_seat(wall, along, 4.0, 1.25, aim, snap_step(false, 1)),
        "the grid made no difference to where the door went"
    );
}

/// A double door frames a hole its own leaves fit through.
///
/// A framed wall used to reserve `DOOR_WIDE` for any door at all, because an
/// opening's width was implied by its kind - so a double door got a single door's
/// sixteen atoms and its two leaves stood over solid timber. Brett: "double doors
/// dont work when placing them on framed walls."
#[test]
fn a_double_door_gets_a_double_hole() {
    // What the one table says each opening needs cleared, and what its leaf spans.
    let clear_of = |what: &'static str| opening_of(&PartKind::Prop(what)).map(|opens| opens.clear);
    let single = clear_of("door").expect("a door");
    let double = clear_of("door-double").expect("a double door");
    assert_eq!(single, DOOR_WIDE);
    assert_eq!(
        double,
        DOOR_WIDE * 2,
        "a double door needs two leaves' worth"
    );

    // And the leaves really do span that: the reason the number is what it is.
    let spans = |what: &'static str| {
        let body = body_of(&PartKind::Prop(what), None);
        let (mut low, mut high) = (f32::INFINITY, f32::NEG_INFINITY);
        for piece in &body {
            // The leaves, not the latches: a latch is a stud of gold on the face.
            if piece.size.y < 1.0 {
                continue;
            }
            low = low.min(piece.at.x - piece.size.x * 0.5);
            high = high.max(piece.at.x + piece.size.x * 0.5);
        }
        high - low
    };
    for (leaf, clear) in [("door-leaf", single), ("door-double-leaf", double)] {
        let wants = spans(leaf);
        let got = clear as f32 * ATOM;
        assert!(
            (wants - got).abs() < 1e-4,
            "{leaf} spans {wants}m but its opening clears {got}m"
        );
    }

    // The wall really frames the wider hole: a long enough wall, one double door.
    let span = (6.0 / ATOM) as i32;
    let tall = (WALL_HIGH / ATOM) as i32;
    let holes = openings_at(
        span,
        tall,
        &[
            Some(Hole {
                wide: double,
                ..Hole::plain(Opening::Door, 0.0)
            }),
            None,
            None,
            None,
        ],
    );
    assert_eq!(holes.len(), 1, "the wall refused a door it has room for");
    assert_eq!(
        holes[0].1, double,
        "the wall framed {} atoms for a door that needs {double}",
        holes[0].1
    );
}

/// A framed wall's name carries an unusual width, and leaves a usual one out.
///
/// The second half is what keeps every drawing already on disk readable: a door of
/// the ordinary width writes exactly the name it always wrote.
#[test]
fn a_framed_wall_names_its_openings() {
    let plain = PartKind::Wall {
        framed: true,
        long: 3.0,
        high: WALL_HIGH,
        openings: [Some(Hole::plain(Opening::Door, 0.0)), None, None, None],
    };
    let name = part_name(&plain);
    assert_eq!(name, "wall-3x2.5xfxd0", "an ordinary door gained a width");
    assert!(kind_from_name(&name) == Some(plain), "it did not read back");

    let wide = PartKind::Wall {
        long: 4.0,
        framed: true,
        high: WALL_HIGH,
        openings: [
            Some(Hole {
                wide: DOOR_WIDE * 2,
                ..Hole::plain(Opening::Door, 0.5)
            }),
            None,
            None,
            None,
        ],
    };
    let name = part_name(&wide);
    assert!(
        name.ends_with("@32"),
        "the width never reached the name: {name}"
    );
    assert!(
        kind_from_name(&name) == Some(wide),
        "a wide door did not read back"
    );

    // A wall with NO framing word is a plain one - and it holds its openings just the
    // same, which is the whole point of there being one wall instead of two. A width left
    // unsaid is the usual width for its kind.
    let plain = kind_from_name("wall-3x2.5xd0.25").expect("a plain wall opens");
    let PartKind::Wall {
        framed, openings, ..
    } = plain
    else {
        panic!("not a wall")
    };
    assert!(!framed, "a wall with no framing word came back framed");
    let hole = openings[0].expect("its door");
    assert_eq!(hole.wide, DOOR_WIDE, "the door came back the wrong width");
    assert!((hole.at - 0.25).abs() < 1e-6);

    // And a plain wall says itself plainly: no framing word, and nothing else either.
    assert_eq!(part_name(&PartKind::wall(3.0)), "wall-3x2.5");
}

/// A swept bench is an empty work: one level, one phase, nothing in it.
///
/// Despawning the standing parts only emptied the phase on the stage. Every other
/// phase and every other level went on existing as records, so switching back
/// brought them out again and a save wrote them out - a bench that looked swept and
/// was not. Brett: "sweeping the bench should sweep everystage and everything."
///
/// Checked on `Stages` itself, since what was wrong was never what the eye saw.
#[test]
fn a_swept_bench_keeps_no_level_at_all() {
    let part = Placed {
        part: "wall-2".to_string(),
        at: [0.0; 3],
        yaw: 0.0,
        tilt: 0.0,
        ramp: None,
        shade: 0.7,
        stage: "walls".to_string(),
        flip: false,
        group: None,
        loose: false,
        material: String::new(),
    };
    // A work well under way: two levels, several phases apiece, parts in all of them.
    let busy = Stages::of(vec![
        Level {
            name: "base".into(),
            phases: vec![vec![part.clone()], vec![part.clone(), part.clone()]],
        },
        Level {
            name: "forge".into(),
            phases: vec![vec![part.clone()]],
        },
    ]);
    assert_eq!(busy.all().len(), 2);
    assert!(busy.all().iter().any(|level| !level.phases[0].is_empty()));

    // What a sweep leaves: the same thing a bench opens with.
    let swept = Stages::default();
    assert_eq!(swept.all().len(), 1, "a swept bench kept a level");
    assert_eq!(swept.count(), 1, "a swept bench kept a phase");
    assert_eq!(swept.showing(), 0);
    for level in swept.all() {
        for phase in &level.phases {
            assert!(phase.is_empty(), "a swept bench kept parts in a phase");
        }
    }
    // And nothing of the busy work survives into it.
    assert!(
        swept.all().iter().all(|level| level.name.is_empty()),
        "a swept bench remembered a level's name"
    );
}

#[cfg(test)]
/// A group is carried as one thing, in every mode but the brush.
///
/// Brett: "When I group a lot of items into a group, I should be able to move the group as
/// one piece right?", and "currently Normal mode ignores the group as well."
///
/// The slide has always carried the rest of a choice along with the part its handle hangs
/// on. What was missing is the handle: it was hung on the ONE part chosen, so the moment
/// there were two there was nothing to take hold of - the offer written for a single part
/// and the answer written for a set, which is this bench's oldest fault.
#[test]
fn a_group_is_carried_as_one_thing() {
    let record = Placed {
        part: part_name(&PartKind::wall(2.0)),
        at: [0.0, 0.0, 0.0],
        yaw: 0.0,
        tilt: 0.0,
        ramp: None,
        shade: 0.7,
        stage: "walls".to_string(),
        flip: false,
        loose: false,
        material: String::new(),
        group: Some(1),
    };
    use crate::gizmo::{Grip, ToolMode, handles_for_choice};
    for count in [2usize, 4, 12] {
        for mode in [ToolMode::Normal, ToolMode::Move, ToolMode::Resize] {
            let worn = handles_for_choice(mode, count, &record);
            assert_eq!(
                worn.len(),
                3,
                "a group of {count} wears {} handles - it has nothing to take hold of",
                worn.len()
            );
            assert!(
                worn.iter().all(|(.., grip)| matches!(grip, Grip::Slide)),
                "a group of {count} is offered a handle that would size or pitch one of \
                 them - several things cannot be stretched at once"
            );
        }
        // The brush is the exception, and stays one: the part IS the handle there.
        assert!(
            handles_for_choice(ToolMode::Paint, count, &record).is_empty(),
            "the brush grew handles"
        );
    }
    // And ONE part still wears exactly what it wore: nothing standing, its own when
    // sized. A group's rule may not quietly become everything's.
    assert!(handles_for_choice(ToolMode::Normal, 1, &record).is_empty());
    let alone = handles_for_choice(ToolMode::Resize, 1, &record);
    assert!(
        alone
            .iter()
            .any(|(.., grip)| matches!(grip, Grip::Size { .. }))
            && alone
                .iter()
                .any(|(.., grip)| matches!(grip, Grip::Rise { .. })),
        "a wall on its own has lost its length or its height"
    );
}

/// A post is a beam stood up: placed at a height, and pulled to another.
///
/// Brett: "pole should be exactly like the beam only verticle", and then "pole, corner is
/// obsolete once we make the new pole." It was a prop - one height, forever, with no handle
/// on it - so the post for a cottage and the post for a tower were the same part and
/// neither could be made.
#[test]
fn a_post_stands_as_tall_as_it_is_drawn() {
    let entry = STRUCTURE
        .iter()
        .find(|entry| entry.label == "POLE")
        .expect("the shelf has lost its post");
    let PartKind::Pole(high) = entry.kind else {
        panic!("the shelf's post is not a post")
    };
    assert!(high > 1.0, "the shelf's post is {high}m - a stub");

    // Its body follows its height, which is the whole of what it is.
    for want in [1.0f32, 2.5, 6.0] {
        let tall = body_of(&PartKind::Pole(want), None)
            .iter()
            .map(|Slab { at, size, .. }| at.y + size.y * 0.5)
            .fold(0.0f32, f32::max);
        assert!(
            (tall - want).abs() < 1e-4,
            "a post asked for {want}m stands {tall}m"
        );
    }

    // THE GOLD HANDLE, both halves: every part that is offered one answers when it is
    // pulled. Offered and unanswered is the fault this bench keeps repeating - the handle
    // appeared on every wall and moved only the framed ones - and it cannot be seen by
    // looking at either side alone.
    for kind in [
        PartKind::Pole(2.5),
        PartKind::Foundation(2.0, 2.0, 0.5),
        PartKind::wall(4.0),
        PartKind::Wall {
            long: 4.0,
            high: WALL_HIGH,
            framed: true,
            openings: [None; MOST_OPENINGS],
        },
    ] {
        let Some(was) = crate::gizmo::stands_at(&kind) else {
            panic!("{} is not offered a height at all", part_name(&kind))
        };
        let Some(made) = crate::gizmo::risen(kind, was + 0.5) else {
            panic!(
                "{} is offered a height and does not answer",
                part_name(&kind)
            )
        };
        assert_eq!(
            crate::gizmo::stands_at(&made),
            Some(was + 0.5),
            "{} did not take the new height",
            part_name(&kind)
        );
    }
    // And nothing else is offered one, so the answer above is not answering for parts
    // that never asked.
    for kind in [PartKind::Beam(2.0, 0.0, 0.0), PartKind::Floor(2.0, 2.0)] {
        assert!(
            crate::gizmo::stands_at(&kind).is_none(),
            "{} is offered a height it has no use for",
            part_name(&kind)
        );
    }

    // A post drawn before it had a height of its own still opens, at the height it had.
    assert!(
        matches!(kind_from_name("prop:pole"), Some(PartKind::Pole(high)) if (high - WALL_HIGH).abs() < 1e-4),
        "the old corner posts no longer open"
    );
    let name = part_name(&PartKind::Pole(3.75));
    assert!(
        matches!(kind_from_name(&name), Some(PartKind::Pole(high)) if (high - 3.75).abs() < 1e-4),
        "{name} did not come back"
    );
}

/// The parts a maker PULLS are handed over whole, not as a stub to stretch.
///
/// Brett, of the gable and then the beam: "I like how the wall works where you have say a
/// 2m wall and you place it and then stretch it." Two halves as always - the shelf has to
/// hand over a real part, AND the handles have to be willing to size one - and a part that
/// went back to being a run would fail here rather than in a maker's hands.
#[test]
fn what_you_pull_is_placed_whole() {
    for label in ["WALL", "GABLE", "BEAM"] {
        let entry = STRUCTURE
            .iter()
            .find(|entry| entry.label == label)
            .unwrap_or_else(|| panic!("the shelf has lost its {label}"));
        assert!(
            entry.kind.run_axes().is_none(),
            "{label} is a run: it would want stretching out of a stub"
        );
        let Some((long, rebuild)) = length_of(&entry.kind) else {
            panic!("{label} wears no length handle once it is down")
        };
        assert!(
            long > 1.0,
            "the shelf's {label} is {long}m - a stub, not something to place and pull"
        );
        let Some((now, _)) = length_of(&rebuild(6.0)) else {
            panic!("pulling a {label} made something with no length at all")
        };
        assert!(
            (now - 6.0).abs() < 1e-4,
            "{label} did not take the new length: {now}"
        );
    }
}

mod bars {
    use super::*;

    /// A window's dark bars survive being written to a name and read back.
    ///
    /// The name is what a `.baz` holds, so a flag that does not round-trip is a flag that
    /// looks set until the work is reopened - which is how the `kind` field was lost once
    /// before, silently, for a whole release.
    #[test]
    fn black_bars_survive_a_round_trip() {
        let dark = PartKind::Wall {
            long: 4.0,
            high: WALL_HIGH,
            framed: true,
            openings: [
                Some(Hole {
                    dark: true,
                    ..Hole::plain(Opening::Window, 0.0)
                }),
                None,
                None,
                None,
            ],
        };
        let name = part_name(&dark);
        let Some(PartKind::Wall {
            framed: true,
            openings,
            ..
        }) = kind_from_name(&name)
        else {
            panic!("a framed wall did not read back: {name}");
        };
        assert!(
            openings[0].expect("its window").dark,
            "the bars came back as timber: {name}"
        );

        // And a wall drawn before bars could be dark writes exactly the name it always
        // wrote, so every saved building reads back byte for byte the same.
        let plain = PartKind::Wall {
            framed: true,
            long: 4.0,
            high: WALL_HIGH,
            openings: [Some(Hole::plain(Opening::Window, 0.0)), None, None, None],
        };
        let said = part_name(&plain);
        assert!(!said.contains('!'), "a plain wall's name changed: {said}");
        let Some(PartKind::Wall {
            framed: true,
            openings,
            ..
        }) = kind_from_name(&said)
        else {
            panic!("it did not read back");
        };
        assert!(!openings[0].expect("its window").dark);
    }
}

#[cfg(test)]
mod framing {
    use super::*;

    /// A brace meets the timber above and below it squarely, at any wall size.
    ///
    /// Both of a brace's ends land on horizontal timber - the sill under it, the rail over -
    /// so both end faces are horizontal and the brace is a parallelogram. That is what the
    /// saw runs are for, and getting one wrong shows as ends that lean too far and leave a
    /// triangle of daylight in every corner. Brett, with a picture of a squat wall: "when
    /// framed walls get short the lines dont stay clean."
    ///
    /// Read off the pieces themselves, corner by corner, because the fault is invisible in
    /// the numbers that made them: a run measured in the wrong units is still a run.
    #[test]
    fn a_brace_meets_its_timber_squarely() {
        for high in [1.25f32, 1.75, 2.5, 3.5] {
            for long in [2.0f32, 4.0, 9.0] {
                let body = body_of(
                    &PartKind::Wall {
                        long,
                        high,
                        framed: true,
                        openings: [None; MOST_OPENINGS],
                    },
                    None,
                );
                // The course the braces stand in, from the one place that says where a
                // wall's courses are.
                let tall = (high / ATOM).round() as i32;
                let (inner_foot, low_tall, ..) = courses_of(tall);
                let (sill, rail) = (
                    inner_foot as f32 * ATOM,
                    (inner_foot + low_tall) as f32 * ATOM,
                );
                let braces = body.iter().filter(|slab| slab.cant.abs() > 1e-3);
                let mut seen = 0;
                for brace in braces {
                    seen += 1;
                    // IT FILLS THE COURSE. Cutting the ends flat takes the brace's own
                    // width off its climb, so a brace as long as the bay's diagonal
                    // stops short at BOTH ends and leaves a line of plaster under the
                    // rail and over the sill. Brett: "there is still a gap on the top",
                    // and then "bottom too".
                    let turn = Mat2::from_angle(brace.cant);
                    let half = Vec2::new(brace.size.x, brace.size.y) * 0.5;
                    let (mut low, mut top) = (f32::INFINITY, f32::NEG_INFINITY);
                    for sx in [-1.0f32, 1.0] {
                        for sy in [-1.0f32, 1.0] {
                            let run = if sx < 0.0 { brace.cut.x } else { brace.cut.y };
                            let sawn = if (sy > 0.0 && run > 0.0) || (sy < 0.0 && run < 0.0) {
                                run.abs()
                            } else {
                                0.0
                            };
                            let corner = Vec2::new(brace.at.x, brace.at.y)
                                + turn * Vec2::new(sx * (half.x - sawn), sy * half.y);
                            low = low.min(corner.y);
                            top = top.max(corner.y);
                        }
                    }
                    assert!(
                        (low - sill).abs() < 1e-3 && (top - rail).abs() < 1e-3,
                        "a brace in a {long}m wall {high}m high stands {low}..{top} where \
                         its course is {sill}..{rail} - it leaves a gap at one end or \
                         runs past it"
                    );
                    let turn = Mat2::from_angle(brace.cant);
                    let half = Vec2::new(brace.size.x, brace.size.y) * 0.5;
                    // Each end of the piece, as its two corners stand in the world.
                    for sx in [-1.0f32, 1.0] {
                        let run = if sx < 0.0 { brace.cut.x } else { brace.cut.y };
                        let mut ends = Vec::new();
                        for sy in [-1.0f32, 1.0] {
                            // A positive run cuts the top face back, a negative one the
                            // foot - see `cut_mesh`.
                            let sawn = if (sy > 0.0 && run > 0.0) || (sy < 0.0 && run < 0.0) {
                                run.abs()
                            } else {
                                0.0
                            };
                            ends.push(
                                Vec2::new(brace.at.x, brace.at.y)
                                    + turn * Vec2::new(sx * (half.x - sawn), sy * half.y),
                            );
                        }
                        let lean = (ends[0].y - ends[1].y).abs();
                        assert!(
                            lean < 1e-3,
                            "a brace in a {long}m wall {high}m high has an end leaning \
                             {lean} out of true - it cannot sit flat on the timber it \
                             lands on"
                        );
                    }
                }
                assert!(
                    seen >= 2,
                    "only {seen} braces in a {long}m wall {high}m high"
                );
            }
        }
    }

    /// Framing a wall changes only its framing.
    ///
    /// The right-click rebuilds the part from a name, so anything the rebuild forgets is
    /// silently lost - and a maker who frames a wall to see how it looks would find their
    /// door moved or their height reset, with nothing to point at.
    #[test]
    fn framing_keeps_everything_else() {
        let plain = PartKind::Wall {
            long: 4.0,
            high: 3.0,
            framed: false,
            openings: [
                Some(Hole {
                    dark: true,
                    ..Hole::plain(Opening::Window, 0.75)
                }),
                None,
                None,
                None,
            ],
        };
        // What the menu does, in the one line that matters.
        let PartKind::Wall {
            long,
            high,
            openings,
            ..
        } = plain
        else {
            panic!("not a wall")
        };
        let framed = PartKind::Wall {
            long,
            high,
            framed: true,
            openings,
        };
        let back = kind_from_name(&part_name(&framed)).expect("it reads back");
        let PartKind::Wall {
            long,
            high,
            framed,
            openings,
        } = back
        else {
            panic!("not a wall")
        };
        assert!(framed, "it did not come back framed");
        assert!((long - 4.0).abs() < 1e-6, "its length moved");
        assert!((high - 3.0).abs() < 1e-6, "its height moved");
        let hole = openings[0].expect("its window");
        assert!((hole.at - 0.75).abs() < 1e-6, "its window moved");
        assert!(hole.dark, "its black bars went back to timber");
    }

    /// Every wall is offered the framing line, and offered the OTHER state.
    ///
    /// Offering and answering are two different matches, and the last two bugs were both
    /// one of them moving without the other - a height handle that appeared and did
    /// nothing when pulled.
    #[test]
    fn every_wall_is_offered_the_other_state() {
        for framed in [false, true] {
            let wall = PartKind::Wall {
                long: 3.0,
                high: WALL_HIGH,
                framed,
                openings: [None; MOST_OPENINGS],
            };
            let deeds = deeds_for(&wall);
            assert!(
                deeds.contains(&Deed::Frame(!framed)),
                "a {} wall was not offered the other state",
                if framed { "framed" } else { "plain" }
            );
            assert!(
                !deeds.contains(&Deed::Frame(framed)),
                "a wall was offered what it already is"
            );
        }
    }
}

#[cfg(test)]
mod windows {
    use super::*;

    fn a_wall_with_a_window(framed: bool) -> Vec<Slab> {
        body_of(
            &PartKind::Wall {
                long: 4.0,
                high: WALL_HIGH,
                framed,
                openings: [Some(Hole::plain(Opening::Window, 0.0)), None, None, None],
            },
            None,
        )
    }

    /// A window is the same window in a plain wall as in a framed one.
    ///
    /// Brett: "the windows that go on normal walls and the windows that punch into framed
    /// walls are totally different. They should work the same and basically be the same
    /// window." They were two systems: a framed wall was TOLD about an opening and solved
    /// around it, while a plain wall was cut into `Seg` leftovers - at a different width,
    /// and with no glazing at all.
    #[test]
    fn a_window_is_the_same_window_in_either_wall() {
        let plain = a_wall_with_a_window(false);
        let framed = a_wall_with_a_window(true);

        // THE PANES. The bars are the thinnest timber in a wall, and a plain wall used to
        // have none of them - a hole with daylight through it.
        let bars = |body: &Vec<Slab>| {
            body.iter()
                .filter(|Slab { size, .. }| size.x < WALL_THICK && size.z < WALL_THICK)
                .count()
        };
        assert!(bars(&plain) > 0, "a plain wall's window has no bars in it");
        assert_eq!(
            bars(&plain),
            bars(&framed),
            "the two walls glaze their windows differently"
        );

        // THE HOLE ITSELF, in the same place and of the same size. Read as the gap in the
        // wall's own substance rather than from the numbers that made it.
        let clear = |body: &Vec<Slab>| {
            let widest = body
                .iter()
                .filter(|Slab { size, .. }| size.z >= WALL_THICK - 1e-4)
                .map(|Slab { at, size, .. }| (at.y - size.y * 0.5, at.y + size.y * 0.5))
                .fold((f32::INFINITY, f32::NEG_INFINITY), |had, (low, high)| {
                    (had.0.min(low), had.1.max(high))
                });
            widest
        };
        let (plain_low, plain_high) = clear(&plain);
        let (framed_low, framed_high) = clear(&framed);
        assert!(
            (plain_low - framed_low).abs() < 1e-4 && (plain_high - framed_high).abs() < 1e-4,
            "the walls do not even stand the same height: {plain_low}..{plain_high} \
             against {framed_low}..{framed_high}"
        );
    }

    /// A plain wall's jamb frames the window and stops there.
    ///
    /// It used to reach for a framed wall's plates, which put a post down the wall to the
    /// floor beside every window - and z-fighting with the plaster it stood in, because the
    /// wall fills the band a jamb had no business occupying. Both faults, one number.
    #[test]
    fn a_plain_walls_jamb_stops_at_the_window() {
        let body = a_wall_with_a_window(false);
        // The opening's own band, read off the wall: the plaster leaves a gap there.
        let (hole_low, hole_high) = {
            let framed = a_wall_with_a_window(true);
            let bars: Vec<&Slab> = framed
                .iter()
                .filter(|Slab { size, .. }| size.x < WALL_THICK && size.z < WALL_THICK)
                .collect();
            let low = bars
                .iter()
                .map(|Slab { at, size, .. }| at.y - size.y * 0.5)
                .fold(f32::INFINITY, f32::min);
            let high = bars
                .iter()
                .map(|Slab { at, size, .. }| at.y + size.y * 0.5)
                .fold(f32::NEG_INFINITY, f32::max);
            (low, high)
        };
        // A jamb is a narrow full-thickness timber. None of them may reach below the
        // window they frame.
        let lowest_jamb = body
            .iter()
            .filter(|Slab { size, .. }| size.z >= WALL_THICK - 1e-4 && size.x < 0.3 && size.y > 0.3)
            .map(|Slab { at, size, .. }| at.y - size.y * 0.5)
            .fold(f32::INFINITY, f32::min);
        assert!(
            lowest_jamb.is_infinite() || lowest_jamb >= hole_low - 1e-3,
            "a jamb reaches down to {lowest_jamb} where the window starts at {hole_low}: \
             a post beside the window, and z-fighting with the plaster around it"
        );
        let _ = hole_high;
    }

    /// Nothing in a plain wall stands in the same place as anything else.
    ///
    /// Z-fighting is two solids at one depth, and it is invisible in the numbers unless
    /// something compares them - it shows up as speckle on a maker's screen and nowhere
    /// else. A plain wall drew plaster over the whole head and apron AND a lintel and sill
    /// inside them, which is exactly that.
    #[test]
    fn a_wall_does_not_fight_itself() {
        // A PLAIN wall, strictly: its plaster and its window's frame are laid by two
        // different pieces of reasoning, and where they met was the speckle Brett saw.
        no_two_solids_share_space(&a_wall_with_a_window(false), false);

        // A FRAMED wall, at the one place this touched: a window sits ON the rail, so the
        // rail gives way where a sill lands. The rest of a framed wall's carpentry has
        // long-standing overlaps - braces and noggings running into the corner posts -
        // which have never shown, being all one wood at one shade. Not chased here, and
        // not pretended away either.
        let framed = a_wall_with_a_window(true);
        let sill = framed
            .iter()
            .find(|Slab { size, .. }| size.z > WALL_THICK + 1e-4)
            .expect("a framed window has a proud sill");
        for other in &framed {
            if std::ptr::eq(other, sill) || other.size.z > WALL_THICK + 1e-4 {
                continue;
            }
            let apart = |at_a: f32, s_a: f32, at_b: f32, s_b: f32| {
                (at_a - at_b).abs() >= (s_a + s_b) * 0.5 - 1e-3
            };
            assert!(
                apart(sill.at.x, sill.size.x, other.at.x, other.size.x)
                    || apart(sill.at.y, sill.size.y, other.at.y, other.size.y),
                "the sill shares its place with {:?} {:?} - the rail did not give way",
                other.at,
                other.size
            );
        }
    }

    /// A window is the size it was given, in any wall you put it in.
    ///
    /// The whole of what Brett asked for, in one assertion: "i want to uncouple the windows
    /// size and pane count from the wall height." A window WAS the wall's upper course, so
    /// the same window in a cottage and in a hall was two different windows - and the one
    /// the townhall wanted, three panes tall over its door, was not a window any wall would
    /// give.
    #[test]
    fn a_window_is_the_size_it_was_given() {
        // The clear opening's height, read off the mullion that divides it - the one piece
        // that is exactly as tall as the glass.
        let glass = |wall_high: f32, high: i32| {
            body_of(
                &PartKind::Wall {
                    long: 6.0,
                    high: wall_high,
                    framed: true,
                    openings: [
                        Some(Hole {
                            high,
                            lift: LOWEST_SILL,
                            ..Hole::plain(Opening::Window, 0.0)
                        }),
                        None,
                        None,
                        None,
                    ],
                },
                None,
            )
            .iter()
            .filter(|Slab { size, .. }| size.z < WALL_THICK - 1e-4 && size.x < 0.2 && size.y > 0.2)
            .map(|Slab { size, .. }| size.y)
            .fold(0.0f32, f32::max)
        };
        // Three panes up, in a cottage wall, a hall wall and one between.
        let want = panes_across(3) as f32 * ATOM;
        for wall_high in [2.5, 3.0, 3.5, 4.0] {
            let got = glass(wall_high, panes_across(3));
            assert!(
                (got - want).abs() < 1e-4,
                "the same window is {got} tall in a wall of {wall_high} and {want} in \
                 another - the wall is still deciding"
            );
        }
    }

    /// And the panes follow the size, exactly, both ways.
    ///
    /// A window divides into as many panes as it has room for, which is the rule that made
    /// the count the wall's business too. Asking for three and getting three is the whole
    /// contract, and it holds because the atoms and the panes are exact inverses.
    #[test]
    fn a_window_has_the_panes_it_was_asked_for() {
        for across in 1..=MOST_PANES {
            for up in 1..=MOST_PANES {
                let panes = WindowPanes { across, up };
                let body = body_of(&panes.window(), None);
                // A mullion is thin across and tall; a transom is the other way about.
                let thin = |body: &[Slab], up: bool| {
                    body.iter()
                        .filter(|Slab { at, size, .. }| {
                            let _ = at;
                            size.z < WALL_THICK - 1e-4
                                && if up { size.y < 0.2 } else { size.x < 0.2 }
                                && if up { size.x > 0.2 } else { size.y > 0.2 }
                        })
                        .count() as i32
                };
                assert_eq!(
                    thin(&body, false),
                    across - 1,
                    "a window of {across} panes across has the wrong count of mullions"
                );
                assert_eq!(
                    thin(&body, true),
                    up - 1,
                    "a window of {up} panes up has the wrong count of transoms"
                );
            }
        }
    }

    /// The size a maker chose is the size the shelf hands back, and the size the menu marks.
    ///
    /// A townhall wants seven windows of one size. Without this a maker sizes one through
    /// the menu and the shelf gives them the two-by-two again for the next - six more trips
    /// through a menu to build one facade.
    #[test]
    fn the_shelf_hands_back_the_size_you_chose() {
        let shelf = PartKind::Window {
            wide: WINDOW_WIDE,
            high: WINDOW_WIDE,
        };
        let chosen = WindowPanes { across: 2, up: 3 };
        assert!(
            crate::builder::from_the_shelf(shelf, chosen) == chosen.window(),
            "the shelf handed back a window of a size nobody asked for"
        );
        // And nothing else on the shelf is touched by it.
        let wall = PartKind::wall(4.0);
        assert!(
            crate::builder::from_the_shelf(wall, chosen) == wall,
            "the shelf resized something that was not a window"
        );

        // The menu marks the line the window is standing on - BOTH halves, since a drawer
        // that offers four sizes and marks none leaves a maker counting panes by eye.
        let wall = PartKind::Wall {
            long: 4.0,
            high: WALL_HIGH,
            framed: true,
            openings: [
                Some(Hole {
                    wide: panes_across(2),
                    high: panes_across(3),
                    ..Hole::plain(Opening::Window, 0.0)
                }),
                None,
                None,
                None,
            ],
        };
        for (deed, want) in [
            (Deed::Panes { up: true, count: 3 }, true),
            (Deed::Panes { up: true, count: 2 }, false),
            (
                Deed::Panes {
                    up: false,
                    count: 2,
                },
                true,
            ),
            (
                Deed::Panes {
                    up: false,
                    count: 3,
                },
                false,
            ),
        ] {
            assert_eq!(
                deed.is_standing(&wall, "walls", ""),
                want,
                "the menu marks the wrong line for a window of two panes by three"
            );
        }
        // And the drawers are offered at all, which is the other half again: a size nobody
        // can reach is a size nobody has.
        let deeds = deeds_for(&wall);
        for drawer in [PANES_ACROSS, PANES_UP] {
            assert!(
                deeds.contains(&Deed::More(drawer)),
                "a wall with a window does not offer {drawer}"
            );
            assert_eq!(
                deeds_in(drawer).len() as i32,
                MOST_PANES,
                "{drawer} does not offer every size"
            );
        }
    }

    /// Every opening is closed at the top, in either kind of wall.
    ///
    /// A plain wall had no head at all: its plaster carried straight on over the hole, so a
    /// window stood there with a sill proud under it and nothing whatever across the top.
    /// Brett, with a picture of one: "Can we get the top of the window fixed here? I am just
    /// talking about a sill on the top." Which is what a head is - the sill upside down.
    #[test]
    fn every_opening_has_a_head() {
        for framed in [false, true] {
            for what in [Opening::Window, Opening::Door] {
                let hole = Hole::plain(what, 0.0);
                let wall = PartKind::Wall {
                    long: 5.0,
                    high: WALL_HIGH,
                    framed,
                    openings: [Some(hole), None, None, None],
                };
                // Where the opening's head actually is, read off the wall rather than off
                // the hole: the clamps are the solver's business.
                let tall = (WALL_HIGH / ATOM).round() as i32;
                let (_, _, _, hy, hh, _) = openings_at(
                    (5.0 / ATOM).round() as i32,
                    tall,
                    &[Some(hole), None, None, None],
                )[0];
                let head = (hy + hh) as f32 * ATOM;
                let over = body_of(&wall, None)
                    .into_iter()
                    .find(|Slab { at, size, .. }| {
                        // Standing proud of both faces, sitting on the opening's head, and
                        // spanning at least the clear opening.
                        size.z > WALL_THICK + 1e-4
                            && (at.y - size.y * 0.5 - head).abs() < 1e-3
                            && size.x >= hole.wide as f32 * ATOM - 1e-3
                    });
                assert!(
                    over.is_some(),
                    "a {} wall's {} has nothing across the top of it",
                    if framed { "framed" } else { "plain" },
                    if what == Opening::Door {
                        "door"
                    } else {
                        "window"
                    }
                );
            }
        }
    }

    /// A window goes where it is put, and takes its frame with it.
    ///
    /// Brett: "I really want to rethink windows. I should be able to place the window atom
    /// perfect anywhere on the wall." It could not: a window's height was not a number
    /// anywhere. It WAS the wall's upper course, so the only thing it could be told was how
    /// far along - and the one degree of freedom it lacked is the one that makes a townhall.
    #[test]
    fn a_window_goes_where_it_is_put() {
        // The same window, in the same wall, at three heights - the one its kind gives it
        // and two of its own.
        let at_band = |band: Band| {
            body_of(
                &PartKind::Wall {
                    long: 4.0,
                    high: WALL_HIGH,
                    framed: true,
                    openings: [
                        Some(Hole {
                            lift: band.foot,
                            high: band.rise,
                            ..Hole::plain(Opening::Window, 0.0)
                        }),
                        None,
                        None,
                        None,
                    ],
                },
                None,
            )
        };
        // Where the glass is, read off the wall: the proud sill is under it and nothing
        // else in a wall stands proud of both faces.
        // The LOWEST of the pieces standing proud of both faces: a window has two of
        // them now, a sill under it and a head over, and "the first one in the list" is
        // not a question about geometry.
        let sill_of = |body: &[Slab]| {
            body.iter()
                .filter(|Slab { size, .. }| size.z > WALL_THICK + 1e-4)
                .map(|Slab { at, .. }| at.y)
                .fold(f32::INFINITY, f32::min)
        };

        let usual = band_of(Opening::Window, (WALL_HIGH / ATOM).round() as i32);
        // A hole made from nothing but its kind stands where that kind stands - which is
        // the one thing a wall's courses still decide, and only at the moment of making.
        let born = Hole::plain(Opening::Window, 0.0);
        assert!(
            born.lift == usual.foot && born.high == usual.rise,
            "a window made from its kind alone did not start at its kind's own band"
        );

        // A metre lower, and half a metre higher than that. Whole atoms both ways.
        for step in [-16, 8] {
            let moved = Band {
                foot: usual.foot + step,
                ..usual
            };
            let want = sill_of(&at_band(usual)) + step as f32 * ATOM;
            let got = sill_of(&at_band(moved));
            assert!(
                (got - want).abs() < 1e-4,
                "a window lifted by {step} atoms put its sill at {got}, not {want}"
            );
        }
    }

    /// And it does not fight the wall at whatever height it has been put.
    ///
    /// The rail was a window's sill, so a window lifted off the rail brings a sill of its
    /// own into atoms the rail already had - which is the speckle Brett has now found twice.
    /// The rail gives way to a sill wherever the sill lands, not only where it used to.
    #[test]
    fn a_lifted_window_does_not_fight_the_wall() {
        let tall = (WALL_HIGH / ATOM).round() as i32;
        let usual = band_of(Opening::Window, tall);
        let wall = |framed: bool, foot: i32| {
            body_of(
                &PartKind::Wall {
                    long: 4.0,
                    high: WALL_HIGH,
                    framed,
                    openings: [
                        Some(Hole {
                            lift: foot,
                            ..Hole::plain(Opening::Window, 0.0)
                        }),
                        None,
                        None,
                        None,
                    ],
                },
                None,
            )
        };
        // Every height between the sill plate and the head plate, an atom at a time: this
        // is cheap and the faults are never where you expect them.
        // Every height a hand can put one at - which now stops a head's depth short of
        // the plate, so the window always has its own head over it.
        for foot in LOWEST_SILL..=tall - PLATE_TALL * 2 - usual.rise {
            // A PLAIN wall, strictly - nothing in one may share space with anything.
            no_two_solids_share_space(&wall(false, foot), false);

            // A FRAMED wall, at the sill, which is the piece that meets the rail. The rest
            // of a framed wall's carpentry has long-standing overlaps where braces run into
            // the corner posts; not chased here, and not pretended away either.
            // BOTH pieces that stand proud - the sill under it and the head over -
            // since either can land in the rail's own band, and the head is the one
            // that had nothing telling the rail to give way to it.
            let framed = wall(true, foot);
            let proud: Vec<&Slab> = framed
                .iter()
                .filter(|Slab { size, .. }| size.z > WALL_THICK + 1e-4)
                .collect();
            assert_eq!(
                proud.len(),
                2,
                "a framed window wears a proud sill and a proud head, not {}",
                proud.len()
            );
            for piece in &proud {
                for other in &framed {
                    // A BRACE is not its own box: it is drawn long and swung, so the box
                    // it was cut from sticks out past the bay at both ends. Asking where
                    // its box is answers a question nobody asked.
                    if std::ptr::eq(*piece, other)
                        || other.size.z > WALL_THICK + 1e-4
                        || other.cant != 0.0
                    {
                        continue;
                    }
                    let apart = |at_a: f32, s_a: f32, at_b: f32, s_b: f32| {
                        (at_a - at_b).abs() >= (s_a + s_b) * 0.5 - 1e-3
                    };
                    assert!(
                        apart(piece.at.x, piece.size.x, other.at.x, other.size.x)
                            || apart(piece.at.y, piece.size.y, other.at.y, other.size.y),
                        "a window with its foot at {foot} has a proud piece at {:?} in \
                         the same place as {:?} {:?}",
                        piece.at,
                        other.at,
                        other.size
                    );
                }
            }
        }
    }

    /// A wall closes up over and under a window wherever the window has been put.
    ///
    /// The framing was worked out by COURSE, and a window that filled its course left
    /// nothing to fill: the rail was its sill and the head plate carried its lintel. Set one
    /// low and the course it stood in had no idea, so the wall showed daylight under the
    /// sill and a post half the height of the wall over the head.
    #[test]
    fn a_lifted_window_leaves_no_hole() {
        let tall = (WALL_HIGH / ATOM).round() as i32;
        let usual = band_of(Opening::Window, tall);
        for foot in [LOWEST_SILL, usual.foot - 8, usual.foot - 3, usual.foot] {
            let band = Band { foot, ..usual };
            let body = body_of(
                &PartKind::Wall {
                    long: 4.0,
                    high: WALL_HIGH,
                    framed: true,
                    openings: [
                        Some(Hole {
                            lift: band.foot,
                            high: band.rise,
                            ..Hole::plain(Opening::Window, 0.0)
                        }),
                        None,
                        None,
                        None,
                    ],
                },
                None,
            );
            // Straight up the middle of the window's own column, atom by atom, between the
            // wall's two plates. Every atom is either the opening itself or something
            // solid; an atom that is neither is a hole a maker can see through.
            for row in PLATE_TALL..tall - PLATE_TALL {
                let y = (row as f32 + 0.5) * ATOM;
                if row >= band.foot && row < band.foot + band.rise {
                    continue;
                }
                // A brace does not close a wall - it is a diagonal, and its box is not
                // even where it is. Only the pieces that fill can be said to fill.
                let filled = body.iter().any(|Slab { at, size, cant, .. }| {
                    *cant == 0.0 && at.x.abs() < size.x * 0.5 && (at.y - y).abs() < size.y * 0.5
                });
                assert!(
                    filled,
                    "a window with its foot at {foot} leaves the wall open at {y} m"
                );
            }
        }
    }

    /// A window that has been moved says so, and a window that has not says nothing new.
    ///
    /// The second half is what keeps every wall in every drawing spelled the way it was
    /// spelled: a band is written only when it is not the band the wall would have given.
    #[test]
    fn a_moved_window_is_spelled_out() {
        let wall = |band: Band| PartKind::Wall {
            long: 4.0,
            high: WALL_HIGH,
            framed: true,
            openings: [
                Some(Hole {
                    lift: band.foot,
                    high: band.rise,
                    ..Hole::plain(Opening::Window, 0.5)
                }),
                None,
                None,
                None,
            ],
        };
        let usual = band_of(Opening::Window, (WALL_HIGH / ATOM).round() as i32);
        assert_eq!(
            part_name(&wall(usual)),
            "wall-4x2.5xfxw0.5",
            "a window at its course is spelled a new way, and every wall ever drawn \
             with one reads back as something else"
        );

        let moved = Band { foot: 7, rise: 12 };
        let said = part_name(&wall(moved));
        let read = kind_from_name(&said).expect("a wall the bench spelled, it can read");
        assert_eq!(
            part_name(&read),
            said,
            "a moved window does not survive its own name"
        );
        let PartKind::Wall { openings, .. } = read else {
            panic!("read back as something other than a wall")
        };
        let hole = openings[0].expect("the window went missing");
        assert!(
            hole.lift == moved.foot && hole.high == moved.rise && (hole.at - 0.5).abs() < 1e-4,
            "the window came back at a different place from the one it was written at"
        );
    }

    /// The ghost stands where the window will land - at every height, not just its own.
    ///
    /// Two halves again: the hand LIFTS the ghost and the punch TELLS the wall, off the same
    /// aim. Written out twice they agree until one of them is touched, and then a maker
    /// slides a window along a wall at one height and it lands at another.
    #[test]
    fn the_ghost_stands_where_the_window_lands() {
        let tall = (WALL_HIGH / ATOM).round() as i32;
        let usual = band_of(Opening::Window, tall);
        // The LOWEST of the pieces standing proud of both faces: a window has two of
        // them now, a sill under it and a head over, and "the first one in the list" is
        // not a question about geometry.
        let sill_of = |body: &[Slab]| {
            body.iter()
                .filter(|Slab { size, .. }| size.z > WALL_THICK + 1e-4)
                .map(|Slab { at, .. }| at.y)
                .fold(f32::INFINITY, f32::min)
        };
        let held = sill_of(&body_of(&PartKind::Prop("window"), None));
        for foot in [LOWEST_SILL, usual.foot - 5, usual.foot, usual.foot + 4] {
            let landed = sill_of(&body_of(
                &PartKind::Wall {
                    long: 4.0,
                    high: WALL_HIGH,
                    framed: true,
                    openings: [
                        Some(Hole {
                            lift: foot,
                            ..Hole::plain(Opening::Window, 0.0)
                        }),
                        None,
                        None,
                        None,
                    ],
                },
                None,
            ));
            let shown = held + crate::builder::ghost_lift(foot);
            assert!(
                (shown - landed).abs() < 1e-4,
                "the ghost shows a window with its sill at {shown} and it lands at {landed}"
            );
        }
    }

    /// Aiming where a window has always gone puts it exactly there.
    ///
    /// The strides up a wall are measured from the COURSE, not from the floor, and this is
    /// the test that says why. A quarter-metre stride off the floor cannot land on a course
    /// an eighth of a metre up: every window placed with the ordinary grid would have missed
    /// the rail by two atoms, carried a band of its own in its name, and broken the rail it
    /// was meant to sit on.
    #[test]
    fn the_ordinary_aim_lands_on_the_course() {
        let tall = (WALL_HIGH / ATOM).round() as i32;
        let usual = band_of(Opening::Window, tall);
        // The cursor on the middle of the glass, on a wall standing on the ground.
        let middle = (usual.foot as f32 + usual.rise as f32 * 0.5) * ATOM;
        for grid in [1, 2, 4, 8, 16] {
            let step = snap_step(false, grid);
            assert_eq!(
                crate::builder::opening_lift(0.0, tall, usual, middle, step),
                usual.foot,
                "aimed at the course on a grid of {grid}, the window landed off it"
            );
        }
        // And a stride away is a stride away, whichever stride is set - the grid counts
        // atoms, so a grid of four moves a window a quarter-metre at a time. Downward,
        // where an ordinary wall has room: there are three atoms over a window at its
        // course and a metre and a half under it.
        for grid in [2, 4, 8] {
            let step = snap_step(false, grid);
            let down = grid as f32 * ATOM;
            assert_eq!(
                crate::builder::opening_lift(0.0, tall, usual, middle - down, step),
                usual.foot - grid,
                "a stride down on a grid of {grid} did not move the window one stride"
            );
        }
        // Never through its own sill plate, and never up into the head plate, however
        // wildly it is aimed.
        let fine = snap_step(true, 16);
        assert_eq!(
            crate::builder::opening_lift(0.0, tall, usual, -5.0, fine),
            LOWEST_SILL,
            "a window aimed at the ground sank into the wall's sill plate"
        );
        assert_eq!(
            crate::builder::opening_lift(0.0, tall, usual, 5.0, fine),
            tall - PLATE_TALL * 2 - usual.rise,
            "a window aimed over the wall left no room for its own head"
        );
    }

    /// Every piece of the ghost stands clear of the wall it is being aimed at.
    ///
    /// A window's frame is drawn at the wall's own thickness and its bars are set back in
    /// the reveal - both right for a window standing in a hole, and both invisible in a wall
    /// that has not got the hole yet. Brett sent a picture of it: three faint sticks, which
    /// were the proud sill and the two jambs fighting the wall's own face for the same
    /// atoms. Everything else was inside the timber.
    #[test]
    fn the_ghost_stands_clear_of_the_wall() {
        // The wall's own face, from the middle of the wall the ghost is aimed at.
        let face = WALL_THICK * 0.5;
        for held in ["window", "door", "door-double", "doorway"] {
            for Slab { at, size, .. } in body_of(&PartKind::Prop(held), None) {
                let front = crate::builder::GHOST_PROUD + at.z + size.z * 0.5;
                assert!(
                    front > face + 1e-4,
                    "a {held}'s ghost has a piece buried in the wall: its face reaches \
                     {front} where the wall's own is at {face}"
                );
            }
        }
    }

    /// The check itself: no two full-thickness solids may occupy the same place.
    fn no_two_solids_share_space(body: &[Slab], framed: bool) {
        // Only the wall's own substance and frame - the panes are set back in the reveal
        // and are meant to sit inside the opening.
        let solid: Vec<&Slab> = body
            .iter()
            .filter(|Slab { size, .. }| size.z >= WALL_THICK - 1e-4)
            .collect();
        let which = if framed { "framed" } else { "plain" };
        let overlaps = |a: &Slab, b: &Slab| {
            let over = |at_a: f32, s_a: f32, at_b: f32, s_b: f32| {
                (at_a - at_b).abs() < (s_a + s_b) * 0.5 - 1e-3
            };
            over(a.at.x, a.size.x, b.at.x, b.size.x) && over(a.at.y, a.size.y, b.at.y, b.size.y)
        };
        for (i, one) in solid.iter().enumerate() {
            for other in solid.iter().skip(i + 1) {
                assert!(
                    !overlaps(one, other),
                    "two solids share space in a {which} wall: {:?} {:?} against {:?} {:?}",
                    one.at,
                    one.size,
                    other.at,
                    other.size
                );
            }
        }
    }

    /// The cursor on the glass finds the window; the cursor on the wall finds the wall.
    ///
    /// A window is a hole a wall was told about, so picking one up means asking the wall
    /// which opening was struck. Getting this loose in either direction is worse than not
    /// having it: too generous and a maker clicking plaster loses their window, too mean
    /// and the window cannot be picked up at all.
    #[test]
    fn a_wall_says_which_opening_was_clicked() {
        let wall = PartKind::Wall {
            long: 4.0,
            high: WALL_HIGH,
            framed: true,
            openings: [Some(Hole::plain(Opening::Window, 0.0)), None, None, None],
        };
        let at = Vec3::ZERO;
        // Where the glass is: the middle of the wall, up in the window's own band. Read
        // off the wall rather than guessed, so the test cannot drift from the geometry.
        let body = body_of(&wall, None);
        let bars: Vec<f32> = body
            .iter()
            .filter(|Slab { size, .. }| size.x < WALL_THICK && size.z < WALL_THICK)
            .map(|Slab { at, .. }| at.y)
            .collect();
        let middle_y = bars.iter().sum::<f32>() / bars.len().max(1) as f32;

        assert_eq!(
            crate::builder::opening_under(&wall, at, 0.0, Vec3::new(0.0, middle_y, 0.0)),
            Some(0),
            "the cursor on the glass did not find the window"
        );
        // Below it, on the plaster: that is the wall, and taking the window out from
        // there would be a maker losing a window they never aimed at.
        assert_eq!(
            crate::builder::opening_under(&wall, at, 0.0, Vec3::new(0.0, 0.2, 0.0)),
            None,
            "the cursor low on the wall took the window out"
        );
        // And along the wall, well past the opening.
        assert_eq!(
            crate::builder::opening_under(&wall, at, 0.0, Vec3::new(1.8, middle_y, 0.0)),
            None,
            "the cursor at the wall's end took the window out"
        );
    }

    /// The window in hand is the window that lands.
    ///
    /// The ghost was a frame of its own invention - a quarter wider than the opening it
    /// punches, at a height of its own, always four panes however the wall would have
    /// glazed it. A maker slid one thing along a wall and got another.
    #[test]
    fn the_ghost_is_what_it_becomes() {
        let held = body_of(&PartKind::Prop("window"), None);
        let placed = a_wall_with_a_window(true);

        // A BAR is set back in the reveal and narrow one way: a mullion is thin across, a
        // transom thin up. A panel is set back too and narrow neither way, which is what
        // told them apart.
        let is_bar = |size: &Vec3| size.z < WALL_THICK - 1e-4 && (size.x < 0.2 || size.y < 0.2);
        let panes = |body: &[Slab]| body.iter().filter(|Slab { size, .. }| is_bar(size)).count();
        assert_eq!(
            panes(&held),
            panes(&placed),
            "the ghost is glazed differently from the window it becomes"
        );

        // The same clear width, read off the bars that divide it: the ghost was a quarter
        // metre wider than the hole it punches.
        let widest = |body: &[Slab]| {
            body.iter()
                .filter(|Slab { size, .. }| is_bar(size))
                .map(|Slab { size, .. }| size.x)
                .fold(0.0f32, f32::max)
        };
        assert!(
            (widest(&held) - widest(&placed)).abs() < 1e-4,
            "the ghost spans {} where the window spans {}",
            widest(&held),
            widest(&placed)
        );
    }

    /// No timber crosses a doorway.
    ///
    /// A door reaches the ground, so every course the wall lays has to break for it - and
    /// when the rail learned to give way to a window's SILL it stopped giving way to
    /// anything else, and carried straight across the doorway. A double door showed it
    /// first, being wide enough that the rail had somewhere to be seen.
    #[test]
    fn nothing_is_laid_across_a_doorway() {
        for wide in [DOOR_WIDE, DOOR_WIDE * 2] {
            let wall = PartKind::Wall {
                long: 5.0,
                high: WALL_HIGH,
                framed: true,
                openings: [
                    Some(Hole {
                        wide,
                        ..Hole::plain(Opening::Door, 0.0)
                    }),
                    None,
                    None,
                    None,
                ],
            };
            // The doorway's own clear span, in metres either side of the middle.
            let half = wide as f32 * ATOM * 0.5;
            for Slab { at, size, .. } in body_of(&wall, None) {
                // A leaf and its furniture belong in the opening; the WALL's timbers do
                // not. Everything the wall lays is full thickness.
                if size.z < WALL_THICK - 1e-4 {
                    continue;
                }
                let (low, high) = (at.x - size.x * 0.5, at.x + size.x * 0.5);
                // A whole atom in, not a whisker: a brace is a rotated slab whose upright
                // bounding box clips the jamb by a hundredth of a metre while its timber
                // does not. A rail carried across a doorway intrudes by the whole width.
                let bite = ATOM;
                let inside = low < half - bite && high > -half + bite;
                let above = at.y - size.y * 0.5 >= DOOR_HIGH as f32 * ATOM - 1e-3;
                assert!(
                    !inside || above,
                    "a timber lies across a {}-atom doorway: at {:?} size {:?}",
                    wide,
                    at,
                    size
                );
            }
        }
    }

    /// And a wall with no opening is still one plain box.
    ///
    /// The cheapest thing a wall can be, and the commonest - drawing it in pieces because
    /// the code now knows how would be paying for openings nobody asked for.
    #[test]
    fn an_unpunched_plain_wall_is_one_slab() {
        let body = body_of(&PartKind::wall(4.0), None);
        assert_eq!(body.len(), 1, "a plain wall came apart for no reason");
    }
}

#[cfg(test)]
mod materials {
    use super::*;

    /// What a part is BUILT of is not what it is painted.
    ///
    /// Brett: "The color shouldn't have anything to do with that. That's just what you
    /// painted in the palette." A wall painted pale is a wall painted pale; a wall MADE of
    /// stone is one the village quarries for. The bake already carried `cloth`, which is
    /// the ramp - so a game reading that as the material would charge for whitewash.
    #[test]
    fn a_material_is_not_a_colour() {
        let mut record = Placed {
            part: part_name(&PartKind::wall(3.0)),
            at: [0.0, 0.0, 0.0],
            yaw: 0.0,
            tilt: 0.0,
            ramp: Some("bone".to_string()),
            shade: 0.8,
            stage: "walls".to_string(),
            flip: false,
            group: None,
            loose: false,
            material: String::new(),
        };
        // Painted pale and built of stone: two facts, and neither is the other.
        record.material = "stone".to_string();
        assert_eq!(record.ramp.as_deref(), Some("bone"), "the paint moved");
        assert_eq!(record.material, "stone");

        // A work carries both through a save and back.
        let work = Workbench {
            levels: vec![Level {
                name: String::new(),
                phases: vec![vec![record.clone()]],
            }],
            ..default()
        };
        let said = serde_json::to_string(&work).expect("written");
        let back: Workbench = serde_json::from_str(&said).expect("read back");
        let part = &back.levels[0].phases[0][0];
        assert_eq!(part.material, "stone", "the material was lost in the file");
        assert_eq!(part.ramp.as_deref(), Some("bone"));
    }

    /// UNSAID is not "wood".
    ///
    /// A part nobody has spoken for writes no material at all, so a game may charge what it
    /// likes for it - which is the game's decision rather than a default the bench smuggles
    /// in behind its back.
    #[test]
    fn unsaid_stays_unsaid() {
        let plain = Placed {
            part: part_name(&PartKind::wall(3.0)),
            at: [0.0, 0.0, 0.0],
            yaw: 0.0,
            tilt: 0.0,
            ramp: None,
            shade: 0.5,
            stage: "walls".to_string(),
            flip: false,
            group: None,
            loose: false,
            material: String::new(),
        };
        let said = serde_json::to_string(&plain).expect("written");
        assert!(
            !said.contains("wood"),
            "a part nobody spoke for was given a material: {said}"
        );
    }

    /// The bench knows three words, and a project may know its own.
    #[test]
    fn the_bench_has_a_starting_point() {
        assert!(crate::project::BENCH_MATERIALS.contains(&"wood"));
        assert!(crate::project::BENCH_MATERIALS.contains(&"stone"));
        assert!(crate::project::BENCH_MATERIALS.contains(&"clay"));
        // And they are the fallback, so a project that has said nothing still has words.
        assert!(!crate::project::materials().is_empty());
    }
}

#[cfg(test)]
/// The bell is one continuous stack, mouth to headstock.
///
/// Brett drew one in the mockup and asked for it once the tower stood: "Can you make the
/// bell?" It is three pieces and a yoke, and the whole of whether it reads as a bell is
/// whether they MEET - a waist a hair narrower than the mouth it sits on is a step, and a
/// step is what a maker sees from across the room.
#[test]
fn the_bell_is_one_continuous_stack() {
    let mut cast = body_of(&PartKind::Prop("bell"), None);
    assert!(cast.len() >= 3, "a bell of {} pieces", cast.len());
    cast.sort_by(|a, b| a.at.y.partial_cmp(&b.at.y).unwrap());
    for pair in cast.windows(2) {
        let (under, over) = (&pair[0], &pair[1]);
        let top = under.at.y + under.size.y * 0.5;
        let foot = over.at.y - over.size.y * 0.5;
        assert!(
            (top - foot).abs() < 1e-4,
            "the bell parts company at {top}: the next piece starts at {foot}"
        );
        // Where a piece tapers, what stands on it is exactly what it tapered TO. Wider is
        // a lip, narrower is a step, and a bell is neither.
        if let Shape::Hip(keep, _) = under.shape {
            let shrunk = under.size.x * keep;
            assert!(
                (shrunk - over.size.x).abs() < 1e-3 || over.size.x > under.size.x,
                "the bell tapers to {shrunk} and then carries {} on it",
                over.size.x
            );
        }
    }
    // And it rests on its own nought like everything else, so it sets down on a beam
    // rather than half through one.
    let foot = cast
        .iter()
        .map(|Slab { at, size, .. }| at.y - size.y * 0.5)
        .fold(f32::INFINITY, f32::min);
    assert!(foot.abs() < 1e-4, "the bell's mouth hangs at {foot}");
}

mod snapping {
    use super::*;

    /// A face lying up is answered with its own height; a face standing up seats itself.
    ///
    /// Brett: "When snapping a object like a wall or gable to a face, it only snaps them to
    /// side faces not top faces." It was true and it was one line: a side face clung flush
    /// and returned, where a top face only SEEDED the placement and left the height to the
    /// vote under the footprint - which takes the lower of two equal answers on purpose, so
    /// that a part half over a wall settles beside it rather than climbing it. Right for a
    /// part being nudged about, wrong for one a maker has pointed at a wall head.
    #[test]
    fn a_face_lying_up_says_where_a_part_lands() {
        let head = Vec3::new(1.0, 2.5, -3.0);
        // A WALL HEAD, a floor, a foundation top: the part lands on the face itself.
        assert_eq!(
            crate::builder::face_seat(Vec3::Y, head),
            Some(2.5),
            "a part aimed at a wall head does not land on it"
        );
        // A roof at the bench's own pitch is a face to stand on too.
        let slope = Quat::from_rotation_x(ROOF_PITCH_DEGREES.to_radians()) * Vec3::Y;
        assert_eq!(
            crate::builder::face_seat(slope, head),
            Some(2.5),
            "a roof at {ROOF_PITCH_DEGREES} degrees is not a face to set anything on"
        );
        // A WALL'S SIDE answers nothing here: it clings flush, which is its own arithmetic.
        for side in [Vec3::X, Vec3::Z, -Vec3::X] {
            assert!(
                crate::builder::face_seat(side, head).is_none(),
                "a face standing up was answered as one lying up"
            );
            assert!(
                crate::builder::face_stands_up(side),
                "a wall's own side is not being taken as a face to hang against"
            );
        }
        // And the two are never both true, which is what would make a snap depend on
        // which branch was written first.
        for tilt in [0.0f32, 15.0, 30.0, 45.0, 60.0, 75.0, 90.0] {
            let normal = Quat::from_rotation_x(tilt.to_radians()) * Vec3::Y;
            assert!(
                !(crate::builder::face_lies_up(normal) && crate::builder::face_stands_up(normal)),
                "a face at {tilt} degrees is both a top and a side"
            );
        }
        // The gap is deliberate: a face leaning further than a roof is neither, and gets
        // no face snap at all. Asserted so that closing it is a decision somebody makes.
        let steep = Quat::from_rotation_x(60f32.to_radians()) * Vec3::Y;
        assert!(
            !crate::builder::face_lies_up(steep) && !crate::builder::face_stands_up(steep),
            "the gap between the two has been closed - which may be right, but not by \
             accident"
        );
    }
}

mod gables {
    use super::*;

    /// A framed gable is the same triangle, with the wall's own timbers on it.
    ///
    /// Every piece has to stay inside the outline: a rake board laid along the slope hangs
    /// its inner corner below the foot unless it is stepped up the slope, and that would be
    /// a tab of timber lapping over the wall underneath.
    #[test]
    fn a_framed_gable_keeps_inside_its_own_triangle() {
        for long in [2.0f32, 4.0, 7.0] {
            for degrees in [PITCH_LEAST, ROOF_PITCH_DEGREES, PITCH_MOST] {
                let framed = PartKind::Gable {
                    long,
                    pitch: degrees,
                    framed: true,
                };
                let plain = PartKind::gable(long, degrees);
                // The outline, taken off the plain one: the framed gable may not exceed it.
                let high = body_of(&plain, None)
                    .iter()
                    .map(|Slab { at, size, .. }| at.y + size.y * 0.5)
                    .fold(0.0f32, f32::max);
                // The slope the triangle ACTUALLY has, off its own measured peak: a
                // gable's height is snapped to the lattice, so a two-metre gable asked
                // for thirty degrees is drawn at twenty-nine and a third, and a test
                // that restated the arithmetic would be checking a different triangle.
                let rise = high / (long * 0.5);
                let body = body_of(&framed, None);
                // A gable with room in it is framed; a sliver at the shallowest pitch is
                // barely taller than its own foot plate, and there is nothing to frame.
                if high > WALL_HIGH * 0.25 {
                    assert!(
                        body.len() > 3,
                        "a {long}m gable at {degrees} degrees is bare: {} pieces",
                        body.len()
                    );
                }
                for Slab {
                    at,
                    size,
                    cant,
                    cut,
                    shape,
                    ..
                } in &body
                {
                    // The TIMBERS. The plaster behind them is the triangle itself - a
                    // wedge, whose box corners are exactly the two the mesh leaves out.
                    if !matches!(shape, Shape::Box) {
                        continue;
                    }
                    // Every corner, turned the way the piece is turned - and pulled in
                    // where the saw has been: a mitred end's corner is not its box's.
                    let turn = Mat2::from_angle(*cant);
                    for sx in [-0.5f32, 0.5] {
                        for sy in [-0.5f32, 0.5] {
                            let run = if sx < 0.0 { cut.x } else { cut.y };
                            // A positive run cuts the top face back, a negative one the
                            // foot - see `cut_mesh`.
                            let sawn = if (sy > 0.0 && run > 0.0) || (sy < 0.0 && run < 0.0) {
                                run.abs()
                            } else {
                                0.0
                            };
                            let corner = Vec2::new(at.x, at.y)
                                + turn * Vec2::new(size.x * sx - sx.signum() * sawn, size.y * sy);
                            assert!(
                                corner.y > -1e-3,
                                "a piece of a {long}m gable at {degrees} degrees hangs \
                                 {} below its foot",
                                -corner.y
                            );
                            // Inside the slope: the triangle's own edge at this x.
                            let roof = (long * 0.5 - corner.x.abs()) * rise;
                            assert!(
                                corner.y <= roof.max(0.0) + 1e-3,
                                "a piece of a {long}m gable at {degrees} degrees stands \
                                 {} above the slope at x={}",
                                corner.y - roof,
                                corner.x
                            );
                        }
                    }
                }
            }
        }
    }

    /// Every stud's head meets the rake it stands under, both shoulders.
    ///
    /// Brett, with a picture of the gap: "the vert pieces don't go all the way up, we can
    /// make angles now so we should be able to seam it perfectly." They stopped square at
    /// whatever the rake's underside measured over their middle, which leaves a wedge of
    /// plaster above every stud in the gable - and the fix is the same saw the rakes are
    /// mitred with.
    ///
    /// Measured off the RAKE, not off the arithmetic that made it: the underside is read
    /// from the board that is actually there, so the two cannot drift apart.
    #[test]
    fn a_studs_head_meets_the_rake() {
        for long in [6.0f32, 9.0, 12.0] {
            for degrees in [25.0f32, ROOF_PITCH_DEGREES, 50.0] {
                let body = body_of(
                    &PartKind::Gable {
                        long,
                        pitch: degrees,
                        framed: true,
                    },
                    None,
                );
                let upright = std::f32::consts::FRAC_PI_2;
                // The rakes: laid at the slope's own angle, which is neither flat nor
                // upright. One each side.
                let rakes: Vec<&Slab> = body
                    .iter()
                    .filter(|slab| {
                        slab.cant.abs() > 1e-3 && (slab.cant.abs() - upright).abs() > 1e-3
                    })
                    .collect();
                assert_eq!(rakes.len(), 2, "a framed gable has two rakes");
                // Where a rake's underside stands at a given x, off the board itself.
                let underside = |x: f32| {
                    let rake = rakes
                        .iter()
                        .find(|slab| (slab.at.x < 0.0) == (x < 0.0))
                        .expect("a rake on this side");
                    let turn = Mat2::from_angle(rake.cant);
                    let inner =
                        Vec2::new(rake.at.x, rake.at.y) + turn * Vec2::new(0.0, -rake.size.y * 0.5);
                    let along = turn * Vec2::new(1.0, 0.0);
                    inner.y + (x - inner.x) * along.y / along.x
                };
                // The studs: upright pieces, and the king post, which is square.
                let studs = body.iter().filter(|slab| {
                    slab.ramp == "wood"
                        && ((slab.cant - upright).abs() < 1e-3
                            || (slab.cant.abs() < 1e-3 && slab.size.y > slab.size.x))
                });
                let (mut seen, mut mitred) = (0, 0);
                for stud in studs {
                    seen += 1;
                    if stud.cut.y.abs() > 1e-4 {
                        mitred += 1;
                    }
                    // ITS HEAD: the two highest corners of the piece, whichever way it
                    // was laid and whatever the saw took. Asking for "the top two" rather
                    // than for a named pair is what lets the square king post and the
                    // mitred studs be checked by one question.
                    let turn = Mat2::from_angle(stud.cant);
                    let half = Vec2::new(stud.size.x, stud.size.y) * 0.5;
                    let mut corners: Vec<Vec2> = Vec::new();
                    for sx in [-1.0f32, 1.0] {
                        for sy in [-1.0f32, 1.0] {
                            let run = if sx < 0.0 { stud.cut.x } else { stud.cut.y };
                            let sawn = if (sy > 0.0 && run > 0.0) || (sy < 0.0 && run < 0.0) {
                                run.abs()
                            } else {
                                0.0
                            };
                            corners.push(
                                Vec2::new(stud.at.x, stud.at.y)
                                    + turn * Vec2::new(sx * half.x - sx * sawn, sy * half.y),
                            );
                        }
                    }
                    corners.sort_by(|a, b| b.y.partial_cmp(&a.y).unwrap());
                    for corner in &corners[..2] {
                        let gap = underside(corner.x) - corner.y;
                        assert!(
                            gap.abs() < 1e-3,
                            "a stud at x={} in a {long}m gable at {degrees} degrees leaves \
                             {gap} between its head and the rake",
                            stud.at.x
                        );
                    }
                }
                assert!(seen >= 3, "only {seen} studs to check in a {long}m gable");
                assert!(
                    mitred >= 2,
                    "only {mitred} of the {seen} studs in a {long}m gable are cut at all - \
                     the seam is being checked on square heads"
                );
            }
        }
    }

    /// The framing is the wall's framing: plaster set back, timber proud of it.
    ///
    /// Which is what makes a framed gable read as the same building as the framed wall it
    /// stands on - the shadow line down a stud is the same shadow line, because it is the
    /// same two thicknesses.
    #[test]
    fn a_framed_gable_is_framed_like_a_wall() {
        let framed = body_of(
            &PartKind::Gable {
                long: 6.0,
                pitch: ROOF_PITCH_DEGREES,
                framed: true,
            },
            None,
        );
        let plaster: Vec<&Slab> = framed.iter().filter(|slab| slab.ramp == "bone").collect();
        assert_eq!(
            plaster.len(),
            1,
            "a framed gable's infill is one wedge behind everything"
        );
        assert!(
            (plaster[0].size.z - (WALL_THICK - (INFILL_SET * 2) as f32 * ATOM)).abs() < 1e-4,
            "the plaster is not set back, so no timber stands proud of it"
        );
        for timber in framed.iter().filter(|slab| slab.ramp == "wood") {
            assert!(
                (timber.size.z - WALL_THICK).abs() < 1e-4,
                "a timber is not the wall's own thickness"
            );
        }
        // A KING POST under the peak, which is what a gable is framed around: an even
        // number of bays, so there is a division at the middle.
        let middle = framed
            .iter()
            .filter(|slab| slab.ramp == "wood" && slab.cant == 0.0)
            .any(|Slab { at, size, .. }| at.x.abs() < 1e-3 && size.y > size.x);
        assert!(middle, "a framed gable has no post under its peak");
    }

    /// And the menu offers the framing on a gable, both ways.
    #[test]
    fn a_gable_is_offered_the_other_state() {
        for framed in [false, true] {
            let gable = PartKind::Gable {
                long: 4.0,
                pitch: ROOF_PITCH_DEGREES,
                framed,
            };
            assert!(
                deeds_for(&gable).contains(&Deed::Frame(!framed)),
                "a {} gable is not offered the other state",
                if framed { "framed" } else { "plain" }
            );
        }
    }
}

mod roofing {
    use super::*;

    /// A ceiling remembers which roof it raises, through its name and back.
    ///
    /// The whole reason a ceiling is its own part rather than a floor: the choice is made
    /// while it is still a rectangle, and it has to survive being saved and reopened - a
    /// forgotten choice would raise a gable over a hall meant to be hipped, and look
    /// deliberate.
    #[test]
    fn a_ceiling_remembers_its_roof() {
        for hipped in [false, true] {
            let ceiling = PartKind::Ceiling {
                long: 6.0,
                deep: 4.0,
                hipped,
                across: false,
            };
            let name = part_name(&ceiling);
            let Some(PartKind::Ceiling {
                long,
                deep,
                hipped: back,
                across: false,
            }) = kind_from_name(&name)
            else {
                panic!("a ceiling did not read back: {name}");
            };
            assert_eq!(back, hipped, "it forgot its roof: {name}");
            assert!(
                (long - 6.0).abs() < 1e-6 && (deep - 4.0).abs() < 1e-6,
                "{name}"
            );
        }
        // A gable ceiling writes no extra word, so the plain case stays the plain name.
        assert_eq!(
            part_name(&PartKind::Ceiling {
                long: 4.0,
                deep: 4.0,
                hipped: false,
                across: false
            }),
            "ceiling-4x4"
        );
    }

    /// A ceiling counts as structure, so it climbs onto what is under it.
    ///
    /// Brett: "It is very hard to get the ceiling to sit ontop of the wall." It was: a
    /// part that is not structure does not look for what holds it up, so a ceiling had to
    /// be flown to the top of a wall by eye and by hand. Everything else that spans a room
    /// - a floor, a footing, a roof - was already in this list, and the ceiling was added
    /// to the shelf without being added here.
    #[test]
    fn a_ceiling_rests_on_what_is_under_it() {
        let ceiling = PartKind::Ceiling {
            long: 4.0,
            deep: 4.0,
            hipped: false,
            across: false,
        };
        assert!(
            crate::builder::is_structure(&ceiling),
            "a ceiling that is not structure never finds the wall tops"
        );
        // And a floor is too, which is the behaviour being matched.
        assert!(crate::builder::is_structure(&PartKind::Floor(2.0, 2.0)));
    }

    /// The ridge drawn on a ceiling says which way, and which kind.
    ///
    /// It is the only thing telling a maker what GENERATE ROOF will do before they press
    /// it, so a beam pointing the wrong way would be worse than none - they would trust it.
    #[test]
    fn a_ceilings_ridge_shows_what_it_will_raise() {
        // The beam is whatever the ceiling draws that is not the slab itself.
        let ridge_of = |long: f32, deep: f32, hipped: bool| {
            let body = body_of(
                &PartKind::Ceiling {
                    long,
                    deep,
                    hipped,
                    across: false,
                },
                None,
            );
            let beam = body
                .iter()
                .find(|Slab { ramp, .. }| ramp == "wood")
                .expect("a ceiling draws its ridge");
            (beam.size.x, beam.size.z)
        };

        // A long ceiling: the ridge runs its length, along X.
        let (x, z) = ridge_of(6.0, 4.0, false);
        assert!(
            (x - 6.0).abs() < 1e-4,
            "a gable ridge is not the full length"
        );
        assert!(z < x, "the ridge is drawn across the building");

        // A DEEP one: the same beam, swung a quarter, exactly as the roof will be.
        let (x, z) = ridge_of(4.0, 6.0, false);
        assert!(
            (z - 6.0).abs() < 1e-4,
            "the ridge did not swing with the shape"
        );
        assert!(x < z);

        // HIPPED: shorter than the building, because the slopes come in at both ends.
        let (gable, _) = ridge_of(6.0, 4.0, false);
        let (hip, _) = ridge_of(6.0, 4.0, true);
        assert!(
            hip < gable,
            "a hipped ridge {hip} is not shorter than a gable's {gable}, so the two \
             ceilings look the same"
        );
        assert!(hip > 0.0, "a hipped ridge vanished");
    }

    /// R flips the ridge, and what is raised follows it.
    ///
    /// The beam and the roof read the same field, so they cannot disagree - but a maker
    /// pressing R and getting a beam that swings while the roof does not would trust the
    /// beam, which is the one thing it must never be wrong about.
    #[test]
    fn r_swings_the_ridge_and_the_roof_follows() {
        let beam_along_x = |long: f32, deep: f32, across: bool| {
            let body = body_of(
                &PartKind::Ceiling {
                    long,
                    deep,
                    hipped: false,
                    across,
                },
                None,
            );
            let beam = body
                .iter()
                .find(|Slab { ramp, .. }| ramp == "wood")
                .expect("its ridge");
            beam.size.x > beam.size.z
        };
        // A long ceiling lays its ridge along itself; flipped, it lays it across.
        assert!(
            beam_along_x(6.0, 4.0, false),
            "the ridge did not run the long way"
        );
        assert!(!beam_along_x(6.0, 4.0, true), "R did not swing the ridge");
        // And a deep one starts across and flips to along.
        assert!(!beam_along_x(4.0, 6.0, false));
        assert!(beam_along_x(4.0, 6.0, true));

        // What GENERATE raises, worked out the way the menu works it out.
        let raised_long = |w: f32, d: f32, across: bool| {
            if (w >= d) != across { (w, d) } else { (d, w) }
        };
        // Flipped, a 6x4 ceiling raises a roof whose ridge is the SHORT side - the cross
        // wing whose gable faces the street, which is the whole reason for the flip.
        assert_eq!(raised_long(6.0, 4.0, false), (6.0, 4.0));
        assert_eq!(raised_long(6.0, 4.0, true), (4.0, 6.0));
    }

    /// The ceiling stops exactly where its gables begin.
    ///
    /// Both sit on the wall plate, so a ceiling reaching the full length would share its
    /// last quarter-metre with the gable that lands there - two surfaces at one depth,
    /// which a renderer flickers over. The two measurements have to agree exactly, and
    /// this is what says so: an overlap flickers, a gap shows daylight.
    #[test]
    fn a_ceiling_stops_where_its_gable_starts() {
        let (long, deep) = (6.0, 4.0);
        let ceiling = body_of(
            &PartKind::Ceiling {
                long,
                deep,
                hipped: false,
                across: false,
            },
            None,
        );
        // The slab, which is the bone one - the wood is its ridge beam.
        let slab = ceiling
            .iter()
            .find(|Slab { ramp, .. }| ramp == "bone")
            .expect("its slab");
        let ceiling_end = slab.at.x + slab.size.x * 0.5;

        // Where the roof's gable will stand, read off the roof itself.
        let roof = body_of(
            &PartKind::GableRoof(long, deep, ROOF_OVERHANG, ROOF_PITCH_DEGREES),
            None,
        );
        let gable_inner = roof
            .iter()
            .filter(|Slab { size, .. }| size.x <= GABLE_THICK + 1e-4)
            .map(|Slab { at, size, .. }| at.x - size.x * 0.5)
            .fold(f32::NEG_INFINITY, f32::max);

        assert!(
            (ceiling_end - gable_inner).abs() < 1e-4,
            "the ceiling ends at {ceiling_end} and its gable begins at {gable_inner}: \
             an overlap flickers, a gap shows daylight"
        );

        // A HIPPED ceiling keeps its full reach: there are no gables to give way to, and
        // the slopes come down on all four sides onto a ceiling that should meet the walls.
        let hipped = body_of(
            &PartKind::Ceiling {
                long,
                deep,
                hipped: true,
                across: false,
            },
            None,
        );
        let full = hipped
            .iter()
            .find(|Slab { ramp, .. }| ramp == "bone")
            .expect("its slab");
        assert!(
            (full.size.x - long).abs() < 1e-4,
            "a hipped ceiling gave way to gables it will never have"
        );
    }

    /// A roof generated over a ceiling covers it, and rests ON it.
    ///
    /// The arithmetic the menu does, checked here rather than by eye: a roof whose eaves
    /// sank into the ceiling or floated over it would look nearly right from the working
    /// perch and be wrong in the game.
    #[test]
    fn a_generated_roof_fits_its_ceiling() {
        // What the menu works out, in the same order it works it out.
        let roof_for = |w: f32, d: f32| {
            let (long, span, turn) = if w >= d {
                (w, d, 0.0)
            } else {
                (d, w, std::f32::consts::FRAC_PI_2)
            };
            (
                PartKind::GableRoof(long, span, ROOF_OVERHANG, ROOF_PITCH_DEGREES),
                turn,
            )
        };

        // A long ceiling: the ridge runs its length, and the roof is not turned.
        let (made, turn) = roof_for(6.0, 4.0);
        let PartKind::GableRoof(long, span, ..) = made else {
            panic!("not a gable roof")
        };
        assert!(
            (long - 6.0).abs() < 1e-6,
            "the ridge does not run the long way"
        );
        assert!((span - 4.0).abs() < 1e-6);
        assert!(turn.abs() < 1e-6, "a long ceiling turned its roof");

        // A DEEP one: the same roof, turned a quarter, so the ridge still runs the long
        // way rather than across the building.
        let (made, turn) = roof_for(4.0, 6.0);
        let PartKind::GableRoof(long, span, ..) = made else {
            panic!("not a gable roof")
        };
        assert!(
            (long - 6.0).abs() < 1e-6,
            "the ridge ran across the building"
        );
        assert!((span - 4.0).abs() < 1e-6);
        assert!(
            (turn - std::f32::consts::FRAC_PI_2).abs() < 1e-6,
            "a deep ceiling did not turn its roof"
        );

        // AND IT SEATS ON THE CEILING'S TOP. A gable roof's eaves rest at its own nought,
        // and a floor spans nought to its thickness - so the lift is exactly that.
        let ceiling = body_of(&PartKind::Floor(6.0, 4.0), None);
        let top = ceiling
            .iter()
            .map(|Slab { at, size, .. }| at.y + size.y * 0.5)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            (top - FLOOR_THICK).abs() < 1e-6,
            "a ceiling's top is at {top}, so a roof lifted by {FLOOR_THICK} would not meet it"
        );

        // The roof itself starts at its own nought, which is what makes that lift right.
        let lowest = body_of(&made, None)
            .iter()
            .map(|Slab { at, size, .. }| at.y - size.y * 0.5)
            .fold(f32::INFINITY, f32::min);
        assert!(
            lowest > -0.2,
            "a roof's lowest timber is at {lowest}, well under its own nought - it would \
             sink into the ceiling"
        );
    }
}
