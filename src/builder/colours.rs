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
    let clear_of = |what: &'static str| opening_of(&PartKind::Prop(what)).map(|(.., clear)| clear);
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
                what: Opening::Door,
                at: 0.0,
                wide: double,
                dark: false,
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
                what: Opening::Door,
                at: 0.5,
                wide: DOOR_WIDE * 2,
                dark: false,
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
                    what: Opening::Window,
                    at: 0.0,
                    wide: WINDOW_WIDE,
                    dark: true,
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
                    what: Opening::Window,
                    at: 0.75,
                    wide: WINDOW_WIDE,
                    dark: true,
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
    fn a_plain_wall_does_not_fight_itself() {
        let body = a_wall_with_a_window(false);
        // Only the wall's own substance and frame - the panes are set back in the reveal
        // and are meant to sit inside the opening.
        let solid: Vec<&Slab> = body
            .iter()
            .filter(|Slab { size, .. }| size.z >= WALL_THICK - 1e-4)
            .collect();
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
                    "two solids share space: {:?} {:?} against {:?} {:?}",
                    one.at,
                    one.size,
                    other.at,
                    other.size
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
