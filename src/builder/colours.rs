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
