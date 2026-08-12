//! THE OPENING — which game the bench is being asked to work for.
//!
//! The bench used to open whichever project it had opened last, without a word.
//! That was right while it served one game and wrong the moment it served two:
//! Divus Factus and Fly on the Wall look identical on the bench, and the only
//! thing that said which was a word in the title bar. Brett, opening it and
//! finding somebody else's game: "shouldnt it ask what project to open?"
//!
//! # It only asks when it has not been told
//!
//! A path on the command line, or in `OPIFICIUM_PROJECT`, is an instruction - from
//! a script, from the launcher, or from this screen reopening the bench - and it is
//! obeyed in silence. Only a bench with nothing to go on asks. See
//! [`crate::project::named_outright`].
//!
//! # Why it is its own little program
//!
//! The palette, the bodies and the shelf of saved work are all read while the
//! plugins are being built, which is before any Bevy interface could exist to ask a
//! question. So this runs as an app of its own - a window, a camera and a card,
//! nothing else - and when a game is chosen it relaunches the bench pointed at it.
//! A fresh process cannot have read one game's colours and then be handed another's,
//! which is the same reason the rail's own switcher relaunches.
//!
//! It paints in the bench's own ramps, because no game has been chosen yet and
//! there is nothing else to paint with. See [`crate::look::BENCH_RAMPS`].

use bevy::prelude::*;
use bevy::text::FontSize;

use crate::look::{Fonts, Palette, theme};

/// One game the bench has worked in before.
#[derive(Component)]
struct AGameBefore(std::path::PathBuf);

/// The button that goes looking for a game it has never seen.
#[derive(Component)]
struct AGameElsewhere;

/// Nothing today, thank you.
#[derive(Component)]
struct NoGameAtAll;

/// Asks which game, and never returns: whatever is chosen, the bench reopens
/// pointed at it, and this process is done.
pub fn ask() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Opificium".to_string(),
                // Smaller than the bench: this is a question, not a workshop.
                resolution: bevy::window::WindowResolution::new(720, 560),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(crate::look::LookPlugin)
        .add_systems(Startup, raise_the_question)
        .add_systems(Update, (light_the_choices, take_the_answer).chain())
        .run();
}

fn raise_the_question(mut commands: Commands, fonts: Res<Fonts>, palette: Res<Palette>) {
    commands.spawn(Camera2d);

    let sheet = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.035, 0.04, 0.05)),
        ))
        .id();

    commands.spawn((
        Text::new("THE OPIFICIUM"),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: FontSize::Px(26.0),
            ..default()
        },
        TextColor(theme::accent(&palette)),
        ChildOf(sheet),
    ));
    commands.spawn((
        Text::new(concat!(
            "the maker's own bench - v",
            env!("CARGO_PKG_VERSION")
        )),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: FontSize::Px(13.0),
            ..default()
        },
        TextColor(theme::text_dim(&palette)),
        Node {
            margin: UiRect::bottom(Val::Px(26.0)),
            ..default()
        },
        ChildOf(sheet),
    ));

    // The list, which is the whole point: a maker recognises a game by NAME, and
    // the bench has been keeping this list all along.
    let worked_in = crate::project::recent();
    commands.spawn((
        Text::new(if worked_in.is_empty() {
            "NO GAME YET"
        } else {
            "WHICH GAME?"
        }),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: FontSize::Px(12.0),
            ..default()
        },
        TextColor(theme::text_dim(&palette)),
        Node {
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        },
        ChildOf(sheet),
    ));

    for (which, road) in worked_in.iter().enumerate() {
        // The first is the one the bench WOULD have opened without asking, and
        // return still opens it - so somebody deep in one game pays a keystroke
        // rather than a decision.
        let button = a_choice(
            &mut commands,
            &fonts,
            &palette,
            sheet,
            crate::project::called(road).to_uppercase(),
            if which == 0 {
                Some("the last one you worked in - return")
            } else {
                None
            },
            which == 0,
        );
        commands.entity(button).insert(AGameBefore(road.clone()));
    }

    let elsewhere = a_choice(
        &mut commands,
        &fonts,
        &palette,
        sheet,
        "OPEN A GAME...".to_string(),
        Some("pick a game's own folder - the bench makes its own inside it"),
        worked_in.is_empty(),
    );
    commands.entity(elsewhere).insert(AGameElsewhere);

    let away = a_choice(
        &mut commands,
        &fonts,
        &palette,
        sheet,
        "NOT NOW".to_string(),
        Some("esc"),
        false,
    );
    commands.entity(away).insert(NoGameAtAll);
}

/// One thing a maker can choose, with a quiet word under it.
fn a_choice(
    commands: &mut Commands,
    fonts: &Fonts,
    palette: &Palette,
    sheet: Entity,
    label: String,
    tale: Option<&str>,
    lit: bool,
) -> Entity {
    let button = commands
        .spawn((
            Interaction::default(),
            Node {
                width: Val::Px(420.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(2.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(9.0)),
                margin: UiRect::bottom(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(if lit { 0.35 } else { 0.18 })),
            BorderColor::all(if lit {
                theme::accent(palette).with_alpha(0.7)
            } else {
                theme::panel_border(palette)
            }),
            ChildOf(sheet),
        ))
        .id();
    commands.spawn((
        Text::new(label),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(if lit {
            theme::accent(palette)
        } else {
            theme::text_dim(palette)
        }),
        ChildOf(button),
    ));
    if let Some(tale) = tale {
        commands.spawn((
            Text::new(tale),
            TextFont {
                font: fonts.text.clone().into(),
                font_size: FontSize::Px(11.0),
                ..default()
            },
            TextColor(theme::text_dim(palette).with_alpha(0.6)),
            ChildOf(button),
        ));
    }
    button
}

/// Whatever the hand rests on brightens.
fn light_the_choices(
    palette: Res<Palette>,
    mut choices: Query<(&Interaction, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (touch, mut fill) in &mut choices {
        let wanted = if *touch == Interaction::None {
            Color::BLACK.with_alpha(0.18)
        } else {
            theme::accent(&palette).with_alpha(0.16)
        };
        if fill.0 != wanted {
            *fill = BackgroundColor(wanted);
        }
    }
}

/// Carries out the choice, and leaves.
#[allow(clippy::too_many_arguments)]
fn take_the_answer(
    _main_thread: bevy::ecs::system::NonSendMarker,
    keys: Res<ButtonInput<KeyCode>>,
    mut leaving: MessageWriter<AppExit>,
    before: Query<(&Interaction, &AGameBefore)>,
    elsewhere: Query<&Interaction, With<AGameElsewhere>>,
    away: Query<&Interaction, With<NoGameAtAll>>,
    first: Query<&AGameBefore>,
) {
    let pressed = |touch: &Interaction| *touch == Interaction::Pressed;

    // Nothing today. The bench closes rather than opening on a game nobody asked
    // for, which is the whole complaint this screen answers.
    if away.iter().any(pressed) || keys.just_pressed(KeyCode::Escape) {
        leaving.write(AppExit::Success);
        return;
    }

    // Return takes the one the bench would have opened on its own.
    let by_key = keys
        .just_pressed(KeyCode::Enter)
        .then(|| first.iter().next().map(|game| game.0.clone()))
        .flatten();

    let chosen = by_key
        .or_else(|| {
            before
                .iter()
                .find(|(touch, _)| pressed(touch))
                .map(|(_, game)| game.0.clone())
        })
        .or_else(|| {
            if !elsewhere.iter().any(pressed) {
                return None;
            }
            // A GAME's own folder, not the bench's inside it: a maker knows where
            // their game is and should not have to know what this program calls
            // the corner of it that it works in.
            let picked = rfd::FileDialog::new()
                .set_title("Open a game's folder")
                .pick_folder()?;
            match crate::project::start_a_project(&picked) {
                Ok(root) => Some(root),
                Err(why) => {
                    warn!("could not start a project in {}: {why}", picked.display());
                    None
                }
            }
        });

    let Some(road) = chosen else {
        return;
    };
    // Reopened rather than continued: see this module's own word on why.
    match crate::project::relaunch(&road) {
        Ok(_) => {
            info!("opening {}", road.display());
            leaving.write(AppExit::Success);
        }
        Err(why) => warn!("could not open {}: {why}", road.display()),
    }
}
