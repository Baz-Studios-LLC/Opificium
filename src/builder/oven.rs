//! The bake: a work resolved into boxes a game can draw.

use super::*;

/// Bakes one work into what the game can eat: plain boxes with resolved
/// colours, and the marks that say what the place is FOR.
///
/// This used to live inside a `#[test]`, which meant a maker could only carry a
/// building into the game from a source tree with cargo on it. The bench in the
/// launcher had nowhere to send its work at all - Brett: "At what point does it
/// install its own files?" - so the bake is ordinary code now, and the test and
/// the BAKE button both go through it.
/// The bounds of one drawing, ignoring the scale reference.
pub(crate) fn bounds_of(parts: &[Placed]) -> (Vec3, Vec3) {
    // The bounds of everything that is not a scale reference, so
    // the building can be recentred on its own footprint.
    let mut low = Vec3::splat(f32::INFINITY);
    let mut high = Vec3::splat(f32::NEG_INFINITY);
    for record in parts {
        if record.part == "prop:mannequin" {
            continue;
        }
        let Some(kind) = kind_from_name(&record.part) else {
            continue;
        };
        let turn = pose(record.yaw, record.tilt, record.flip);
        for Slab { mut at, size, .. } in body_of(&kind, None) {
            if record.flip {
                at.x = -at.x;
            }
            let centre = Vec3::from(record.at) + turn * at;
            let reach = (turn * (size * 0.5)).abs();
            low = low.min(centre - reach);
            high = high.max(centre + reach);
        }
    }
    (low, high)
}

/// One phase, resolved: the boxes a game can draw, and the marks that say what the
/// place is FOR, in a frame whose origin is `middle`.
///
/// Lifted out of `bake_a_work` when a work grew levels. It used to bake exactly one
/// drawing - the last - and every phase a maker had authored was discarded on the
/// way out; now the same arithmetic runs once per phase, and the shared `middle` is
/// what keeps an upgrade standing where the building it upgrades stands.
pub(crate) fn bake_one_phase(
    parts: &[Placed],
    palette: &Palette,
    middle: Vec3,
) -> (Vec<String>, Vec<String>) {
    let mut boxes: Vec<String> = Vec::new();
    // What, where, which way - and whether a hand put it there.
    let mut marks: Vec<(String, Vec3, f32, bool)> = Vec::new();
    let say = |v: Vec3| format!("[{:.4}, {:.4}, {:.4}]", v.x, v.y, v.z);

    for record in parts {
        if record.part == "prop:mannequin" {
            continue;
        }
        let Some(kind) = kind_from_name(&record.part) else {
            continue;
        };
        let turn = pose(record.yaw, record.tilt, record.flip);
        let anchor = Vec3::from(record.at) - middle;

        // What the place is for, read from the widgets that say
        // so and from the furniture that means it.
        let mark = |what: &str, at: Vec3, yaw: f32| (what.to_string(), at, yaw, false);
        match kind {
            PartKind::Widget(what) => {
                marks.push((what.to_string(), anchor, record.yaw, true));
                continue;
            }
            // Beds and seats say nothing on their own: their
            // figures are set down WITH them and can be taken
            // away, so a chair with no sitter on it is a chair
            // nobody sits in. Only furniture with no figure to
            // show still speaks for itself.
            PartKind::Prop("cradle") => marks.push(mark("sleep", anchor, record.yaw)),
            PartKind::Prop("hearth") => {
                marks.push(mark("fire", anchor, record.yaw));
                marks.push(mark("smoke", anchor, record.yaw));
            }
            PartKind::Prop("table") => marks.push(mark("table", anchor, record.yaw)),
            PartKind::Prop("chest" | "cupboard" | "wardrobe" | "shelves") => {
                marks.push(mark("store", anchor, record.yaw))
            }
            PartKind::Prop("anvil" | "loom") => marks.push(mark("work", anchor, record.yaw)),
            PartKind::Prop("candle") => marks.push(mark("light", anchor, record.yaw)),
            _ => {}
        }

        // The body itself, as boxes the game can simply draw.
        let repaint = record.ramp.as_deref().map(|r| (r, record.shade));
        for Slab {
            mut at,
            size,
            ramp,
            shade,
            clarity,
            shape,
            mut lean,
            mut cant,
            cut,
        } in body_of(&kind, repaint)
        {
            if record.flip {
                at.x = -at.x;
                lean = -lean;
                // A mirrored wall's braces lean the other way, or a mirrored
                // pair comes out leaning the same way as the original and the
                // whole point of mirroring it is lost.
                cant = -cant;
            }
            let centre = anchor + turn * at;
            // A piece that leans or cants carries its own angles into the
            // turn the game will draw it with.
            let turn = turn * Quat::from_rotation_z(cant) * Quat::from_rotation_x(lean);
            let colour = palette.shade(&ramp, shade).to_srgba();
            // A hip's two fractions ride in its form, since the game has to
            // build the same frustum and a name alone cannot say how far in the
            // deck stands. See FORMATS.md.
            let hip = match shape {
                Shape::Hip(x, z) => format!("hip:{x:.4}x{z:.4}"),
                _ => String::new(),
            };
            // A cut rides in the form the same way, and for the same reason: the
            // game builds the mesh itself and a name alone cannot say how far
            // the saw travelled. As FRACTIONS of the piece's own length, because
            // the game scales a unit box and never sees the metres.
            //
            // This is what "mitre" and "mitre-back" used to be, and they could
            // only ever say ALL of one end. See FORMATS.md.
            let cut_form = if cut != Vec2::ZERO && matches!(shape, Shape::Box) {
                format!("cut:{:.4}x{:.4}", cut.x / size.x, cut.y / size.x)
            } else {
                String::new()
            };
            let form = match shape {
                Shape::Box if !cut_form.is_empty() => cut_form.as_str(),
                Shape::Box => "box",
                Shape::Wedge => "wedge",
                Shape::Ridge => "ridge",
                Shape::Hip(..) => hip.as_str(),
            };
            let stage = match kind {
                PartKind::Gable(..)
                | PartKind::Ridge(..)
                | PartKind::GableRoof(..)
                | PartKind::Roof(..)
                | PartKind::Chimney(..) => "roof",
                _ => record.stage.as_str(),
            };
            // The cloth is named as well as resolved: the game
            // re-dyes a house's own walls and roof per building,
            // the way it always rolled its own, and leaves every
            // other piece exactly as it was painted.
            boxes.push(format!(
                "    {{\"at\": {}, \"size\": {}, \"turn\": [{:.5}, {:.5}, {:.5}, {:.5}], \
                 \"rgb\": [{}, {}, {}], \"alpha\": {:.2}, \"form\": \"{form}\", \
                 \"cloth\": \"{ramp}:{shade}\", \"stage\": \"{}\"}}",
                say(centre),
                say(size),
                turn.x,
                turn.y,
                turn.z,
                turn.w,
                (colour.red * 255.0).round() as u8,
                (colour.green * 255.0).round() as u8,
                (colour.blue * 255.0).round() as u8,
                clarity,
                stage,
            ));
        }
    }

    // A widget laid by hand overrules the same meaning derived
    // from the furniture under it: a sleeping figure set on a bed
    // to check the fit is that bed's sleeping place, not a second
    // one beside it.
    let by_hand: Vec<(String, Vec3)> = marks
        .iter()
        .filter(|(.., hand)| *hand)
        .map(|(what, at, ..)| (what.clone(), *at))
        .collect();
    marks.retain(|(what, at, _, hand)| {
        *hand
            || !by_hand
                .iter()
                .any(|(other, spot)| other == what && (spot.x - at.x).hypot(spot.z - at.z) < 0.8)
    });
    let marks: Vec<String> = marks
        .iter()
        .map(|(what, at, yaw, _)| {
            format!(
                "    {{\"mark\": \"{what}\", \"at\": {}, \"yaw\": {yaw:.4}}}",
                say(*at)
            )
        })
        .collect();

    (boxes, marks)
}

/// Every level of a work, whichever shape it was saved in.
///
/// One place, so opening a work and baking it can never disagree about what it
/// holds. Levels, then phases without levels, then the one flat list from before
/// either existed.
pub(crate) fn levels_of(work: &Workbench) -> Vec<Level> {
    if !work.levels.is_empty() {
        return work.levels.clone();
    }
    vec![Level {
        name: String::new(),
        phases: if work.stages.is_empty() {
            stages_from_flat(&work.parts)
        } else {
            work.stages.clone()
        },
    }]
}

/// Bakes one work into what a game can eat: every level, every phase of each.
///
/// # Where the origin is
///
/// The BASE level's finished footprint, and every level is measured from it. An
/// upgrade has to land on the building it replaces, so it cannot be recentred on
/// its own bounds - a forge added to one end would shunt the whole blacksmith
/// sideways the day it was built.
///
/// # Why `boxes` and `marks` are still at the top
///
/// They are the BASE level, finished - exactly what a reader of format 1 saw, and
/// exactly what it saw before levels existed. A game reads `levels` when it is
/// ready to and needs no change until then. See FORMATS.md.
pub(crate) fn bake_a_work(
    work: &Workbench,
    palette: &Palette,
    name: &str,
) -> (String, usize, usize) {
    let levels = levels_of(work);
    let nothing: Vec<Placed> = Vec::new();
    let finished = |level: &Level| level.phases.last().unwrap_or(&nothing).clone();

    let base = levels.first().map(finished).unwrap_or_default();
    let (low, high) = bounds_of(&base);
    let middle = Vec3::new((low.x + high.x) * 0.5, 0.0, (low.z + high.z) * 0.5);

    let mut written: Vec<String> = Vec::new();
    for level in &levels {
        let mut phases: Vec<String> = Vec::new();
        for phase in &level.phases {
            let (boxes, _) = bake_one_phase(phase, palette, middle);
            phases.push(format!(
                "        {{\"boxes\": [\n{}\n        ]}}",
                boxes.join(",\n")
            ));
        }
        // The FINISHED building is what the village clears a plot for and what it
        // means by the place: its footprint and its marks, not those of a phase
        // halfway up.
        let done = finished(level);
        let (_, marks) = bake_one_phase(&done, palette, middle);
        let (low, high) = bounds_of(&done);
        // Measured from the SHARED origin, so a level that reaches further than the
        // base says so.
        let reach_w = (low.x - middle.x).abs().max((high.x - middle.x).abs());
        let reach_d = (low.z - middle.z).abs().max((high.z - middle.z).abs());
        written.push(format!(
            "    {{\"name\": \"{}\", \"half_w\": {reach_w:.4}, \"half_d\": {reach_d:.4}, \
             \"high\": {:.4},\n      \"phases\": [\n{}\n      ],\n      \"marks\": [\n{}\n      ]}}",
            level.name,
            high.y,
            phases.join(",\n"),
            marks.join(",\n"),
        ));
    }

    // And the base level, finished, written the older way as well.
    let (boxes, marks) = bake_one_phase(&base, palette, middle);
    let span = high - low;
    let json = format!(
        "{{\n  \"format\": 2,\n  \"name\": \"{name}\",\n  \
         \"half_w\": {:.4},\n  \"half_d\": {:.4},\n  \"high\": {:.4},\n  \
         \"boxes\": [\n{}\n  ],\n  \"marks\": [\n{}\n  ],\n  \
         \"levels\": [\n{}\n  ]\n}}\n",
        span.x * 0.5,
        span.z * 0.5,
        high.y,
        boxes.join(",\n"),
        marks.join(",\n"),
        written.join(",\n"),
    );
    (json, boxes.len(), marks.len())
}
