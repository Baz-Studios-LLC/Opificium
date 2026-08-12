//! Painting what already stands.

use super::*;

/// Keys that steer what the hand holds. Esc empties it.
#[allow(clippy::too_many_arguments)]
/// The colour the paint tool lays down.
///
/// `None` is not a colour but the absence of one: painting with an empty brush
/// strips a part back to the colours its own body was drawn with, which is the
/// only way back once a part has been painted.
#[derive(Resource)]
pub struct Brush {
    pub ramp: Option<String>,
    pub shade: f32,
}

impl Default for Brush {
    fn default() -> Self {
        Brush {
            ramp: Some("wood".to_string()),
            shade: 0.5,
        }
    }
}

/// The swatch a picked step belongs to.
///
/// `Palette::shade` reads the nearest of five steps, so this changes no colour -
/// it only moves the brush onto a value the palette has a square for.
pub(super) fn nearest_swatch(shade: f32) -> f32 {
    crate::builder::SWATCHES
        .into_iter()
        .min_by(|a, b| (a - shade).abs().total_cmp(&(b - shade).abs()))
        .unwrap_or(shade)
}

/// Paints what is already standing.
///
/// The colour keys only ever spoke to the HAND: a part took its ramp and shade
/// from whatever was held when it went down, and after that the only way to
/// change a wall's colour was to delete the wall. Brett asked whether a building
/// could be painted, and it could not.
///
/// A mode rather than a modifier, on Brett's suggestion — PAINT sits with MOVE
/// and RESIZE on the bar, so the tool is somewhere you can see rather than a key
/// you have to know. In it, clicking a part paints it: the bench's own picking
/// already turns a click into a selection, and in this mode a selection IS the
/// stroke, so there is no second way of pointing at a part to keep in step with
/// the first.
///
/// The brush takes the same four keys the hand uses for the same job — `[` and
/// `]` through the ramps, `-` and `=` darker and brighter — because a maker
/// should not have to learn the colour keys twice. `\` empties the brush, and
/// painting with an empty brush gives a part its own colours back.
///
/// Shift paints the whole building instead of the one part: a shade at a time is
/// how a colour gets found, all at once is what happens when it has been.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_the_work(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    hovered: Res<Hovered>,
    palette: Res<Palette>,
    naming: Res<Naming>,
    mode: Res<crate::gizmo::ToolMode>,
    selected: Res<crate::gizmo::Selected>,
    mut brush: ResMut<Brush>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut placed: Query<(Entity, &mut Placed), Without<Ghost>>,
) {
    if *mode != crate::gizmo::ToolMode::Paint || naming.0.is_some() {
        return;
    }

    // The brush first, so a key and a click on the same frame paint the colour
    // the maker just chose rather than the one before it.
    let ramps: Vec<&str> = palette.names().collect();
    if !ramps.is_empty() {
        let step = i32::from(keys.just_pressed(KeyCode::BracketRight))
            - i32::from(keys.just_pressed(KeyCode::BracketLeft));
        if step != 0 {
            let here = brush
                .ramp
                .as_deref()
                .and_then(|r| ramps.iter().position(|n| *n == r))
                .unwrap_or(0);
            let next = (here as i32 + step).rem_euclid(ramps.len() as i32) as usize;
            brush.ramp = Some(ramps[next].to_string());
        }
    }
    let by =
        f32::from(keys.just_pressed(KeyCode::Equal)) - f32::from(keys.just_pressed(KeyCode::Minus));
    if by != 0.0 {
        brush.shade = (brush.shade + by * 0.25).clamp(0.0, 1.0);
        if brush.ramp.is_none() {
            brush.ramp = Some(ramps.first().copied().unwrap_or("wood").to_string());
        }
    }
    if keys.just_pressed(KeyCode::Backslash) {
        brush.ramp = None;
    }

    // THE DROPPER. Alt-click takes the colour of whatever is under the cursor
    // instead of putting one there.
    //
    // It reads the PIECE, not the part: most of what a maker wants to copy has
    // never been repainted - the timbers of a framed wall are wood and its panels
    // are bone, and the wall's own `ramp` is None - so a dropper that read the
    // record would come up empty on exactly the colours worth having. See
    // `Hit::wearing`.
    //
    // The step is snapped to a swatch, which costs nothing and buys the gold ring:
    // `Palette::shade` reads the nearest of five steps, so an authored 0.65 and the
    // swatch's 0.75 are the same colour on screen - but only the swatch's value
    // lands on a square the palette can mark as armed.
    if keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
        && buttons.just_pressed(MouseButton::Left)
    {
        if let Some(hit) = hovered.build {
            let (ramp, shade) = hit.wearing;
            brush.ramp = Some(ramp.to_string());
            brush.shade = nearest_swatch(shade);
            info!("the brush takes {ramp} at {:.2}", brush.shade);
        }
        // Whether or not it found anything: an alt-click is never a stroke, and a
        // dropper that missed and painted instead would be a poor tool.
        return;
    }

    // A stroke is a CLICK ON A PART, and nothing else. Changing the brush used
    // to repaint whatever was standing selected - meant as a way to hold a wall
    // and walk the ramps watching it change, and wrong: arming a colour is
    // choosing, not doing, and a maker choosing a colour has not said where they
    // want it yet. Brett: "Arming a color shouldnt paint."
    if !selected.is_changed() {
        return;
    }
    if selected.is_empty() {
        return;
    }
    let whole = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    for (part, mut record) in &mut placed {
        if !whole && !selected.holds(part) {
            continue;
        }
        let Some(kind) = kind_from_name(&record.part) else {
            continue;
        };
        if record.ramp.as_deref() == brush.ramp.as_deref()
            && (record.shade - brush.shade).abs() < 1e-4
        {
            continue;
        }
        record.ramp = brush.ramp.clone();
        record.shade = brush.shade;
        let copy = record.clone();
        commands.entity(part).despawn_related::<Children>();
        dress_part(
            &mut commands,
            &mut meshes,
            &mut materials,
            &palette,
            &kind,
            &copy,
            part,
            false,
        );
    }
}
