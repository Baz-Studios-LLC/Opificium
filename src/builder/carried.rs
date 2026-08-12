//! The drawer of work already carried into the game, and taking one back out.

use super::*;

/// Where the drawings that SHIPPED are, from the bench's side of the fence.
///
/// In a bundle the bench stands in the same folder as the game, so the game's
/// assets are beside it. In a source tree the bench is its own crate one level
/// down, so they are one level up.
pub(crate) fn shipped_buildings() -> Option<std::path::PathBuf> {
    let mut roads: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe()
        && let Some(beside) = exe.parent()
    {
        roads.push(beside.join("assets/buildings"));
    }
    roads.push(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../assets/buildings"));
    roads.push("assets/buildings".into());
    roads.into_iter().find(|road| road.is_dir())
}

/// Just enough of a baked file to list it.
#[derive(serde::Deserialize)]
pub(crate) struct CarriedFile {
    name: String,
    #[serde(default)]
    kind: String,
}

/// Hangs the list of what the game will raise.
pub(crate) fn hang_the_carried(
    mut commands: Commands,
    fonts: Res<Fonts>,
    palette: Res<Palette>,
    drawer: Option<Res<CarriedDrawer>>,
    mut stale: ResMut<CarriedStale>,
) {
    if !stale.0 {
        return;
    }
    let Some(drawer) = drawer else {
        return;
    };
    stale.0 = false;
    commands.entity(drawer.0).despawn_related::<Children>();

    // EVERYTHING the game will raise, not merely what this maker put there.
    // Brett, looking at a drawer that said nothing while two of his own
    // buildings stood in the village: "I dont see it?" - because those shipped
    // inside the app, and a list of what the game will raise that leaves out
    // most of what the game will raise is a list nobody can trust.
    let mut carried: Vec<(std::path::PathBuf, CarriedFile, bool)> = Vec::new();
    for (home, mine) in [
        (shipped_buildings(), false),
        (Some(carried_home("buildings")), true),
    ] {
        let Some(home) = home else {
            continue;
        };
        for path in std::fs::read_dir(&home)
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|kind| kind == "json"))
        {
            let Some(file) = std::fs::read_to_string(&path)
                .ok()
                .and_then(|text| serde_json::from_str::<CarriedFile>(&text).ok())
            else {
                continue;
            };
            // The game's own rule, said here too: a maker's drawing of a given
            // name replaces the one that shipped under it, so the list shows
            // one row and it is the one that will actually be raised.
            match carried.iter().position(|(_, had, _)| had.name == file.name) {
                Some(standing) => carried[standing] = (path, file, mine),
                None => carried.push((path, file, mine)),
            }
        }
    }
    carried.sort_by(|a, b| a.1.name.cmp(&b.1.name));

    if carried.is_empty() {
        commands.spawn((
            Text::new("nothing carried in yet"),
            TextFont {
                font: fonts.text.clone().into(),
                font_size: FontSize::Px(11.0),
                ..default()
            },
            TextColor(theme::text_dim(&palette).with_alpha(0.7)),
            ChildOf(drawer.0),
        ));
        return;
    }
    for (path, file, mine) in carried {
        let row = CarriedRow {
            path,
            name: file.name.clone(),
        };
        let button = commands
            .spawn((
                row.clone(),
                Interaction::default(),
                Node {
                    width: Val::Percent(100.0),
                    min_width: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::axes(Val::Px(9.0), Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::BLACK.with_alpha(0.18)),
                BorderColor::all(theme::panel_border(&palette)),
                ChildOf(drawer.0),
            ))
            .id();
        let name = button_label(
            &mut commands,
            &fonts,
            &palette,
            button,
            Box::leak(file.name.to_uppercase().into_boxed_str()),
        );
        // A saved name is usually one long word with nothing a line breaker
        // would call a gap, and a word that will not break hangs off the shelf.
        commands.entity(name).insert(TextLayout::new(
            bevy::text::Justify::Left,
            bevy::text::LineBreak::AnyCharacter,
        ));
        let said = crate::project::kinds()
            .iter()
            .find(|kind| kind.word == file.kind)
            .map(|kind| kind.said())
            .unwrap_or_else(|| "BY ITS NAME".to_string());
        commands.spawn((
            Text::new(if mine {
                said.clone()
            } else {
                format!("{said}  -  shipped")
            }),
            TextFont {
                font: fonts.text.clone().into(),
                font_size: FontSize::Px(10.0),
                ..default()
            },
            TextColor(theme::text_dim(&palette).with_alpha(0.75)),
            ChildOf(button),
        ));
        commands.spawn((
            crate::rail::Word(if mine {
                "Take it back out of the game"
            } else {
                "This one shipped inside the game - carry in your own of the same name to replace it"
            }),
            ChildOf(button),
        ));
        // Only a maker's own can be taken out. What shipped lives inside the
        // app, is replaced whole by the next update, and deleting it would be a
        // hole rather than a choice.
        if !mine {
            commands.entity(button).remove::<CarriedRow>();
        }
    }
}

/// A press on a row asks; a press on the card answers.
#[allow(clippy::too_many_arguments)]
pub(crate) fn take_one_back_out(
    mut commands: Commands,
    fonts: Res<Fonts>,
    palette: Res<Palette>,
    naming: Res<Naming>,
    mut stale: ResMut<CarriedStale>,
    rows: Query<(&Interaction, &CarriedRow), Changed<Interaction>>,
    cards: Query<Entity, With<RemovalCard>>,
    yes: Query<(&Interaction, &RemovalYes), Changed<Interaction>>,
    no: Query<&Interaction, (Changed<Interaction>, With<RemovalNo>)>,
) {
    // Answering first, so a press cannot both raise a card and answer it.
    if let Some((_, chosen)) = yes
        .iter()
        .find(|(touch, _)| **touch == Interaction::Pressed)
    {
        match std::fs::remove_file(&chosen.0.path) {
            Ok(()) => info!("took {} back out of the game", chosen.0.name),
            Err(why) => warn!("could not remove {}: {why}", chosen.0.path.display()),
        }
        stale.0 = true;
        for card in &cards {
            commands.entity(card).despawn();
        }
        return;
    }
    if no.iter().any(|touch| *touch == Interaction::Pressed) {
        for card in &cards {
            commands.entity(card).despawn();
        }
        return;
    }
    if !cards.is_empty() || naming.0.is_some() {
        return;
    }
    let Some((_, row)) = rows
        .iter()
        .find(|(touch, _)| **touch == Interaction::Pressed)
    else {
        return;
    };
    // The asking. Taking a building out of the village is not something to do on
    // one press of a row a maker might have brushed while reading it.
    let card = commands
        .spawn((
            RemovalCard,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(40.0),
                margin: UiRect {
                    left: Val::Px(-170.0),
                    ..default()
                },
                width: Val::Px(340.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(18.0)),
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme::panel_bg()),
            BorderColor::all(theme::accent(&palette).with_alpha(0.7)),
            GlobalZIndex(50),
        ))
        .id();
    commands.spawn((
        Text::new(format!("TAKE {} OUT OF THE GAME?", row.name.to_uppercase())),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(theme::accent(&palette)),
        ChildOf(card),
    ));
    commands.spawn((
        Text::new("the drawing on the bench is untouched - only the game's copy goes"),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: FontSize::Px(11.0),
            ..default()
        },
        TextColor(theme::text_dim(&palette).with_alpha(0.8)),
        ChildOf(card),
    ));
    let answers = commands
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
    for take in [true, false] {
        let button = commands
            .spawn((
                Interaction::default(),
                Node {
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::BLACK.with_alpha(0.18)),
                BorderColor::all(if take {
                    theme::accent(&palette).with_alpha(0.7)
                } else {
                    theme::panel_border(&palette)
                }),
                ChildOf(answers),
            ))
            .id();
        if take {
            commands.entity(button).insert(RemovalYes(row.clone()));
        } else {
            commands.entity(button).insert(RemovalNo);
        }
        commands.spawn((
            Text::new(if take { "TAKE IT OUT" } else { "LEAVE IT" }),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: FontSize::Px(12.0),
                ..default()
            },
            TextColor(if take {
                theme::accent(&palette)
            } else {
                theme::text_dim(&palette)
            }),
            ChildOf(button),
        ));
    }
}

/// A button that carries the work into the game.
#[derive(Component)]
pub(crate) struct BakeButton;

/// Where baked work is carried so a game can read it.
///
/// The project says where that is - only the game knows - and until it
/// does, the bake stops in the project's own `baked` folder and the
/// carrying is done by hand. The bench used to write straight into one
/// particular game's application-support folder, which is exactly the
/// assumption that kept it from serving any other.
///
/// NOT a bundle. A bundle is replaced whole on the next update and is not
/// writable besides, so a building baked into it would last until Tuesday.
pub(crate) fn carried_home(under: &str) -> std::path::PathBuf {
    match crate::project::install() {
        // `install` NAMES the folder - both manifests in the studio say
        // `.../assets/buildings` - so joining `under` onto it again wrote to
        // `.../assets/buildings/buildings`, which no game reads. Nothing had
        // caught it because nothing had yet baked through an install path.
        //
        // When clips need carrying too, the honest fix is for `install` to name
        // a ROOT and for every manifest to lose its last component; until then
        // one destination needs no sub-folder to tell it apart from the others.
        Some(into) => into,
        None => crate::project::baked().join(under),
    }
}
