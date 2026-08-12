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
            PartKind::Wall(long) => (long, true),
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
    let made = PartKind::Wall(((high - low) * 16.0).round() / 16.0);
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
pub fn opening_of(kind: &PartKind) -> Option<(f32, f32, f32, bool)> {
    match kind {
        PartKind::Prop("door") => Some((1.25, 2.125, 0.0, true)),
        // Twice the leaf, so twice the hole.
        PartKind::Prop("door-double") => Some((2.25, 2.125, 0.0, true)),
        // A bare doorway needs no widget: the gap itself is the portal,
        // and a widget would only say it twice.
        PartKind::Prop("doorway") => Some((1.25, 2.125, 0.0, false)),
        PartKind::Prop("window") => Some((1.25, 2.0, 0.75, false)),
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
pub fn opening_seat(wall_at: Vec3, along: Vec3, length: f32, wide: f32, point: Vec3) -> f32 {
    let half = length * 0.5;
    let reach = (half - wide * 0.5).max(0.0);
    let t = (point - wall_at).dot(along).clamp(-reach, reach);
    ((t * 16.0).round() / 16.0).clamp(-reach, reach)
}

pub(crate) fn punchable_length(record: &Placed) -> Option<f32> {
    match kind_from_name(&record.part)? {
        PartKind::Wall(long) => Some(long),
        // A framed wall takes an opening too - it just does not have to be cut
        // to take one. Being punchable is what lets a window's ghost seat
        // itself along the wall as you aim, which is the same arithmetic
        // whether the wall is going to be parted or is going to reframe itself.
        PartKind::Framed { long, .. } => Some(long),
        PartKind::Seg { long, high, lift }
            if lift.abs() < 0.01 && (high - WALL_HIGH).abs() < 0.05 =>
        {
            Some(long)
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
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
    hand: &Hand,
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
    let middle = opening_seat(wall_at, along, length, wide, wall_at + along * t);

    // A FRAMED wall is not cut. It is told.
    //
    // Everything below this parts a plain wall into the pieces an opening
    // leaves - a jamb strip either side, a header over, a sill under - because
    // a plain wall is a single box and a box cannot have a hole in it. A framed
    // wall already knows what an opening is: it frames one, declines to panel
    // it, and divides the bays either side of it. So the wall simply gains the
    // opening where it was aimed, and re-solves.
    let reframed = if let Some(PartKind::Framed {
        long,
        high,
        mut openings,
    }) = kind_from_name(&record.part)
    {
        let want = Some((
            if is_door {
                Opening::Door
            } else {
                Opening::Window
            },
            on_the_lattice(middle),
        ));
        // Into the first empty slot, so a second window joins the first rather
        // than replacing it. With every slot taken, the nearest one moves - a
        // wall that quietly ignores the drop would look broken, and a maker who
        // has filled a wall with doors is asking to move one.
        match openings.iter_mut().find(|slot| slot.is_none()) {
            Some(slot) => *slot = want,
            None => {
                let nearest = openings.iter_mut().flatten().min_by(|(_, a), (_, b)| {
                    let (a, b) = ((a - middle).abs(), (b - middle).abs());
                    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
                });
                if let Some(hole) = nearest {
                    *hole = want.expect("just built");
                }
            }
        }
        let made = PartKind::Framed {
            long,
            high,
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
            group: None,
        };
        spawn_part(commands, meshes, materials, palette, &kind, &piece, false);
    }

    // The frame takes the wall's own line and turn.
    // The hand knows which opening it holds; `is_door` only decides
    // whether a routing widget rides along.
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
                group: None,
            };
            spawn_part(commands, meshes, materials, palette, &widget, &mark, false);
        }
    }
    true
}

// ---------------------------------------------------------------- the file
