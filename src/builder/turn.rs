//! Turning, tilting and mirroring what stands.

use super::*;

/// SHIFT: more of the same.
///
/// Gather another part into what is chosen, paint the whole of something rather than the
/// piece under the cursor, keep the tool in hand after setting one down - and, on a key
/// rather than a click, step the other way. Every one of those is "and again" or "and the
/// rest", which is what shift means in a file manager, a drawing program and a builder
/// alike.
pub(crate) fn held_shift(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
}

/// ALT: the other way of doing it.
///
/// Take the colour under the cursor instead of laying one down; place on the bench's own
/// sixteenths instead of the grid the G key set. Photoshop's eyedropper and Blender's
/// precision drag are both this key, and both are "not the ordinary action, the other one".
///
/// It carries the fine snap because SHIFT could not: shift is read at the very moment a
/// part is set down, so one key for both would have meant never placing several parts on
/// the coarse grid, nor one part on the fine one.
pub(crate) fn held_fine(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
}

/// Shift-R turns the WHOLE work a quarter, about its own middle.
///
/// A house is drawn facing whichever way the maker happened to start, and
/// finding out at the end that it wants to face the other way should not
/// mean rebuilding it. The middle is snapped to the lattice before
/// anything turns, so a quarter turn about it lands every part back on
/// the lattice exactly - no drift, however many times it is spun.
pub(crate) fn turn_the_work(
    keys: Res<ButtonInput<KeyCode>>,
    bench: Res<Bench>,
    naming: Res<Naming>,
    dims: Res<DimsEntry>,
    hand: Res<Hand>,
    mut parts: Query<(&mut Transform, &mut Placed), Without<Ghost>>,
) {
    // NOT WHILE A HAND IS FULL. R belongs to what you are holding, and shift is what
    // you hold to keep placing - so a maker turning a row of books to face the other
    // way, with shift down to set down another, spun the entire village instead.
    // Brett: "When holding books to place and pressing R the entire build rotates."
    if *bench != Bench::Builder
        || naming.0.is_some()
        || dims.0.is_some()
        || hand.kind.is_some()
        || !keys.just_pressed(KeyCode::KeyR)
        || !held_shift(&keys)
    {
        return;
    }
    let mut low = Vec3::splat(f32::INFINITY);
    let mut high = Vec3::splat(f32::NEG_INFINITY);
    for (_, record) in &parts {
        low = low.min(Vec3::from(record.at));
        high = high.max(Vec3::from(record.at));
    }
    if !low.x.is_finite() {
        return;
    }
    let middle = (low + high) * 0.5;
    let onto = |v: f32| (v * 16.0).round() / 16.0;
    let (mx, mz) = (onto(middle.x), onto(middle.z));

    for (mut transform, mut record) in &mut parts {
        let at = Vec3::from(record.at);
        // A quarter turn about Y sends (x, z) to (z, -x).
        let (dx, dz) = (at.x - mx, at.z - mz);
        record.at = [mx + dz, at.y, mz - dx];
        record.yaw = (record.yaw + std::f32::consts::FRAC_PI_2).rem_euclid(std::f32::consts::TAU);
        transform.translation = Vec3::from(record.at);
        transform.rotation = pose(record.yaw, record.tilt, record.flip);
    }
}

/// T tilts the selected part a notch, the way R turns it. Tilt was the
/// hand's alone until now: a piece already set down could be turned but
/// never leaned, so getting it wrong meant picking it up and starting the
/// approach again.
pub(crate) fn tilt_part(
    keys: Res<ButtonInput<KeyCode>>,
    bench: Res<Bench>,
    naming: Res<Naming>,
    dims: Res<DimsEntry>,
    selected: Res<crate::gizmo::Selected>,
    mut parts: Query<(&mut Transform, &mut Placed), Without<Ghost>>,
) {
    if *bench != Bench::Builder
        || naming.0.is_some()
        || dims.0.is_some()
        || !keys.just_pressed(KeyCode::KeyT)
    {
        return;
    }
    let Some(part) = selected.lead() else {
        return;
    };
    let Ok((mut transform, mut record)) = parts.get_mut(part) else {
        return;
    };
    let step = if held_shift(&keys) {
        -15f32.to_radians()
    } else {
        15f32.to_radians()
    };
    record.tilt = (record.tilt + step).rem_euclid(std::f32::consts::TAU);
    transform.rotation = pose(record.yaw, record.tilt, record.flip);
}

pub(crate) fn turn_part(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    bench: Res<Bench>,
    naming: Res<Naming>,
    dims: Res<DimsEntry>,
    palette: Res<Palette>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    selected: Res<crate::gizmo::Selected>,
    hand: Res<Hand>,
    mut parts: Query<(&mut Transform, &mut Placed), Without<Ghost>>,
) {
    // The same rule: a full hand turns what it is holding and nothing else. R did BOTH
    // at once, so a maker with a part in hand and something still chosen turned the
    // ghost and the chosen thing on one press.
    if *bench != Bench::Builder
        || naming.0.is_some()
        || dims.0.is_some()
        || hand.kind.is_some()
        || !keys.just_pressed(KeyCode::KeyR)
        || held_shift(&keys)
    {
        return;
    }
    let Some(part) = selected.lead() else {
        return;
    };
    let Ok((mut transform, mut record)) = parts.get_mut(part) else {
        return;
    };
    // A CEILING flips its ridge instead of turning, and has to be REDRAWN for it: the
    // beam is part of its body, so unlike a yaw this changes what the part is made of.
    if let Some(PartKind::Ceiling {
        long,
        deep,
        hipped,
        across,
    }) = kind_from_name(&record.part)
    {
        let made = PartKind::Ceiling {
            long,
            deep,
            hipped,
            across: !across,
        };
        record.part = part_name(&made);
        let copy = record.clone();
        commands.entity(part).despawn_related::<Children>();
        dress_part(
            &mut commands,
            &mut meshes,
            &mut materials,
            &palette,
            &made,
            &copy,
            part,
            false,
        );
        return;
    }
    record.yaw = (record.yaw + std::f32::consts::FRAC_PI_2).rem_euclid(std::f32::consts::TAU);
    transform.rotation = pose(record.yaw, record.tilt, record.flip);
}
