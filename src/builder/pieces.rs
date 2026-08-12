//! Pieces: a cluster of parts kept to bring into other works.

use super::*;

/// Asks what the work is, and where it should go, before carrying it in.
///
/// The bake used to be a `cargo test`, which meant a building could only be
/// carried in from a source tree - so Opificium in the launcher build was a
/// sketchpad whose work had nowhere to go. Brett: "At what point does it install
/// its own files?" It goes into the game's own folder now, in one press, and the
/// game reads that folder alongside the drawings that shipped with it.
///
/// The press raises the card rather than writing at once, because a building
/// needs two things said about it and only one of them was ever in the file
/// name: what it is CALLED, and what it IS.
#[allow(clippy::too_many_arguments)]
/// Raises the card for a piece that has just been gathered.
pub(crate) fn name_the_piece(
    mut commands: Commands,
    fonts: Res<Fonts>,
    palette: Res<Palette>,
    mut wants: ResMut<PieceWantsAName>,
    mut naming: ResMut<Naming>,
) {
    if !wants.0 {
        return;
    }
    wants.0 = false;
    if naming.0.is_some() {
        return;
    }
    naming.0 = Some(String::new());
    naming.1 = NamingFor::AsAPiece;
    raise_naming_card(&mut commands, &fonts, &palette, NamingFor::AsAPiece, 0);
}

/// Hangs the drawer of kept pieces, and hangs it again when one is added.
pub(crate) fn hang_the_pieces(
    mut commands: Commands,
    fonts: Res<Fonts>,
    palette: Res<Palette>,
    drawer: Option<Res<PieceDrawer>>,
    mut stale: ResMut<PiecesStale>,
) {
    if !stale.0 {
        return;
    }
    let Some(drawer) = drawer else {
        return;
    };
    stale.0 = false;
    commands.entity(drawer.0).despawn_related::<Children>();

    let home = pieces_home();
    let mut kept: Vec<(std::path::PathBuf, String, usize)> = std::fs::read_dir(&home)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| is_a_work(path))
        .filter_map(|path| {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|text| serde_json::from_str::<Piece>(&text).ok())
                .filter(|piece| piece.kind == "piece")
                .map(|piece| (path, piece.name, piece.parts.len()))
        })
        .collect();
    kept.sort_by(|a, b| a.1.cmp(&b.1));

    if kept.is_empty() {
        commands.spawn((
            Text::new("choose a few parts, right-click, keep as a piece"),
            TextFont {
                font: fonts.text.clone().into(),
                font_size: FontSize::Px(10.0),
                ..default()
            },
            TextColor(theme::text_dim(&palette).with_alpha(0.7)),
            TextLayout::new(
                bevy::text::Justify::Left,
                bevy::text::LineBreak::WordBoundary,
            ),
            ChildOf(drawer.0),
        ));
        return;
    }
    for (path, name, parts) in kept {
        let button = commands
            .spawn((
                PieceButton(path),
                Interaction::default(),
                Node {
                    width: Val::Percent(100.0),
                    min_width: Val::Px(0.0),
                    padding: UiRect::axes(Val::Px(9.0), Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::BLACK.with_alpha(0.18)),
                BorderColor::all(theme::panel_border(&palette)),
                ChildOf(drawer.0),
            ))
            .id();
        let label = button_label(
            &mut commands,
            &fonts,
            &palette,
            button,
            Box::leak(format!("{} - {parts}", name.to_uppercase()).into_boxed_str()),
        );
        commands.entity(label).insert(TextLayout::new(
            bevy::text::Justify::Left,
            bevy::text::LineBreak::AnyCharacter,
        ));
        commands.spawn((
            crate::rail::Word("Take it in hand - click to set it down, R turns it, esc drops it"),
            ChildOf(button),
        ));
    }
}

/// The drawer the pieces hang in.
#[derive(Resource)]
pub(crate) struct PieceDrawer(pub(crate) Entity);

/// Whether that drawer is out of date.
#[derive(Resource)]
pub(crate) struct PiecesStale(pub(crate) bool);

impl Default for PiecesStale {
    fn default() -> Self {
        PiecesStale(true)
    }
}

/// Takes a kept piece into the hand, turns it, sets it down, or drops it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn wield_a_piece(
    mut commands: Commands,
    // Bundled: Bevy's parameter ceiling is sixteen, and holding a whole cluster
    // takes more than one hand.
    told: (
        Res<Bench>,
        Res<ButtonInput<KeyCode>>,
        Res<ButtonInput<MouseButton>>,
        Res<Naming>,
        Res<Palette>,
        Res<SnapGrid>,
    ),
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    hovers: Query<&Interaction>,
    taken: Query<(&Interaction, &PieceButton), Changed<Interaction>>,
    mut held: ResMut<PieceInHand>,
    mut hand: ResMut<Hand>,
    ghosts: Query<Entity, With<PieceGhost>>,
    plain_ghosts: Query<Entity, With<Ghost>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut standing: Query<&mut Transform, With<PieceGhost>>,
    already: Query<&Placed, Without<Ghost>>,
) {
    let (bench, keys, buttons, naming, palette, snap_grid) = told;
    if *bench != Bench::Builder || naming.0.is_some() {
        return;
    }
    // Taking one up. The ordinary hand empties: two things held at once is two
    // things placed by one click.
    if let Some((_, piece)) = taken
        .iter()
        .find(|(touch, _)| **touch == Interaction::Pressed)
    {
        if let Some(read) = std::fs::read_to_string(&piece.0)
            .ok()
            .and_then(|text| serde_json::from_str::<Piece>(&text).ok())
        {
            *hand = Hand::default();
            for ghost in &plain_ghosts {
                commands.entity(ghost).despawn();
            }
            held.parts = read.parts;
            held.name = read.name;
            held.yaw = 0.0;
            for ghost in &ghosts {
                commands.entity(ghost).despawn();
            }
            for record in &held.parts {
                if let Some(kind) = kind_from_name(&record.part) {
                    let ghost = spawn_part(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &palette,
                        &kind,
                        record,
                        true,
                    );
                    commands.entity(ghost).insert(PieceGhost);
                }
            }
        }
        return;
    }
    if held.parts.is_empty() {
        return;
    }
    // Dropping it.
    if keys.just_pressed(KeyCode::Escape) {
        held.parts.clear();
        for ghost in &ghosts {
            commands.entity(ghost).despawn();
        }
        return;
    }
    // Turning it, in quarter turns, the way everything else turns.
    if keys.just_pressed(KeyCode::KeyR) {
        held.yaw = (held.yaw + std::f32::consts::FRAC_PI_2).rem_euclid(std::f32::consts::TAU);
    }
    let Some(point) = cursor_point(&windows, &cameras, 0.0) else {
        return;
    };
    let grid = 16.0 / snap_grid.0 as f32;
    let at = Vec3::new(
        (point.x * grid).round() / grid,
        0.0,
        (point.z * grid).round() / grid,
    );
    let spin = Quat::from_rotation_y(held.yaw);
    // The ghost follows, turned and snapped, so what a maker sees is what will
    // stand there.
    for (mut transform, record) in standing.iter_mut().zip(held.parts.iter()) {
        transform.translation = at + spin * Vec3::from(record.at);
        transform.rotation = pose(record.yaw + held.yaw, record.tilt, record.flip);
    }
    let over_ui = hovers
        .iter()
        .any(|interaction| *interaction != Interaction::None);
    if !buttons.just_pressed(MouseButton::Left) || over_ui {
        return;
    }
    // Setting it down: every part at once, as ONE fresh group, so what was one
    // thing where it was kept is one thing where it lands.
    // Fresh in THIS work: a piece kept beside a longhouse knows nothing about
    // the groups on the bench it is being set down on.
    let together = a_fresh_group(already.iter());
    for record in held.parts.clone() {
        let Some(kind) = kind_from_name(&record.part) else {
            continue;
        };
        let mut set = record.clone();
        set.at = (at + spin * Vec3::from(record.at)).into();
        set.yaw = (record.yaw + held.yaw).rem_euclid(std::f32::consts::TAU);
        set.group = Some(together);
        spawn_part(
            &mut commands,
            &mut meshes,
            &mut materials,
            &palette,
            &kind,
            &set,
            false,
        );
    }
    info!(
        "set down the piece {} - {} parts",
        held.name,
        held.parts.len()
    );
}
