//! The boxes each kind of part is made of.

use super::*;

/// The boxes a part is made of, in its own local space, resting on y = 0.
pub(crate) fn body_of(kind: &PartKind, repaint: Option<(&str, f32)>) -> Vec<Slab> {
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
            if !framed {
                return vec![slab(
                    0.0,
                    high * 0.5,
                    0.0,
                    *long,
                    *high,
                    WALL_THICK,
                    "wood",
                    0.7,
                )];
            }
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

            course(&mut body, 0, PLATE_TALL);
            course(&mut body, tall - PLATE_TALL, PLATE_TALL);
            course(&mut body, rail_foot, PLATE_TALL);

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

            // The opening's own frame: a jamb on each side running the full
            // clear height, a lintel over it, and a sill under a window. This
            // is the timber the hole ADDS - the whole reason nothing has to be
            // cut out of anything.
            for (what, hx, hw, hy, hh, dark) in &holes {
                let jamb = jamb_of(*what);
                // A jamb runs from the opening's own foot, which for a door is
                // the ground, up to the head plate.
                let jamb_foot = (*hy).min(inner_foot);
                let jamb_tall = inner_foot + inner_tall - jamb_foot;
                timber(&mut body, hx - jamb, jamb, jamb_foot, jamb_tall);
                timber(&mut body, hx + hw, jamb, jamb_foot, jamb_tall);
                // The lintel fills whatever is left between the opening's head
                // and the head plate, rather than being one plate thick and
                // leaving a strip of nothing over it. Where the opening reaches
                // the plate there is nothing left to fill and no lintel: a
                // window has the plate itself over it.
                let over = (tall - PLATE_TALL) - (hy + hh);
                if over > 0 {
                    timber(&mut body, *hx, *hw, hy + hh, over);
                }
                if *what == Opening::Window {
                    // A sill only where the rail is not already under it, for
                    // the same reason as the lintel above.
                    if *hy > high_foot {
                        timber(&mut body, *hx, *hw, hy - PLATE_TALL, PLATE_TALL);
                    }

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
                    let panes =
                        |across: i32| (across as f32 / PANE_WANTED as f32).round().max(1.0) as i32;
                    let (cols, rows) = (panes(*hw), panes(*hh));
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

            // The framing, course by course - and the spans worked out FOR EACH
            // COURSE rather than once for the wall.
            //
            // An opening only interrupts the courses it actually stands in. In
            // the courses it does not, the wall between its jambs is ordinary
            // wall and wants ordinary framing: its own bays, its own studs, its
            // own braces. Worked out once for the whole wall instead, the
            // opening's entire column was left out top to bottom - which first
            // showed as a hole under every window, and then, once that was
            // filled with a single panel, as an apron with no timber in it
            // while the bays either side of it were braced.
            for (foot, rise) in [(inner_foot, low_tall), (high_foot, high_tall)] {
                if rise <= 0 {
                    continue;
                }
                let mut spans: Vec<(i32, i32)> = Vec::new();
                let mut from = POST_WIDE;
                for (what, hx, hw, hy, hh, _) in &holes {
                    let jamb = jamb_of(*what);
                    if foot < hy + hh && foot + rise > *hy {
                        // The opening stands in this course: the wall stops at
                        // its jamb and picks up again past the other one.
                        spans.push((from, hx - jamb));
                        from = hx + hw + jamb;
                    } else {
                        // It does not: the wall between its jambs is ordinary
                        // wall in this course and wants ordinary framing. An
                        // apron under a window is a bay like any other, and
                        // gets its own studs and its own braces.
                        spans.push((from, hx - jamb));
                        spans.push((*hx, hx + hw));
                        from = hx + hw + jamb;
                    }
                }
                spans.push((from, span - POST_WIDE));

                for (from, to) in spans {
                    let width = (to - from).max(0);
                    if width <= 0 {
                        continue;
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
                        timber(&mut body, edge - STUD_WIDE / 2, STUD_WIDE, foot, rise);
                    }

                    // Braces, in the low course only, a pair to a bay rising to
                    // meet at its middle.
                    if foot == inner_foot {
                        for pair in edges.windows(2) {
                            let (a, b) = (pair[0], pair[1]);
                            let bay = (b - a) as f32 * ATOM;
                            let up = rise as f32 * ATOM;
                            let half = bay * 0.5;
                            if half <= ATOM * 2.0 || up <= ATOM {
                                continue;
                            }
                            // Both ends meet horizontal timber - the sill below,
                            // the rail above - so both end faces are horizontal,
                            // which makes the brace a parallelogram. The run that
                            // leans a face by the brace's own pitch is its width
                            // against the tangent of that pitch, and the signs
                            // are opposite because one end cuts the top and the
                            // other the bottom.
                            let angle = up.atan2(half);
                            let reach = (half * half + up * up).sqrt() + ATOM;
                            let wide = STUD_WIDE as f32 * ATOM;
                            let run = (wide / angle.tan().max(1e-3)) / reach;
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
                                    Vec2::new(side * run, -side * run),
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
                }
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
        PartKind::Ceiling { long, deep, .. } => vec![slab(
            0.0,
            FLOOR_THICK * 0.5,
            0.0,
            *long,
            FLOOR_THICK,
            *deep,
            "bone",
            0.8,
        )],
        PartKind::Foundation(w, d, high) => {
            let high = on_the_lattice(*high).max(ATOM);
            vec![slab(0.0, high * 0.5, 0.0, *w, high, *d, "stone", 0.55)]
        }
        PartKind::Roof(w, d) => vec![slab(0.0, 0.0625, 0.0, *w, 0.125, *d, "earth", 0.4)],
        PartKind::RoofRun => vec![slab(0.0, 0.0625, 0.0, 0.25, 0.125, 0.25, "earth", 0.4)],
        PartKind::Gable(long, degrees) => {
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
            vec![wedge(
                0.0,
                high * 0.5,
                0.0,
                *long,
                high,
                WALL_THICK,
                "wood",
                0.65,
            )]
        }
        PartKind::GableRun => vec![wedge(
            0.0, 0.0625, 0.0, 0.25, 0.125, WALL_THICK, "wood", 0.65,
        )],
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
            let end = 0.25;
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
        PartKind::GableRoofRun => {
            vec![leaning(
                0.0, 0.0625, 0.0, 0.25, 0.125, 0.25, "earth", 0.4, 0.0,
            )]
        }
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
        PartKind::BeamRun => {
            vec![slab(0.0, 0.1875, 0.0, 0.25, 0.375, 0.375, "wood", 0.45)]
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
        PartKind::HipRoof(long, span, over, pitch) => {
            // The roof's own footprint, eaves and all.
            let half_long = (long * 0.5 + over).max(ATOM);
            let half_span = (span * 0.5 + over).max(ATOM);
            // How far the slope runs IN from the eave, the same on all four
            // sides: half of the shorter half-extent, which leaves a deck of the
            // same proportion whatever shape the building is.
            let run = on_the_lattice(half_long.min(half_span) * 0.5).max(ATOM);
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
        PartKind::HipRoofRun => body_of(
            &PartKind::HipRoof(0.25, 0.25, 0.0, ROOF_PITCH_DEGREES),
            None,
        ),
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
        PartKind::Prop("pole") => vec![
            // The corner post: shoulders over both wall ends at a meeting,
            // a shade darker so the frame reads against the panels.
            slab(
                0.0,
                WALL_HIGH * 0.5,
                0.0,
                0.375,
                WALL_HIGH,
                0.375,
                "wood",
                0.45,
            ),
        ],
        PartKind::Prop("door") => vec![
            // Jambs, lintel board, and the leaf itself, all on the
            // lattice, with a gold latch.
            slab(-0.5625, 1.0, 0.0, 0.125, 2.0, 0.375, "wood", 0.45),
            slab(0.5625, 1.0, 0.0, 0.125, 2.0, 0.375, "wood", 0.45),
            slab(0.0, 2.0625, 0.0, 1.25, 0.125, 0.375, "wood", 0.45),
            slab(0.0, 1.0, 0.0625, 1.0, 2.0, 0.125, "wood", 0.35),
            slab(0.375, 1.0, 0.125, 0.125, 0.125, 0.125, "cloth-gold", 0.8),
        ],
        PartKind::Prop("door-double") => vec![
            // The hall door: two leaves meeting in the middle, for the
            // buildings a village walks into rather than through - a hall, a
            // barn, a granary taking a cart. Two full metres of clear opening
            // against the single door's one.
            //
            // The single door's own construction, widened: jambs on the
            // lattice, a lintel board across both leaves, and each leaf the
            // same width as the single's, so a double reads as two of the
            // doors already in the world rather than as a different thing.
            slab(-1.0625, 1.0, 0.0, 0.125, 2.0, 0.375, "wood", 0.45),
            slab(1.0625, 1.0, 0.0, 0.125, 2.0, 0.375, "wood", 0.45),
            slab(0.0, 2.0625, 0.0, 2.25, 0.125, 0.375, "wood", 0.45),
            slab(-0.5, 1.0, 0.0625, 1.0, 2.0, 0.125, "wood", 0.35),
            slab(0.5, 1.0, 0.0625, 1.0, 2.0, 0.125, "wood", 0.35),
            // Latches where the leaves meet, an eighth in from each free
            // edge - the same hand's reach as the single door's.
            slab(-0.125, 1.0, 0.125, 0.125, 0.125, 0.125, "cloth-gold", 0.8),
            slab(0.125, 1.0, 0.125, 0.125, 0.125, 0.125, "cloth-gold", 0.8),
        ],
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
        PartKind::Prop("doorway") => vec![
            // An opening with no leaf: jambs and a lintel, for the ways
            // between rooms that never wanted a door.
            slab(-0.5625, 1.0, 0.0, 0.125, 2.0, 0.375, "wood", 0.45),
            slab(0.5625, 1.0, 0.0, 0.125, 2.0, 0.375, "wood", 0.45),
            slab(0.0, 2.0625, 0.0, 1.25, 0.125, 0.375, "wood", 0.45),
        ],
        PartKind::Prop("window") => vec![
            // An opening in a timber frame with its bars across it, and
            // no glass in it at all: glass is a later century than this
            // village. Jambs, sill, lintel, and the two muntins.
            slab(-0.5625, 1.375, 0.0, 0.125, 1.125, 0.375, "wood", 0.45),
            slab(0.5625, 1.375, 0.0, 0.125, 1.125, 0.375, "wood", 0.45),
            slab(0.0, 0.8125, 0.0, 1.25, 0.125, 0.375, "wood", 0.45),
            slab(0.0, 1.9375, 0.0, 1.25, 0.125, 0.375, "wood", 0.45),
            slab(0.0, 1.375, -0.03125, 0.125, 1.0, 0.1875, "wood", 0.4),
            slab(0.0, 1.375, -0.03125, 1.0, 0.125, 0.1875, "wood", 0.4),
        ],
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
    // A repainted part carries its choice into every structural slab.
    if let Some((ramp, shade)) = repaint {
        for piece in &mut slabs {
            if piece.ramp == "wood" || piece.ramp == "earth" || piece.ramp.starts_with("cloth") {
                piece.ramp = ramp.to_string();
                piece.shade = shade;
            }
        }
    }
    slabs
}
