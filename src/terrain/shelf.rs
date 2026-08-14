//! The terrain bench's shelf: the tools, the brush, and what is under the
//! pointer.
//!
//! The builder's shelf exactly — same edge, same width, same border on the one
//! side it touches — because a bench whose panels each float somewhere near an
//! edge is four benches that happen to share a window.

use bevy::prelude::*;

use crate::Bench;
use crate::look::{Fonts, Palette, theme};
use crate::terrain::edit::Brushing;
use crate::terrain::ground::Ground;
use crate::terrain::{Brush, MAX_RADIUS, MAX_STRENGTH, MIN_RADIUS, MIN_STRENGTH, Said};

/// The keys the six tools sit on, in order.
const KEYS: [&str; 8] = ["1", "2", "3", "4", "5", "6", "7", "8"];

/// Each tool wears a colour, and the ring on the ground wears the same one, so
/// what is under the pointer always matches what is lit on the shelf. Taken from
/// the open game's ramps like everything else here.
pub fn tool_colour(how: Brushing, palette: &Palette) -> Color {
    match how {
        Brushing::Raise => palette.shade("grass", 0.95),
        Brushing::Lower => palette.shade("cloth-rust", 0.9),
        Brushing::Smooth => palette.shade("cloth-blue", 0.95),
        Brushing::Flatten => palette.shade("cloth-gold", 0.95),
        Brushing::Path => palette.shade("cloth-purple", 0.95),
        Brushing::Roughen => palette.shade("cloth-teal", 0.95),
        Brushing::Erode => palette.shade("earth", 0.95),
        Brushing::Ramp => palette.shade("cloth-pink", 0.95),
    }
}

#[derive(Component)]
struct GroundShelf;

/// The button that asks for a world to shape.
#[derive(Component)]
pub struct OpenWorld;

#[derive(Component)]
struct ToolRow(Brushing);

#[derive(Component)]
struct ToolWord(Brushing);

/// A live number on the shelf. One component and one system, rather than a
/// marker type for each.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Reading {
    World,
    Radius,
    Strength,
    Where,
    High,
    Sculpted,
    History,
    Said,
}

pub struct ShelfPlugin;

impl Plugin for ShelfPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, hang_the_shelf)
            .add_systems(Update, (show_the_shelf, light_the_tool, say_the_numbers));
    }
}

fn hang_the_shelf(mut commands: Commands, fonts: Res<Fonts>, palette: Res<Palette>) {
    let shelf = commands
        .spawn((
            GroundShelf,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(crate::menu::BAR_HIGH),
                bottom: Val::Px(0.0),
                width: Val::Px(crate::look::PANEL_WIDE),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::left(Val::Px(1.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(theme::panel_bg()),
            BorderColor::all(theme::panel_border(&palette)),
            crate::look::Scrollable,
            bevy::ui::ScrollPosition::default(),
            Visibility::Hidden,
        ))
        .id();

    // Which world, first, because nothing below it means anything until one is
    // open. A world is not a project: the bench is a tool you bring ground to,
    // so this sits here rather than on the opening screen.
    heading(&mut commands, &fonts, &palette, shelf, "THE WORLD");
    commands.spawn((
        Reading::World,
        Text::new("none open"),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: crate::look::text_at(10.0),
            ..default()
        },
        TextColor(theme::text_dim(&palette)),
        Node {
            padding: UiRect::axes(Val::Px(6.0), Val::Px(1.0)),
            ..default()
        },
        ChildOf(shelf),
    ));
    let open = commands
        .spawn((
            OpenWorld,
            Button,
            Node {
                margin: UiRect::top(Val::Px(5.0)),
                padding: UiRect::axes(Val::Px(9.0), Val::Px(6.0)),
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.25)),
            BorderColor::all(theme::panel_border(&palette)),
            ChildOf(shelf),
        ))
        .id();
    commands.spawn((
        Text::new("OPEN A WORLD..."),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: crate::look::text_at(10.0),
            ..default()
        },
        TextColor(theme::accent(&palette)),
        ChildOf(open),
    ));
    commands.spawn((
        crate::rail::Word("Pick a world's map image - the folder it sits in is the world"),
        ChildOf(open),
    ));

    heading(&mut commands, &fonts, &palette, shelf, "THE TOOLS");
    for (key, how) in KEYS.iter().zip(Brushing::ALL) {
        tool_row(&mut commands, &fonts, &palette, shelf, key, how);
    }

    heading(&mut commands, &fonts, &palette, shelf, "THE BRUSH");
    reading(&mut commands, &fonts, &palette, shelf, "Radius", Reading::Radius);
    reading(&mut commands, &fonts, &palette, shelf, "Strength", Reading::Strength);

    heading(&mut commands, &fonts, &palette, shelf, "UNDER THE POINTER");
    reading(&mut commands, &fonts, &palette, shelf, "Where", Reading::Where);
    reading(&mut commands, &fonts, &palette, shelf, "Ground", Reading::High);

    heading(&mut commands, &fonts, &palette, shelf, "THE GROUND YOU MADE");
    reading(&mut commands, &fonts, &palette, shelf, "Sculpted", Reading::Sculpted);
    reading(&mut commands, &fonts, &palette, shelf, "History", Reading::History);

    heading(&mut commands, &fonts, &palette, shelf, "THE KEYS");
    for (keys, deed) in [
        ("Drag", "lay the tool down"),
        ("Right drag", "take it back off"),
        ("[ ]", "brush radius"),
        ("- =", "brush strength"),
        ("Shift drag", "turn the eye"),
        ("Middle drag", "pan, wheel zooms"),
        ("Shift 1-6", "the drafting angles"),
        ("Ctrl Z", "undo, Ctrl Y redo"),
        ("Ctrl S", "keep it"),
    ] {
        key_row(&mut commands, &fonts, &palette, shelf, keys, deed);
    }

    // What just happened, when it left no mark to look at. Kept last so it never
    // moves anything above it as it comes and goes.
    commands.spawn((
        Reading::Said,
        Text::new(""),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: crate::look::text_at(10.0),
            ..default()
        },
        TextColor(theme::accent(&palette)),
        Node {
            margin: UiRect::top(Val::Px(10.0)),
            ..default()
        },
        ChildOf(shelf),
    ));
}

fn heading(
    commands: &mut Commands,
    fonts: &Fonts,
    palette: &Palette,
    shelf: Entity,
    word: &str,
) {
    commands.spawn((
        Text::new(word.to_string()),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: crate::look::text_at(9.0),
            ..default()
        },
        TextColor(theme::text_dim(palette).with_alpha(0.55)),
        Node {
            margin: UiRect::top(Val::Px(10.0)).with_bottom(Val::Px(2.0)),
            ..default()
        },
        ChildOf(shelf),
    ));
}

fn tool_row(
    commands: &mut Commands,
    fonts: &Fonts,
    palette: &Palette,
    shelf: Entity,
    key: &str,
    how: Brushing,
) {
    let row = commands
        .spawn((
            ToolRow(how),
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            ChildOf(shelf),
        ))
        .id();

    // A boxed letter, so the key reads as a key rather than as the first word of
    // the tool's name.
    let cap = commands
        .spawn((
            Node {
                width: Val::Px(18.0),
                justify_content: JustifyContent::Center,
                padding: UiRect::axes(Val::Px(0.0), Val::Px(1.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.25)),
            BorderColor::all(theme::panel_border(palette)),
            ChildOf(row),
        ))
        .id();
    commands.spawn((
        Text::new(key.to_string()),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: crate::look::text_at(9.0),
            ..default()
        },
        TextColor(theme::text_dim(palette).with_alpha(0.7)),
        ChildOf(cap),
    ));

    commands.spawn((
        ToolWord(how),
        Text::new(how.name().to_string()),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: crate::look::text_at(10.0),
            ..default()
        },
        TextColor(theme::text_dim(palette)),
        ChildOf(row),
    ));
    commands.spawn((crate::rail::Word(how.said()), ChildOf(row)));
}

fn reading(
    commands: &mut Commands,
    fonts: &Fonts,
    palette: &Palette,
    shelf: Entity,
    label: &str,
    which: Reading,
) {
    let row = commands
        .spawn((
            Node {
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(10.0),
                padding: UiRect::axes(Val::Px(6.0), Val::Px(1.0)),
                ..default()
            },
            ChildOf(shelf),
        ))
        .id();
    commands.spawn((
        Text::new(label.to_string()),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: crate::look::text_at(10.0),
            ..default()
        },
        TextColor(theme::text_dim(palette).with_alpha(0.6)),
        ChildOf(row),
    ));
    commands.spawn((
        which,
        Text::new("-"),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: crate::look::text_at(10.0),
            ..default()
        },
        TextColor(theme::text_dim(palette)),
        ChildOf(row),
    ));
}

fn key_row(
    commands: &mut Commands,
    fonts: &Fonts,
    palette: &Palette,
    shelf: Entity,
    keys: &str,
    deed: &str,
) {
    let row = commands
        .spawn((
            Node {
                column_gap: Val::Px(8.0),
                padding: UiRect::axes(Val::Px(6.0), Val::Px(1.0)),
                ..default()
            },
            ChildOf(shelf),
        ))
        .id();
    commands.spawn((
        Text::new(keys.to_string()),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: crate::look::text_at(9.0),
            ..default()
        },
        TextColor(theme::text_dim(palette).with_alpha(0.75)),
        Node {
            width: Val::Px(74.0),
            ..default()
        },
        ChildOf(row),
    ));
    commands.spawn((
        Text::new(deed.to_string()),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: crate::look::text_at(9.0),
            ..default()
        },
        TextColor(theme::text_dim(palette).with_alpha(0.45)),
        ChildOf(row),
    ));
}

/// The shelf stands only at its own bench.
fn show_the_shelf(
    bench: Res<Bench>,
    showing: Res<crate::look::Showing>,
    mut shelves: Query<&mut Visibility, With<GroundShelf>>,
) {
    if !bench.is_changed() && !showing.is_changed() {
        return;
    }
    let out = *bench == Bench::Terrain && showing.wanted(crate::look::Tool::Shelf);
    for mut visibility in &mut shelves {
        *visibility = if out {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn light_the_tool(
    brush: Res<Brush>,
    palette: Res<Palette>,
    mut rows: Query<(&ToolRow, &mut BackgroundColor)>,
    mut words: Query<(&ToolWord, &mut TextColor)>,
) {
    if !brush.is_changed() {
        return;
    }
    for (row, mut background) in &mut rows {
        background.0 = if row.0 == brush.how {
            tool_colour(row.0, &palette).with_alpha(0.18)
        } else {
            Color::NONE
        };
    }
    for (word, mut colour) in &mut words {
        colour.0 = if word.0 == brush.how {
            tool_colour(word.0, &palette)
        } else {
            theme::text_dim(&palette)
        };
    }
}

fn say_the_numbers(
    brush: Res<Brush>,
    ground: Option<Res<Ground>>,
    said: Res<Said>,
    mut readings: Query<(&Reading, &mut Text)>,
) {
    let tally = ground
        .as_ref()
        .and_then(|ground| ground.sculpt().read().ok().map(|s| crate::terrain::tally(&s)));

    for (which, mut text) in &mut readings {
        let word = match which {
            Reading::World => match ground.as_ref() {
                Some(ground) => crate::terrain::opened::called(ground.folder()),
                None => "none open".to_string(),
            },
            Reading::Radius => bar(brush.radius, MIN_RADIUS, MAX_RADIUS, &format!("{:.0} m", brush.radius)),
            Reading::Strength => bar(
                brush.strength,
                MIN_STRENGTH,
                MAX_STRENGTH,
                &format!("{:.0} m/s", brush.strength),
            ),
            Reading::Where => match brush.on {
                Some(on) => format!("{:.0}, {:.0}", on.x, on.z),
                None => "off the ground".to_string(),
            },
            Reading::High => match brush.on {
                Some(on) => format!("{:.1} m", on.y),
                None => "-".to_string(),
            },
            Reading::Sculpted => match tally {
                Some((cells, unsaved, _, _)) => {
                    format!("{cells}{}", if unsaved { " unkept" } else { "" })
                }
                None => "-".to_string(),
            },
            Reading::History => match tally {
                Some((_, _, back, forward)) => match (back, forward) {
                    (false, false) => "nothing yet".to_string(),
                    (true, false) => "can undo".to_string(),
                    (false, true) => "can redo".to_string(),
                    (true, true) => "undo, redo".to_string(),
                },
                None => "-".to_string(),
            },
            Reading::Said => {
                if said.1 > 0.0 {
                    said.0.clone()
                } else {
                    String::new()
                }
            }
        };
        if text.0 != word {
            text.0 = word;
        }
    }
}

/// A number drawn as a short bar of blocks as well as said, so a glance is
/// enough to know whether the brush is small or large without reading it.
fn bar(value: f32, low: f32, high: f32, said: &str) -> String {
    // Placed on a LOG scale, because the radius runs four to six hundred and a
    // linear bar would spend nine tenths of itself on the top half of the range
    // and show no difference at all across the sizes most used.
    let t = ((value / low).ln() / (high / low).ln()).clamp(0.0, 1.0);
    let filled = (t * 6.0).round() as usize;
    let mut out = String::new();
    for i in 0..6 {
        out.push(if i < filled { '#' } else { '.' });
    }
    format!("{out}  {said}")
}
