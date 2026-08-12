//! Turning, tilting and mirroring what stands.

use super::*;

/// R turns whatever is selected, in whichever mode: the hand's own
/// quarter-turn belongs to placing, but a part already standing should
/// answer the same key.
#[allow(clippy::too_many_arguments)]
pub(crate) fn held_shift(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
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
    mut parts: Query<(&mut Transform, &mut Placed), Without<Ghost>>,
) {
    if *bench != Bench::Builder
        || naming.0.is_some()
        || dims.0.is_some()
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
    record.yaw = (record.yaw + std::f32::consts::FRAC_PI_2).rem_euclid(std::f32::consts::TAU);
    transform.rotation = pose(record.yaw, record.tilt, record.flip);
}
