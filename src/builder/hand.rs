//! The hand: what it holds, where the ground is, and how a part finds its place.

use super::*;

pub(crate) fn steer_hand(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    palette: Res<Palette>,
    naming: Res<Naming>,
    mut hand: ResMut<Hand>,
    ghosts: Query<Entity, With<Ghost>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if hand.kind.is_none() || naming.0.is_some() {
        return;
    }
    // Backspace as well, which is what the key marked "delete" reports on a
    // Mac. Without it, the one key a maker would try to empty their hand with
    // did nothing at all, and escape was the only way - which is exactly the
    // roundabout Brett described.
    if keys.just_pressed(KeyCode::Escape)
        || keys.just_pressed(KeyCode::Delete)
        || keys.just_pressed(KeyCode::Backspace)
    {
        if hand.anchor.is_some() {
            hand.anchor = None;
        } else {
            *hand = Hand::default();
            for ghost in &ghosts {
                commands.entity(ghost).despawn();
            }
        }
        return;
    }
    let mut redress = false;
    if keys.just_pressed(KeyCode::KeyR) {
        hand.yaw += std::f32::consts::FRAC_PI_2;
    }
    if keys.just_pressed(KeyCode::KeyT) {
        // A whole turn, not a quarter. Stopping at ninety meant the tilt
        // wrapped back to flat just as it reached upright, so nothing
        // could ever be stood on its end - and shift walks it back, since
        // twenty-three presses to undo one is not a control.
        let step = if held_shift(&keys) {
            -15f32.to_radians()
        } else {
            15f32.to_radians()
        };
        hand.tilt = (hand.tilt + step).rem_euclid(std::f32::consts::TAU);
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        hand.lift = (hand.lift + 0.25).min(8.0);
    }
    if keys.just_pressed(KeyCode::KeyE) {
        hand.lift = (hand.lift - 0.25).max(0.0);
    }
    let ramps: Vec<&str> = palette.names().collect();
    if keys.just_pressed(KeyCode::BracketRight) && !ramps.is_empty() {
        let here = hand
            .ramp
            .as_deref()
            .and_then(|r| ramps.iter().position(|n| *n == r))
            .unwrap_or(0);
        hand.ramp = Some(ramps[(here + 1) % ramps.len()].to_string());
        redress = true;
    }
    if keys.just_pressed(KeyCode::BracketLeft) && !ramps.is_empty() {
        let here = hand
            .ramp
            .as_deref()
            .and_then(|r| ramps.iter().position(|n| *n == r))
            .unwrap_or(0);
        hand.ramp = Some(ramps[(here + ramps.len() - 1) % ramps.len()].to_string());
        redress = true;
    }
    if keys.just_pressed(KeyCode::Minus) {
        hand.shade = (hand.shade - 0.25).max(0.0);
        redress = true;
    }
    if keys.just_pressed(KeyCode::Equal) {
        hand.shade = (hand.shade + 0.25).min(1.0);
        redress = true;
    }
    if redress {
        dress_ghost(
            &mut commands,
            &mut meshes,
            &mut materials,
            &palette,
            &hand,
            &ghosts,
        );
    }
}

/// Where the cursor's ray meets the working plane (the grid, lifted).
pub(crate) fn cursor_point(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    lift: f32,
) -> Option<Vec3> {
    let window = windows.iter().next()?;
    let cursor = window.cursor_position()?;
    let (camera, camera_at) = cameras.iter().next()?;
    let ray = camera.viewport_to_world(camera_at, cursor).ok()?;
    let reach = ray.intersect_plane(Vec3::Y * lift, InfinitePlane3d::new(Vec3::Y))?;
    Some(ray.get_point(reach))
}

/// How far a part reaches along its own X and Z, measured from the boxes it is
/// actually made of.
///
/// Which is not always the number the handle is pulling. A chimney's handle
/// asks for its DROP and a flight's asks for its RUN, and a run only comes in
/// whole treads - so the number asked for and the part that answers can differ,
/// and a mover that trusted the asking slid the part sideways while the geometry
/// stood still in jumps. Brett: "when you resize the stai height right now they
/// slide to the side while resizing...the chiney does it as well."
pub fn extent_of(kind: &PartKind) -> Vec2 {
    let mut low = Vec3::splat(f32::INFINITY);
    let mut high = Vec3::splat(f32::NEG_INFINITY);
    for Slab {
        at,
        size,
        lean,
        cant,
        ..
    } in body_of(kind, None)
    {
        // A leaning slab reaches further across than it is thick and less far
        // along than it is long. Measuring its unturned box makes a stair rail
        // - which is longer than the flight it runs beside - the widest thing
        // about the flight, and everything downstream believes it.
        let turn = Quat::from_rotation_z(cant) * Quat::from_rotation_x(lean);
        let half = size * 0.5;
        let reach = (turn * Vec3::new(half.x, 0.0, 0.0)).abs()
            + (turn * Vec3::new(0.0, half.y, 0.0)).abs()
            + (turn * Vec3::new(0.0, 0.0, half.z)).abs();
        low = low.min(at - reach);
        high = high.max(at + reach);
    }
    if !low.x.is_finite() {
        return Vec2::ZERO;
    }
    Vec2::new(high.x - low.x, high.z - low.z)
}

/// How a flight divides: how many treads, how tall each riser, how deep each
/// tread.
///
/// A stair is a rhythm rather than a size. Asked for a height, it takes the
/// number of EVEN steps that comes nearest - uneven steps are the one thing a
/// foot notices - and the run follows from the count.
pub fn stair_rhythm(rise: f32) -> (i32, f32, f32) {
    let riser = ATOM * 3.0;
    // Six atoms of tread to three of rise: about twenty-seven degrees, which is
    // a stair somebody could carry a sack up. Four atoms made a ladder at
    // forty-nine, and it left no room between the newels for the rail to be
    // anything but a stub - Brett: "we can make the stair treads deeper if that
    // helps." It does, and it looks like a stair.
    let tread = ATOM * 6.0;
    (((rise / riser).round() as i32).clamp(2, 24), riser, tread)
}

/// Whether this kind counts as structure - walls, their leavings, floors,
/// roofs, foundations and steps. Structure rests only on structure; props
/// rest on anything.
pub(crate) fn is_structure(kind: &PartKind) -> bool {
    matches!(
        kind,
        PartKind::Wall { long: _, .. }
            | PartKind::Seg { .. }
            | PartKind::Floor(..)
            | PartKind::FloorRun
            | PartKind::Foundation(..)
            | PartKind::FoundationRun
            | PartKind::Roof(..)
            | PartKind::RoofRun
            | PartKind::Trim { .. }
            | PartKind::TrimRun { .. }
            | PartKind::SegRun { .. }
            | PartKind::Gable(..)
            | PartKind::GableRun
            | PartKind::Ridge(..)
            | PartKind::Chimney(..)
            | PartKind::RidgeRun
            | PartKind::GableRoof(..)
            | PartKind::HipRoof(..)
            | PartKind::GableRoofRun
            | PartKind::Stairs { .. }
            | PartKind::Rail { .. }
            | PartKind::Prop("steps")
            | PartKind::Prop("pole")
    )
}

/// The carried part's footprint, spoken as sample points: its centre and
/// four corners, drawn in slightly so edge-kisses do not flicker.
pub(crate) fn footprint_samples(kind: &PartKind, at: Vec3, yaw: f32) -> Vec<Vec3> {
    let mut low = Vec3::splat(f32::INFINITY);
    let mut high = Vec3::splat(f32::NEG_INFINITY);
    for Slab {
        at: slab_at, size, ..
    } in body_of(kind, None)
    {
        low = low.min(slab_at - size * 0.5);
        high = high.max(slab_at + size * 0.5);
    }
    if !low.x.is_finite() {
        return vec![at];
    }
    let spin = Quat::from_rotation_y(yaw);
    let mut samples = vec![at];
    for (cx, cz) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let corner = Vec3::new(
            if cx < 0.0 { low.x } else { high.x } * 0.9,
            0.0,
            if cz < 0.0 { low.z } else { high.z } * 0.9,
        );
        samples.push(at + spin * corner);
    }
    samples
}

/// The height of whatever stands beneath a point: the highest slab top
/// whose footprint holds it. Widgets hold nothing up; structure is picky
/// about what it stands on.
pub(crate) fn support_height(
    placed: &Query<(Entity, &Transform, &Placed, &Visibility), Without<Ghost>>,
    samples: &[Vec3],
    carrying_structure: bool,
    except: Option<Entity>,
) -> f32 {
    // What holds up each sample of the footprint, kept apart rather than merged
    // into one highest-anywhere answer.
    //
    // The highest was wrong the moment a part came NEAR something tall: one
    // corner brushing a wall lifted the whole thing onto the wall, which is
    // exactly what a maker sees as "it jumps to being on top of the wall" when
    // they were only trying to set it against one. What holds a part up is what
    // most of it is standing on.
    let mut under = vec![0.0f32; samples.len()];
    for (entity, transform, record, showing) in placed {
        // A wall the cutaway has taken away holds nothing up and
        // catches nothing: what you cannot see, you cannot build on.
        if *showing == Visibility::Hidden {
            continue;
        }
        if Some(entity) == except {
            continue;
        }
        let Some(kind) = kind_from_name(&record.part) else {
            continue;
        };
        if matches!(kind, PartKind::Widget(_)) {
            continue;
        }
        if carrying_structure && !is_structure(&kind) {
            continue;
        }
        let turn = pose(record.yaw, record.tilt, record.flip);
        let repaint = record.ramp.as_deref().map(|r| (r, record.shade));
        for Slab { mut at, size, .. } in body_of(&kind, repaint) {
            if record.flip {
                at.x = -at.x;
            }
            let face_y = at.y + size.y * 0.5;
            for (which, sample) in samples.iter().enumerate() {
                // Where the sample's own column meets this piece's top
                // face. The turn carries (lx, face_y, lz) into the world,
                // so its x and z rows are two equations in lx and lz -
                // which is how a ridge finds the top of a SLOPING roof
                // instead of sliding off to the wall beneath it.
                let base = Vec3::new(sample.x, 0.0, sample.z) - transform.translation;
                let cx = turn * Vec3::X;
                let cy = turn * Vec3::Y;
                let cz = turn * Vec3::Z;
                let det = cx.x * cz.z - cz.x * cx.z;
                if det.abs() < 1e-5 {
                    continue;
                }
                let rx = base.x - cy.x * face_y;
                let rz = base.z - cy.z * face_y;
                let lx = (rx * cz.z - cz.x * rz) / det;
                let lz = (cx.x * rz - rx * cx.z) / det;
                if (lx - at.x).abs() <= size.x * 0.5 && (lz - at.z).abs() <= size.z * 0.5 {
                    let world_y = transform.translation.y + (turn * Vec3::new(lx, face_y, lz)).y;
                    // Every sample answered, not the first one that hits: the
                    // vote below needs all of them, and the old `break` left
                    // four of the five unasked.
                    under[which] = under[which].max(world_y);
                }
            }
        }
    }
    seated_at(&under)
}

/// The height most of a footprint agrees on.
///
/// Quantised to a sixty-fourth first, because two samples on one floor can
/// differ in the last bit of a float and would otherwise count as two opinions.
/// Most samples wins, and the LOWER of two equal counts - so a part half over a
/// wall settles beside it rather than climbing it, which is the whole point.
pub fn seated_at(under: &[f32]) -> f32 {
    let mut votes: Vec<(i64, f32, usize)> = Vec::new();
    for height in under {
        let key = (height * 64.0).round() as i64;
        match votes.iter_mut().find(|(had, ..)| *had == key) {
            Some((_, _, count)) => *count += 1,
            None => votes.push((key, *height, 1)),
        }
    }
    votes
        .into_iter()
        .max_by(|a, b| a.2.cmp(&b.2).then(b.1.total_cmp(&a.1)))
        .map(|(_, height, _)| height)
        .unwrap_or(0.0)
}

/// A platform's top rectangle: foundations and floors, the things walls
/// stand on and line up against.
pub(crate) struct PlatformRect {
    at: Vec3,
    yaw: f32,
    half: Vec2,
}

pub(crate) fn platform_rects(
    placed: &Query<(Entity, &Transform, &Placed, &Visibility), Without<Ghost>>,
) -> Vec<PlatformRect> {
    let mut rects = Vec::new();
    for (_, transform, record, showing) in placed {
        if *showing == Visibility::Hidden {
            continue;
        }
        let Some(kind) = kind_from_name(&record.part) else {
            continue;
        };
        if !matches!(kind, PartKind::Floor(..) | PartKind::Foundation(..)) {
            continue;
        }
        let mut low = Vec3::splat(f32::INFINITY);
        let mut high = Vec3::splat(f32::NEG_INFINITY);
        for Slab { at, size, .. } in body_of(&kind, None) {
            low = low.min(at - size * 0.5);
            high = high.max(at + size * 0.5);
        }
        rects.push(PlatformRect {
            at: transform.translation,
            yaw: record.yaw,
            half: Vec2::new((high.x - low.x) * 0.5, (high.z - low.z) * 0.5),
        });
    }
    rects
}

/// Wall centrelines sit half a wall inside the platform edge, which
/// puts the timber's OUTER FACE flush with the stone's: fully seated,
/// no gap, no overhang. Corners still meet cleanly because platform
/// corners pull walls only along their own line - the flush snap owns
/// the sideways part - and the pole caps the centreline crossing.
pub(crate) const PLINTH_REVEAL: f32 = WALL_THICK * 0.5;

/// The ends of every standing full-height wall piece, for the magnets.
/// Every standing wall end, with the direction it points out of its own
/// wall - the joint math needs to know which way a tip faces.
pub(crate) fn wall_ends(
    placed: &Query<(Entity, &Transform, &Placed, &Visibility), Without<Ghost>>,
) -> Vec<(Vec3, Vec3)> {
    let mut ends = Vec::new();
    for (_, transform, record, showing) in placed {
        if *showing == Visibility::Hidden {
            continue;
        }
        let long = match kind_from_name(&record.part) {
            Some(PartKind::Wall { long, .. }) => long,
            Some(PartKind::Seg { long, lift, .. }) if lift == 0.0 => long,
            _ => continue,
        };
        let along = Quat::from_rotation_y(record.yaw) * Vec3::X;
        ends.push((transform.translation + along * (long * 0.5), along));
        ends.push((transform.translation - along * (long * 0.5), -along));
    }
    ends
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn move_ghost(
    mut commands: Commands,
    bench: Res<Bench>,
    hand: Res<Hand>,
    mode: Res<SnapMode>,
    snap_grid: Res<SnapGrid>,
    hovered: Res<Hovered>,
    selected: Res<crate::gizmo::Selected>,
    mut ghost_shapes: Query<&mut Visibility, With<Ghost>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    placed: Query<(Entity, &Transform, &Placed, &Visibility), Without<Ghost>>,
    mut ghosts: Query<(Entity, &mut Transform, &Placed), With<Ghost>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<Palette>,
) {
    if *bench != Bench::Builder {
        return;
    }
    // While the arrows are out, the ghost stands aside entirely - fine
    // tuning wants a clear view and no accidental placements.
    let tuning = !selected.is_empty();
    for mut visibility in &mut ghost_shapes {
        let wanted = if tuning {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
    if tuning {
        return;
    }
    let Some(kind_now) = hand.kind else {
        return;
    };

    // A stretch tool with its anchor down draws itself from the anchor
    // to the cursor and listens to nothing else.
    if let Some(axes) = kind_now.run_axes()
        && let Some(anchor) = hand.anchor
    {
        let Some(point) = cursor_point(&windows, &cameras, anchor.y) else {
            return;
        };
        let grid = snap_step(held_shift(&keys), snap_grid.0);
        let mut to = Vec3::new(
            (point.x * grid).round() / grid,
            anchor.y,
            (point.z * grid).round() / grid,
        );
        // The drawn end answers the same magnets as any wall end: joint
        // crossings, wall ends and platform corners pull it off the
        // plain grid, so a stretched wall can actually MEET a seated
        // one instead of stopping a half-thickness short.
        let half_thick = WALL_THICK * 0.5;
        let mut stops: Vec<Vec3> = Vec::new();
        for (end, out) in wall_ends(&placed) {
            stops.push(end);
            stops.push(end - out * half_thick);
        }
        for platform in platform_rects(&placed) {
            let spin = Quat::from_rotation_y(platform.yaw);
            for (sx, sz) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                stops.push(
                    platform.at + spin * Vec3::new(sx * platform.half.x, 0.0, sz * platform.half.y),
                );
            }
        }
        let mut best: Option<(f32, Vec3)> = None;
        for stop in stops {
            let gap = Vec2::new(stop.x - to.x, stop.z - to.z).length();
            if gap < 0.4 && best.as_ref().is_none_or(|(b, _)| gap < *b) {
                best = Some((gap, stop));
            }
        }
        if let Some((_, stop)) = best {
            to.x = stop.x;
            to.z = stop.z;
        }
        let reach = to - anchor;
        let (made, centre, yaw) = if axes == 1 {
            let on_x = reach.x.abs() >= reach.z.abs();
            let signed = if on_x { reach.x } else { reach.z };
            let long = signed.abs().max(0.25);
            let dir = if on_x {
                Vec3::X * signed.signum()
            } else {
                Vec3::Z * signed.signum()
            };
            (
                kind_now.run_made(long, 0.0),
                anchor + dir * (long * 0.5),
                if on_x {
                    0.0
                } else {
                    std::f32::consts::FRAC_PI_2
                },
            )
        } else {
            let w = reach.x.abs().max(0.25);
            let d = reach.z.abs().max(0.25);
            // R turns a whole roof a quarter, so the ridge can run the
            // other way over the same rectangle: the part is laid
            // crosswise and its two sides swap.
            let crossed = hand.yaw.rem_euclid(std::f32::consts::PI) > 0.7;
            let made = if crossed {
                kind_now.run_made(d, w)
            } else {
                kind_now.run_made(w, d)
            };
            (
                made,
                anchor + Vec3::new(w * 0.5 * reach.x.signum(), 0.0, d * 0.5 * reach.z.signum()),
                if crossed {
                    std::f32::consts::FRAC_PI_2
                } else {
                    0.0
                },
            )
        };
        let record = Placed {
            part: part_name(&made),
            at: centre.into(),
            yaw,
            tilt: 0.0,
            ramp: hand.ramp.clone(),
            shade: hand.shade,
            stage: hand.stage.clone(),
            flip: hand.flip,
            loose: false,
            group: None,
        };
        // A whole roof draws as a flat plane while it is being sized -
        // the footprint it will cover - and becomes a roof when the
        // second click lands. Far easier to judge than two slopes
        // swinging about in the air.
        let shown = match made {
            PartKind::GableRoof(w, d, _, _) => PartKind::RoofPlan(w, d),
            other => other,
        };
        // Redraw only when the drawn size changed; otherwise carry the
        // ghost along.
        let stale = ghosts
            .iter()
            .next()
            .map(|(_, _, held)| held.part != record.part)
            .unwrap_or(true);
        if stale {
            for (ghost, _, _) in &ghosts {
                commands.entity(ghost).despawn();
            }
            spawn_part(
                &mut commands,
                &mut meshes,
                &mut materials,
                &palette,
                &shown,
                &record,
                true,
            );
        } else {
            for (_, mut transform, _) in &mut ghosts {
                transform.translation = centre;
                transform.rotation = Quat::from_rotation_y(yaw);
            }
        }
        return;
    }

    // Face-aware placement: the part clings to the face the cursor
    // points at. A side is clung to flush at the aimed course and is
    // final; the top of a thing seeds the position and then passes
    // through the magnets like any other placement, so a wall set down
    // on a foundation's top still seats flush to its edges.
    // A door or a window does not stand ON a wall, it stands IN one - and
    // it belongs to walls alone. Shown any other way the ghost clings to
    // whatever the cursor finds, roofs included, and leaps a metre the
    // instant the aim slips off the timber. It seats itself here with the
    // punch's own arithmetic, so what you see is where it goes.
    if let Some((wide, ..)) = opening_of(&kind_now) {
        let seat = hovered
            .build
            .filter(|hit| hit.normal.y.abs() < 0.3)
            .and_then(|hit| {
                let (_, wall_at, record, _) = placed.get(hit.entity).ok()?;
                let length = punchable_length(record)?;
                let along = Quat::from_rotation_y(record.yaw) * Vec3::X;
                let step = snap_step(held_shift(&keys), snap_grid.0);
                let middle =
                    opening_seat(wall_at.translation, along, length, wide, hit.point, step);
                Some((wall_at.translation + along * middle, record.yaw))
            });
        // Nothing punchable under the cursor: hold still. A ghost that
        // jumps to the ground whenever the aim wanders is worse than one
        // that waits where it was last wanted.
        let Some((seat, wall_yaw)) = seat else {
            return;
        };
        for (_, mut transform, _) in &mut ghosts {
            transform.translation = seat;
            transform.rotation = pose(wall_yaw, hand.tilt, hand.flip);
        }
        return;
    }

    let mut seeded: Option<Vec3> = None;
    if mode.face
        && let Some(hit) = hovered.build
    {
        if hit.normal.y > 0.7 {
            let per = 16.0 / snap_grid.0 as f32;
            seeded = Some(Vec3::new(
                (hit.point.x * per).round() / per,
                0.0,
                (hit.point.z * per).round() / per,
            ));
        } else if hit.normal.y.abs() < 0.3 {
            // My reach along the face's normal: how far my centre must
            // stand off so my body kisses the face.
            let mut low = Vec3::splat(f32::INFINITY);
            let mut high = Vec3::splat(f32::NEG_INFINITY);
            for Slab { at, size, .. } in body_of(&kind_now, None) {
                low = low.min(at - size * 0.5);
                high = high.max(at + size * 0.5);
            }
            let spin = Quat::from_rotation_y(hand.yaw);
            let mut reach = 0.0f32;
            for (cx, cz) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                let corner = spin
                    * Vec3::new(
                        if cx < 0.0 { low.x } else { high.x },
                        0.0,
                        if cz < 0.0 { low.z } else { high.z },
                    );
                reach = reach.max(corner.dot(hit.normal).abs());
            }
            // Along the face: quarter-metre order. Up the face: courses
            // measured from the part's own base, so trim stacks in rings.
            let per = 16.0 / snap_grid.0 as f32;
            let tangent = Vec3::Y.cross(hit.normal).normalize_or_zero();
            let along = (hit.point.dot(tangent) * per).round() / per;
            let course = ((hit.point.y - hit.base_y).max(0.0) * per).round() / per + hit.base_y;
            let anchor = hit.point - tangent * hit.point.dot(tangent) + tangent * along;
            let snapped = Vec3::new(
                (anchor + hit.normal * reach).x,
                course + hand.lift,
                (anchor + hit.normal * reach).z,
            );
            for (_, mut transform, _) in &mut ghosts {
                transform.translation = snapped;
                transform.rotation = pose(hand.yaw, hand.tilt, hand.flip);
            }
            return;
        }
    }

    let mut snapped = match seeded {
        Some(seed) => seed,
        None => {
            let Some(point) = cursor_point(&windows, &cameras, hand.lift) else {
                return;
            };
            Vec3::ZERO + point
        }
    };
    // Quarter-metre snap by default; holding shift tightens the grid to
    // five centimetres for the odd exact nestling.
    if seeded.is_none() {
        let grid = snap_step(held_shift(&keys), snap_grid.0);
        snapped = Vec3::new(
            (snapped.x * grid).round() / grid,
            0.0,
            (snapped.z * grid).round() / grid,
        );
    }

    let kind = kind_now;

    // Walls click to wall ends - a butt joint or a square corner - and
    // the corner pole magnetizes to the same points it exists to cover.
    let magnetic = matches!(kind, PartKind::Wall { long: _, .. })
        || kind == PartKind::Prop("pole")
        || kind.run_axes() == Some(1);
    if magnetic {
        let mut ends = wall_ends(&placed);
        let platforms = platform_rects(&placed);
        // The pole magnetizes to centreline crossings at platform corners
        // - the exact point two flush walls meet - alongside wall ends.
        if kind == PartKind::Prop("pole") || kind.run_axes() == Some(1) {
            for platform in &platforms {
                let spin = Quat::from_rotation_y(platform.yaw);
                for (sx, sz) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                    ends.push((
                        platform.at
                            + spin
                                * Vec3::new(
                                    sx * (platform.half.x - PLINTH_REVEAL),
                                    0.0,
                                    sz * (platform.half.y - PLINTH_REVEAL),
                                ),
                        Vec3::ZERO,
                    ));
                }
            }
        }
        let my_dir = Quat::from_rotation_y(hand.yaw) * Vec3::X;
        let my_ends: Vec<(Vec3, Vec3)> = match kind {
            PartKind::Wall { long, .. } => {
                vec![
                    (snapped + my_dir * (long * 0.5), my_dir),
                    (snapped - my_dir * (long * 0.5), -my_dir),
                ]
            }
            _ => vec![(snapped, Vec3::ZERO)],
        };
        let half_thick = WALL_THICK * 0.5;
        let mut pull: Option<(f32, Vec3)> = None;
        for (mine, my_out) in &my_ends {
            for (theirs, their_out) in &ends {
                // The joint decides the target. Perpendicular tips overlap
                // into a full corner block, outer faces flush both ways; a
                // continuation meets end to end; a pole takes the
                // centreline crossing itself.
                let target = if *my_out == Vec3::ZERO {
                    *theirs - *their_out * half_thick
                } else if my_out.dot(*their_out).abs() < 0.35 {
                    *theirs - *their_out * half_thick + *my_out * half_thick
                } else {
                    *theirs
                };
                let gap = Vec3::new(target.x - mine.x, 0.0, target.z - mine.z);
                let reach = gap.length();
                if reach < 0.4 && pull.as_ref().is_none_or(|(best, _)| reach < *best) {
                    pull = Some((reach, gap));
                }
            }
        }
        if let Some((_, gap)) = pull {
            snapped += gap;
        } else if let PartKind::Wall { long: my_len, .. } = kind {
            // No wall end took hold. A wall running parallel to a platform
            // edge seats flush onto it - outer face to the stone's face -
            // and platform corners then slide it ALONG its line only, so
            // the flush seat is never yanked sideways.
            let mut best: Option<(f32, Vec3)> = None;
            for platform in &platforms {
                let spin = Quat::from_rotation_y(platform.yaw);
                let faces = [
                    (
                        spin * Vec3::X,
                        spin * Vec3::Z,
                        platform.half.y,
                        platform.half.x,
                    ),
                    (
                        spin * Vec3::Z,
                        spin * Vec3::X,
                        platform.half.x,
                        platform.half.y,
                    ),
                ];
                for (along_edge, outward, half_out, half_along) in faces {
                    if my_dir.dot(along_edge).abs() < 0.92 {
                        continue;
                    }
                    for side in [-1.0f32, 1.0] {
                        let line = platform.at + outward * side * (half_out - PLINTH_REVEAL);
                        let offset = snapped - line;
                        let across = offset.dot(outward);
                        let along = offset.dot(along_edge);
                        if across.abs() < 0.45
                            && along.abs() < half_along + 0.3
                            && best.as_ref().is_none_or(|(b, _)| across.abs() < *b)
                        {
                            best = Some((across.abs(), outward * -across));
                        }
                    }
                }
            }
            if let Some((_, shift)) = best {
                snapped += shift;
                // Corner slide: my nearest end walks along my line to the
                // platform corner's projection, and no further than that.
                let mut slide: Option<(f32, f32)> = None;
                for platform in &platforms {
                    let spin = Quat::from_rotation_y(platform.yaw);
                    for (sx, sz) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                        let corner = platform.at
                            + spin * Vec3::new(sx * platform.half.x, 0.0, sz * platform.half.y);
                        for end_sign in [-1.0f32, 1.0] {
                            let my_end = snapped + my_dir * (end_sign * my_len * 0.5);
                            let to_corner = corner - my_end;
                            let along = to_corner.dot(my_dir);
                            let sideways = (to_corner - my_dir * along).length();
                            if along.abs() < 0.4
                                && sideways < 0.45
                                && slide.as_ref().is_none_or(|(b, _)| along.abs() < *b)
                            {
                                slide = Some((along.abs(), along));
                            }
                        }
                    }
                }
                if let Some((_, along)) = slide {
                    snapped += my_dir * along;
                }
            }
        }
    }

    // Whatever lies beneath carries the part; Q and E add height on top
    // of that, so a roof panel rides the wall tops on its own.
    let samples = footprint_samples(&kind, snapped, hand.yaw);
    let support = support_height(&placed, &samples, is_structure(&kind), None);
    // A tilted part rests its DOWNHILL EDGE on what carries it and
    // rises from there - a pitched panel's eave sits on the wall plate
    // instead of half the slope swinging down into the room.
    let mut eave = 0.0;
    if hand.tilt.abs() > 0.001 {
        let mut deep = 0.0f32;
        for Slab { at, size, .. } in body_of(&kind, None) {
            deep = deep.max((at.z.abs() + size.z * 0.5) * 2.0);
        }
        eave = deep * 0.5 * hand.tilt.abs().sin();
    }
    snapped.y = support + hand.lift + eave;

    for (_, mut transform, _) in &mut ghosts {
        transform.translation = snapped;
        transform.rotation = pose(hand.yaw, hand.tilt, hand.flip);
    }
}

/// What the cursor's ray touched first: the part, where, and through
/// which face - the face is what placement clings to.
#[derive(Clone, Copy)]
pub struct Hit {
    /// Which part was struck - unread today, but the grab and future
    /// tools (measure) will want to know.
    #[allow(dead_code)]
    pub entity: Entity,
    pub point: Vec3,
    pub normal: Vec3,
    pub base_y: f32,
    /// The colour of the very piece under the cursor: its ramp, and its step.
    ///
    /// The colour SEEN, not the part's own field. Most of what a maker points at
    /// has never been repainted - a framed wall is wood timbers and bone panels,
    /// and its `ramp` is None - so a dropper reading the record would come up empty
    /// on exactly the colours worth copying. `body_of` is handed the repaint before
    /// this is read, so a part that HAS been painted answers with the paint.
    ///
    /// The ramp is KEPT rather than owned, so a `Hit` stays `Copy` and every reader
    /// of one goes on costing nothing. A palette holds two dozen ramp words and they
    /// are interned once each - see [`crate::project::a_kept_word`], which exists
    /// for the same reason a mark's word does.
    pub wearing: (&'static str, f32),
}

/// The cursor's findings, shared by the glow, the grab and the ghost:
/// `grab` is the first thing touched (widgets included), `build` the
/// first solid face a part could cling to.
#[derive(Resource, Default)]
pub struct Hovered {
    pub grab: Option<Entity>,
    pub build: Option<Hit>,
}

/// Whether placement clings to the face under the cursor, or ignores
/// faces and works the ground plane alone. F walks between them.
#[derive(Resource)]
pub struct SnapMode {
    pub face: bool,
}

impl Default for SnapMode {
    fn default() -> Self {
        SnapMode { face: true }
    }
}

/// The placement grid's step, in atoms. G cycles it; shift always
/// drops to a single atom while held.
#[derive(Resource)]
pub struct SnapGrid(pub i32);

impl Default for SnapGrid {
    fn default() -> Self {
        SnapGrid(4)
    }
}

/// The shelf line that says which mode the hand is in.
#[derive(Component)]
pub(crate) struct SnapModeText;

/// Exact dimensions being typed for the selected part, while the card
/// is up. Every other key on the bench holds its tongue.
#[derive(Resource, Default)]
pub struct DimsEntry(pub Option<String>);

/// The dimensions card at the window's foot and the text inside it.
#[derive(Component)]
pub(crate) struct DimsCard;

#[derive(Component)]
pub(crate) struct DimsText;

#[allow(clippy::type_complexity)]
pub(crate) fn ray_scan(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    placed: &Query<(Entity, &Transform, &Placed, &Visibility), Without<Ghost>>,
) -> (Option<Entity>, Option<Hit>) {
    let Some(ray) = windows
        .iter()
        .next()
        .and_then(|window| window.cursor_position())
        .and_then(|cursor| {
            let (camera, camera_at) = cameras.iter().next()?;
            camera.viewport_to_world(camera_at, cursor).ok()
        })
    else {
        return (None, None);
    };

    // First thing touched at all (the grab), and first SOLID face (the
    // build target) - widgets are markers, not masonry.
    let mut first_any: Option<(Entity, f32)> = None;
    let mut first_solid: Option<(f32, Hit)> = None;
    for (entity, transform, record, showing) in placed {
        // A wall the cutaway has taken away holds nothing up and
        // catches nothing: what you cannot see, you cannot build on.
        if *showing == Visibility::Hidden {
            continue;
        }
        let Some(kind) = kind_from_name(&record.part) else {
            continue;
        };
        let spin = Quat::from_rotation_y(record.yaw) * Quat::from_rotation_x(record.tilt);
        let inverse = spin.inverse();
        let origin = inverse * (ray.origin - transform.translation);
        let toward = inverse * Vec3::from(ray.direction);
        let repaint = record.ramp.as_deref().map(|r| (r, record.shade));
        for Slab {
            at,
            size,
            ref ramp,
            shade,
            ..
        } in body_of(&kind, repaint)
        {
            let low = at - size * 0.5;
            let high = at + size * 0.5;
            let mut enter = f32::NEG_INFINITY;
            let mut leave = f32::INFINITY;
            let mut face = Vec3::Y;
            let mut missed = false;
            for axis in 0..3 {
                let (o, d, lo, hi) = (origin[axis], toward[axis], low[axis], high[axis]);
                if d.abs() < 1e-6 {
                    if o < lo || o > hi {
                        missed = true;
                        break;
                    }
                    continue;
                }
                let a = (lo - o) / d;
                let b = (hi - o) / d;
                let near = a.min(b);
                if near > enter {
                    enter = near;
                    let mut normal = Vec3::ZERO;
                    normal[axis] = -toward[axis].signum();
                    face = normal;
                }
                leave = leave.min(a.max(b));
            }
            if missed || enter > leave || leave < 0.0 {
                continue;
            }
            let reach = enter.max(0.0);
            if first_any.is_none_or(|(_, t)| reach < t) {
                first_any = Some((entity, reach));
            }
            if !matches!(kind, PartKind::Widget(_))
                && first_solid.as_ref().is_none_or(|(t, _)| reach < *t)
            {
                first_solid = Some((
                    reach,
                    Hit {
                        entity,
                        point: ray.get_point(reach),
                        normal: (spin * face).normalize_or_zero(),
                        base_y: transform.translation.y,
                        wearing: (crate::project::a_kept_word(ramp), shade),
                    },
                ));
            }
        }
    }
    (
        first_any.map(|(entity, _)| entity),
        first_solid.map(|(_, hit)| hit),
    )
}

/// Keeps the hovered part known, and lights it softly gold while the
/// hand is empty - what glows is what a click will take.
#[allow(clippy::too_many_arguments)]
pub(crate) fn feel_ahead(
    bench: Res<Bench>,
    naming: Res<Naming>,
    mut hovered: ResMut<Hovered>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    placed: Query<(Entity, &Transform, &Placed, &Visibility), Without<Ghost>>,
    hovers: Query<&Interaction>,
) {
    let over_ui = hovers
        .iter()
        .any(|interaction| *interaction != Interaction::None);
    let (fresh, build) = if *bench == Bench::Builder && naming.0.is_none() && !over_ui {
        ray_scan(&windows, &cameras, &placed)
    } else {
        (None, None)
    };
    hovered.build = build;
    if fresh == hovered.grab {
        return;
    }
    hovered.grab = fresh;
}
