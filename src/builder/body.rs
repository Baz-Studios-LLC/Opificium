//! The boxes each kind of part is made of.

use super::*;

/// The boxes a part is made of, in its own local space, resting on y = 0 - and
/// wearing whatever a maker has painted it.
pub(crate) fn body_of(kind: &PartKind, repaint: Option<(&str, f32)>) -> Vec<Slab> {
    let mut slabs = shape_of(kind);
    // A repainted part carries its choice into everything it is MADE of - and
    // leaves its plaster standing where the plaster is the field between something
    // else.
    //
    // Applied out here, where no arm can slip past it. It used to sit at the tail
    // of the match, and two arms return early out of the middle of it: a plain
    // wall and a plain gable never reached the paint at all, so neither could be
    // painted however the brush was armed. A part that leaves by another door does
    // not get dressed. Brett: "Walls and foundations can't be painted?"
    if let Some((ramp, shade)) = repaint {
        let field = leaves_its_plaster(kind, &slabs);
        for piece in &mut slabs {
            if field && piece.ramp == PLASTER {
                continue;
            }
            piece.ramp = ramp.to_string();
            piece.shade = shade;
        }
    }
    slabs
}

/// What a wall's panels, a gable's field and a clock's dial are drawn in.
pub(crate) const PLASTER: &str = "bone";

/// Whether the brush should leave this part's plaster where it is.
///
/// It was a LIST of ramps the brush was allowed to touch, and a list is the wrong
/// shape for the question. Stone was missing from it, so a foundation could not be
/// painted at all - nor a plinth, a stone rail or a stone trim - and sand was
/// missing, so a basket could not either, and every prop drawn in plaster alone
/// could not be painted while plaster was on the forbidden list for a reason that
/// had nothing to do with baskets.
///
/// The reason is CONTRAST. A half-timbered wall is timber and plaster, a clock is
/// a case and a dial, and painting either one a single colour is not what anybody
/// means by painting it. So plaster stands only where there is something else for
/// it to stand against; a part made of nothing but plaster is a part made of
/// plaster, and the brush paints it.
pub(crate) fn leaves_its_plaster(kind: &PartKind, slabs: &[Slab]) -> bool {
    match kind {
        // A ceiling is plaster seen from below and that is the whole of it. Its
        // ridge beam is a promise about the roof it will raise rather than a
        // material in the room, so it does not make the plaster a field.
        PartKind::Ceiling { .. } => false,
        _ => slabs.iter().any(|piece| piece.ramp != PLASTER),
    }
}

fn shape_of(kind: &PartKind) -> Vec<Slab> {
    let slab = |x: f32, y: f32, z: f32, sx: f32, sy: f32, sz: f32, ramp: &str, shade: f32| Slab {
        at: Vec3::new(x, y, z),
        size: Vec3::new(sx, sy, sz),
        ramp: ramp.to_string(),
        shade,
        clarity: 1.0,
        shape: Shape::Box,
        lean: 0.0,
        cant: 0.0,
        cut: Vec2::ZERO,
    };
    // A wedge: the gable's own shape.
    let wedge = |x: f32, y: f32, z: f32, sx: f32, sy: f32, sz: f32, ramp: &str, shade: f32| Slab {
        at: Vec3::new(x, y, z),
        size: Vec3::new(sx, sy, sz),
        ramp: ramp.to_string(),
        shade,
        clarity: 1.0,
        shape: Shape::Wedge,
        lean: 0.0,
        cant: 0.0,
        cut: Vec2::ZERO,
    };
    // A piece that leans on its own, about its length: the two slopes of
    // a whole roof, and whatever else wants an angle inside a part.
    #[allow(clippy::too_many_arguments)]
    let leaning =
        |x: f32, y: f32, z: f32, sx: f32, sy: f32, sz: f32, ramp: &str, shade: f32, lean: f32| {
            Slab {
                at: Vec3::new(x, y, z),
                size: Vec3::new(sx, sy, sz),
                ramp: ramp.to_string(),
                shade,
                clarity: 1.0,
                shape: Shape::Box,
                lean,
                cant: 0.0,
                cut: Vec2::ZERO,
            }
        };
    // A piece swung WITHIN its part's face rather than out of it: the diagonal
    // brace in a wall, and anything else that lies across a flat thing at an
    // angle. `leaning` turns a piece about its own length and takes it out of
    // the plane; this keeps it in.
    #[allow(clippy::too_many_arguments)]
    let canted = |x: f32,
                  y: f32,
                  z: f32,
                  sx: f32,
                  sy: f32,
                  sz: f32,
                  ramp: &str,
                  shade: f32,
                  cant: f32,
                  cut: Vec2| {
        Slab {
            at: Vec3::new(x, y, z),
            size: Vec3::new(sx, sy, sz),
            ramp: ramp.to_string(),
            shade,
            clarity: 1.0,
            shape: Shape::Box,
            lean: 0.0,
            cant,
            cut,
        }
    };
    // A ridge cap: the same triangle, laid along the part's length.
    let ridge = |x: f32, y: f32, z: f32, sx: f32, sy: f32, sz: f32, ramp: &str, shade: f32| Slab {
        at: Vec3::new(x, y, z),
        size: Vec3::new(sx, sy, sz),
        ramp: ramp.to_string(),
        shade,
        clarity: 1.0,
        shape: Shape::Ridge,
        lean: 0.0,
        cant: 0.0,
        cut: Vec2::ZERO,
    };
    // Glass: the world shows through it.
    #[allow(unused_variables)]
    // Kept for whatever wants to be seen through next - a lantern's
    // horn pane, water in a trough.
    #[allow(unused_variables)]
    let glass = |x: f32, y: f32, z: f32, sx: f32, sy: f32, sz: f32, ramp: &str, shade: f32| Slab {
        at: Vec3::new(x, y, z),
        size: Vec3::new(sx, sy, sz),
        ramp: ramp.to_string(),
        shade,
        clarity: 0.35,
        shape: Shape::Box,
        lean: 0.0,
        cant: 0.0,
        cut: Vec2::ZERO,
    };
    let mut slabs = match kind {
        // The wall that frames itself. See `PartKind::Framed`.
        //
        // Everything is worked out in whole atoms from the left end and turned
        // into metres once, at the very end, when the slab is cut. Nothing here
        // is snapped, because nothing here is ever off.
        PartKind::Wall {
            long,
            high,
            framed,
            openings,
        } => {
            // A plain wall is one slab, as it always was. A framed one is SOLVED below.
            // Both are the same part now, so a right-click can turn one into the other -
            // and an opening will mean the same thing in either, which is what makes a
            // window one window instead of two different things.
            let span = (long / ATOM).round().max(POST_WIDE as f32 * 2.0) as i32;
            let tall = (high / ATOM).round().max((PLATE_TALL * 3 + 8) as f32) as i32;
            let mut body = Vec::new();

            // Atoms to metres, and the wall's own left end to its middle. The
            // one conversion in the whole solve.
            let across = |from: i32, wide: i32| -> (f32, f32) {
                let middle = (from as f32 + wide as f32 * 0.5) - span as f32 * 0.5;
                (middle * ATOM, wide as f32 * ATOM)
            };
            let timber = |body: &mut Vec<Slab>, from: i32, wide: i32, foot: i32, rise: i32| {
                if wide <= 0 || rise <= 0 {
                    return;
                }
                let (x, w) = across(from, wide);
                body.push(slab(
                    x,
                    (foot as f32 + rise as f32 * 0.5) * ATOM,
                    0.0,
                    w,
                    rise as f32 * ATOM,
                    WALL_THICK,
                    "wood",
                    0.62,
                ));
            };

            let inner_foot = PLATE_TALL;
            let inner_tall = tall - PLATE_TALL * 2;
            let low_tall = ((inner_tall - PLATE_TALL) * 2 / 5).max(4);
            let rail_foot = inner_foot + low_tall;
            let high_foot = rail_foot + PLATE_TALL;
            let high_tall = (tall - PLATE_TALL - high_foot).max(0);

            // Where the hole is, if there is one: its clear edges and the band
            // of wall it takes out. Nothing is subtracted anywhere - this is
            // simply the region the framing works AROUND, and the panels and
            // the rails and the studs all ask it whether they are wanted.
            let holes = openings_at(span, tall, openings);
            // THE HEAD over an opening: a plate's worth of timber, or whatever is
            // left where the opening reaches the top of the wall.
            //
            // A plain wall had none at all. Its plaster simply carried on over the
            // hole, so a window had a sill standing proud under it and nothing
            // whatever across the top - Brett, with a picture of one: "Can we get
            // the top of the window fixed here? I am just talking about a sill on
            // the top." Which is exactly what it is: the sill upside down.
            //
            // Asked in ONE place because three parts of the wall have to agree
            // about it - the plaster that stops short of it, the lintel that
            // starts above it, and the rail that gives way to it.
            let head_over = |hy: i32, hh: i32| {
                let room = if *framed { tall - PLATE_TALL } else { tall } - (hy + hh);
                PLATE_TALL.min(room.max(0))
            };
            // A horizontal member, laid in as many pieces as the openings
            // standing in its band leave. One that crosses no opening runs the
            // whole length in a single timber.
            let course = |body: &mut Vec<Slab>, foot: i32, rise: i32| {
                let mut from = 0;
                for (what, hx, hw, hy, hh, _) in &holes {
                    if foot + rise <= *hy || foot >= hy + hh {
                        continue;
                    }
                    let jamb = jamb_of(*what);
                    timber(body, from, hx - jamb - from, foot, rise);
                    from = hx + hw + jamb;
                }
                timber(body, from, span - from, foot, rise);
            };

            // WHAT FILLS THE WALL is the only thing framing decides. Everything below -
            // the jambs, the lintel, the sill, the panes - belongs to the OPENING and is
            // the same either way. Brett: "we need to unify the two window systems."
            if !framed {
                // Plaster: the wall drawn AROUND its openings rather than cut into pieces
                // after the fact. A column between each pair, a head over each, an apron
                // under it. "A plain wall is a single box and a box cannot have a hole in
                // it" was true while a plain wall was its own species; it is one part now,
                // and a box that cannot have a hole is a box drawn in more than one piece.
                let plaster = |body: &mut Vec<Slab>, from: i32, wide: i32, foot: i32, rise: i32| {
                    if wide <= 0 || rise <= 0 {
                        return;
                    }
                    let (x, w) = across(from, wide);
                    body.push(slab(
                        x,
                        (foot as f32 + rise as f32 * 0.5) * ATOM,
                        0.0,
                        w,
                        rise as f32 * ATOM,
                        WALL_THICK,
                        "wood",
                        0.7,
                    ));
                };
                let mut from = 0;
                for (what, hx, hw, hy, hh, _) in &holes {
                    let jamb = jamb_of(*what);
                    plaster(&mut body, from, hx - jamb - from, 0, tall);
                    // Over the opening, and under it - a door reaches the ground and has
                    // no apron, a window has both. Both stop a plate's depth short, where
                    // the head and the sill stand.
                    let head = hy + hh + head_over(*hy, *hh);
                    plaster(&mut body, hx - jamb, hw + jamb * 2, head, tall - head);
                    // Stopping a plate's depth short, where the sill will stand.
                    plaster(&mut body, hx - jamb, hw + jamb * 2, 0, hy - PLATE_TALL);
                    // Nothing beside the sill: it reaches past the reveal on both sides
                    // and covers the jambs' own columns itself.
                    from = hx + hw + jamb;
                }
                plaster(&mut body, from, span - from, 0, tall);
            } else {
                course(&mut body, 0, PLATE_TALL);
                course(&mut body, tall - PLATE_TALL, PLATE_TALL);
                // THE RAIL, broken where a window's sill lands. A window sits ON the
                // rail - the rail was its sill - so a sill of its own would have stood in
                // the same atoms, which is two solids at one depth however proud one of
                // them is.
                {
                    // BETWEEN THE POSTS, not through them. A corner post runs the whole
                    // clear height and the rail sits inside that band, so a rail starting
                    // at the wall's edge ran its last four atoms inside the post - two
                    // timbers of the same wood at one depth, which never showed because
                    // they are the same colour.
                    let mut from = POST_WIDE;
                    for (what, hx, hw, hy, hh, _) in &holes {
                        // It gives way to THE WHOLE OPENING - its sill, its glass and its
                        // head - because anything of the opening's standing in the rail's
                        // own band is two solids in the same atoms. A door reaches the
                        // ground, so a rail carried across one would lie over the doorway;
                        // a window's sill and head are timbers the rail does not cross but
                        // must still leave room for.
                        //
                        // One test where there were two, and the second one only knew about
                        // the sill: a window whose head landed in the rail's band drew both
                        // of them there.
                        let sill = if *what == Opening::Window {
                            PLATE_TALL
                        } else {
                            0
                        };
                        let (low, high) = (hy - sill, hy + hh + head_over(*hy, *hh));
                        if low >= rail_foot + PLATE_TALL || high <= rail_foot {
                            continue;
                        }
                        let jamb = jamb_of(*what);
                        timber(&mut body, from, hx - jamb - from, rail_foot, PLATE_TALL);
                        from = hx + hw + jamb;
                    }
                    timber(
                        &mut body,
                        from,
                        span - POST_WIDE - from,
                        rail_foot,
                        PLATE_TALL,
                    );
                }

                // A post at each end, running the whole clear height behind the
                // rail - a corner post is one timber, not two stacked.
                timber(&mut body, 0, POST_WIDE, inner_foot, inner_tall);
                timber(
                    &mut body,
                    span - POST_WIDE,
                    POST_WIDE,
                    inner_foot,
                    inner_tall,
                );
            }

            // The opening's own frame: a jamb on each side running the full
            // clear height, a lintel over it, and a sill under a window. This
            // is the timber the hole ADDS - the whole reason nothing has to be
            // cut out of anything.
            for (what, hx, hw, hy, hh, dark) in &holes {
                let jamb = jamb_of(*what);
                // A jamb runs from the opening's own foot, which for a door is
                // the ground, up to the head plate.
                //
                // In a PLAIN wall it runs the opening and no further. Those plates are a
                // framed wall's, and reaching for them here put a post down the wall to
                // the floor beside every window - and z-fighting with the plaster it stood
                // in, since the wall fills the band a jamb has no business occupying.
                let (jamb_foot, jamb_tall) = if *framed {
                    let foot = (*hy).min(inner_foot);
                    (foot, inner_foot + inner_tall - foot)
                } else {
                    (*hy, *hh)
                };
                timber(&mut body, hx - jamb, jamb, jamb_foot, jamb_tall);
                timber(&mut body, hx + hw, jamb, jamb_foot, jamb_tall);

                // WHAT A SILL AND A HEAD SPAN, which is the same question twice: past
                // the reveal in a plain wall, where the jambs frame the opening and the
                // timber covers their plaster columns; between the jambs in a framed
                // one, where they run the whole clear height and a wider piece would
                // stand inside them.
                let (crown_from, crown_wide) = if *framed {
                    (*hx, *hw)
                } else {
                    (hx - jamb, hw + jamb * 2)
                };
                // A CROWN OVER EVERY OPENING, standing proud exactly as the sill does.
                // A window with a sill under it and bare plaster over it reads as a hole
                // somebody forgot to finish, which is what it was.
                let head = head_over(*hy, *hh);
                if head > 0 {
                    let (x, w) = across(crown_from, crown_wide);
                    body.push(slab(
                        x,
                        (hy + hh) as f32 * ATOM + head as f32 * ATOM * 0.5,
                        0.0,
                        w,
                        head as f32 * ATOM,
                        WALL_THICK + SILL_PROUD * 2.0,
                        "wood",
                        0.62,
                    ));
                }
                // And the lintel over THAT, where a framed wall has room left between
                // the head and its head plate: the timber that carries the wall across
                // the hole. A plain wall has no plate, so what a lintel filled there was
                // ordinary wall drawn a second time in the atoms the plaster already
                // occupies - which is the z-fighting a plain wall's window used to show.
                let lintel = lintel_of((tall - PLATE_TALL) - (hy + hh));
                if *framed && lintel > head {
                    timber(&mut body, *hx, *hw, hy + hh + head, lintel - head);
                }
                if *what == Opening::Window {
                    // A sill only where the rail is not already under it, for
                    // the same reason as the lintel above - and in a plain wall the
                    // plaster stops short to leave room for it, so it is the one timber
                    // under a plain window rather than a second skin over the first.
                    // A SILL IN EITHER WALL, standing an atom proud of both faces. The
                    // rail gives way to it in a framed wall - see the courses below -
                    // because a sill and a rail in the same atoms is the flicker this was
                    // meant to avoid rather than cause.
                    let (x, w) = across(crown_from, crown_wide);
                    body.push(slab(
                        x,
                        (hy - PLATE_TALL) as f32 * ATOM + PLATE_TALL as f32 * ATOM * 0.5,
                        0.0,
                        w,
                        PLATE_TALL as f32 * ATOM,
                        WALL_THICK + SILL_PROUD * 2.0,
                        "wood",
                        0.62,
                    ));

                    // The cross in the light: a mullion up the middle and a
                    // transom across it, dividing the opening into four panes.
                    //
                    // Set back with the plaster rather than standing at the
                    // wall's full thickness, because a bar is joinery in the
                    // reveal and not part of the frame carrying the wall over
                    // the hole.
                    let thin = WALL_THICK - (INFILL_SET * 2) as f32 * ATOM;
                    let bar = |body: &mut Vec<Slab>, from: i32, wide: i32, foot: i32, rise: i32| {
                        let (x, w) = across(from, wide);
                        body.push(slab(
                            x,
                            (foot as f32 + rise as f32 * 0.5) * ATOM,
                            0.0,
                            w,
                            rise as f32 * ATOM,
                            thin,
                            // Painted joinery or bare timber. Dark bars are what a hall's
                            // windows wear against pale plaster; a cottage keeps its wood.
                            "wood",
                            if *dark { 0.12 } else { 0.55 },
                        ));
                    };
                    // As many bars as the opening has room for panes. One mullion
                    // divides it into two lights, two into three, and the same across -
                    // so the glazing follows the window's size instead of always being
                    // the same four panes whatever shape the hole is.
                    let (cols, rows) = (panes_in(*hw), panes_in(*hh));
                    for at in 1..cols {
                        let x = hx + (hw * at) / cols - BAR_WIDE / 2;
                        bar(&mut body, x, BAR_WIDE, *hy, *hh);
                    }
                    for at in 1..rows {
                        let y = hy + (hh * at) / rows - BAR_WIDE / 2;
                        bar(&mut body, *hx, *hw, y, BAR_WIDE);
                    }
                }
            }

            // A plain wall is done: it has its plaster and its openings are framed. What
            // follows lays bays and studs, which is the whole of what framing means.
            if !framed {
                return body;
            }

            // ORDINARY WALL, framed: bays across it, a stud at each division,
            // braces in the pairs that carry, and the panels between.
            //
            // A REGION rather than a course, because an opening's own column is
            // ordinary wall too - above its lintel and below its sill - and the
            // wall it is standing in has no idea where those bands are. Worked
            // out by course instead, the opening's entire column was left out top
            // to bottom, which showed as a hole under every window.
            let frame =
                |body: &mut Vec<Slab>, from: i32, to: i32, foot: i32, rise: i32, braced: bool| {
                    let width = (to - from).max(0);
                    if width <= 0 || rise <= 0 {
                        return;
                    }
                    let bays = ((width as f32 / BAY_WANTED as f32).round() as i32).max(1);
                    let mut edge = from;
                    let mut edges = vec![edge];
                    for w in into_bays(width, bays) {
                        edge += w;
                        edges.push(edge);
                    }

                    // Studs at the interior divisions, spanning exactly this
                    // course's clear height.
                    for edge in &edges[1..edges.len() - 1] {
                        timber(body, edge - STUD_WIDE / 2, STUD_WIDE, foot, rise);
                    }

                    // Braces, in the low course only, a pair to a bay rising to
                    // meet at its middle.
                    if braced {
                        for pair in edges.windows(2) {
                            let (a, b) = (pair[0], pair[1]);
                            let bay = (b - a) as f32 * ATOM;
                            let up = rise as f32 * ATOM;
                            let half = bay * 0.5;
                            // Nor in a band too shallow to brace: a pair of
                            // braces in a strip under a low window's sill is
                            // two chips of timber, not a brace.
                            if half <= ATOM * 2.0 || up <= ATOM * 3.0 {
                                continue;
                            }
                            // Both ends meet horizontal timber - the sill below,
                            // the rail above - so both end faces are horizontal,
                            // which makes the brace a parallelogram. The run that
                            // leans a face by the brace's own pitch is its width
                            // against the tangent of that pitch, and the signs
                            // are opposite because one end cuts the top and the
                            // other the bottom.
                            //
                            // IN METRES, like every other cut on this bench: the run is
                            // a distance along the piece, and the mesh divides it by the
                            // piece's own length itself. Divided by that length HERE as
                            // well, it came out over by a factor of the length - which is
                            // nothing much on a brace a metre long and grows as the wall
                            // gets squat, since a shorter brace divides by a smaller
                            // number. Brett, with a picture of one: "when framed walls get
                            // short the lines dont stay clean." Its ends were leaning a
                            // third of a metre out of true and leaving a triangle of
                            // daylight in every corner.
                            let angle = up.atan2(half);
                            let wide = STUD_WIDE as f32 * ATOM;
                            let run = wide / angle.tan().max(1e-3);
                            // LONGER BY EXACTLY WHAT THE SAW TAKES. A square-ended
                            // brace as long as the bay's own diagonal reaches from
                            // the sill to the rail; cut its ends flat and it loses
                            // its own width off the climb, so it stops short at both
                            // and leaves a line of plaster under the rail. Brett:
                            // "This is way better but there is still a gap on the
                            // top."
                            //
                            // The diagonal plus the run gives it back: the piece
                            // spans `reach * sin - wide * cos` up the wall, which is
                            // the course exactly when reach is the hypotenuse plus
                            // the run.
                            let reach = (half * half + up * up).sqrt() + run;
                            let (x, _) = across(a, b - a);
                            for side in [-1.0f32, 1.0] {
                                body.push(canted(
                                    x - side * half * 0.5,
                                    (foot as f32 + rise as f32 * 0.5) * ATOM,
                                    0.0,
                                    reach,
                                    wide,
                                    WALL_THICK,
                                    "wood",
                                    0.62,
                                    side * angle,
                                    // Opposite hands, and THIS way round: at the end
                                    // that stands high the saw takes the underside, and
                                    // at the end that lands low it takes the top. The
                                    // other way about leans both faces the wrong way by
                                    // twice the angle, which is what put a triangle of
                                    // daylight under every brace.
                                    Vec2::new(-side * run, side * run),
                                ));
                            }
                        }
                    }

                    // And the panels: what the framing LEAVES, cut to the gap
                    // and set back so the timber stands proud of it.
                    for pair in edges.windows(2) {
                        let (mut a, mut b) = (pair[0], pair[1]);
                        if a > from {
                            a += STUD_WIDE / 2;
                        }
                        if b < to {
                            b -= STUD_WIDE / 2;
                        }
                        if b <= a {
                            continue;
                        }
                        let (x, w) = across(a, b - a);
                        body.push(slab(
                            x,
                            (foot as f32 + rise as f32 * 0.5) * ATOM,
                            0.0,
                            w,
                            rise as f32 * ATOM,
                            // Thinner than the wall by an atom on each face, so
                            // the timber stands proud of the plaster on both
                            // sides. That shadow line is the whole look; flush,
                            // it is a painted stripe.
                            WALL_THICK - (INFILL_SET * 2) as f32 * ATOM,
                            "bone",
                            0.9,
                        ));
                    }
                };

            // The courses, each broken at every opening - whether the opening
            // stands in that course or not. What is between an opening's jambs
            // belongs to the opening, top to bottom, and is framed below with the
            // bands it actually leaves.
            for (foot, rise) in [(inner_foot, low_tall), (high_foot, high_tall)] {
                let mut from = POST_WIDE;
                for (what, hx, hw, ..) in &holes {
                    let jamb = jamb_of(*what);
                    frame(&mut body, from, hx - jamb, foot, rise, foot == inner_foot);
                    from = hx + hw + jamb;
                }
                frame(
                    &mut body,
                    from,
                    span - POST_WIDE,
                    foot,
                    rise,
                    foot == inner_foot,
                );
            }

            // And each opening's own column: the wall under its sill, and the
            // wall over its lintel. Both are nought for a window filling the
            // course it was born in - which is why this was never needed until a
            // window could be set anywhere up the wall.
            for (what, hx, hw, hy, hh, _) in &holes {
                // A window hangs its sill a plate below its foot; a door has
                // none, and nothing under it but the floor.
                let sill = if *what == Opening::Window {
                    PLATE_TALL
                } else {
                    0
                };
                // Braced, because this is the apron under a window and an apron
                // is a bay like any other - the bays either side of it are.
                frame(
                    &mut body,
                    *hx,
                    hx + hw,
                    inner_foot,
                    hy - sill - inner_foot,
                    true,
                );
                let over = hy + hh + lintel_of((tall - PLATE_TALL) - (hy + hh));
                frame(
                    &mut body,
                    *hx,
                    hx + hw,
                    over,
                    tall - PLATE_TALL - over,
                    false,
                );
            }
            body
        }
        PartKind::SegRun { high, lift } => vec![slab(
            0.0,
            lift + high * 0.5,
            0.0,
            0.25,
            *high,
            WALL_THICK,
            "wood",
            0.7,
        )],
        PartKind::Seg { long, high, lift } => {
            // NO lap. A wall piece is exactly what it measures.
            //
            // It used to be drawn half an atom longer and half an atom taller so
            // its seams could not open - and half an atom is off the lattice, so
            // it stood proud of the wall top, proud of its neighbours, and proud
            // of the corner it ended at. Brett found all three: "it seems to get
            // slightly taller than the other walls", then "same problem,
            // different axis", then the rule that settles it - "everything
            // should respect the grid so all items line up. Whole atoms only."
            //
            // Which turns out to close the seams the lap was for. A sixteenth is
            // a power of two: two pieces meeting on the lattice work out the
            // same edge bit for bit, from different centres, and there is
            // nothing left for the rasteriser to leave a hairline in. The
            // hairline Brett photographed was a piece off the lattice, not a
            // piece that met its neighbour too exactly.
            vec![slab(
                0.0,
                lift + high * 0.5,
                0.0,
                *long,
                *high,
                WALL_THICK,
                "wood",
                0.7,
            )]
        }
        PartKind::Floor(w, d) => vec![slab(0.0, 0.0625, 0.0, *w, 0.125, *d, "wood", 0.5)],
        // The same slab a floor is, because that is what a ceiling is - the difference
        // between them is which side of it a villager stands on, and what it will raise.
        //
        // In LIGHT BONE rather than the floor's wood: it is plaster seen from underneath,
        // and on the bench it is the one thing that tells a ceiling from a floor at a
        // glance when both are lying flat in a half-built room.
        PartKind::Ceiling {
            long,
            deep,
            hipped,
            across,
        } => {
            // WHICH WAY THE RIDGE RUNS decides the whole part, so it is settled first: the
            // long side by default, the other one once a maker has pressed R.
            let along_x = (long >= deep) != *across;
            // THE CEILING GIVES WAY TO THE GABLES. Brett: "there's z fighting from the
            // ceiling and the gable... The ceiling should actually shrink and give way to
            // the gable."
            //
            // Both share the wall plate now, so the ceiling's last quarter-metre at each
            // end of the ridge stood in the same atoms as the gable that lands there - two
            // surfaces at one depth, which is what a renderer flickers over. The ceiling
            // stops short instead, by exactly the thickness the gable will be.
            //
            // Only where a gable actually lands: a hipped roof has none, and slopes down
            // all four sides onto a ceiling that should reach the walls.
            let yield_to_gables = if *hipped { 0.0 } else { GABLE_THICK * 2.0 };
            let (slab_x, slab_z) = if along_x {
                ((*long - yield_to_gables).max(ATOM), *deep)
            } else {
                (*long, (*deep - yield_to_gables).max(ATOM))
            };
            let mut body = vec![slab(
                0.0,
                FLOOR_THICK * 0.5,
                0.0,
                slab_x,
                FLOOR_THICK,
                slab_z,
                "bone",
                0.8,
            )];
            // THE RIDGE IT WILL RAISE, laid on top where a maker can see it. Brett: "A
            // ceiling should have a ridge beam so you know how the roof will generate."
            //
            // It answers both questions a ceiling cannot otherwise be asked. WHICH WAY -
            // the ridge runs the long side, and on a ceiling dragged deeper than it is
            // long the beam swings a quarter, which is exactly what the roof will do.
            // AND WHICH KIND - a gable's ridge runs the whole length, a hip's stops
            // short at both ends where the slopes come in, so the two look different
            // before either is raised.
            let (side, across) = if along_x {
                (*long, *deep)
            } else {
                (*deep, *long)
            };
            // Worked out the way `HipRoof` works it out, so the beam is a promise rather
            // than an impression.
            let ridge = if *hipped {
                let half_side = side * 0.5 + ROOF_OVERHANG;
                let half_across = across * 0.5 + ROOF_OVERHANG;
                let run = on_the_lattice(half_side.min(half_across) * 0.5).max(ATOM);
                ((half_side - run) * 2.0).max(ATOM)
            } else {
                side
            };
            let thick = ATOM * 2.0;
            let (sx, sz) = if along_x {
                (ridge, thick)
            } else {
                (thick, ridge)
            };
            body.push(slab(
                0.0,
                FLOOR_THICK + thick * 0.5,
                0.0,
                sx,
                thick,
                sz,
                "wood",
                0.45,
            ));
            body
        }
        PartKind::Foundation(w, d, high) => {
            let high = on_the_lattice(*high).max(ATOM);
            vec![slab(0.0, high * 0.5, 0.0, *w, high, *d, "stone", 0.55)]
        }
        PartKind::Roof(w, d) => vec![slab(0.0, 0.0625, 0.0, *w, 0.125, *d, "earth", 0.4)],
        PartKind::Gable {
            long,
            pitch: degrees,
            framed,
        } => {
            // One clean slope each way, at the gable's OWN pitch.
            //
            // It used to be the bench's one pitch, with a comment promising that
            // a gable "meets the roof panels exactly" - true while every roof in
            // the world stood at thirty degrees, and false from the moment a
            // roof could be pulled steeper. A steep roof on a thirty degree
            // gable is a wall with daylight over it, which is the opposite of
            // the medieval look the pitch was added for.
            //
            // Snapped to the lattice like everything else, so a gable's peak is
            // somewhere another part can meet it.
            let pitch = degrees.clamp(PITCH_LEAST, PITCH_MOST).to_radians();
            let high = ((long * 0.5 * pitch.tan()) * 16.0).round() / 16.0;
            if !framed {
                return vec![wedge(
                    0.0,
                    high * 0.5,
                    0.0,
                    *long,
                    high,
                    WALL_THICK,
                    "wood",
                    0.65,
                )];
            }

            // HALF-TIMBERED, out of the same parts a framed wall is made of: the
            // plaster set back an atom on each face, and the timber standing proud
            // of it, so the shadow line down each stud is the same shadow line the
            // wall below has. Brett: "Look at the mock up for the framing patterns."
            //
            // The infill is the WHOLE triangle rather than a panel per bay, which
            // is what makes this possible without a new shape: a gable's bays are
            // cut off by the slope, and a box cannot be a triangle. One wedge
            // behind everything can never leave a hole, and what a maker sees
            // between the timbers is the same plaster either way.
            let half = long * 0.5;
            let plate = PLATE_TALL as f32 * ATOM;
            // THE ANGLE THE TRIANGLE ACTUALLY HAS, which is not quite the pitch it
            // was asked for: the peak is snapped to the lattice like every other
            // face on this bench, so a two-metre gable at thirty degrees is drawn
            // at twenty-nine and a third. Framing it at the asked-for angle hangs
            // the rake a few millimetres below its own foot - the same two-halves
            // fault as ever, one number worked out twice.
            let slope = (high / half.max(1e-3)).atan();
            // What the rake takes out of a stud's height: a plate measured
            // PERPENDICULAR to the slope stands taller than a plate on the level.
            let under = plate / slope.cos();
            let mut body = vec![wedge(
                0.0,
                high * 0.5,
                0.0,
                *long,
                high,
                WALL_THICK - (INFILL_SET * 2) as f32 * ATOM,
                "bone",
                0.9,
            )];
            // Where the triangle first has a plate's worth of room in it, measured
            // along the foot. Everything level is held between these: past them the
            // gable is thinner than the timber, and a plate laid the full length
            // would stand out through the slope at both corners with the roof
            // coming down onto it.
            let step = plate / slope.tan().max(1e-3);
            let level = half - step;
            // The plate along its foot, where the gable sits on the wall head.
            if level > 0.0 {
                body.push(slab(
                    0.0,
                    plate * 0.5,
                    0.0,
                    level * 2.0,
                    plate,
                    WALL_THICK,
                    "wood",
                    0.62,
                ));
            }
            // A RAKE up each slope, its top face on the slope itself: the board a
            // roof's edge lands against. Laid as one timber from eave to peak,
            // where the two of them cross in a lozenge no wider than the plate -
            // the same tolerance the wall's braces take running into its posts,
            // and all one wood at one shade.
            let reach = (half * half + high * high).sqrt();
            // Its ends are square, so it has to START up the slope: a board whose
            // top face lies on the slope hangs its inner corner below the foot by
            // exactly a plate's rise, and that would be a tab of timber lapping
            // over the wall below. Stepped up by that much, its lowest corner lands
            // ON the foot.
            let long_enough = reach - step;
            if long_enough > ATOM {
                for side in [-1.0f32, 1.0] {
                    // From this eave up to the peak: the way along, and the way
                    // INTO the triangle, which is where the board's own thickness
                    // has to go if its face is to lie on the slope.
                    let along = Vec2::new(-side * half / reach, high / reach);
                    let inward = Vec2::new(-side * high / reach, -half / reach);
                    let middle = Vec2::new(side * half, 0.0) + along * (step + long_enough * 0.5);
                    let seat = middle + inward * plate * 0.5;
                    // MITRED AT THE PEAK. Both boards are as thick as a plate and
                    // both want their outer face to reach the apex, so their inner
                    // faces have to stop on the centreline or each one comes out
                    // through the other slope - which at sixty degrees is a nub of
                    // timber standing a fifth of a metre outside the roof.
                    //
                    // The saw travels a plate's rise along the board while crossing
                    // it, and the cut is NEGATIVE because it is the inner face that
                    // is shortened, not the one lying on the slope.
                    let mitre = -plate * slope.tan();
                    body.push(canted(
                        seat.x,
                        seat.y,
                        0.0,
                        long_enough,
                        plate,
                        WALL_THICK,
                        "wood",
                        0.62,
                        -side * slope,
                        // Whichever end of the board the peak is at: the one the
                        // rotation put uphill.
                        if side < 0.0 {
                            Vec2::new(0.0, mitre)
                        } else {
                            Vec2::new(mitre, 0.0)
                        },
                    ));
                }
            }
            // And the studs. An EVEN number of bays, always, so there is a
            // division at the middle and a king post standing under the peak -
            // which is what a gable is framed around, and what the drawing shows.
            let span = (long / ATOM).round().max(1.0) as i32;
            let bays = (((span as f32 / BAY_WANTED as f32).round() as i32).max(2) + 1) / 2 * 2;
            let mut edge = 0;
            for w in into_bays(span, bays) {
                edge += w;
                if edge >= span {
                    break;
                }
                // A STUD IS CUT TO THE RAKE, both corners of its head landing on the
                // board's underside. It used to stop square at whatever the underside
                // measured at its middle, which leaves a wedge of plaster over every
                // stud in the gable - Brett: "the vert pieces don't go all the way up,
                // we can make angles now so we should be able to seam it perfectly."
                //
                // NOT snapped to the lattice, and it is the one thing here that is not:
                // a rake's underside is a diagonal and lands between atoms nearly
                // everywhere, so a stud rounded onto the lattice is a stud that has
                // stopped meeting it. A mitre follows the timber it meets.
                let at = (edge as f32 - span as f32 * 0.5) * ATOM;
                let wide = STUD_WIDE as f32 * ATOM;
                let underside = |from_middle: f32| {
                    high * (1.0 - (from_middle.max(0.0) / half).min(1.0)) - under
                };
                // Its two head corners: the one nearer the peak stands taller.
                let tall = underside(at.abs() - wide * 0.5);
                let short = underside(at.abs() + wide * 0.5);
                // Nothing shorter than a stub of timber nobody would cut: out at the
                // eaves the triangle runs out of height, and a stud an atom tall reads
                // as a mistake rather than as framing.
                if short - plate < ATOM * 4.0 {
                    continue;
                }
                // THE KING POST straddles the middle, where the two undersides meet
                // over it in a V. It gets a square head just under the peak: a saw can
                // only cut one of its shoulders here, and a post pointed on one side is
                // a post that looks broken.
                if at.abs() < wide * 0.5 {
                    body.push(slab(
                        at,
                        (plate + short) * 0.5,
                        0.0,
                        wide,
                        short - plate,
                        WALL_THICK,
                        "wood",
                        0.62,
                    ));
                    continue;
                }
                // Laid along its own length and stood upright, so the saw runs across
                // its head rather than down its side: a cut takes the END of a piece,
                // and a stud's end is its head only once the piece is turned.
                let mitre = tall - short;
                body.push(canted(
                    at,
                    (plate + tall) * 0.5,
                    0.0,
                    tall - plate,
                    wide,
                    WALL_THICK,
                    "wood",
                    0.62,
                    std::f32::consts::FRAC_PI_2,
                    // Whichever shoulder is the low one: turned upright, the piece's
                    // own top face is the shoulder facing the near eave.
                    if at < 0.0 {
                        Vec2::new(0.0, mitre)
                    } else {
                        Vec2::new(0.0, -mitre)
                    },
                ));
            }
            body
        }
        PartKind::Ridge(long) => {
            // Half a metre across, a quarter tall: the bench's own pitch
            // again, so it sits down onto two 45 degree slopes.
            vec![ridge(0.0, 0.0625, 0.0, *long, 0.125, 0.5, "earth", 0.35)]
        }
        PartKind::RidgeRun => vec![ridge(0.0, 0.0625, 0.0, 0.25, 0.125, 0.5, "earth", 0.35)],
        PartKind::GableRoof(long, span, over, degrees) => {
            // Both slopes at once, meeting over the middle: no lining up
            // two panels and hoping. The eaves rest at y=0, so the part
            // seats straight onto the wall tops.
            let pitch = degrees.clamp(PITCH_LEAST, PITCH_MOST).to_radians();
            let half = span * 0.5;
            let rise = half * pitch.tan();
            let slope = (half * half + rise * rise).sqrt();
            let thick = 0.125;
            // The slopes reach past the building on every side by the
            // overhang; the gables stay where the walls are.
            let mut sides = Vec::new();
            for way in [-1.0_f32, 1.0] {
                sides.push(leaning(
                    0.0,
                    // Down the slope as well as out along it. The panel grows
                    // by the overhang and its middle has to travel half that
                    // distance ALONG its own pitch to keep the ridge end where
                    // it was - the outward half was here and the downward half
                    // was not, so every pull on the eaves lifted the whole roof
                    // off the gable it was sitting on. Brett found it at once.
                    rise * 0.5 + thick * 0.5 - over * 0.5 * pitch.sin(),
                    way * (half * 0.5 + over * 0.5 * pitch.cos()),
                    long + over * 2.0,
                    thick,
                    slope + over,
                    "earth",
                    0.4,
                    way * pitch,
                ));
            }
            // And the two ends closed: a gable apiece, standing just
            // inside the slopes so the roof laps over them.
            let end = GABLE_THICK;
            for way in [-1.0_f32, 1.0] {
                sides.push(ridge(
                    way * (long * 0.5 - end * 0.5),
                    rise * 0.5,
                    0.0,
                    end,
                    rise,
                    span * 0.995,
                    "wood",
                    0.65,
                ));
            }
            sides
        }
        PartKind::RoofPlan(w, d) => vec![
            slab(0.0, 0.0625, 0.0, *w, 0.125, *d, "earth", 0.4),
            // The ridge line: gold down the way the ridge will run, so
            // the roof can be turned with R before it is set down.
            slab(0.0, 0.1875, 0.0, *w, 0.125, 0.25, "cloth-gold", 0.85),
            // And the two eaves it will come down to.
            slab(
                0.0,
                0.1875,
                -*d * 0.5 + 0.0625,
                *w,
                0.125,
                0.125,
                "cloth-gold",
                0.5,
            ),
            slab(
                0.0,
                0.1875,
                *d * 0.5 - 0.0625,
                *w,
                0.125,
                0.125,
                "cloth-gold",
                0.5,
            ),
        ],
        PartKind::Beam(long, cut_high, cut_low) => {
            // The corner post's own timber, laid over. It rests ON its origin
            // rather than straddling it, the way the post stands on its foot -
            // so a beam dropped on a wall top sits on the wall rather than half
            // inside it.
            //
            // A cut end is the square box stopping short and a prism finishing
            // the job: full height where it joins, tapering to nothing where the
            // saw came out. The two ends want opposite hands of that prism,
            // since the full face points inward at both.
            let thick = 0.375;
            let (high, low) = (cut_high.max(0.0), cut_low.max(0.0));
            // ONE timber, cut at both ends.
            //
            // This used to be three pieces: a square middle with a mitre prism
            // stuck on each end, in opposite hands, because a cut was a SHAPE
            // and no shape could cut both ends. A beam cut at one end had two
            // pieces and a beam cut at neither had one, so its own length meant
            // something different in each case and every seam between them was
            // a place two faces could disagree.
            //
            // A cut is a property of the box now, so a beam is a beam.
            vec![Slab {
                at: Vec3::new(0.0, 0.1875, 0.0),
                size: Vec3::new(long.max(ATOM), thick, thick),
                ramp: "wood".to_string(),
                shade: 0.45,
                clarity: 1.0,
                shape: Shape::Box,
                lean: 0.0,
                cant: 0.0,
                cut: Vec2::new(low, high),
            }]
        }
        PartKind::Trim { long, stone } => {
            let (ramp, shade) = if *stone {
                ("stone", 0.55)
            } else {
                ("wood", 0.5)
            };
            vec![slab(0.0, 0.15625, 0.0, *long, 0.3125, 0.125, ramp, shade)]
        }
        PartKind::TrimRun { stone } => {
            let (ramp, shade) = if *stone {
                ("stone", 0.55)
            } else {
                ("wood", 0.5)
            };
            vec![slab(0.0, 0.15625, 0.0, 0.25, 0.3125, 0.125, ramp, shade)]
        }
        PartKind::Prop("bed") => vec![
            // Long enough for the villager who will lie in it: the
            // sleeper is 1.75 head to heel, so the mattress is 1.875 and
            // the frame two metres. The old bed was a metre and a half,
            // and everyone's feet hung off the end of it.
            slab(0.0, 0.25, 0.0, 0.875, 0.25, 2.0, "wood", 0.55),
            slab(0.0, 0.46875, 0.0, 0.75, 0.1875, 1.875, "bone", 0.8),
            slab(0.0, 0.5625, 0.6875, 0.5, 0.125, 0.375, "bone", 0.95),
        ],
        PartKind::Prop("bed-double") => vec![
            // Room for two who each take three-quarters of a metre with
            // their arms at their sides: a mattress of one and three
            // quarters, so nobody sleeps inside their spouse.
            slab(0.0, 0.25, 0.0, 1.875, 0.25, 2.0, "wood", 0.55),
            slab(0.0, 0.46875, 0.0, 1.75, 0.1875, 1.875, "bone", 0.8),
            slab(-0.46875, 0.5625, 0.6875, 0.5625, 0.125, 0.375, "bone", 0.95),
            slab(0.40625, 0.5625, 0.6875, 0.5625, 0.125, 0.375, "bone", 0.95),
        ],
        PartKind::Prop("table") => {
            let mut parts = vec![slab(0.0, 0.75, 0.0, 1.5, 0.125, 0.875, "wood", 0.65)];
            for (sx, sz) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                parts.push(slab(
                    sx * 0.625,
                    0.375,
                    sz * 0.3125,
                    0.125,
                    0.75,
                    0.125,
                    "wood",
                    0.5,
                ));
            }
            parts
        }
        PartKind::Prop("stool") => vec![
            // Seven units to the seat, and every edge on the lattice.
            slab(0.0, 0.40625, 0.0, 0.375, 0.0625, 0.375, "wood", 0.6),
            slab(0.0, 0.1875, 0.0, 0.25, 0.375, 0.25, "wood", 0.45),
        ],
        PartKind::Prop("hearth") => vec![
            slab(0.0, 0.40625, 0.0, 0.875, 0.8125, 0.625, "stone", 0.6),
            slab(0.0, 0.5625, 0.09375, 0.625, 0.5, 0.4375, "stone", 0.25),
        ],
        PartKind::Chimney(drop) => vec![
            // A stack of dressed stone with a capped throat, and a shaft
            // that reaches down as far as it is told: set on a roof it
            // buries itself in the slope instead of perching on it, and
            // pulled further it runs to the hearth below.
            slab(
                0.0,
                (2.0 - drop) * 0.5,
                0.0,
                0.875,
                2.0 + drop,
                0.875,
                "stone",
                0.5,
            ),
            slab(0.0, 2.0625, 0.0, 1.0, 0.125, 1.0, "stone", 0.6),
            slab(0.0, 2.25, 0.0, 0.75, 0.25, 0.75, "stone", 0.45),
            slab(0.0, 2.4375, 0.0, 0.875, 0.125, 0.875, "stone", 0.6),
        ],
        PartKind::Prop("chair") => vec![
            slab(
                -0.03125, 0.40625, -0.03125, 0.4375, 0.0625, 0.4375, "wood", 0.6,
            ),
            slab(
                0.03125, 0.1875, 0.03125, 0.3125, 0.375, 0.3125, "wood", 0.45,
            ),
            // The back, standing where a back actually meets a back.
            slab(-0.03125, 0.75, -0.21875, 0.4375, 0.75, 0.0625, "wood", 0.55),
        ],
        PartKind::Prop("bench") => vec![
            // Wide enough for the two bodies it claims to seat, and cut
            // to the same seven units as every other seat.
            slab(0.0, 0.40625, 0.0, 1.75, 0.0625, 0.375, "wood", 0.6),
            slab(-0.75, 0.1875, 0.03125, 0.125, 0.375, 0.3125, "wood", 0.45),
            slab(0.75, 0.1875, 0.03125, 0.125, 0.375, 0.3125, "wood", 0.45),
        ],
        PartKind::Prop("couch") => vec![
            // A padded settle: a timber plinth with cloth laid in it.
            // The plinth stands BACK from the front of the cushion, the
            // way an upholstered seat is built - a sitter's shins have to
            // hang somewhere, and a solid block to the front edge is the
            // one place they cannot.
            slab(0.0, 0.1875, -0.0625, 1.875, 0.375, 0.625, "wood", 0.5),
            slab(
                -0.90625, 0.3125, 0.03125, 0.1875, 0.625, 0.8125, "wood", 0.55,
            ),
            slab(
                0.96875, 0.3125, 0.03125, 0.1875, 0.625, 0.8125, "wood", 0.55,
            ),
            slab(0.0, 0.75, -0.28125, 1.875, 0.75, 0.1875, "wood", 0.55),
            // The cushions: two to sit on, two to lean against.
            slab(
                -0.46875,
                0.4375,
                0.09375,
                0.8125,
                0.125,
                0.6875,
                "cloth-wine",
                0.6,
            ),
            slab(
                0.40625,
                0.4375,
                0.09375,
                0.8125,
                0.125,
                0.6875,
                "cloth-wine",
                0.6,
            ),
            slab(
                -0.46875,
                0.71875,
                -0.1875,
                0.8125,
                0.4375,
                0.125,
                "cloth-wine",
                0.5,
            ),
            slab(
                0.40625,
                0.71875,
                -0.1875,
                0.8125,
                0.4375,
                0.125,
                "cloth-wine",
                0.5,
            ),
        ],
        PartKind::Prop("chest") => vec![
            slab(0.03125, 0.25, 0.0, 0.8125, 0.5, 0.5, "wood", 0.5),
            slab(0.03125, 0.5, 0.03125, 0.8125, 0.125, 0.5625, "wood", 0.35),
            slab(
                0.0,
                0.34375,
                0.28125,
                0.125,
                0.1875,
                0.0625,
                "cloth-gold",
                0.7,
            ),
        ],
        PartKind::HipRoof(long, span, over, pitch, deck) => {
            // The roof's own footprint, eaves and all.
            let half_long = (long * 0.5 + over).max(ATOM);
            let half_span = (span * 0.5 + over).max(ATOM);
            // How far the slope runs IN from the eave, the same on all four
            // sides - and how much of that run the DECK keeps.
            //
            // At a half it is the roof this bench always drew. At nought the
            // slopes run the whole way in and meet: a ridge along the longer
            // axis, and a point when the roof is square.
            let reach = half_long.min(half_span);
            let run = on_the_lattice(reach * (1.0 - deck.clamp(0.0, 1.0))).clamp(ATOM, reach);
            let high = on_the_lattice(run * pitch.to_radians().tan()).max(ATOM);
            // What the deck keeps of the box, as a fraction of each half-extent:
            // the slope runs in the same DISTANCE on every side, so the two
            // fractions differ whenever the roof is not square.
            let keep_x = ((half_long - run) / half_long).clamp(0.0, 1.0);
            let keep_z = ((half_span - run) / half_span).clamp(0.0, 1.0);
            vec![Slab {
                at: Vec3::new(0.0, high * 0.5, 0.0),
                size: Vec3::new(half_long * 2.0, high, half_span * 2.0),
                ramp: "earth".to_string(),
                shade: 0.4,
                clarity: 1.0,
                shape: Shape::Hip(keep_x, keep_z),
                lean: 0.0,
                cant: 0.0,
                cut: Vec2::ZERO,
            }]
        }
        PartKind::Rail { long, hand, stone } => {
            let (ramp, post_shade, rail_shade, pin_shade) = if *stone {
                ("stone", 0.5, 0.55, 0.58)
            } else {
                ("wood", 0.4, 0.5, 0.45)
            };
            let hand = on_the_lattice(*hand).clamp(0.375, 2.0);
            let cap = RAIL_POST;
            let long = long.max(RAIL_POST * 2.0);
            let mut body = vec![
                // A newel at each end, capped above the rail the way a flight's
                // are, so the two meet without a seam to see.
                slab(
                    -long * 0.5 + RAIL_POST * 0.5,
                    (hand + cap) * 0.5,
                    0.0,
                    RAIL_POST,
                    hand + cap,
                    RAIL_POST,
                    ramp,
                    post_shade,
                ),
                slab(
                    long * 0.5 - RAIL_POST * 0.5,
                    (hand + cap) * 0.5,
                    0.0,
                    RAIL_POST,
                    hand + cap,
                    RAIL_POST,
                    ramp,
                    post_shade,
                ),
                // The rail itself, from newel to newel.
                slab(
                    0.0,
                    hand,
                    0.0,
                    long - RAIL_POST,
                    RAIL_THICK,
                    RAIL_THICK,
                    ramp,
                    rail_shade,
                ),
            ];
            // Balusters at a pace apart, spread evenly between the newels so a
            // rail of any length looks made rather than cut off.
            let span = long - RAIL_POST * 2.0;
            let gaps = (span / RAIL_GAP).round().max(1.0);
            for step in 1..gaps as i32 {
                let x = -span * 0.5 + span * (step as f32 / gaps);
                // Snapped, like the newels: an even spread is not the lattice,
                // and a baluster off it puts four faces off it.
                body.push(slab(
                    on_the_lattice(x),
                    (hand - RAIL_THICK * 0.5) * 0.5,
                    0.0,
                    RAIL_PIN,
                    hand - RAIL_THICK * 0.5,
                    RAIL_PIN,
                    ramp,
                    pin_shade,
                ));
            }
            body
        }
        PartKind::RailRun { stone } => body_of(
            &PartKind::Rail {
                long: 0.25,
                hand: RAIL_HIGH,
                stone: *stone,
            },
            None,
        ),
        PartKind::Prop("barrel") => vec![
            slab(0.03125, 0.375, 0.03125, 0.5625, 0.75, 0.5625, "wood", 0.55),
            // The hoops stand PROUD of the staves. Drawn flush - the same width
            // and depth as the barrel itself - their sides sat in the same
            // planes as its sides, and two surfaces at one depth is a fight the
            // renderer settles differently every frame: the stripes Brett
            // photographed. A hoop stands out on a real barrel anyway.
            // Eleven atoms to the barrel's nine: proud by an atom all round, and
            // ODD like the body it wraps. An even hoop on a body centred half an
            // atom off lands its own faces half an atom off.
            slab(
                0.03125, 0.15625, 0.03125, 0.6875, 0.0625, 0.6875, "stone", 0.45,
            ),
            slab(
                0.03125, 0.53125, 0.03125, 0.6875, 0.0625, 0.6875, "stone", 0.45,
            ),
        ],
        PartKind::Prop("crate") => vec![
            slab(0.0, 0.3125, 0.0, 0.625, 0.625, 0.625, "wood", 0.6),
            // The lid rests on the crate rather than sinking into its top face.
            slab(0.0, 0.65625, 0.0, 0.5, 0.0625, 0.5, "wood", 0.4),
        ],
        PartKind::Prop("desk") => {
            // THE CLERK'S DESK, for the room where a village keeps its word. A table
            // is four legs and a top; a desk is a carcase - panelled ends, a drawer
            // course under the lid and a back to it - which is what tells the two
            // apart across a hall.
            //
            // At the table's own height, because a village has one height for the
            // things people sit at.
            let a = ATOM;
            let mut body = vec![
                // The lid, overhanging its carcase by an atom all round.
                slab(
                    0.0,
                    a * 12.0,
                    0.0,
                    a * 24.0,
                    a * 2.0,
                    a * 12.0,
                    "wood",
                    0.65,
                ),
                // The back, which is what stops it being a table.
                slab(
                    0.0,
                    a * 5.5,
                    -a * 4.5,
                    a * 20.0,
                    a * 11.0,
                    a * 1.0,
                    "wood",
                    0.5,
                ),
            ];
            for side in [-1.0f32, 1.0] {
                // Panelled ends, standing to the floor.
                body.push(slab(
                    side * a * 10.0,
                    a * 5.5,
                    0.0,
                    a * 2.0,
                    a * 11.0,
                    a * 10.0,
                    "wood",
                    0.5,
                ));
                // Two drawers under the lid, each with a knob in the middle.
                body.push(slab(
                    side * a * 5.0,
                    a * 9.5,
                    a * 5.5,
                    a * 8.0,
                    a * 3.0,
                    a * 1.0,
                    "wood",
                    0.65,
                ));
                body.push(slab(
                    side * a * 5.0,
                    a * 10.0,
                    a * 6.5,
                    a * 2.0,
                    a * 2.0,
                    a * 1.0,
                    "cloth-gold",
                    0.8,
                ));
            }
            body
        }
        PartKind::Prop("lectern") => {
            // A STAND TO SPEAK FROM: a splayed foot, a post, and a board leaning
            // back at a reading angle with a lip to stop the book sliding off it.
            let a = ATOM;
            let lean = 20f32.to_radians();
            vec![
                slab(0.0, a * 1.0, 0.0, a * 10.0, a * 2.0, a * 8.0, "wood", 0.45),
                slab(0.0, a * 9.0, 0.0, a * 4.0, a * 14.0, a * 4.0, "wood", 0.5),
                leaning(
                    0.0,
                    a * 17.0,
                    0.0,
                    a * 14.0,
                    a * 1.0,
                    a * 10.0,
                    "wood",
                    0.6,
                    lean,
                ),
                leaning(
                    0.0,
                    a * 15.5,
                    a * 4.5,
                    a * 14.0,
                    a * 1.0,
                    a * 1.0,
                    "wood",
                    0.45,
                    lean,
                ),
            ]
        }
        PartKind::Prop("books") => {
            // A ROW OF BOOKS, for a shelf or a desk or the arm of a chair. Standing
            // on its own nought like everything else, so it is set on a shelf and
            // lifted to it rather than sunk through it.
            //
            // Each a different height and a different cloth, because a row of
            // identical books is a pattern rather than a shelf - and the last one
            // leaning on its neighbours, which is what a shelf nobody has tidied
            // looks like.
            let a = ATOM;
            let spines = [
                (-a * 5.0, 7.0, "cloth-wine", 0.45),
                (-a * 3.0, 8.0, "cloth-blue", 0.4),
                (-a * 1.0, 6.0, "cloth-green", 0.45),
                (a * 1.0, 8.0, "cloth-sable", 0.5),
                (a * 3.0, 7.0, "cloth-rust", 0.5),
            ];
            let mut body: Vec<Slab> = spines
                .into_iter()
                .map(|(x, tall, cloth, shade)| {
                    slab(
                        x,
                        a * tall * 0.5,
                        0.0,
                        a * 2.0,
                        a * tall,
                        a * 4.0,
                        cloth,
                        shade,
                    )
                })
                .collect();
            // The one that has been put back in a hurry.
            body.push(canted(
                a * 5.5,
                a * 3.5,
                0.0,
                a * 2.0,
                a * 7.0,
                a * 4.0,
                "cloth-red",
                0.45,
                -12f32.to_radians(),
                Vec2::ZERO,
            ));
            body
        }
        PartKind::Prop("shelves") => vec![
            slab(
                -0.40625, 0.8125, 0.03125, 0.0625, 1.625, 0.3125, "wood", 0.5,
            ),
            slab(0.40625, 0.8125, 0.03125, 0.0625, 1.625, 0.3125, "wood", 0.5),
            slab(0.0, 0.53125, 0.03125, 0.875, 0.0625, 0.3125, "wood", 0.65),
            slab(0.0, 1.03125, 0.03125, 0.875, 0.0625, 0.3125, "wood", 0.65),
            slab(0.0, 1.53125, 0.03125, 0.875, 0.0625, 0.3125, "wood", 0.65),
        ],
        PartKind::Prop("cupboard") => vec![
            slab(0.0, 0.75, -0.03125, 0.875, 1.5, 0.4375, "wood", 0.5),
            slab(
                0.03125, 0.78125, 0.21875, 0.8125, 1.3125, 0.0625, "wood", 0.65,
            ),
            slab(
                0.09375,
                0.71875,
                0.28125,
                0.0625,
                0.1875,
                0.0625,
                "cloth-gold",
                0.6,
            ),
        ],
        PartKind::Prop("pot") => vec![
            slab(0.0, 0.1875, 0.0, 0.375, 0.375, 0.375, "stone", 0.3),
            slab(
                -0.03125, 0.40625, -0.03125, 0.4375, 0.0625, 0.4375, "stone", 0.45,
            ),
        ],
        PartKind::Prop("basket") => vec![
            slab(
                -0.03125, 0.15625, -0.03125, 0.4375, 0.3125, 0.4375, "sand", 0.55,
            ),
            slab(0.0, 0.28125, 0.0, 0.5, 0.0625, 0.5, "sand", 0.4),
        ],
        PartKind::Prop("rug") => vec![
            slab(0.0, 0.03125, 0.0, 1.375, 0.0625, 0.875, "cloth-red", 0.55),
            slab(0.0, 0.03125, 0.0, 1.125, 0.0625, 0.625, "cloth-red", 0.75),
        ],
        PartKind::Prop("woodpile") => vec![
            slab(0.0, 0.125, -0.03125, 1.0, 0.25, 0.6875, "wood", 0.4),
            slab(0.0, 0.34375, 0.0, 1.0, 0.1875, 0.5, "wood", 0.5),
            slab(0.0, 0.46875, 0.03125, 1.0, 0.1875, 0.3125, "wood", 0.6),
        ],
        PartKind::Prop("candle") => vec![
            slab(
                0.03125, 0.03125, 0.03125, 0.3125, 0.0625, 0.3125, "stone", 0.5,
            ),
            slab(0.03125, 0.625, 0.03125, 0.0625, 1.125, 0.0625, "wood", 0.4),
            slab(0.0, 1.1875, 0.0, 0.125, 0.125, 0.125, "bone", 0.95),
            slab(
                0.03125,
                1.3125,
                0.03125,
                0.0625,
                0.125,
                0.0625,
                "cloth-gold",
                0.95,
            ),
        ],
        PartKind::Prop("sack") => vec![
            slab(
                -0.03125, 0.21875, -0.03125, 0.4375, 0.4375, 0.4375, "bone", 0.6,
            ),
            slab(
                -0.03125, 0.4375, -0.03125, 0.1875, 0.125, 0.1875, "bone", 0.45,
            ),
        ],
        PartKind::Prop("trough") => vec![
            slab(
                -0.03125, 0.15625, -0.03125, 1.1875, 0.3125, 0.4375, "wood", 0.45,
            ),
            slab(
                0.03125, 0.28125, 0.03125, 1.0625, 0.0625, 0.3125, "water", 0.7,
            ),
        ],
        PartKind::Pole(high) => vec![
            // The corner post: shoulders over both wall ends at a meeting,
            // a shade darker so the frame reads against the panels. The beam's
            // own section, stood on its foot.
            slab(
                0.0,
                high * 0.5,
                0.0,
                0.375,
                high.max(ATOM),
                0.375,
                "wood",
                0.45,
            ),
        ],
        // A post drawn before one had a height of its own.
        PartKind::Prop("pole") => body_of(&PartKind::Pole(WALL_HIGH), None),
        PartKind::Door { double, leaf } => {
            // ONE CONSTRUCTION, four doors. They were three hand-written lists and
            // the fourth was missing: jambs on the lattice, a lintel board across
            // whatever they frame, and a leaf hung in it or not.
            //
            // A double is the single WIDENED rather than a different thing - each
            // leaf the same metre as the single's - so a hall door reads as two of
            // the doors already in the village.
            let clear = if *double { 2.0 } else { 1.0 };
            let jamb = clear * 0.5 + 0.0625;
            let mut body = vec![
                slab(-jamb, 1.0, 0.0, 0.125, 2.0, 0.375, "wood", 0.45),
                slab(jamb, 1.0, 0.0, 0.125, 2.0, 0.375, "wood", 0.45),
                slab(0.0, 2.0625, 0.0, clear + 0.25, 0.125, 0.375, "wood", 0.45),
            ];
            // And what hangs in it, on the lanes the opening's own leaves take -
            // one per leaf, from the one table that says where they are.
            if *leaf {
                for lane in door_lanes(kind) {
                    // The latch on the leaf's FREE edge: the far side of a single,
                    // and the middle where a pair meet.
                    let toward = if *double { -lane.signum() } else { 1.0 };
                    body.push(slab(*lane, 1.0, 0.0625, 1.0, 2.0, 0.125, "wood", 0.35));
                    body.push(slab(
                        lane + toward * 0.375,
                        1.0,
                        0.125,
                        0.125,
                        0.125,
                        0.125,
                        "cloth-gold",
                        0.8,
                    ));
                }
            }
            body
        }
        // The leaf on its own, with no jambs and no lintel around it.
        //
        // For a wall that has already framed its own opening. The door prop is
        // a frame AND a leaf, which is right when a plain wall has been cut and
        // has nothing of its own - but a framed wall gathers jambs and a lintel
        // when it takes the opening, so setting the whole prop down draws a
        // second frame inside the first. Skipping the prop altogether threw the
        // leaf out with the frame, and left a doorway with no door in it.
        //
        // The same leaf and the same latch, at the same places, so a door reads
        // the same whichever kind of wall it hangs in.
        PartKind::Prop("door-leaf") => vec![
            slab(0.0, 1.0, 0.0625, 1.0, 2.0, 0.125, "wood", 0.35),
            slab(0.375, 1.0, 0.125, 0.125, 0.125, 0.125, "cloth-gold", 0.8),
        ],
        PartKind::Prop("door-double-leaf") => vec![
            slab(-0.5, 1.0, 0.0625, 1.0, 2.0, 0.125, "wood", 0.35),
            slab(0.5, 1.0, 0.0625, 1.0, 2.0, 0.125, "wood", 0.35),
            slab(-0.125, 1.0, 0.125, 0.125, 0.125, 0.125, "cloth-gold", 0.8),
            slab(0.125, 1.0, 0.125, 0.125, 0.125, 0.125, "cloth-gold", 0.8),
        ],
        // A window drawn before one had a size of its own. Read as the size it
        // was, rather than as nothing at all - a name that reads back as nothing
        // is a part that vanishes at the next respawn and never reaches a game.
        PartKind::Prop("window") => shape_of(&PartKind::Window {
            wide: WINDOW_WIDE,
            high: ghost_band().rise,
        }),
        PartKind::Window { wide, high } => {
            // WHAT IT WILL BECOME, drawn from the numbers that will make it. Brett: "right
            // now the window only slides at one size, can we make it more representative of
            // what it will look like when placed?"
            //
            // It was a frame of its own invention - a quarter wider than the opening it
            // punches, at a height of its own, and always four panes however the wall it
            // was aimed at would have glazed it. Every one of those is read here instead,
            // so the thing sliding along the wall is the thing that lands in it.
            //
            // Its own size, and the ordinary wall's foot: the size is the PART'S now, so
            // the ghost is exactly what lands however tall the wall is - only how far up
            // is left for the hand, which lifts it from this foot to where it is aimed.
            let (wide, rise) = (*wide, *high);
            let foot = ghost_band().foot;
            let jamb = jamb_of(Opening::Window);
            let (half_w, half_j) = (wide as f32 * 0.5 * ATOM, jamb as f32 * ATOM);
            let middle = (foot as f32 + rise as f32 * 0.5) * ATOM;
            let mut body = vec![
                // The jambs either side, and the sill under, standing proud as it will.
                slab(
                    -half_w - half_j * 0.5,
                    middle,
                    0.0,
                    half_j,
                    rise as f32 * ATOM,
                    WALL_THICK,
                    "wood",
                    0.62,
                ),
                slab(
                    half_w + half_j * 0.5,
                    middle,
                    0.0,
                    half_j,
                    rise as f32 * ATOM,
                    WALL_THICK,
                    "wood",
                    0.62,
                ),
                slab(
                    0.0,
                    (foot - PLATE_TALL) as f32 * ATOM + PLATE_TALL as f32 * ATOM * 0.5,
                    0.0,
                    wide as f32 * ATOM,
                    PLATE_TALL as f32 * ATOM,
                    WALL_THICK + SILL_PROUD * 2.0,
                    "wood",
                    0.62,
                ),
                // And the head over it, which is the piece that closes the shape.
                // Without it the ghost is an open U and a maker can see where a window
                // starts but not where it stops - the one thing they are judging. Proud,
                // like the sill and like the head that lands.
                slab(
                    0.0,
                    (foot + rise) as f32 * ATOM + PLATE_TALL as f32 * ATOM * 0.5,
                    0.0,
                    wide as f32 * ATOM,
                    PLATE_TALL as f32 * ATOM,
                    WALL_THICK + SILL_PROUD * 2.0,
                    "wood",
                    0.62,
                ),
            ];
            // And the panes it will be divided into, by the same rule the wall uses.
            let thin = WALL_THICK - (INFILL_SET * 2) as f32 * ATOM;
            let bar = |body: &mut Vec<Slab>, x: f32, y: f32, w: f32, h: f32| {
                body.push(slab(x, y, 0.0, w, h, thin, "wood", 0.55));
            };
            // In WHOLE ATOMS, dividing the way the wall divides: every face on this bench
            // lands on the lattice, and a bar placed by float division sits between two of
            // them - which a test catches and a maker would find as a seam that will not
            // meet anything.
            let (cols, rows) = (panes_in(wide), panes_in(rise));
            for at in 1..cols {
                let from = (wide * at) / cols - BAR_WIDE / 2;
                bar(
                    &mut body,
                    (from as f32 + BAR_WIDE as f32 * 0.5 - wide as f32 * 0.5) * ATOM,
                    middle,
                    BAR_WIDE as f32 * ATOM,
                    rise as f32 * ATOM,
                );
            }
            for at in 1..rows {
                let up = foot + (rise * at) / rows - BAR_WIDE / 2;
                bar(
                    &mut body,
                    0.0,
                    (up as f32 + BAR_WIDE as f32 * 0.5) * ATOM,
                    wide as f32 * ATOM,
                    BAR_WIDE as f32 * ATOM,
                );
            }
            body
        }
        PartKind::Stairs {
            rise,
            wide,
            stone,
            rail_stone,
            hand,
        } => {
            // A stair is a rhythm, not a size: pick the number of steps that
            // gets nearest the rise asked for, then let every tread be equal.
            // Uneven steps are the one thing a foot notices.
            let (steps, riser, tread) = stair_rhythm(*rise);
            // The one difference between a timber flight and a stone one, asked
            // twice: once of the treads and once of everything a hand touches.
            let (ramp, tread_shade) = if *stone {
                ("stone", 0.6)
            } else {
                ("wood", 0.45)
            };
            let (rail_ramp, post_shade, rail_shade, pin_shade) = if *rail_stone {
                ("stone", 0.5, 0.55, 0.58)
            } else {
                ("wood", 0.4, 0.5, 0.45)
            };
            let rise = steps as f32 * riser;
            let run = steps as f32 * tread;
            let wide = wide.max(0.375);
            let mut body: Vec<Slab> = Vec::new();
            // Treads, each a solid block from the ground - the way the stone
            // steps are drawn, and the way a mason would actually build it.
            for step in 0..steps {
                let high = (step + 1) as f32 * riser;
                body.push(slab(
                    0.0,
                    high * 0.5,
                    -run * 0.5 + (step as f32 + 0.5) * tread,
                    wide,
                    high,
                    tread,
                    ramp,
                    tread_shade,
                ));
            }
            // A newel at each corner, a rail between each pair running at the
            // flight's own angle, and balusters standing on the treads.
            // The railing's own measurements, shared with the flat rail so a
            // landing carries straight on from a flight.
            let post = RAIL_POST;
            let hand = hand.clamp(0.375, 2.0);
            let cap = RAIL_POST;
            let rail_thick = RAIL_THICK;
            // The TREADS stand proud of the rail by a sixteenth, on every side.
            //
            // Flush is the one thing that cannot be: a newel's face in exactly
            // the same plane as a tread's is a fight the renderer settles
            // differently frame to frame - Brett: "We have a z ordering issue."
            // The first answer stood the rail proud instead, and he was right to
            // send it back - "i would like the stairs to stick out a 1/16 not
            // the rail" - because the flight is the thing and the rail is
            // fitted to it. Set in at the front and back as well as the sides,
            // "so that that z fighting is gone" everywhere rather than along one
            // pair of faces.
            let reveal = 0.0625_f32;
            let inset = wide * 0.5 - post * 0.5 - reveal;
            let foot_z = -run * 0.5 + post * 0.5 + reveal;
            // The head newel's back is FLUSH with the flight's back, so the two
            // make one face that meets a wall together - Brett: "If the back of
            // the stair and the pole are alinged it will hit the wall perfect."
            //
            // Flush is safe here and nowhere else on this part, because the
            // newel stands ON the top tread rather than in it: the two share the
            // plane but never the space, so there is nothing for a renderer to
            // settle. Which is the same rule the face audit uses.
            //
            // It hung an ATOM past for a while, to meet a wall without a slot,
            // and bought that with a post hanging in the air whenever there was
            // no wall - which Brett saw at once.
            let head_z = run * 0.5 - post * 0.5;
            let span = head_z - foot_z;
            // The rail stops INSIDE the newels rather than at their centres.
            //
            // A leaning box has square ends, and a square end cut across a slope
            // reaches further at one corner than the other - so the rail poked
            // out of the newel's face at the top. Brett: "the beam is protruding
            // from the front. Can't we miter that to prevent that?" A mitre is
            // the carpenter's answer and the bench cannot draw one here: its
            // mitre cuts across a part's X and this rail runs along Z.
            //
            // Pulling both ends back by half a newel does the same job. The ends
            // are then buried in the posts, where nothing can see whether they
            // are square, and the rail keeps its own line and pitch exactly -
            // both ends move along it by the same amount.
            // Half a newel off each end, measured ALONG the rail rather than
            // across the ground. Scaling the slope length by a ratio of z spans
            // took far more off a steep flight than a post's worth, and left the
            // rail floating between its posts.
            let line = span.hypot(rise);
            let rail_len = (line - post).max(line * 0.5);
            // And it lies on the middle of its own LINE. The head newel hangs a
            // lap past the top tread, which moved the line's midpoint off the
            // part's origin - and the rail stayed at the origin, an atom adrift
            // of the posts it is supposed to join.
            let rail_mid = (foot_z + head_z) * 0.5;
            // A slab's length lies along Z, and leaning about X carries its far
            // end UP when the angle is negative - see `dress_part`.
            let lean = -(rise / span).atan();
            // Where the rail's own line stands at a given point along the run.
            let rail_at = |z: f32| hand + rise * ((z - foot_z) / span);
            for side in [-1.0_f32, 1.0] {
                // The newels stand a CAP above the rail rather than stopping at
                // it. A rail's square end meeting a post's top corner leaves the
                // seam in plain sight, at both ends - Brett: "this join isnt
                // great. Same with the top." Run the post past the joint and the
                // joint is inside the post.
                body.push(slab(
                    side * inset,
                    (hand + cap) * 0.5,
                    foot_z,
                    post,
                    hand + cap,
                    post,
                    rail_ramp,
                    post_shade,
                ));
                body.push(slab(
                    side * inset,
                    rise + (hand + cap) * 0.5,
                    head_z,
                    post,
                    hand + cap,
                    post,
                    rail_ramp,
                    post_shade,
                ));
                body.push(leaning(
                    side * inset,
                    rise * 0.5 + hand,
                    rail_mid,
                    rail_thick,
                    rail_thick,
                    rail_len,
                    rail_ramp,
                    rail_shade,
                    lean,
                ));
                // Balusters, every other tread: each stands on the step under it
                // and reaches the rail's underside, so they shorten and lengthen
                // with the slope the way real ones do. Brett: "a verticle pole
                // gets added every so often to look more like real railing".
                // One on every tread now that a tread is six atoms deep. At
                // four they stood a hand apart and had to be thinned out.
                for step in 1..steps {
                    let z = -run * 0.5 + (step as f32 + 0.5) * tread;
                    // Clear of the newels by half a post, rather than a whole
                    // one: a wider newel had swallowed the window entirely, and
                    // a short flight came out with a rail and nothing under it.
                    if z <= foot_z + post * 0.5 || z >= head_z - post * 0.5 {
                        continue;
                    }
                    let stood = (step + 1) as f32 * riser;
                    let under = rail_at(z) - rail_thick * 0.5;
                    let tall = under - stood;
                    if tall < 0.125 {
                        continue;
                    }
                    // On the lattice, like everything else: a baluster is a
                    // whole atom square, and stands a whole number of atoms
                    // tall. The last hair of it disappears into the rail above,
                    // which is where a joiner would want it anyway.
                    let tall = (tall / ATOM).round().max(1.0) * ATOM;
                    body.push(slab(
                        side * inset,
                        stood + tall * 0.5,
                        z,
                        RAIL_PIN,
                        tall,
                        RAIL_PIN,
                        rail_ramp,
                        pin_shade,
                    ));
                }
            }
            body
        }
        PartKind::Prop("steps") => vec![
            // Three treads rising the foundation's 0.375, from +X.
            slab(0.375, 0.0625, 0.0, 0.375, 0.125, 1.25, "stone", 0.6),
            slab(0.0, 0.125, 0.0, 0.375, 0.25, 1.25, "stone", 0.55),
            slab(-0.375, 0.1875, 0.0, 0.375, 0.375, 1.25, "stone", 0.5),
        ],
        PartKind::Prop("cradle") => vec![
            slab(0.03125, 0.28125, 0.0, 0.5625, 0.3125, 0.875, "wood", 0.55),
            slab(-0.03125, 0.4375, 0.3125, 0.4375, 0.125, 0.25, "bone", 0.9),
            slab(0.0, 0.09375, -0.375, 0.625, 0.0625, 0.125, "wood", 0.4),
            slab(0.0, 0.09375, 0.375, 0.625, 0.0625, 0.125, "wood", 0.4),
        ],
        PartKind::Prop("wardrobe") => vec![
            slab(0.0, 0.9375, 0.0, 1.125, 1.875, 0.5, "wood", 0.5),
            slab(-0.25, 0.96875, 0.28125, 0.5, 1.6875, 0.0625, "wood", 0.62),
            slab(0.25, 0.96875, 0.28125, 0.5, 1.6875, 0.0625, "wood", 0.62),
            slab(
                0.03125,
                0.96875,
                0.28125,
                0.0625,
                0.3125,
                0.0625,
                "cloth-gold",
                0.7,
            ),
        ],
        PartKind::Prop("side-table") => vec![
            slab(0.0, 0.53125, 0.0, 0.625, 0.0625, 0.625, "wood", 0.65),
            slab(-0.21875, 0.25, -0.21875, 0.0625, 0.5, 0.0625, "wood", 0.5),
            slab(0.21875, 0.25, -0.21875, 0.0625, 0.5, 0.0625, "wood", 0.5),
            slab(-0.21875, 0.25, 0.21875, 0.0625, 0.5, 0.0625, "wood", 0.5),
            slab(0.21875, 0.25, 0.21875, 0.0625, 0.5, 0.0625, "wood", 0.5),
        ],
        PartKind::Clock(wide) => {
            // THE FACE, AND NO HANDS. Brett: "I wonder if we should make it hands
            // free and have the game create and animate the hands?" - which is the
            // line this bench already draws everywhere else: the bake speaks static
            // boxes, so anything that MOVES belongs to the game. A door's leaf is
            // geometry and its routing mark is the village's business; a clock's
            // face is geometry and its hands are. See the `clock` mark in the oven,
            // which carries this dial's width so a village can hang hands on it
            // that are the right size.
            //
            // AN OCTAGON, in bands. Brett: "Maybe an octogon for the clock face?"
            // Two squares crossed at a half-quarter make a STAR rather than an
            // octagon - a rotated square's corners stand out past the first one's
            // edges - and a chamfer can only be taken away, which is the one thing
            // a world of boxes cannot do. Three bands can: a full-width course
            // through the middle and a narrower one over and under, which is a
            // square with its corners off.
            //
            // Worked in whole ATOMS from the face's own corner, like a framed wall
            // and for the same reason: a band's width is three quarters of another
            // band's, and three quarters of an odd number of atoms is a face half
            // an atom off the lattice. Integers cannot land there.
            let across = ((wide / ATOM).round() as i32).max(8) & !1;
            // A pair of faces, lowest first - a mark on the left of the dial is
            // built out from the middle and comes back the other way round.
            trait Ordered {
                fn min_max(self) -> (i32, i32);
            }
            impl Ordered for (i32, i32) {
                fn min_max(self) -> (i32, i32) {
                    (self.0.min(self.1), self.0.max(self.1))
                }
            }
            // A piece from its own four faces, so nothing has to be centred by
            // halving anything.
            let piece_at = |x: (i32, i32), y: (i32, i32), z: (i32, i32), ramp: &str, shade: f32| {
                slab(
                    (x.0 + x.1) as f32 * 0.5 * ATOM,
                    (y.0 + y.1) as f32 * 0.5 * ATOM,
                    (z.0 + z.1) as f32 * 0.5 * ATOM,
                    (x.1 - x.0) as f32 * ATOM,
                    (y.1 - y.0) as f32 * ATOM,
                    (z.1 - z.0) as f32 * ATOM,
                    ramp,
                    shade,
                )
            };
            let piece =
                |body: &mut Vec<Slab>,
                 x: (i32, i32),
                 y: (i32, i32),
                 z: (i32, i32),
                 ramp: &str,
                 shade: f32| { body.push(piece_at(x, y, z, ramp, shade)) };
            // A TRUE OCTAGON, in three pieces with two real edges each.
            //
            // It was five stepped bands, and before that three, and Brett saw
            // straight through both: "this is not an octogon", and then the
            // question that settles it - "cant we do angles now, does it have to
            // be steps?" It does not. A cut takes a box's END off at an angle, so a
            // box cut at both ends is a TRAPEZIUM, and an octagon is two of those
            // and a rectangle: a foot-cut band flaring up to full width, the full
            // width through the middle, and a top-cut band closing in again.
            //
            // The run equals the band's own height, which is what makes the edge
            // forty-five degrees - the same arithmetic the gable's rakes and the
            // wall's braces are cut by.
            let octagon = |body: &mut Vec<Slab>, w: i32, z: (i32, i32), ramp: &str, shade: f32| {
                // A regular octagon takes a bit under a third off each corner.
                let corner = (((w as f32 * 0.293).round() as i32).max(1)).min((w - 2) / 2);
                let middle = w - corner * 2;
                let run = corner as f32 * ATOM;
                for (band, cut) in [
                    ((0, corner), Vec2::new(-run, -run)),
                    ((corner, corner + middle), Vec2::ZERO),
                    ((corner + middle, w), Vec2::new(run, run)),
                ] {
                    let mut cut_to = piece_at((-w / 2, w / 2), band, z, ramp, shade);
                    cut_to.cut = cut;
                    body.push(cut_to);
                }
            };
            let mut body = Vec::new();
            // The rim behind, and the dial standing an atom proud of it.
            octagon(&mut body, across, (0, 2), "wood", 0.35);
            let dial = across - 4;
            let inset = (across - dial) / 2;
            let mut face = Vec::new();
            octagon(&mut face, dial, (2, 3), "bone", 0.95);
            for slab in &mut face {
                slab.at.y += inset as f32 * ATOM;
            }
            body.append(&mut face);
            // Four marks on the dial, at the quarters. Twelve would be a dozen
            // slabs on a face read from across a square; the quarters are what says
            // "a clock" at that distance, and the game's own hands say the rest.
            //
            // Measured as a RADIUS from the middle of the dial, so they ring it at
            // the same inset however wide it is drawn.
            let middle = across / 2;
            let (near, far) = ((across / 2 - 5).max(1), (across / 2 - 3).max(2));
            if far > near {
                for side in [-1, 1] {
                    // At noon and six: standing up.
                    piece(
                        &mut body,
                        (-1, 1),
                        (middle + side * near, middle + side * far).min_max(),
                        (3, 4),
                        "wood",
                        0.3,
                    );
                    // At three and nine: lying across.
                    piece(
                        &mut body,
                        (side * near, side * far).min_max(),
                        (middle - 1, middle + 1),
                        (3, 4),
                        "wood",
                        0.3,
                    );
                }
            }
            body
        }
        PartKind::Prop("bell") => {
            // A BELL, for the belfry the townhall's tower already has an opening
            // for. Brett drew one in the mockup and asked for it once the tower
            // stood: "Can you make the bell?"
            //
            // Built the way everything here is built - out of the shapes the bake
            // can speak - and a bell is the one thing in a village that a
            // truncated pyramid is exactly right for: wide at the mouth, drawn in
            // at the shoulder. The hip roof's own shape, three times smaller and
            // stacked, is a bell's silhouette.
            //
            // Every face on a whole atom: eight across the mouth, six at the
            // waist, four at the shoulder, and ten tall to the top of its yoke.
            let bronze = |y: f32, wide: f32, tall: f32, keep: f32| Slab {
                at: Vec3::new(0.0, y, 0.0),
                size: Vec3::new(wide, tall, wide),
                ramp: "cloth-gold".to_string(),
                shade: 0.45,
                clarity: 1.0,
                shape: Shape::Hip(keep, keep),
                lean: 0.0,
                cant: 0.0,
                cut: Vec2::ZERO,
            };
            vec![
                // The mouth, flaring out where it is struck.
                bronze(ATOM * 1.5, ATOM * 8.0, ATOM * 3.0, 6.0 / 8.0),
                // The waist, drawing in.
                bronze(ATOM * 4.5, ATOM * 6.0, ATOM * 3.0, 4.0 / 6.0),
                // The crown it is cast with, square and solid.
                slab(
                    0.0,
                    ATOM * 7.0,
                    0.0,
                    ATOM * 4.0,
                    ATOM * 2.0,
                    ATOM * 4.0,
                    "cloth-gold",
                    0.5,
                ),
                // And the headstock it swings from, reaching past the bell on both
                // sides so it can rest across two beams.
                slab(
                    0.0,
                    ATOM * 9.0,
                    0.0,
                    ATOM * 10.0,
                    ATOM * 2.0,
                    ATOM * 2.0,
                    "wood",
                    0.4,
                ),
            ]
        }
        PartKind::Prop("anvil") => vec![
            slab(-0.03125, 0.15625, 0.0, 0.4375, 0.3125, 0.375, "wood", 0.4),
            slab(0.0, 0.40625, 0.0, 0.25, 0.1875, 0.25, "stone", 0.3),
            slab(-0.03125, 0.5, 0.0, 0.6875, 0.125, 0.25, "stone", 0.45),
        ],
        PartKind::Prop("loom") => vec![
            slab(
                -0.46875, 0.6875, 0.03125, 0.0625, 1.375, 0.0625, "wood", 0.5,
            ),
            slab(0.53125, 0.6875, 0.03125, 0.0625, 1.375, 0.0625, "wood", 0.5),
            slab(
                0.03125, 1.34375, 0.03125, 1.0625, 0.0625, 0.0625, "wood", 0.6,
            ),
            slab(
                0.03125, 0.34375, 0.03125, 1.0625, 0.0625, 0.0625, "wood", 0.6,
            ),
            slab(0.0, 0.8125, 0.03125, 0.875, 0.875, 0.0625, "cloth-red", 0.6),
        ],
        PartKind::Prop("planter") => vec![
            slab(0.0, 0.15625, 0.0, 0.875, 0.3125, 0.375, "earth", 0.4),
            slab(
                -0.21875, 0.40625, -0.03125, 0.1875, 0.1875, 0.1875, "grass", 0.6,
            ),
            slab(0.125, 0.4375, 0.0625, 0.25, 0.25, 0.25, "grass", 0.5),
            slab(
                0.34375, 0.375, -0.03125, 0.1875, 0.125, 0.1875, "grass", 0.7,
            ),
        ],
        PartKind::Prop("fence") => vec![
            // The posts stand ON the fence's own ends and are square in
            // plan, so two fences meeting at a corner put a post in
            // exactly the same place and the two become one post rather
            // than two overlapping ones. Inset posts could never do that:
            // whichever way you turned the second fence, its post landed
            // a finger's width off the first.
            slab(-0.75, 0.4375, 0.0, 0.125, 0.875, 0.125, "wood", 0.45),
            slab(0.75, 0.4375, 0.0, 0.125, 0.875, 0.125, "wood", 0.45),
            slab(0.0, 0.6875, 0.0, 1.5, 0.125, 0.125, "wood", 0.55),
            slab(0.0, 0.375, 0.0, 1.5, 0.125, 0.125, "wood", 0.55),
        ],
        PartKind::Prop("ladder") => vec![
            slab(
                -0.15625, 1.1875, 0.03125, 0.0625, 2.375, 0.0625, "wood", 0.5,
            ),
            slab(0.15625, 1.1875, 0.03125, 0.0625, 2.375, 0.0625, "wood", 0.5),
            slab(0.0, 0.40625, 0.03125, 0.375, 0.0625, 0.0625, "wood", 0.6),
            slab(0.0, 0.84375, 0.03125, 0.375, 0.0625, 0.0625, "wood", 0.6),
            slab(0.0, 1.28125, 0.03125, 0.375, 0.0625, 0.0625, "wood", 0.6),
            slab(0.0, 1.78125, 0.03125, 0.375, 0.0625, 0.0625, "wood", 0.6),
            slab(0.0, 2.21875, 0.03125, 0.375, 0.0625, 0.0625, "wood", 0.6),
        ],
        PartKind::Prop("mannequin") => vec![
            // The game's adult, boxed in bone: a measuring stick with a
            // face. Skipped on import - reference, not furniture.
            slab(-0.125, 0.3125, 0.0, 0.125, 0.625, 0.125, "bone", 0.6),
            slab(0.125, 0.3125, 0.0, 0.125, 0.625, 0.125, "bone", 0.6),
            slab(-0.03125, 0.90625, 0.0, 0.4375, 0.5625, 0.25, "bone", 0.75),
            slab(-0.25, 0.875, 0.0, 0.125, 0.5, 0.125, "bone", 0.6),
            slab(0.25, 0.875, 0.0, 0.125, 0.5, 0.125, "bone", 0.6),
            slab(
                -0.03125, 1.40625, -0.03125, 0.4375, 0.4375, 0.4375, "bone", 0.85,
            ),
        ],
        PartKind::Prop(_) => vec![],
        PartKind::Widget(name) => {
            // A mark the project has not declared still DRAWS - as a plain
            // block in bone - because a work opened in the wrong project should
            // show what it holds rather than lose it.
            let found = crate::project::widgets()
                .iter()
                .find(|mark| mark.word == *name);
            let (ramp, shade) =
                found.map_or(("bone", 0.5), |mark| (mark.ramp.as_str(), mark.shade));
            // The sleeping place is shaped like the sleeper: a body laid
            // out at the game's own proportions, head toward the widget's
            // own facing. No guessing whether a bed is long enough, or
            // which end the pillow wants to be.
            if *name == "sleep" {
                // The village's median adult, measured from its own
                // genome: 1.75 head to heel, a half-metre head, a torso
                // seven units across. Head at the facing end.
                return vec![
                    slab(0.625, 0.25, 0.0, 0.5, 0.5, 0.5, ramp, shade),
                    slab(0.0625, 0.125, -0.03125, 0.625, 0.25, 0.4375, ramp, shade),
                    // Arms alongside.
                    slab(0.0625, 0.09375, -0.28125, 0.5, 0.1875, 0.1875, ramp, shade),
                    slab(0.0625, 0.09375, 0.34375, 0.5, 0.1875, 0.1875, ramp, shade),
                    // Legs, to the foot of the bed.
                    slab(
                        -0.5625, 0.09375, -0.15625, 0.625, 0.1875, 0.1875, ramp, shade,
                    ),
                    slab(
                        -0.5625, 0.09375, 0.09375, 0.625, 0.1875, 0.1875, ramp, shade,
                    ),
                ];
            }
            // Smoke has one direction and it is not a compass point. Every
            // other widget wears a nose along its own +X to say which way
            // it faces - and that nose CANNOT be tilted up, because tilt
            // turns a part about its own X and the nose lies along it. So
            // this one is drawn rising instead of pointing: a plume
            // thinning as it goes, which is the whole of what the mark
            // means.
            // A candle throws its light every way at once, so it is the
            // third of the directionless three. A taper with a flame on
            // it: slimmer and taller than the hearth's, so the two read
            // apart at a glance across a dark room.
            if *name == "light" {
                return vec![
                    slab(0.0, 0.25, 0.0, 0.125, 0.5, 0.125, ramp, shade),
                    slab(0.0, 0.5625, 0.0, 0.25, 0.125, 0.25, ramp, shade),
                    slab(0.0, 0.6875, 0.0, 0.125, 0.125, 0.125, ramp, shade),
                ];
            }
            // A fire is the same case: people gather at one from every
            // side, so a nose pointing at one of them says something the
            // eye cannot check. Drawn as a flame instead - and the yaw is
            // still written into the mark either way, so if a hearth's
            // open side ever comes to matter the direction is already
            // there in the file, carried by the hearth's own turn.
            if *name == "fire" {
                return vec![
                    slab(0.0, 0.09375, 0.0, 0.375, 0.1875, 0.375, ramp, shade),
                    slab(0.0, 0.3125, 0.0, 0.25, 0.25, 0.25, ramp, shade),
                    slab(0.0, 0.5, 0.0, 0.125, 0.125, 0.125, ramp, shade),
                ];
            }
            if *name == "smoke" {
                return vec![
                    slab(0.0, 0.1875, 0.0, 0.375, 0.375, 0.375, ramp, shade),
                    slab(0.0, 0.5, 0.0, 0.25, 0.25, 0.25, ramp, shade),
                    slab(0.0, 0.6875, 0.0, 0.125, 0.125, 0.125, ramp, shade),
                ];
            }
            // A seat is shaped like the sitter: knees toward the facing,
            // so a stool's height and a table's clearance can be judged
            // by eye instead of by arithmetic.
            if *name == "sit" {
                // The same adult, sat on a stool's own seat: hips at
                // seven units, the crown a shade over a metre and a half.
                // Knees toward the facing.
                // The seat surface is where the THIGHS rest - seven
                // units up, which is what every seat in this catalogue
                // is cut to. The first version hung its hip joint there
                // instead, so the thighs lay a hand's breadth inside
                // every chair, stool and cushion in the house.
                return vec![
                    // The crown stays where it always was: 25 units.
                    slab(0.0, 1.3125, 0.0, 0.5, 0.5, 0.5, ramp, shade),
                    // Torso: hip to shoulder, deep front to back.
                    slab(0.0, 0.84375, -0.03125, 0.25, 0.4375, 0.4375, ramp, shade),
                    // Arms hanging at the sides.
                    slab(
                        -0.03125, 0.84375, -0.28125, 0.1875, 0.4375, 0.1875, ramp, shade,
                    ),
                    slab(
                        -0.03125, 0.84375, 0.34375, 0.1875, 0.4375, 0.1875, ramp, shade,
                    ),
                    // Thighs, resting ON the seat, forward to the knees.
                    slab(
                        0.21875, 0.53125, -0.15625, 0.4375, 0.1875, 0.1875, ramp, shade,
                    ),
                    slab(
                        0.21875, 0.53125, 0.09375, 0.4375, 0.1875, 0.1875, ramp, shade,
                    ),
                    // Shins, from the knee down to the floor.
                    slab(
                        0.46875, 0.21875, -0.15625, 0.1875, 0.4375, 0.1875, ramp, shade,
                    ),
                    slab(
                        0.46875, 0.21875, 0.09375, 0.1875, 0.4375, 0.1875, ramp, shade,
                    ),
                ];
            }
            vec![
                slab(0.0, 0.1875, 0.0, 0.375, 0.375, 0.375, ramp, shade),
                // The nose: which way the widget faces.
                slab(0.28125, 0.1875, 0.0, 0.1875, 0.125, 0.125, ramp, shade),
            ]
        }
    };
    slabs
}
