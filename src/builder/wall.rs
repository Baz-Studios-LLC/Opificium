//! Punching an opening through a wall, and healing one closed again.

use super::*;

/// Taking an opening out of a wall closes the wall back up: the pieces
/// the punch left - the sides, the header, a window's sill - merge into
/// one whole wall again, and a door's routing widget goes with it.
pub(crate) fn heal_wall(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
    placed: &Query<(Entity, &Transform, &Placed, &Visibility), Without<Ghost>>,
    frame: Entity,
) -> bool {
    let Ok((_, frame_at, _, _)) = placed.get(frame) else {
        return false;
    };
    let spot = frame_at.translation;
    heal_wall_at(commands, meshes, materials, palette, placed, frame, spot)
}

/// The same closing, at a spot the frame may since have left - a door
/// dragged along its wall heals the hole it came from.
#[allow(clippy::too_many_arguments)]
pub(crate) fn heal_wall_at(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
    placed: &Query<(Entity, &Transform, &Placed, &Visibility), Without<Ghost>>,
    frame: Entity,
    spot: Vec3,
) -> bool {
    let Ok((_, _, frame_record, _)) = placed.get(frame) else {
        return false;
    };
    // Through `opening_of`, not a literal beside it. This was written as a
    // bare 1.25 - correct while every opening in the shelf happened to be one
    // and a quarter wide, and wrong the moment a double door existed.
    let Some(width) = kind_from_name(&frame_record.part)
        .and_then(|kind| opening_of(&kind))
        .map(|(wide, ..)| wide)
    else {
        return false;
    };
    let along = Quat::from_rotation_y(frame_record.yaw) * Vec3::X;
    let base = spot;

    // Everything standing on this wall's own line, measured along it.
    let mut doomed: Vec<Entity> = Vec::new();
    let mut low = -width * 0.5;
    let mut high = width * 0.5;
    let mut cloth: Option<Placed> = None;
    for (entity, transform, record, showing) in placed {
        // A wall the cutaway has taken away holds nothing up and
        // catches nothing: what you cannot see, you cannot build on.
        if *showing == Visibility::Hidden {
            continue;
        }
        if entity == frame {
            continue;
        }
        let offset = transform.translation - base;
        if (offset.y).abs() > 0.05 {
            continue;
        }
        let reach = offset.dot(along);
        if (offset - along * reach).length() > 0.2 {
            continue;
        }
        // The door's own widget rides along.
        if matches!(kind_from_name(&record.part), Some(PartKind::Widget("door")))
            && reach.abs() < 0.2
        {
            doomed.push(entity);
            continue;
        }
        let Some(kind) = kind_from_name(&record.part) else {
            continue;
        };
        let facing = Quat::from_rotation_y(record.yaw) * Vec3::X;
        if facing.dot(along).abs() < 0.99 {
            continue;
        }
        let (long, full) = match kind {
            PartKind::Wall { long, .. } => (long, true),
            PartKind::Seg { long, high, lift } => {
                (long, lift.abs() < 0.01 && (high - WALL_HIGH).abs() < 0.05)
            }
            _ => continue,
        };
        let (piece_low, piece_high) = (reach - long * 0.5, reach + long * 0.5);
        let fills_opening = reach.abs() < 0.1 && !full;
        let touches_left = (piece_high - low).abs() < 0.1 && full;
        let touches_right = (piece_low - high).abs() < 0.1 && full;
        if !(fills_opening || touches_left || touches_right) {
            continue;
        }
        doomed.push(entity);
        low = low.min(piece_low);
        high = high.max(piece_high);
        if full {
            cloth = Some(record.clone());
        }
    }

    let dressed = cloth.unwrap_or_else(|| frame_record.clone());
    let made = PartKind::wall(((high - low) * 16.0).round() / 16.0);
    let centre = base + along * ((low + high) * 0.5);
    let whole = Placed {
        part: part_name(&made),
        at: centre.into(),
        yaw: frame_record.yaw,
        tilt: 0.0,
        ramp: dressed.ramp.clone(),
        shade: dressed.shade,
        stage: "walls".to_string(),
        flip: false,
        loose: false,
        material: String::new(),
        group: None,
    };
    for piece in doomed {
        commands.entity(piece).despawn();
    }
    spawn_part(commands, meshes, materials, palette, &made, &whole, false);
    true
}

/// Splits the nearest wall around an opening and sets the frame in it.
/// Returns false when no wall stands close enough to take the punch.
#[allow(clippy::too_many_arguments)]
/// A wall the punch may part: pristine, or a full-height leaving from
/// an earlier punch - a second window in the same run is honest work.
/// The hole a part cuts in a wall: how wide, how high its head, how far
/// its sill stands off the floor, and whether a routing widget comes with
/// it. One table, read both by the ghost that SHOWS the placement and by
/// the punch that makes it - they each had their own copy once, and the
/// ghost drifted off onto the roof while the door went into the wall.
///
/// `clear` is the fifth thing it says, and the reason a double door used to fail on
/// a framed wall: the first number is how far a PLAIN wall parts, in metres, which
/// includes the frame the prop brings with it - and a framed wall brings its own
/// frame, so what it needs is the CLEAR span the leaves must fit through, in atoms.
/// Those are different numbers, and only one table should hold either.
pub fn opening_of(kind: &PartKind) -> Option<(f32, f32, f32, bool, i32)> {
    match kind {
        // One leaf a metre wide: sixteen atoms of clear.
        PartKind::Prop("door") => Some((1.25, 2.125, 0.0, true, DOOR_WIDE)),
        // Twice the leaf, so twice the hole - and twice the clear, which is the
        // half that was missing. `door-double-leaf` hangs two metre-wide leaves at
        // either side of the middle, spanning two metres, so a framed wall that
        // reserved a single door's sixteen atoms put them over solid timber.
        PartKind::Prop("door-double") => Some((2.25, 2.125, 0.0, true, DOOR_WIDE * 2)),
        // A bare doorway needs no widget: the gap itself is the portal,
        // and a widget would only say it twice.
        PartKind::Prop("doorway") => Some((1.25, 2.125, 0.0, false, DOOR_WIDE)),
        PartKind::Prop("window") => Some((1.25, 2.0, 0.75, false, WINDOW_WIDE)),
        _ => None,
    }
}

/// Where the routing widgets stand in an opening, measured along the wall from
/// its middle: one lane per leaf.
///
/// Brett's idea, and it needs nothing new anywhere else: the game reads EVERY
/// mark called "door" into a building's list of doorways and steers each walker
/// to the nearest one, so two marks a metre apart in one opening are two lanes,
/// and two villagers meeting at a double door take one each instead of queueing
/// through the same point. The part that knows it has two leaves is the part that
/// should say where they are.
pub fn door_lanes(kind: &PartKind) -> &'static [f32] {
    match kind {
        // One lane per leaf, each on its own leaf's centre.
        PartKind::Prop("door-double") => &[-0.5, 0.5],
        _ => &[0.0],
    }
}

/// Where along a wall an opening aimed at `point` actually lands: on the
/// lattice, and never spilling past either end.
pub fn opening_seat(
    wall_at: Vec3,
    along: Vec3,
    length: f32,
    wide: f32,
    point: Vec3,
    grid: f32,
) -> f32 {
    let half = length * 0.5;
    let reach = (half - wide * 0.5).max(0.0);
    let t = (point - wall_at).dot(along).clamp(-reach, reach);
    // On the grid G sets, not on whole atoms always. An opening was the one thing a
    // hand placed that ignored the grid entirely - Brett: "when placing a door it
    // should respect the grid settings when you press g" - so a maker laying out a
    // wall in quarter metres had to nudge its door by sixteenths.
    //
    // The step is measured from the wall's own middle, so a door moves in grid
    // strides along the timber it is going into rather than along the world.
    ((t * grid).round() / grid).clamp(-reach, reach)
}

/// How far UP a wall an opening aimed at `aim` sets its foot, in atoms.
///
/// The companion to [`opening_seat`], and deliberately the same shape: the same
/// grid, striding up the wall the way that one strides along it. Brett: "I
/// should be able to place the window atom perfect anywhere on the wall" - so G
/// sets the stride and alt drops it to whole atoms, exactly as they do sideways.
///
/// The cursor holds a window by its MIDDLE, which is where a hand thinks it is
/// holding it; the foot is what a wall is told.
///
/// THE STRIDES ARE MEASURED FROM THE COURSE the wall would have put it in, not
/// from the wall's foot, and that is the whole difference between a window you
/// can put anywhere and a window you can no longer put where it belongs. The
/// course is one and an eighth of a metre up an ordinary wall; a stride of a
/// quarter-metre off the FLOOR cannot land on it at all, so every window placed
/// with the ordinary grid would have missed the rail by two atoms and every
/// wall in the village would have been spelled a new way. Off the course, the
/// course is the stop you get for aiming at it, and a stride is a stride away.
pub fn opening_lift(wall_foot: f32, tall: i32, usual: Band, aim: f32, grid: f32) -> i32 {
    let course = usual.foot as f32 * ATOM;
    let want = aim - wall_foot - usual.rise as f32 * ATOM * 0.5 - course;
    let stepped = (want * grid).round() / grid;
    // Never sunk into the sill plate, never up into the head plate - and the
    // second clamp gives way to the first on a wall too short to hold it, since
    // a window with its foot above its head is not a window.
    (usual.foot + (stepped / ATOM).round() as i32).clamp(
        LOWEST_SILL,
        (tall - PLATE_TALL - usual.rise).max(LOWEST_SILL),
    )
}

/// How far an opening's ghost stands off the face it is aimed at.
///
/// Half the wall, so its middle reaches the face, and an atom more so the whole
/// of it is clear of the timber. A ghost inside a wall is a ghost nobody can
/// see: a window's frame is drawn at the wall's own thickness and its bars are
/// set back in the reveal, both of them right for a window standing in a hole
/// and both of them buried in a wall that has not got the hole yet.
pub const GHOST_PROUD: f32 = WALL_THICK * 0.5 + ATOM;

/// How far a window's ghost stands off where it is drawn, to show where it will
/// land.
///
/// The other half of [`ghost_band`], and a function rather than a line in the
/// hand because a ghost lifted by arithmetic written out twice is the trap this
/// bench keeps falling into: the offer moves and the answer does not.
pub fn ghost_lift(foot: i32) -> f32 {
    (foot - ghost_band().foot) as f32 * ATOM
}

pub(crate) fn punchable_length(record: &Placed) -> Option<f32> {
    match kind_from_name(&record.part)? {
        PartKind::Wall { long, .. } => Some(long),
        // A framed wall takes an opening too - it just does not have to be cut
        // to take one. Being punchable is what lets a window's ghost seat
        // itself along the wall as you aim, which is the same arithmetic
        // whether the wall is going to be parted or is going to reframe itself.
        PartKind::Seg { long, high, lift }
            if lift.abs() < 0.01 && (high - WALL_HIGH).abs() < 0.05 =>
        {
            Some(long)
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
/// Which of a wall's openings the cursor is on, if any.
///
/// A window is not a part any more - it is a hole a wall was told about - so grabbing one
/// means asking the wall which of its openings was struck. Brett: "I need to be able to
/// pick windows up."
///
/// Both bands are checked, across the wall and up it, because a wall with a window in it is
/// mostly WALL: a maker clicking the plaster under a window means the wall, and a maker
/// clicking the glass means the window.
pub(crate) fn opening_under(
    kind: &PartKind,
    wall_at: Vec3,
    yaw: f32,
    point: Vec3,
) -> Option<usize> {
    let PartKind::Wall {
        long,
        high,
        openings,
        ..
    } = kind
    else {
        return None;
    };
    let span = (long / ATOM).round().max(POST_WIDE as f32 * 2.0) as i32;
    let tall = (high / ATOM).round().max((PLATE_TALL * 3 + 8) as f32) as i32;
    let along = Quat::from_rotation_y(yaw) * Vec3::X;
    // Along the wall from its own left end, and up from its foot, both in atoms.
    let across = (point - wall_at).dot(along) / ATOM + span as f32 * 0.5;
    let up = (point.y - (wall_at.y - 0.0)) / ATOM;
    for (what, hx, hw, hy, hh, _) in openings_at(span, tall, openings) {
        if across >= hx as f32
            && across <= (hx + hw) as f32
            && up >= hy as f32
            && up <= (hy + hh) as f32
        {
            // Which SLOT it is, so the caller can take that one out and leave the rest.
            let at_metres = (hx as f32 + hw as f32 * 0.5 - span as f32 * 0.5) * ATOM;
            let _ = what;
            return openings.iter().position(|slot| {
                slot.is_some_and(|hole| (hole.at - at_metres).abs() < ATOM * 2.0)
            });
        }
    }
    None
}

pub(crate) fn punch_wall(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
    placed: &Query<(Entity, &Transform, &Placed, &Visibility), Without<Ghost>>,
    aimed: Option<(Entity, Vec3, Vec3)>,
    at: Vec3,
    wide: f32,
    head: f32,
    sill: f32,
    is_door: bool,
    clear: i32,
    hand: &Hand,
    grid: f32,
) -> bool {
    // The wall the cursor's own ray touches wins outright; the search
    // by proximity is the fallback for a blind click.
    let mut best: Option<(Entity, f32, Vec3, f32, f32, Placed)> = None;
    if let Some((touched, point, _)) = aimed
        && let Ok((entity, transform, record, _)) = placed.get(touched)
        && let Some(length) = punchable_length(record)
    {
        let along = Quat::from_rotation_y(record.yaw) * Vec3::X;
        let t = (point - transform.translation).dot(along);
        if t.abs() <= length * 0.5 {
            best = Some((entity, 0.0, along, t, length, record.clone()));
        }
    }
    if best.is_none() {
        for (entity, transform, record, showing) in placed {
            // A wall the cutaway has taken away holds nothing up and
            // catches nothing: what you cannot see, you cannot build on.
            if *showing == Visibility::Hidden {
                continue;
            }
            let Some(length) = punchable_length(record) else {
                continue;
            };
            let along = Quat::from_rotation_y(record.yaw) * Vec3::X;
            let from_centre = at - transform.translation;
            let t = from_centre.dot(along);
            let sideways = (from_centre - along * t).length();
            if sideways > 0.5 || t.abs() > length * 0.5 {
                continue;
            }
            if best.as_ref().is_none_or(|(_, s, ..)| sideways < *s) {
                best = Some((entity, sideways, along, t, length, record.clone()));
            }
        }
    }
    let Some((wall, _, along, t, length, record)) = best else {
        return false;
    };

    // The opening, on the lattice and clamped so it never spills past the
    // wall's ends. The ghost seats itself with this same arithmetic.
    let half = length * 0.5;
    let wall_at = placed
        .get(wall)
        .map(|(_, tf, _, _)| tf.translation)
        .unwrap_or(at);
    let middle = opening_seat(wall_at, along, length, wide, wall_at + along * t, grid);

    // A WALL IS NOT CUT. It is told - framed or plain, now that they are one part.
    //
    // Everything below this parts a wall into the pieces an opening leaves: a jamb strip
    // either side, a header over, a sill under. That was the only way while a plain wall
    // was a single box, and it made a window in a plain wall a different thing from a
    // window in a framed one - a different width, and no glass in it. A wall knows what an
    // opening is now, whichever it is: it frames one, declines to fill it, and draws what
    // is left either side. So the wall gains the opening where it was aimed and re-solves,
    // and what is below is left for the parts that are genuinely not walls.
    let reframed = if let Some(PartKind::Wall {
        framed,
        long,
        high,
        mut openings,
    }) = kind_from_name(&record.part)
    {
        let what = if is_door {
            Opening::Door
        } else {
            Opening::Window
        };
        // WHERE UP THE WALL IT WAS AIMED, at the size that wall's own courses
        // give it: a window slides up and down and keeps its size, rather than
        // growing to fill wherever it has been put.
        //
        // Nothing at all when it landed where its kind would have put it anyway,
        // and nothing for a door, which reaches the floor by definition. That is
        // what keeps a wall punched the ordinary way spelled the ordinary way.
        let tall = (high / ATOM).round().max((PLATE_TALL * 3 + 8) as f32) as i32;
        let usual = band_of(what, tall);
        let band = aimed
            // The aim on THIS wall, not on whatever else the cursor found: the
            // wall may have been chosen by nearness after a blind click, and a
            // height read off some other thing's face is a height nobody meant.
            .filter(|(touched, _, _)| *touched == wall && what == Opening::Window)
            .map(|(_, point, _)| Band {
                foot: opening_lift(wall_at.y, tall, usual, point.y, grid),
                rise: usual.rise,
            })
            .filter(|band| *band != usual);
        let want = Some(Hole {
            what,
            at: on_the_lattice(middle),
            // The width the OPENING needs, carried in from the one table rather
            // than implied by its kind. This is the whole of the fix: a double
            // door frames a hole its leaves fit through.
            wide: clear,
            // Timber until somebody says otherwise; the right-click menu blackens them.
            dark: false,
            band,
        });
        // Into the first empty slot, so a second window joins the first rather
        // than replacing it. With every slot taken, the nearest one moves - a
        // wall that quietly ignores the drop would look broken, and a maker who
        // has filled a wall with doors is asking to move one.
        match openings.iter_mut().find(|slot| slot.is_none()) {
            Some(slot) => *slot = want,
            None => {
                let nearest = openings.iter_mut().flatten().min_by(|a, b| {
                    let (a, b) = ((a.at - middle).abs(), (b.at - middle).abs());
                    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
                });
                if let Some(hole) = nearest {
                    *hole = want.expect("just built");
                }
            }
        }
        // It keeps the framing it had. A plain wall punched for a door is a plain wall
        // with a door in it, not a wall that quietly framed itself.
        let made = PartKind::Wall {
            long,
            high,
            framed,
            openings,
        };
        commands.entity(wall).despawn_related::<Children>();
        let mut reframed = record.clone();
        reframed.part = part_name(&made);
        dress_part(
            commands, meshes, materials, palette, &made, &reframed, wall, false,
        );
        commands.entity(wall).insert(reframed);
        true
    } else {
        false
    };
    let centre_of = |offset: f32| {
        let base = placed
            .get(wall)
            .map(|(_, tf, _, _)| tf.translation)
            .unwrap_or(at);
        base + along * offset
    };

    let mut leavings: Vec<(PartKind, Vec3)> = Vec::new();
    let left = middle - wide * 0.5 + half;
    if left > 0.06 {
        leavings.push((
            PartKind::Seg {
                long: left,
                high: WALL_HIGH,
                lift: 0.0,
            },
            centre_of(-half + left * 0.5),
        ));
    }
    let right = half - (middle + wide * 0.5);
    if right > 0.06 {
        leavings.push((
            PartKind::Seg {
                long: right,
                high: WALL_HIGH,
                lift: 0.0,
            },
            centre_of(half - right * 0.5),
        ));
    }
    if WALL_HIGH - head > 0.06 {
        leavings.push((
            PartKind::Seg {
                long: wide,
                high: WALL_HIGH - head,
                lift: head,
            },
            centre_of(middle),
        ));
    }
    if sill > 0.06 {
        leavings.push((
            PartKind::Seg {
                long: wide,
                high: sill,
                lift: 0.0,
            },
            centre_of(middle),
        ));
    }

    let base = placed
        .get(wall)
        .map(|(_, tf, _, _)| tf.translation)
        .unwrap_or(at);
    // A framed wall reframed itself above and is still standing; only a plain
    // one is taken away and replaced by the pieces its opening leaves.
    if !reframed {
        commands.entity(wall).despawn();
    }
    for (kind, spot) in leavings.into_iter().filter(|_| !reframed) {
        let piece = Placed {
            part: part_name(&kind),
            at: spot.into(),
            yaw: record.yaw,
            tilt: 0.0,
            ramp: record.ramp.clone(),
            shade: record.shade,
            stage: record.stage.clone(),
            flip: false,
            loose: false,
            material: String::new(),
            group: None,
        };
        spawn_part(commands, meshes, materials, palette, &kind, &piece, false);
    }

    // The frame takes the wall's own line and turn.
    // The hand knows which opening it holds; `is_door` only decides
    // whether a routing widget rides along.
    // A WINDOW THE WALL HAS ALREADY DRAWN is not placed a second time. The wall frames
    // its own opening now - jambs, lintel, sill and panes - so setting the window part
    // down as well stood one window inside another, a hand's breadth apart.
    //
    // A DOOR still hangs its leaf: the wall makes the opening and the leaf is the thing
    // that swings in it, which no wall has ever drawn.
    if reframed && !is_door {
        return true;
    }
    let frame_kind = hand.kind.unwrap_or(PartKind::Prop("window"));
    // The frame keeps the wall's own footing - a door in a wall on a
    // foundation stands on the foundation, not sunk to the ground.
    let frame_at = base + along * middle;
    let frame = Placed {
        part: part_name(&frame_kind),
        at: [frame_at.x, base.y, frame_at.z],
        yaw: record.yaw,
        tilt: 0.0,
        ramp: hand.ramp.clone(),
        shade: hand.shade,
        stage: "walls".to_string(),
        flip: hand.flip,
        loose: false,
        material: String::new(),
        group: None,
    };
    // The frame prop IS a frame - jambs, sill, lintel and the bars across -
    // which is exactly what a framed wall already gathered around the opening
    // when it reframed itself. Set down there it draws a second frame inside
    // the first, a few atoms off, and the pair read as a mistake with a shadow
    // in it. So on a plain wall the prop supplies the frame; on a framed wall
    // the wall does, and nothing is set down.
    //
    // The widget below is not geometry and arrives either way: it is how the
    // village knows there is a door here to walk through.
    // On a framed wall, only what the wall does NOT provide: the leaf and its
    // latch. A doorway that was never going to have a door in it leaves
    // nothing at all, because the wall has already drawn the whole of it.
    let hung = match (reframed, frame_kind) {
        (false, _) => Some(frame_kind),
        (true, PartKind::Prop("door")) => Some(PartKind::Prop("door-leaf")),
        (true, PartKind::Prop("door-double")) => Some(PartKind::Prop("door-double-leaf")),
        (true, _) => None,
    };
    if let Some(hung) = hung {
        let mut leaf = frame.clone();
        leaf.part = part_name(&hung);
        spawn_part(commands, meshes, materials, palette, &hung, &leaf, false);
    }

    // A door is a doorway: the routing widget arrives with it, its nose
    // pointing OUT through the opening - the way you were looking when
    // you punched it, since that is the side you were standing on.
    if is_door {
        let widget = PartKind::Widget("door");
        let outward = aimed
            .map(|(_, _, normal)| Vec3::new(normal.x, 0.0, normal.z))
            .filter(|flat| flat.length() > 0.1)
            .map(|flat| flat.normalize())
            .unwrap_or_else(|| Vec3::Y.cross(along).normalize_or_zero());
        // A widget's nose is its local +X.
        let facing = (-outward.z).atan2(outward.x);
        // One per leaf: a double door gets two, so two people can use it at
        // once. See [`door_lanes`].
        for lane in door_lanes(&frame_kind) {
            let stands = frame_at + along * *lane;
            let mark = Placed {
                part: part_name(&widget),
                at: [stands.x, base.y, stands.z],
                yaw: facing,
                tilt: 0.0,
                ramp: None,
                shade: 0.7,
                stage: "widget".to_string(),
                flip: false,
                loose: false,
                material: String::new(),
                group: None,
            };
            spawn_part(commands, meshes, materials, palette, &widget, &mark, false);
        }
    }
    true
}

// ---------------------------------------------------------------- the file
