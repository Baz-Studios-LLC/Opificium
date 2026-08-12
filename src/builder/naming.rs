//! The card that asks for a name, and the one that asks for a measure.

use super::*;

/// The card that asks for the work's name.
pub(crate) fn raise_naming_card(
    commands: &mut Commands,
    fonts: &Fonts,
    palette: &Palette,
    what_for: NamingFor,
    chosen: usize,
) {
    let carrying = what_for == NamingFor::Carrying;
    let card = commands
        .spawn((
            NamingCard,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(40.0),
                margin: UiRect {
                    left: Val::Px(if carrying { -210.0 } else { -170.0 }),
                    ..default()
                },
                width: Val::Px(if carrying { 420.0 } else { 340.0 }),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(18.0)),
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme::panel_bg()),
            BorderColor::all(theme::accent(palette).with_alpha(0.7)),
            GlobalZIndex(50),
        ))
        .id();
    commands.spawn((
        Text::new(match what_for {
            NamingFor::Carrying => "CARRY IT INTO THE GAME",
            NamingFor::AsAPiece => "KEEP IT AS A PIECE",
            NamingFor::Keeping => "NAME THE WORK",
            NamingFor::AKind => "NAME A KIND OF BUILDING",
        }),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(theme::accent(palette)),
        ChildOf(card),
    ));
    commands.spawn((
        NameText,
        Text::new("_"),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(theme::text(palette)),
        Node {
            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            min_width: Val::Px(220.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::BLACK.with_alpha(0.35)),
        BorderColor::all(theme::panel_border(palette)),
        ChildOf(card),
    ));
    commands.spawn((
        Text::new(match what_for {
            NamingFor::Carrying => {
                "the village raises it under this name - esc thinks better of it"
            }
            NamingFor::AsAPiece => "kept for any work, not just this one - esc thinks better of it",
            NamingFor::Keeping => "enter saves - esc thinks better of it",
            // The one warning worth printing on a card. Nothing here can check a
            // word against the game's own vocabulary - that lives in the other
            // program's source - and a word the game does not know costs the
            // building in silence. See `project::kinds`.
            NamingFor::AKind => {
                "the game must already know this word, or it raises nothing - esc goes back"
            }
        }),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: FontSize::Px(11.0),
            ..default()
        },
        TextColor(theme::text_dim(palette).with_alpha(0.8)),
        ChildOf(card),
    ));
    if carrying {
        commands.spawn((
            Text::new("WHAT IS IT?"),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: FontSize::Px(12.0),
                ..default()
            },
            TextColor(theme::text_dim(palette)),
            Node {
                margin: UiRect::top(Val::Px(6.0)),
                ..default()
            },
            ChildOf(card),
        ));
        let kinds = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                ChildOf(card),
            ))
            .id();
        for (index, offered) in crate::project::kinds().iter().enumerate() {
            let label = offered.said();
            let standing = index == chosen;
            let button = commands
                .spawn((
                    KindButton(index),
                    Interaction::default(),
                    Node {
                        padding: UiRect::axes(Val::Px(7.0), Val::Px(3.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::BLACK.with_alpha(if standing { 0.45 } else { 0.18 })),
                    BorderColor::all(if standing {
                        theme::accent(palette).with_alpha(0.8)
                    } else {
                        theme::panel_border(palette)
                    }),
                    ChildOf(kinds),
                ))
                .id();
            commands.spawn((
                Text::new(label.clone()),
                TextFont {
                    font: fonts.display.clone().into(),
                    font_size: FontSize::Px(10.0),
                    ..default()
                },
                TextColor(if standing {
                    theme::accent(palette)
                } else {
                    theme::text_dim(palette)
                }),
                ChildOf(button),
            ));
        }
        // And the way to add one. It sits with the kinds rather than beside the
        // save, because it is a kind - and a project that knows none at all shows
        // this button alone, which is exactly the right first thing to press.
        let adding = commands
            .spawn((
                NewKindButton,
                Interaction::default(),
                Node {
                    padding: UiRect::axes(Val::Px(7.0), Val::Px(3.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::BLACK.with_alpha(0.18)),
                BorderColor::all(theme::accent(palette).with_alpha(0.5)),
                ChildOf(kinds),
            ))
            .id();
        commands.spawn((
            Text::new("+ A NEW KIND"),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: FontSize::Px(10.0),
                ..default()
            },
            TextColor(theme::accent(palette)),
            ChildOf(adding),
        ));
    }
    let row = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                margin: UiRect::top(Val::Px(4.0)),
                ..default()
            },
            ChildOf(card),
        ))
        .id();
    for (label, accent) in [
        (
            match what_for {
                NamingFor::Carrying => "CARRY IN",
                NamingFor::AsAPiece => "KEEP IT",
                NamingFor::Keeping => "SAVE",
                // It does not carry anything in or keep anything: it hands the
                // word back to the card that asked for it.
                NamingFor::AKind => "ADD IT",
            },
            true,
        ),
        ("CANCEL", false),
    ] {
        let button = commands
            .spawn((
                Interaction::default(),
                Node {
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::BLACK.with_alpha(0.18)),
                BorderColor::all(if accent {
                    theme::accent(palette).with_alpha(0.7)
                } else {
                    theme::panel_border(palette)
                }),
                ChildOf(row),
            ))
            .id();
        if accent {
            commands.entity(button).insert(NamingSave);
        } else {
            commands.entity(button).insert(NamingCancel);
        }
        commands.spawn((
            Text::new(label),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: FontSize::Px(12.0),
                ..default()
            },
            TextColor(if accent {
                theme::accent(palette)
            } else {
                theme::text_dim(palette)
            }),
            ChildOf(button),
        ));
    }
}

/// Typing while the card is up: letters, digits and dashes build the name,
/// enter writes the file, escape puts the pen down.
#[allow(clippy::too_many_arguments)]
pub(crate) fn take_the_name(
    mut commands: Commands,
    stages: Res<Stages>,
    palette: Res<Palette>,
    // Bundled: the ceiling is sixteen, and this card has three errands now.
    errands: (
        ResMut<CarryingKind>,
        ResMut<CarriedStale>,
        ResMut<PieceKept>,
        ResMut<PiecesStale>,
        ResMut<NameHeld>,
    ),
    fonts: Res<Fonts>,
    mut keystrokes: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut naming: ResMut<Naming>,
    time: Res<Time>,
    mut work_name: ResMut<WorkName>,
    placed: Query<&Placed, Without<Ghost>>,
    cards: Query<Entity, With<NamingCard>>,
    saves_click: Query<&Interaction, (Changed<Interaction>, With<NamingSave>)>,
    cancels_click: Query<&Interaction, (Changed<Interaction>, With<NamingCancel>)>,
    mut shown: Query<&mut Text, With<NameText>>,
    mut save_labels: Query<(Entity, &mut Text), (With<SaveLabel>, Without<NameText>)>,
) {
    let (mut kind, mut stale, mut kept, mut pieces_stale, mut held) = errands;
    let what_for = naming.1;
    let Some(name) = naming.0.as_mut() else {
        return;
    };
    use bevy::input::keyboard::Key;
    let mut done: Option<bool> = None;
    for stroke in keystrokes.read() {
        if !stroke.state.is_pressed() {
            continue;
        }
        match &stroke.logical_key {
            Key::Character(text) => {
                for letter in text.chars() {
                    let letter = letter.to_ascii_lowercase();
                    if (letter.is_ascii_alphanumeric() || letter == '-') && name.len() < 24 {
                        name.push(letter);
                    }
                }
            }
            Key::Space => {
                if name.len() < 24 && !name.is_empty() {
                    name.push('-');
                }
            }
            Key::Backspace => {
                name.pop();
            }
            Key::Enter => done = Some(true),
            Key::Escape => done = Some(false),
            _ => {}
        }
    }
    if saves_click
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        done = Some(true);
    }
    if cancels_click
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        done = Some(false);
    }
    for mut text in &mut shown {
        let fresh = format!("{name}_");
        if text.0 != fresh {
            *text = Text::new(fresh);
        }
    }
    let Some(saving) = done else {
        return;
    };
    // Naming a kind is the one errand that does not END the card: it came from
    // the carrying card and it goes back there, with the work's name restored and
    // the new word chosen. So it returns early rather than falling through to the
    // teardown at the foot of this function.
    if what_for == NamingFor::AKind {
        let word = name.clone();
        naming.0 = Some(std::mem::take(&mut held.0));
        naming.1 = NamingFor::Carrying;
        if saving && !word.is_empty() {
            match crate::project::add_a_kind(&word) {
                // Chosen as well as added: a maker who has just named a kind means
                // to bake THIS building as one.
                Ok(()) => {
                    if let Some(at) = crate::project::kinds()
                        .iter()
                        .position(|known| known.word == word)
                    {
                        kind.0 = at;
                    }
                    info!("the project knows {word} now");
                }
                Err(why) => warn!("could not add {word}: {why}"),
            }
        }
        for card in &cards {
            commands.entity(card).despawn();
        }
        raise_naming_card(&mut commands, &fonts, &palette, NamingFor::Carrying, kind.0);
        return;
    }
    // The same card, three errands now.
    if saving && what_for == NamingFor::AsAPiece {
        let called = if name.is_empty() {
            "piece".to_string()
        } else {
            name.clone()
        };
        let home = pieces_home();
        let _ = std::fs::create_dir_all(&home);
        let piece = Piece {
            format: 1,
            kind: "piece".to_string(),
            name: called.clone(),
            // Centred on its own middle, so it is set down where the cursor is
            // rather than wherever it happened to be drawn.
            parts: piece_from(&kept.0),
        };
        let said = match serde_json::to_string_pretty(&piece)
            .map_err(|why| why.to_string())
            .and_then(|text| {
                let out = home.join(format!("{called}.{WORK_KIND}"));
                std::fs::write(&out, text).map_err(|why| why.to_string())
            }) {
            Ok(()) => {
                pieces_stale.0 = true;
                format!(
                    "KEPT {} AS A PIECE - {} PARTS",
                    called.to_uppercase(),
                    piece.parts.len()
                )
            }
            Err(why) => {
                warn!("could not keep the piece {called}: {why}");
                "THE PIECE COULD NOT BE WRITTEN".to_string()
            }
        };
        kept.0.clear();
        for (entity, mut text) in &mut save_labels {
            *text = Text::new(said.clone());
            commands.entity(entity).insert(PassingWord {
                back: crate::rail::FOOT_SAYING,
                until: time.elapsed_secs() + 3.0,
            });
        }
    } else if saving && what_for == NamingFor::Carrying {
        let called = if name.is_empty() {
            "untitled".to_string()
        } else {
            name.clone()
        };
        let work = gather_the_work(&called, &stages, placed.iter());
        // A project with no kinds writes NONE, and the game reads the drawing's
        // name instead - which is the older reading, and still a true one.
        let known = crate::project::kinds();
        let (word, label) = known
            .get(kind.0)
            .map_or((String::new(), String::new()), |kind| {
                (kind.word.clone(), kind.said())
            });
        stale.0 = true;
        let said = match carry_into_the_game(&work, &palette, &called, &word) {
            Ok((boxes, marks)) => format!(
                "CARRIED {} IN {} - {boxes} BOXES, {marks} MARKS",
                called.to_uppercase(),
                if label.is_empty() {
                    "TO BE CLAIMED BY ITS NAME".to_string()
                } else {
                    format!("AS A {label}")
                }
            ),
            Err(why) => {
                warn!("could not carry {called} in: {why}");
                "THE GAME'S FOLDER COULD NOT BE WRITTEN".to_string()
            }
        };
        for (entity, mut text) in &mut save_labels {
            *text = Text::new(said.clone());
            commands.entity(entity).insert(PassingWord {
                back: crate::rail::FOOT_SAYING,
                until: time.elapsed_secs() + 3.5,
            });
        }
    } else if saving {
        let written = if name.is_empty() {
            "untitled"
        } else {
            name.as_str()
        };
        if let Some(dir) = bench_path().parent().map(|d| d.to_path_buf()) {
            let _ = std::fs::create_dir_all(&dir);
            // The work's own name is overwritten freely - that is what
            // saving means - but a name some OTHER work holds steps aside
            // rather than clobbering it in silence.
            let ours = work_name.0.as_deref() == Some(written);
            let mut stem = written.to_string();
            let mut path = dir.join(format!("{stem}.{WORK_KIND}"));
            if !ours {
                let mut n = 2;
                while path.exists() || dir.join(format!("{stem}.json")).exists() {
                    stem = format!("{written}-{n}");
                    path = dir.join(format!("{stem}.{WORK_KIND}"));
                    n += 1;
                }
            }
            // A work saved under a name it already held as a `.json` leaves no
            // twin behind: the drawer lists what is in the folder, and two rows
            // reading LONGHOUSE would be two rows a maker has to tell apart by
            // opening them.
            let elder = dir.join(format!("{stem}.json"));
            if ours && elder.exists() && elder != path {
                let _ = std::fs::remove_file(&elder);
            }
            let bench = gather_the_work(&stem, &stages, placed.iter());
            if let Ok(json) = serde_json::to_string_pretty(&bench) {
                let showing = stages.showing.min(bench.stages.len().saturating_sub(1));
                let count = bench.stages.get(showing).map_or(0, Vec::len);
                let _ = std::fs::write(&path, json);
                info!("saved {count} parts to {}", path.display());
                work_name.0 = Some(stem.clone());
                for (entity, mut text) in &mut save_labels {
                    *text = Text::new(format!("SAVED {} - {count} PARTS", stem.to_uppercase()));
                    commands.entity(entity).insert(PassingWord {
                        back: crate::rail::FOOT_SAYING,
                        until: time.elapsed_secs() + 2.5,
                    });
                }
            }
        }
    }
    naming.0 = None;
    for card in &cards {
        commands.entity(card).despawn();
    }
}

/// Passing words return to their old text when their moment ends.
pub(crate) fn settle_words(
    mut commands: Commands,
    time: Res<Time>,
    mut words: Query<(Entity, &PassingWord, &mut Text)>,
) {
    for (entity, word, mut text) in &mut words {
        if time.elapsed_secs() >= word.until {
            *text = Text::new(word.back.to_string());
            commands.entity(entity).remove::<PassingWord>();
        }
    }
}

/// The dimensions card: raised once, shown when it has something to say.
/// While stretch-drawing it reads the live size; with a sized part
/// selected in RESIZE it shows the measure and D opens typed entry -
/// "3.5" for a length, "3.5x6" for a slab - enter applies on the
/// lattice, escape thinks better of it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dims_panel(
    mut commands: Commands,
    mut keystrokes: MessageReader<bevy::input::keyboard::KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    fonts: Res<Fonts>,
    palette: Res<Palette>,
    tool: Res<crate::gizmo::ToolMode>,
    selected: Res<crate::gizmo::Selected>,
    mut entry: ResMut<DimsEntry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cards: Query<&mut Visibility, With<DimsCard>>,
    mut readouts: Query<&mut Text, With<DimsText>>,
    ghosts: Query<&Placed, With<Ghost>>,
    mut parts: Query<(&mut Transform, &mut Placed), Without<Ghost>>,
    mut raised: Local<bool>,
) {
    if !*raised {
        *raised = true;
        let card = commands
            .spawn((
                DimsCard,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    // Clear of the step row, which took the floor when the
                    // stages arrived and hid this underneath itself.
                    bottom: Val::Px(52.0),
                    margin: UiRect::left(Val::Px(-130.0)),
                    width: Val::Px(260.0),
                    justify_content: JustifyContent::Center,
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(theme::panel_bg()),
                BorderColor::all(theme::panel_border(&palette)),
                Visibility::Hidden,
            ))
            .id();
        commands.spawn((
            DimsText,
            Text::new(""),
            TextFont {
                font: fonts.text.clone().into(),
                font_size: FontSize::Px(13.0),
                ..default()
            },
            TextColor(theme::text(&palette)),
            ChildOf(card),
        ));
        return;
    }

    // The selected sized part, if any, and its measure.
    let sized = selected.one().and_then(|part| {
        let (_, record) = parts.get(part).ok()?;
        let kind = kind_from_name(&record.part)?;
        match kind {
            PartKind::Wall(long) => Some((part, long, None, None)),
            PartKind::Seg { long, .. } => Some((part, long, None, None)),
            PartKind::Trim { long, .. } => Some((part, long, None, None)),
            PartKind::Beam(long, ..) => Some((part, long, None, None)),
            PartKind::Ridge(long) => Some((part, long, None, None)),
            // A roof's PITCH rides along with its size. Brett: "incase I want
            // to make multiple buildings with the same sized roof peak" - and a
            // number you can read off one roof is a number you can pull another
            // to, where an angle you can only judge by eye is not.
            PartKind::GableRoof(w, d, _, pitch) => Some((part, w, Some(d), Some(pitch))),
            PartKind::Gable(long, pitch) => Some((part, long, None, Some(pitch))),
            PartKind::Chimney(drop) => Some((part, drop, None, None)),
            PartKind::Floor(w, d) | PartKind::Roof(w, d) => Some((part, w, Some(d), None)),
            PartKind::Foundation(w, d, _) => Some((part, w, Some(d), None)),
            _ => None,
        }
    });

    // Typing takes precedence; then the live stretch; then the measure.
    let mut said: Option<String> = None;
    if let Some(text) = entry.0.as_mut() {
        use bevy::input::keyboard::Key;
        let mut done: Option<bool> = None;
        for stroke in keystrokes.read() {
            if !stroke.state.is_pressed() {
                continue;
            }
            match &stroke.logical_key {
                Key::Character(typed) => {
                    for letter in typed.chars() {
                        let letter = letter.to_ascii_lowercase();
                        if (letter.is_ascii_digit() || letter == '.' || letter == 'x')
                            && text.len() < 12
                        {
                            text.push(letter);
                        }
                    }
                }
                Key::Backspace => {
                    text.pop();
                }
                Key::Enter => done = Some(true),
                Key::Escape => done = Some(false),
                _ => {}
            }
        }
        said = Some(format!("{text}_"));
        if let Some(saving) = done {
            if saving
                && let Some((part, _, had_d, _)) = sized
                && let Ok((mut transform, mut record)) = parts.get_mut(part)
                && let Some(kind) = kind_from_name(&record.part)
            {
                // "3.5" or "3.5x6", snapped onto the lattice, no smaller
                // than one coarse cell, resized around the centre.
                let lattice = |value: f32| ((value * 16.0).round() / 16.0).max(0.25);
                let (w_in, d_in) = match text.split_once('x') {
                    Some((a, b)) => (a.parse::<f32>().ok(), b.parse::<f32>().ok()),
                    None => (text.parse::<f32>().ok(), None),
                };
                // Typed numbers are UNITS - sixteenths of a metre - so a
                // wall is 40 tall and a room 48 wide, no decimals needed.
                let units = |value: f32| lattice(value / 16.0);
                let _ = &lattice;
                if let Some(w) = w_in.map(units) {
                    let d = d_in.map(lattice);
                    let made = match kind {
                        PartKind::Wall(_) => Some(PartKind::Wall(w)),
                        PartKind::Seg { high, lift, .. } => Some(PartKind::Seg {
                            long: w,
                            high,
                            lift,
                        }),
                        PartKind::Trim { stone, .. } => Some(PartKind::Trim { long: w, stone }),
                        PartKind::Floor(_, old) => Some(PartKind::Floor(w, d.unwrap_or(old))),
                        PartKind::Foundation(_, old, high) => {
                            Some(PartKind::Foundation(w, d.unwrap_or(old), high))
                        }
                        PartKind::Roof(_, old) => Some(PartKind::Roof(w, d.unwrap_or(old))),
                        // The pitch rides through a resize: a roof HAS a pitch,
                        // so a wider building wants a taller roof and not a
                        // flatter one.
                        PartKind::GableRoof(_, old, over, pitch) => {
                            Some(PartKind::GableRoof(w, d.unwrap_or(old), over, pitch))
                        }
                        PartKind::Chimney(_) => Some(PartKind::Chimney(w)),
                        // Typed: width, and the height it climbs after the x.
                        PartKind::Stairs {
                            rise,
                            stone,
                            rail_stone,
                            hand,
                            ..
                        } => Some(PartKind::Stairs {
                            rise: d.unwrap_or(rise),
                            wide: w,
                            stone,
                            rail_stone,
                            hand,
                        }),
                        _ => None,
                    };
                    if let Some(made) = made {
                        record.part = part_name(&made);
                        record.at = transform.translation.into();
                        let _ = &mut transform;
                        commands.entity(part).despawn_related::<Children>();
                        dress_part(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &palette,
                            &made,
                            &record,
                            part,
                            false,
                        );
                        let _ = had_d;
                    }
                }
            }
            entry.0 = None;
            said = None;
        }
    } else if let Some(drawn) = ghosts.iter().next().and_then(|g| kind_from_name(&g.part)) {
        // Live measure while stretch-drawing.
        let units = |value: f32| format!("{}", (value * 16.0).round() as i64);
        said = match drawn {
            PartKind::Wall(long) => Some(format!("wall - {}", units(long))),
            PartKind::Trim { long, .. } => Some(format!("trim - {}", units(long))),
            PartKind::Floor(w, d) => Some(format!("floor - {} x {}", units(w), units(d))),
            PartKind::Foundation(w, d, _) => {
                Some(format!("foundation - {} x {}", units(w), units(d)))
            }
            PartKind::Roof(w, d) => Some(format!("roof - {} x {}", units(w), units(d))),
            _ => None,
        };
    } else if *tool == crate::gizmo::ToolMode::Resize
        && let Some((_, w, d, pitch)) = sized
    {
        let units = |value: f32| format!("{}", (value * 16.0).round() as i64);
        let angle = pitch.map_or(String::new(), |degrees| format!("  {degrees:.1}°"));
        said = Some(match d {
            Some(d) => format!("{} x {}{angle} - D to type", units(w), units(d)),
            None => format!("{}{angle} - D to type", units(w)),
        });
        if keys.just_pressed(KeyCode::KeyD) {
            entry.0 = Some(String::new());
        }
    }

    for mut visibility in &mut cards {
        let wanted = if said.is_some() || entry.0.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
    if let Some(word) = said {
        for mut text in &mut readouts {
            if text.0 != word {
                *text = Text::new(word.clone());
            }
        }
    }
}
