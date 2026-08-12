//! Undo, redo, and the clipboard.

use super::*;

/// The bench's memory: whole-state snapshots, since a build is nothing
/// but its list of records. Undo therefore covers everything the same
/// way - placements, punches, stretches, moves, typed sizes, even a
/// template load or a cleared bench.
#[derive(Resource, Default)]
pub struct History {
    past: Vec<Vec<Placed>>,
    future: Vec<Vec<Placed>>,
    current: Vec<Placed>,
    primed: bool,
}

impl History {
    /// Forgets everything, for a change no hand could have made — setting out
    /// another step of the build swaps every part on the bench at once.
    pub(crate) fn forget(&mut self) {
        self.past.clear();
        self.future.clear();
        self.current.clear();
        self.primed = false;
    }
}

pub(crate) fn state_signature(list: &[Placed]) -> String {
    let mut lines: Vec<String> = list
        .iter()
        .map(|record| serde_json::to_string(record).unwrap_or_default())
        .collect();
    lines.sort();
    lines.join("|")
}

/// Notices settled changes and remembers the state they replaced. While
/// the mouse button is down nothing commits, so a whole drag lands as
/// one step.
pub(crate) fn remember(
    buttons: Res<ButtonInput<MouseButton>>,
    mut history: ResMut<History>,
    placed: Query<&Placed, Without<Ghost>>,
) {
    if buttons.pressed(MouseButton::Left) {
        return;
    }
    let now: Vec<Placed> = placed.iter().cloned().collect();
    if !history.primed {
        history.current = now;
        history.primed = true;
        return;
    }
    if state_signature(&now) != state_signature(&history.current) {
        let old = std::mem::replace(&mut history.current, now);
        history.past.push(old);
        if history.past.len() > 50 {
            history.past.remove(0);
        }
        history.future.clear();
    }
}

/// Ctrl or cmd with Z walks back; with Y - or shift-Z - walks forward.
#[allow(clippy::too_many_arguments)]
pub(crate) fn recall(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    bench: Res<Bench>,
    naming: Res<Naming>,
    dims: Res<DimsEntry>,
    mut history: ResMut<History>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<Palette>,
    standing: Query<Entity, (With<Placed>, Without<Ghost>)>,
    mut wish: ResMut<crate::menu::MenuWish>,
) {
    // Taken before the guards below, and before anything can return: a wish
    // spent is a wish that cannot go off later, at some moment the maker has
    // stopped thinking about the menu they pressed.
    let asked_back = wish.taken(crate::menu::MenuDeed::Undo);
    let asked_forward = wish.taken(crate::menu::MenuDeed::Redo);
    if *bench != Bench::Builder || naming.0.is_some() || dims.0.is_some() {
        return;
    }
    let held = keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
        || keys.pressed(KeyCode::SuperLeft)
        || keys.pressed(KeyCode::SuperRight);
    let shifted = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    // The menu asks for the same two things, and does not hold a key down to do
    // it - so `held` gates the KEYS and nothing else.
    let back = asked_back || (held && keys.just_pressed(KeyCode::KeyZ) && !shifted);
    let forward = asked_forward
        || (held
            && (keys.just_pressed(KeyCode::KeyY) || (keys.just_pressed(KeyCode::KeyZ) && shifted)));
    if !back && !forward {
        return;
    }
    let restored = if back {
        let Some(older) = history.past.pop() else {
            return;
        };
        let now = std::mem::replace(&mut history.current, older.clone());
        history.future.push(now);
        older
    } else {
        let Some(newer) = history.future.pop() else {
            return;
        };
        let now = std::mem::replace(&mut history.current, newer.clone());
        history.past.push(now);
        newer
    };
    for part in &standing {
        commands.entity(part).despawn();
    }
    for record in &restored {
        if let Some(kind) = kind_from_name(&record.part) {
            spawn_part(
                &mut commands,
                &mut meshes,
                &mut materials,
                &palette,
                &kind,
                record,
                false,
            );
        }
    }
    info!(
        "{} to a bench of {} parts",
        if back {
            "walked back"
        } else {
            "walked forward"
        },
        restored.len()
    );
}

/// Walking into MOVE or RESIZE empties the hand: the ghost belongs to
/// placement, and those modes are for what already stands.
pub(crate) fn disarm_on_mode(
    mut commands: Commands,
    tool: Res<crate::gizmo::ToolMode>,
    mut hand: ResMut<Hand>,
    ghosts: Query<Entity, With<Ghost>>,
) {
    if tool.is_changed() && *tool != crate::gizmo::ToolMode::Normal && hand.kind.is_some() {
        *hand = Hand::default();
        for ghost in &ghosts {
            commands.entity(ghost).despawn();
        }
    }
}

/// The last part copied, kept whole - its kind, size, turn and paint.
#[derive(Resource, Default)]
pub struct Clipboard(pub Option<Placed>);

/// Cmd or ctrl with C copies what the cursor touches (or what is
/// selected); with V it loads that copy into the hand, ghost and all,
/// so it lands with every snap the bench offers and can be stamped as
/// often as you like.
#[allow(clippy::too_many_arguments)]
pub(crate) fn copy_and_paste(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    bench: Res<Bench>,
    naming: Res<Naming>,
    dims: Res<DimsEntry>,
    hovered: Res<Hovered>,
    gizmo: (Res<crate::gizmo::Selected>, ResMut<crate::gizmo::ToolMode>),
    mut clipboard: ResMut<Clipboard>,
    mut hand: ResMut<Hand>,
    placed: Query<&Placed, Without<Ghost>>,
    ghosts: Query<Entity, With<Ghost>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<Palette>,
    mut wish: ResMut<crate::menu::MenuWish>,
) {
    let (selected, mut tool) = gizmo;
    // As in `recall`: spent up front, so a wish cannot go off later.
    let asked_copy = wish.taken(crate::menu::MenuDeed::Copy);
    let asked_paste = wish.taken(crate::menu::MenuDeed::Paste);
    if *bench != Bench::Builder || naming.0.is_some() || dims.0.is_some() {
        return;
    }
    let held = keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
        || keys.pressed(KeyCode::SuperLeft)
        || keys.pressed(KeyCode::SuperRight);
    // Bracketed, because the `let` chain below binds names the body needs: without
    // them `||` splits the condition and the bindings belong to only one half.
    if (keys.just_pressed(KeyCode::KeyC) && held || asked_copy)
        && let Some(source) = selected.lead().or(hovered.grab)
        && let Ok(record) = placed.get(source)
    {
        clipboard.0 = Some(record.clone());
        info!("copied {}", record.part);
    }
    // Bracketed, because the `let` chain below binds names the body needs: without
    // them `||` splits the condition and the bindings belong to only one half.
    if (keys.just_pressed(KeyCode::KeyV) && held || asked_paste)
        && let Some(record) = clipboard.0.clone()
        && let Some(kind) = kind_from_name(&record.part)
    {
        // Pasting is placing: the hand takes the copy and the modes step
        // back to NORMAL, where placement lives.
        *tool = crate::gizmo::ToolMode::Normal;
        *hand = Hand {
            kind: Some(kind),
            anchor: None,
            flip: record.flip,
            stage: record.stage.clone(),
            yaw: record.yaw,
            tilt: record.tilt,
            lift: 0.0,
            ramp: record.ramp.clone(),
            shade: record.shade,
        };
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

/// M mirrors what the hand holds, or what the arrows hold: the body
/// reflects across its own length and any tilt leans the other way, so
/// a pitched panel's twin completes the gable and an L-corner becomes
/// the other hand.
#[allow(clippy::too_many_arguments)]
pub(crate) fn mirror_part(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    bench: Res<Bench>,
    naming: Res<Naming>,
    dims: Res<DimsEntry>,
    selected: Res<crate::gizmo::Selected>,
    mut hand: ResMut<Hand>,
    ghosts: Query<Entity, With<Ghost>>,
    mut parts: Query<(&mut Transform, &mut Placed), Without<Ghost>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<Palette>,
) {
    if *bench != Bench::Builder
        || naming.0.is_some()
        || dims.0.is_some()
        || !keys.just_pressed(KeyCode::KeyM)
    {
        return;
    }
    // A standing part first: mirroring what you can see beats mirroring
    // what you are about to place.
    if let Some(part) = selected.lead()
        && let Ok((mut transform, mut record)) = parts.get_mut(part)
        && let Some(kind) = kind_from_name(&record.part)
    {
        record.flip = !record.flip;
        transform.rotation = pose(record.yaw, record.tilt, record.flip);
        commands.entity(part).despawn_related::<Children>();
        dress_part(
            &mut commands,
            &mut meshes,
            &mut materials,
            &palette,
            &kind,
            &record,
            part,
            false,
        );
        return;
    }
    if hand.kind.is_some() {
        hand.flip = !hand.flip;
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

/// The opening a frame stands in follows it. Sliding a door or window
/// with the arrows closes the wall it came from and parts the wall it
/// lands in, when the drag lets go.
#[allow(clippy::too_many_arguments)]
pub(crate) fn reflow_openings(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    selected: Res<crate::gizmo::Selected>,
    mut came_from: Local<Option<(Entity, Vec3)>>,
    placed: Query<(Entity, &Transform, &Placed, &Visibility), Without<Ghost>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<Palette>,
) {
    // The one table, reached through the one function. This was a SECOND copy
    // of it, with the same three rows written out again - so a new opening added
    // to the shelf punched its hole correctly when placed and reverted to a
    // door's dimensions the moment the wall under it was reflowed.
    let opening_for =
        |record: &Placed| kind_from_name(&record.part).and_then(|kind| opening_of(&kind));

    if buttons.just_pressed(MouseButton::Left) {
        *came_from = selected
            .lead()
            .and_then(|part| placed.get(part).ok())
            .filter(|(_, _, record, _)| opening_for(record).is_some())
            .map(|(entity, at, _, _)| (entity, at.translation));
        return;
    }
    if !buttons.just_released(MouseButton::Left) {
        return;
    }
    let Some((frame, old_spot)) = came_from.take() else {
        return;
    };
    let Ok((_, at, record, _)) = placed.get(frame) else {
        return;
    };
    let now = at.translation;
    if now.distance(old_spot) < 0.03 {
        return;
    }
    let Some((wide, head, sill, is_door)) = opening_for(record) else {
        return;
    };

    // Close the wall it came from, then part the one it landed in. If
    // nothing stands where it landed, the frame simply keeps its place.
    heal_wall_at(
        &mut commands,
        &mut meshes,
        &mut materials,
        &palette,
        &placed,
        frame,
        old_spot,
    );
    let carried = Hand {
        kind: kind_from_name(&record.part),
        anchor: None,
        flip: record.flip,
        stage: record.stage.clone(),
        yaw: record.yaw,
        tilt: 0.0,
        lift: 0.0,
        ramp: record.ramp.clone(),
        shade: record.shade,
    };
    let punched = punch_wall(
        &mut commands,
        &mut meshes,
        &mut materials,
        &palette,
        &placed,
        None,
        now,
        wide,
        head,
        sill,
        is_door,
        &carried,
    );
    if punched {
        // The punch set a fresh frame of its own in the new opening.
        commands.entity(frame).despawn();
    }
}
