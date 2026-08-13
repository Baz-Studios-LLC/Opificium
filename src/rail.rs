//! The bench rail: the strip of benches down the left edge, in the codex's
//! own dress. New tools become new benches here.

use bevy::prelude::*;

use crate::Bench;
use crate::look::{Fonts, Palette, theme};

/// A button that walks the maker to a bench.
#[derive(Component)]
struct BenchButton(Bench);

/// A button on the top bar that sets the tool mode.
#[derive(Component)]
struct ModeButton(crate::gizmo::ToolMode);

/// A step of the build to set out, by its place in the list.
#[derive(Component, Clone, Copy)]
struct StageButton(usize);

/// The buttons that change how many steps there are.
#[derive(Component, Clone, Copy)]
struct StageDeedButton(crate::builder::StageDeed);

/// The row itself, rebuilt whenever the number of steps changes.
#[derive(Component)]
pub struct StageBar;

/// The row of modes along the top. Buildings only: they move, resize and paint
/// parts, and a body has none. Brett: "the top bar needs differnt buttons on
/// top, those are for buildings."
#[derive(Component)]
pub struct ModeBar;

/// A button that leaves this project and opens one the bench has worked in
/// before.
#[derive(Component)]
struct ProjectButton(std::path::PathBuf);

/// The button that asks for a game's folder.
#[derive(Component)]
pub(crate) struct OpenProjectButton;

/// The gear at the rail's foot, and the settings panel it opens.
#[derive(Component)]
pub(crate) struct SettingsButton;

#[derive(Component)]
struct SettingsPanel;

pub struct RailPlugin;

impl Plugin for RailPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, raise_rail).add_systems(
            Update,
            (
                work_buttons,
                work_mode_bar,
                work_settings,
                work_stage_bar,
                hang_the_stage_bar,
                follow_with_a_word,
                work_projects,
            ),
        );
    }
}

pub fn raise_rail(mut commands: Commands, fonts: Res<Fonts>, palette: Res<Palette>) {
    // The mode bar, top centre, the way the big programs wear it.
    //
    // Centred by a full-width row rather than by arithmetic. It used to be three
    // hundred and twenty pixels wide with a hundred and sixty of negative margin
    // pulling it back over the middle - true only while the buttons happened to
    // add up to that, and the fourth one spilled straight out of the panel it
    // was supposed to be inside. A hard-coded width is a measurement of the
    // contents kept somewhere the contents cannot reach.
    let centring = commands
        .spawn((
            ModeBar,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(crate::menu::BAR_HIGH),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .id();
    let bar = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect {
                    left: Val::Px(1.0),
                    right: Val::Px(1.0),
                    bottom: Val::Px(1.0),
                    ..default()
                },
                ..default()
            },
            BackgroundColor(theme::panel_bg()),
            BorderColor::all(theme::panel_border(&palette)),
            ChildOf(centring),
        ))
        .id();
    /// The width every mode button takes, whatever its word.
    const MODE_BUTTON_WIDTH: f32 = 88.0;
    for (mode, label) in [
        (crate::gizmo::ToolMode::Normal, "NORMAL"),
        (crate::gizmo::ToolMode::Move, "MOVE"),
        (crate::gizmo::ToolMode::Resize, "RESIZE"),
        (crate::gizmo::ToolMode::Paint, "PAINT"),
    ] {
        let button = commands
            .spawn((
                ModeButton(mode),
                Interaction::default(),
                Node {
                    // One width for all four, rather than each sized to its own
                    // word - a row of buttons that step in and out as the eye
                    // runs along them reads as four unrelated things. Wide
                    // enough for NORMAL, which is the longest of them.
                    width: Val::Px(MODE_BUTTON_WIDTH),
                    padding: UiRect::vertical(Val::Px(5.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::BLACK.with_alpha(0.18)),
                BorderColor::all(theme::panel_border(&palette)),
                ChildOf(bar),
            ))
            .id();
        commands.spawn((
            Text::new(label),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: crate::look::text_at(12.0),
                ..default()
            },
            TextColor(theme::accent(&palette)),
            ChildOf(button),
        ));
    }

    // The build steps, on their own row under the modes. Empty here and filled
    // by `hang_the_stage_bar`, because how many steps there are is a property of
    // the work on the bench rather than of the bench itself: a work opens with
    // as many as its file declares, and a maker adds and drops them while
    // working.
    commands.spawn((
        StageBar,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            bottom: Val::Px(0.0),
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            ..default()
        },
    ));

    let rail = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(crate::menu::BAR_HIGH),
                bottom: Val::Px(0.0),
                // The shelf's own width. The two stand either side of the same
                // stage and a bench with mismatched margins reads as a bench
                // that was assembled rather than drawn.
                width: Val::Px(crate::look::PANEL_WIDE),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(14.0)),
                row_gap: Val::Px(8.0),
                border: UiRect::right(Val::Px(1.0)),
                // Without this the wheel did nothing here. `Scrollable` and a
                // `ScrollPosition` are only half of it - a scroll position
                // moves a node's children only if the node CLIPS them, and a
                // node with no overflow set does not. So the rail took the
                // wheel, told the camera to keep its hands off the zoom, and
                // then sat still: the one arrangement that looks like the
                // scroll is broken rather than absent.
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(theme::panel_bg()),
            BorderColor::all(theme::panel_border(&palette)),
            crate::look::Scrollable,
            bevy::ui::ScrollPosition::default(),
        ))
        .id();

    commands.spawn((
        Text::new("THE OPIFICIUM"),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: crate::look::text_at(20.0),
            ..default()
        },
        TextColor(theme::accent(&palette)),
        ChildOf(rail),
    ));
    commands.spawn((
        Text::new(concat!(
            "the maker's own bench - v",
            env!("CARGO_PKG_VERSION")
        )),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: crate::look::text_at(12.0),
            ..default()
        },
        TextColor(theme::text_dim(&palette)),
        Node {
            margin: UiRect::bottom(Val::Px(14.0)),
            ..default()
        },
        ChildOf(rail),
    ));

    // WHICH GAME.
    //
    // The bench holds no game's content, so this is the largest thing on the
    // screen that can be wrong: every colour, every body and every saved work
    // comes out of the project named here. It could only be said on the command
    // line until now, and the window's title bar was the one place it was written
    // down at all - Brett: "this app should support multiple programs, How do I
    // switch it?" A bench that serves any game has to be able to say which one it
    // is serving, and let a hand change it.
    commands.spawn((
        Text::new("THE PROJECT"),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: crate::look::text_at(11.0),
            ..default()
        },
        TextColor(theme::text_dim(&palette)),
        Node {
            // Cancels the lead a drawer head carries, so this label and the name
            // under it sit at the rail's own spacing and read as one thing. A
            // drawer's margin is there to hold it off whatever came before, and
            // what came before this one is its own word.
            margin: UiRect::bottom(Val::Px(-8.0)),
            ..default()
        },
        ChildOf(rail),
    ));
    // The open project's own name IS the head of the drawer, and the others drop
    // out of it - Brett: "this list could get rather long, could we make this a
    // drop down?" A list of every game a maker has ever opened, standing open at
    // the top of the rail, pushes the benches themselves off the bottom of it.
    //
    // The shelf's own drawer, not a second kind of thing that opens: same `+` and
    // `-`, same press to work it, and `work_drawers` already does the working. The
    // one difference is that this head carries a VALUE rather than a category,
    // which is why the dim word above it says what the value is.
    let standing = crate::project::current();
    let inside = crate::builder::drawer(
        &mut commands,
        &fonts,
        &palette,
        rail,
        standing
            .as_ref()
            .map(|project| project.name.to_uppercase())
            .unwrap_or_else(|| "NONE OPEN".to_string()),
        false,
    );

    // Every one the bench has worked in - all twelve it keeps, not a handful.
    // Closed, the drawer costs one line however many there are, which is the whole
    // reason it is a drawer.
    let here = standing.as_ref().map(|project| project.root.clone());
    for road in crate::project::recent()
        .into_iter()
        .filter(|road| Some(road) != here.as_ref())
    {
        let button = project_face(
            &mut commands,
            &fonts,
            &palette,
            inside,
            crate::project::called(&road).to_uppercase(),
            false,
        );
        commands.entity(button).insert((
            ProjectButton(road),
            Word("Leave this project and open that one"),
        ));
    }
    let open = project_face(
        &mut commands,
        &fonts,
        &palette,
        inside,
        "OPEN A GAME...".to_string(),
        true,
    );
    commands.entity(open).insert((
        OpenProjectButton,
        Word(
            "Pick a game's own folder. The bench makes its \
             opificium folder inside it and works from there",
        ),
    ));
    // A hand's breadth before the benches. It cannot hang off the last button in
    // the drawer: that button is inside the body, and a closed body has no
    // margins to give.
    commands.spawn((
        Node {
            height: Val::Px(6.0),
            ..default()
        },
        ChildOf(rail),
    ));

    for (bench, label, tale) in [
        (Bench::Builder, "THE BUILDER", "boxes, ramps and widgets"),
        (Bench::Kiln, "THE KILN", "an image in, a model out"),
        (Bench::Rig, "THE RIG", "a model, looked at closely"),
    ] {
        let button = commands
            .spawn((
                BenchButton(bench),
                Interaction::default(),
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                BackgroundColor(Color::BLACK.with_alpha(0.18)),
                BorderColor::all(theme::panel_border(&palette)),
                ChildOf(rail),
            ))
            .id();
        commands.spawn((
            Text::new(label),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: crate::look::text_at(13.0),
                ..default()
            },
            TextColor(theme::accent(&palette)),
            ChildOf(button),
        ));
        commands.spawn((
            Text::new(tale),
            TextFont {
                font: fonts.text.clone().into(),
                font_size: crate::look::text_at(11.0),
                ..default()
            },
            TextColor(theme::text_dim(&palette)),
            ChildOf(button),
        ));
    }

    // The footer: how the bench speaks to the world.
    let foot = commands
        .spawn((
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                row_gap: Val::Px(3.0),
                ..default()
            },
            ChildOf(rail),
        ))
        .id();
    // The settings panel: hidden until the gear is pressed, carrying
    // the keybinds so the rail itself stays short.
    let panel = commands
        .spawn((
            SettingsPanel,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(200.0),
                bottom: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                padding: UiRect::all(Val::Px(14.0)),
                border: UiRect::all(Val::Px(1.0)),
                // As above: scrollable, and now actually able to scroll.
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(theme::panel_bg()),
            BorderColor::all(theme::accent(&palette).with_alpha(0.5)),
            Visibility::Hidden,
            GlobalZIndex(40),
            crate::look::Scrollable,
            bevy::ui::ScrollPosition::default(),
        ))
        .id();
    commands.spawn((
        Text::new("KEYBINDS"),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: crate::look::text_at(13.0),
            ..default()
        },
        TextColor(theme::accent(&palette)),
        Node {
            margin: UiRect::bottom(Val::Px(2.0)),
            ..default()
        },
        ChildOf(panel),
    ));
    for (cap, tale) in [
        ("click", "place, or pick back up"),
        ("X / del", "remove at the cursor"),
        ("cmd Z / Y", "undo and redo"),
        (
            "cmd C / V",
            "copy what you point at,
paste it into the hand",
        ),
        ("R", "turn a quarter"),
        ("SHIFT R", "turn the whole work"),
        ("T / SHIFT T", "lean it, and lean it back"),
        (
            "M",
            "mirror it - the far half
of a gable, the other hand
of an L",
        ),
        ("Q / E", "lift and lower"),
        ("[ ]", "repaint through the ramps"),
        ("- =", "darker and brighter"),
        (
            "PAINT",
            "colour what is already
standing: click a part,
shift-click the lot,
\\ empties the brush",
        ),
        (
            "alt-click",
            "the dropper: the brush
takes the colour of the
piece under the cursor,
painted or authored",
        ),
        (
            "right-click",
            "a part's menu, or a saved
work's - group, ungroup,
trim, bury",
        ),
        (
            "shift-click",
            "choose several; right-
click to group them",
        ),
        ("esc / del", "empty the hand"),
        (
            "del",
            "with a part chosen and
an empty hand: bury it",
        ),
        (
            "1 - 6",
            "front, right, back, left,\noverhead, and the perch",
        ),
        ("shift", "fine snap, 1 unit"),
        (
            "G",
            "cycle the grid interval:
1, 2, 4, 8, 16 units",
        ),
        ("F", "face snap on and off"),
        (
            "H",
            "the cutaway: whole, roof
off, walls down as well",
        ),
        ("tab", "normal, move, resize;\nclick selects, drag a handle"),
        ("D", "type exact dimensions\nof the resize selection"),
        ("RMB drag", "swing the camera"),
        ("MMB drag", "pull the bench along"),
        ("wheel", "draw near, pull away"),
        ("WASD", "glide over the bench"),
    ] {
        let row = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                },
                ChildOf(panel),
            ))
            .id();
        let chip = commands
            .spawn((
                Node {
                    min_width: Val::Px(52.0),
                    padding: UiRect::axes(Val::Px(5.0), Val::Px(1.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::Center,
                    flex_shrink: 0.0,
                    ..default()
                },
                BackgroundColor(Color::BLACK.with_alpha(0.35)),
                BorderColor::all(theme::panel_border(&palette)),
                ChildOf(row),
            ))
            .id();
        // The machine's own face on the caps, the way manuals do it.
        commands.spawn((
            Text::new(cap),
            TextFont {
                font_size: crate::look::text_at(10.0),
                ..default()
            },
            TextColor(theme::accent(&palette).with_alpha(0.95)),
            ChildOf(chip),
        ));
        commands.spawn((
            Text::new(tale),
            TextFont {
                font: fonts.text.clone().into(),
                font_size: crate::look::text_at(11.0),
                ..default()
            },
            TextColor(theme::text_dim(&palette).with_alpha(0.85)),
            ChildOf(row),
        ));
    }
    // The tools that act on the WORK rather than on the bench, along the foot
    // beside the gear. Brett: "Cluld these buttons be iconized and down by the
    // settings button?" - they were three of the widest words on the rail and
    // none of them is a part, which is what the shelf above is for.
    let tools = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                ..default()
            },
            ChildOf(foot),
        ))
        .id();
    // The gear: a drawn sliders glyph, since the fonts keep no gear.
    let gear = icon_face(&mut commands, &palette, tools);
    commands
        .entity(gear)
        .insert((SettingsButton, Word("The keys, and what they do")));
    for offset in [-6.5f32, 4.0, -2.5] {
        let bar = commands
            .spawn((
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(2.0),
                    ..default()
                },
                BackgroundColor(theme::text_dim(&palette).with_alpha(0.8)),
                ChildOf(gear),
            ))
            .id();
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(12.0 + offset),
                top: Val::Px(-2.0),
                width: Val::Px(6.0),
                height: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(theme::accent(&palette)),
            ChildOf(bar),
        ));
    }

    // SAVE: a chest with its plate, the one glyph everybody already reads as
    // keeping a thing.
    let save = icon_face(&mut commands, &palette, tools);
    commands
        .entity(save)
        .insert((
            crate::builder::SaveButton,
            Word("Save the work under a name you give it"),
        ))
        .insert(BorderColor::all(theme::accent(&palette).with_alpha(0.7)));
    let body = pane(&mut commands, save, 24.0, 21.0, true, &palette);
    commands.entity(body).insert(Node {
        width: Val::Px(24.0),
        height: Val::Px(21.0),
        border: UiRect::all(Val::Px(1.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::FlexEnd,
        padding: UiRect::bottom(Val::Px(3.0)),
        ..default()
    });
    plate(&mut commands, body, 13.0, 7.0, theme::accent(&palette));

    // OPEN A WORK: a page with its lines, and the desktop's own file window
    // behind it.
    let open = icon_face(&mut commands, &palette, tools);
    commands.entity(open).insert((
        crate::builder::OpenWorkButton,
        Word("Open a work from anywhere on the disk"),
    ));
    let page = pane(&mut commands, open, 18.0, 24.0, true, &palette);
    commands.entity(page).insert(Node {
        width: Val::Px(18.0),
        height: Val::Px(24.0),
        border: UiRect::all(Val::Px(1.0)),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        row_gap: Val::Px(4.0),
        ..default()
    });
    for _ in 0..3 {
        plate(&mut commands, page, 11.0, 2.0, theme::text_dim(&palette));
    }

    // BAKE: a house going down into a tray, which is what carrying a work into
    // the game is. It sits beside the save because it is the other half of
    // keeping a thing: one for the bench, one for the world.
    let bake = icon_face(&mut commands, &palette, tools);
    commands.entity(bake).insert((
        crate::builder::BakeButton,
        // What the word MEANS, not the word again. Every other glyph on this row
        // says what pressing it does, and this one had been cut back to its own
        // label - which tells a maker nothing they could not see, about the one
        // button here whose name is a term of art. Brett: "I am not sure I really
        // understand what baking is?"
        Word(
            "Bake: turn this building into a file the game can \
             read, and put it where the game reads it",
        ),
    ));
    let stack = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(2.0),
                ..default()
            },
            ChildOf(bake),
        ))
        .id();
    plate(&mut commands, stack, 12.0, 8.0, theme::accent(&palette));
    plate(&mut commands, stack, 4.0, 5.0, theme::text_dim(&palette));
    pane(&mut commands, stack, 22.0, 6.0, true, &palette);

    // THE BROOM: a handle and a head. It sweeps the bench, and it is the one
    // here that takes something away, so it wears the dimmest border of the
    // four - nothing about it should invite a stray hand.
    let broom = icon_face(&mut commands, &palette, tools);
    commands.entity(broom).insert((
        crate::builder::ClearButton,
        Word(
            "Clear the bench and start again: every part, \
             every phase, every level. What was there is kept in \
             the project's workbench file first",
        ),
    ));
    let stack = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(1.0),
                ..default()
            },
            ChildOf(broom),
        ))
        .id();
    plate(&mut commands, stack, 3.0, 13.0, theme::text_dim(&palette));
    plate(&mut commands, stack, 16.0, 5.0, theme::accent(&palette));

    // The line under them says what the hand is over, and the bench's own word
    // when it is over nothing. Which is where the save's answer lands too: a
    // button with no writing on it cannot tell anybody what it just did.
    commands.spawn((
        crate::builder::SaveLabel,
        FootWord,
        Text::new(FOOT_SAYING),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: crate::look::text_at(11.0),
            ..default()
        },
        TextColor(theme::text_dim(&palette).with_alpha(0.6)),
        Node {
            margin: UiRect::top(Val::Px(8.0)),
            ..default()
        },
        ChildOf(foot),
    ));
}

/// What the foot says when nothing is under the hand.
pub const FOOT_SAYING: &str = "what you save here, the god carries into the world by hand.";

/// The line at the foot of the rail. The save's answer lands here - a glyph
/// with no writing on it cannot say what it just did.
#[derive(Component)]
struct FootWord;

/// What a thing says when the hand rests on it.
///
/// Hung on the button rather than looked up by type, so anything at all can
/// carry a word: the five glyphs do, and the next one will by being spawned
/// with it.
#[derive(Component)]
pub struct Word(pub &'static str);

/// The floating card that shows it, and follows the cursor.
#[derive(Component)]
struct Tooltip;

/// How long a hand rests on a thing before it is asking about it.
///
/// Brett: "maybe it takes a second before it pops up". A tooltip that appears
/// the instant the cursor crosses a button follows the hand around the screen
/// like a fly, and a maker crossing the row to reach the broom would raise four
/// cards on the way.
const DWELL: f32 = 0.6;

/// One project button: the full width of the rail, since what it carries is a
/// name of unknown length rather than a word somebody chose to fit.
///
/// `bright` is for the one that opens a game the bench has never seen. It is the
/// only button here that does something a maker cannot undo by pressing another
/// one, and it is the one they are looking for the first time.
fn project_face(
    commands: &mut Commands,
    fonts: &Fonts,
    palette: &Palette,
    parent: Entity,
    label: String,
    bright: bool,
) -> Entity {
    let button = commands
        .spawn((
            Interaction::default(),
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.18)),
            BorderColor::all(if bright {
                theme::accent(palette).with_alpha(0.6)
            } else {
                theme::panel_border(palette)
            }),
            ChildOf(parent),
        ))
        .id();
    commands.spawn((
        Text::new(label),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: crate::look::text_at(11.0),
            ..default()
        },
        TextColor(if bright {
            theme::accent(palette)
        } else {
            theme::text_dim(palette)
        }),
        ChildOf(button),
    ));
    button
}

/// Opens another project: the picker, and the ones worked in before.
///
/// The bench LEAVES rather than swapping - see `project::relaunch` for why - so
/// the last thing it does here is keep whatever is standing, because nothing on
/// the bench is written automatically and a maker who has drawn for an hour
/// should not lose it to a button that looks harmless.
fn work_projects(
    _main_thread: bevy::ecs::system::NonSendMarker,
    mut leaving: MessageWriter<AppExit>,
    stages: Res<crate::builder::Stages>,
    work_name: Res<crate::builder::WorkName>,
    standing: Query<&crate::builder::Placed, Without<crate::builder::Ghost>>,
    picks: Query<&Interaction, (Changed<Interaction>, With<OpenProjectButton>)>,
    recents: Query<(&Interaction, &ProjectButton), Changed<Interaction>>,
) {
    let wanted = if picks.iter().any(|touch| *touch == Interaction::Pressed) {
        // A GAME's folder, not the bench's own inside it: the maker knows where
        // their game is and should not have to know what this program calls the
        // corner of it that it works in.
        let Some(picked) = rfd::FileDialog::new()
            .set_title("Open a game's folder")
            .pick_folder()
        else {
            return;
        };
        match crate::project::start_a_project(&picked) {
            Ok(root) => Some(root),
            Err(why) => {
                warn!("could not start a project in {}: {why}", picked.display());
                return;
            }
        }
    } else {
        recents
            .iter()
            .find(|(touch, _)| **touch == Interaction::Pressed)
            .map(|(_, button)| button.0.clone())
    };
    let Some(root) = wanted else {
        return;
    };

    if let Some(kept) =
        crate::builder::keep_the_bench(&stages, standing.iter(), work_name.0.as_deref())
    {
        info!("kept the bench at {}", kept.display());
    }
    match crate::project::relaunch(&root) {
        Ok(_) => {
            info!("opening {}", root.display());
            leaving.write(AppExit::Success);
        }
        Err(why) => warn!("could not open {}: {why}", root.display()),
    }
}

/// One icon button's frame: the gear's own, so the four read as one row.
///
/// They grew when the folder left. Four glyphs at the old size left a third of
/// the rail's width empty beside them, and a glyph is a picture - the bigger it
/// is drawn the less it has to be guessed at.
fn icon_face(commands: &mut Commands, palette: &Palette, parent: Entity) -> Entity {
    commands
        .spawn((
            Interaction::default(),
            Node {
                width: Val::Px(44.0),
                height: Val::Px(38.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.18)),
            BorderColor::all(theme::panel_border(palette)),
            ChildOf(parent),
        ))
        .id()
}

/// A drawn rectangle, filled.
fn plate(commands: &mut Commands, parent: Entity, w: f32, h: f32, dye: Color) -> Entity {
    commands
        .spawn((
            Node {
                width: Val::Px(w),
                height: Val::Px(h),
                ..default()
            },
            BackgroundColor(dye),
            ChildOf(parent),
        ))
        .id()
}

/// A drawn rectangle, outlined.
fn pane(
    commands: &mut Commands,
    parent: Entity,
    w: f32,
    h: f32,
    lined: bool,
    palette: &Palette,
) -> Entity {
    commands
        .spawn((
            Node {
                width: Val::Px(w),
                height: Val::Px(h),
                border: UiRect::all(Val::Px(if lined { 1.0 } else { 0.0 })),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.35)),
            BorderColor::all(theme::accent(palette).with_alpha(0.8)),
            ChildOf(parent),
        ))
        .id()
}

/// The tooltip: a card at the cursor, saying what is under it.
///
/// It TRACKS rather than sits, which is Brett's call and the right one - "these
/// tooltips shoul be hovering tooltips like tooltips in wow that are tracking
/// the mouse". A word at the foot of the rail asks the eye to leave the very
/// thing it is asking about, and by the time it has read the word the hand has
/// moved on.
///
/// An icon with no writing on it is a guess until somebody has pressed it once,
/// and the one that empties the bench is not a button to learn by pressing.
#[allow(clippy::too_many_arguments)]
fn follow_with_a_word(
    mut commands: Commands,
    time: Res<Time>,
    fonts: Res<Fonts>,
    palette: Res<Palette>,
    windows: Query<&Window>,
    hovered: Query<(&Interaction, &Word)>,
    cards: Query<Entity, With<Tooltip>>,
    mut nodes: Query<&mut Node, With<Tooltip>>,
    mut showing: Local<Option<&'static str>>,
    mut resting: Local<f32>,
) {
    let under = hovered
        .iter()
        .find(|(touch, _)| **touch != Interaction::None)
        .map(|(_, word)| word.0);
    // The rest has to be on ONE thing: crossing from a button to its neighbour
    // starts the count again, or a hand travelling the row would arrive at the
    // far end with a card already up for something it passed.
    *resting = match (under, *showing) {
        (Some(word), Some(shown)) if word == shown => *resting + time.delta_secs(),
        (Some(_), _) => 0.0,
        (None, _) => 0.0,
    };
    let wanted = under.filter(|_| *resting >= DWELL);

    // Raised and dropped rather than hidden and shown: a tooltip exists while
    // the hand is somewhere, and one kept about with its visibility off is one
    // more thing to remember to keep true.
    let up = !cards.is_empty();
    if under != *showing || up != wanted.is_some() {
        for card in &cards {
            commands.entity(card).despawn();
        }
        if under != *showing {
            *showing = under;
            *resting = 0.0;
        }
        if let Some(word) = wanted {
            let card = commands
                .spawn((
                    Tooltip,
                    Node {
                        position_type: PositionType::Absolute,
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        max_width: Val::Px(320.0),
                        ..default()
                    },
                    BackgroundColor(theme::panel_bg().with_alpha(0.97)),
                    BorderColor::all(theme::panel_border(&palette)),
                    GlobalZIndex(200),
                ))
                .id();
            commands.spawn((
                Text::new(word),
                TextFont {
                    font: fonts.text.clone().into(),
                    font_size: crate::look::text_at(16.0),
                    ..default()
                },
                TextColor(theme::accent(&palette).with_alpha(0.9)),
                ChildOf(card),
            ));
        }
    }
    // And it follows. Below and right of the point, the way every tooltip does,
    // so the cursor never covers the first word of what it raised.
    let Some(at) = windows.iter().next().and_then(|w| w.cursor_position()) else {
        return;
    };
    for mut node in &mut nodes {
        node.left = Val::Px(at.x + 16.0);
        node.top = Val::Px(at.y + 20.0);
    }
}

/// One button of the step row.
/// The width every button on the stage bar takes, step or deed alike. Wide
/// enough for STAGE 10, which is further than any building has needed to go.
///
/// One width, because the row is one row: buttons sized to their own words step in
/// and out as the eye runs along them, and a bare "+" beside "STAGE 10" would make
/// the smallest and the largest of them neighbours. The glyphs are the narrowest
/// things on it now, which is exactly why they are not allowed to shrink.
const STAGE_BUTTON_WIDTH: f32 = 80.0;

fn stage_face(
    commands: &mut Commands,
    fonts: &Fonts,
    palette: &Palette,
    parent: Entity,
    label: String,
    lead: f32,
) -> Entity {
    let button = commands
        .spawn((
            Interaction::default(),
            Node {
                width: Val::Px(STAGE_BUTTON_WIDTH),
                padding: UiRect::vertical(Val::Px(5.0)),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                // A gap before the deeds, so a miss lands on nothing rather
                // than on a step being deleted.
                margin: UiRect::left(Val::Px(lead)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.30)),
            BorderColor::all(theme::panel_border(palette)),
            ChildOf(parent),
        ))
        .id();
    commands.spawn((
        Text::new(label),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: crate::look::text_at(12.0),
            ..default()
        },
        TextColor(theme::accent(palette)),
        ChildOf(button),
    ));
    button
}

/// Rebuilds the row when the number of steps changes.
///
/// A work opens with as many steps as its file says, and a maker adds and drops
/// them while working, so the row cannot be built once at startup and left.
fn hang_the_stage_bar(
    mut commands: Commands,
    fonts: Res<Fonts>,
    palette: Res<Palette>,
    stages: Res<crate::builder::Stages>,
    bars: Query<Entity, With<StageBar>>,
    mut hung: Local<usize>,
) {
    let count = stages.count();
    if count == *hung && !stages.is_changed() {
        return;
    }
    if count == *hung {
        return;
    }
    *hung = count;
    let Some(row) = bars.iter().next() else {
        return;
    };
    commands.entity(row).despawn_related::<Children>();
    // The same panel the modes wear, on the floor rather than the ceiling: one
    // border along the edge it does not touch, and the buttons inside it. A row
    // of bare buttons on the background was the odd one out on this screen.
    let bar = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect {
                    left: Val::Px(1.0),
                    right: Val::Px(1.0),
                    top: Val::Px(1.0),
                    ..default()
                },
                ..default()
            },
            BackgroundColor(theme::panel_bg()),
            BorderColor::all(theme::panel_border(&palette)),
            ChildOf(row),
        ))
        .id();

    for step in 0..count {
        let button = stage_face(
            &mut commands,
            &fonts,
            &palette,
            bar,
            format!("STAGE {}", step + 1),
            0.0,
        );
        commands.entity(button).insert(StageButton(step));
    }
    // Adding and dropping sit apart from the steps, so a miss lands on nothing
    // rather than on a step being deleted.
    for (deed, label) in [
        // The glyph alone. "+ STEP" beside "STAGE 3" read as two kinds of word
        // where one of them is a picture - Brett: "we can have the buttons just be +
        // and - no need for the word step".
        (crate::builder::StageDeed::Add, "+"),
        (crate::builder::StageDeed::Drop, "-"),
        // A step taken from here and put on another. The `+` pair make a NEW
        // step; these two change one that already stands, which is a different
        // job and reads as one.
        (crate::builder::StageDeed::Take, "TAKE"),
        (crate::builder::StageDeed::Put, "PUT"),
    ] {
        let button = stage_face(
            &mut commands,
            &fonts,
            &palette,
            bar,
            label.to_string(),
            // A gap before the deeds, so a miss lands on nothing rather than on a
            // step being dropped. It used to be measured against "+ COPY", a button
            // that has not existed for some time, so it was never applied at all.
            if label == "+" { 12.0 } else { 0.0 },
        );
        commands.entity(button).insert(StageDeedButton(deed));
    }
}

/// Step presses set out that step; the standing step wears the gold.
fn work_stage_bar(
    palette: Res<Palette>,
    stages: Res<crate::builder::Stages>,
    mut wish: ResMut<crate::builder::StageWish>,
    // On the CHANGE into Pressed, not on Pressed. Bevy holds `Pressed` for as long
    // as the button is held down, so one press of `+` was read on every frame until
    // the row was rebuilt with fresh buttons - and the rebuild lags a frame, so a
    // single click added two steps. Brett: "when there is one stage and you press +
    // it adds two instead of 1."
    //
    // The step buttons below need no such guard: pressing STAGE 2 twice asks to show
    // the step already showing, which the test beside them refuses.
    deeds: Query<(&Interaction, &StageDeedButton), Changed<Interaction>>,
    mut buttons: Query<(
        &Interaction,
        &StageButton,
        &mut BorderColor,
        &mut BackgroundColor,
    )>,
) {
    for (interaction, button) in &deeds {
        if *interaction == Interaction::Pressed && wish.0.is_none() {
            wish.0 = Some(button.0);
        }
    }
    for (interaction, button, _, _) in &buttons {
        if *interaction == Interaction::Pressed && button.0 != stages.showing() && wish.0.is_none()
        {
            wish.0 = Some(crate::builder::StageDeed::Show(button.0));
        }
    }
    for (_, button, mut border, mut fill) in &mut buttons {
        let standing = button.0 == stages.showing();
        let dress = BorderColor::all(if standing {
            theme::accent(&palette)
        } else {
            theme::panel_border(&palette)
        });
        if *border != dress {
            *border = dress;
        }
        let wanted = BackgroundColor(if standing {
            Color::srgb(0.075, 0.082, 0.102)
        } else {
            Color::BLACK.with_alpha(0.18)
        });
        if fill.0 != wanted.0 {
            *fill = wanted;
        }
    }
}

/// Mode presses set the tool; the standing mode wears the gold.
fn work_mode_bar(
    palette: Res<Palette>,
    mut mode: ResMut<crate::gizmo::ToolMode>,
    mut buttons: Query<(
        &Interaction,
        &ModeButton,
        &mut BorderColor,
        &mut BackgroundColor,
    )>,
) {
    for (interaction, button, _, _) in &buttons {
        if *interaction == Interaction::Pressed && *mode != button.0 {
            *mode = button.0;
        }
    }
    for (_, button, mut border, mut fill) in &mut buttons {
        let standing = *mode == button.0;
        let dress = BorderColor::all(if standing {
            theme::accent(&palette)
        } else {
            theme::panel_border(&palette)
        });
        if *border != dress {
            *border = dress;
        }
        // Every mode button says the same thing the same way: the fill means
        // STANDING and nothing else. PAINT used to wear the brush's own colour,
        // which made it look chosen while another tool was in hand - Brett:
        // "Paint wasnt active, but the button isnt following the other buttons
        // conventions". The brush has a square of its own at the head of the
        // palette, which is a better place for it anyway: it is only wanted
        // while painting, and there it can be the size of a colour rather than
        // the size of a word.
        let wanted = BackgroundColor(if standing {
            Color::srgb(0.075, 0.082, 0.102)
        } else {
            Color::BLACK.with_alpha(0.18)
        });
        if fill.0 != wanted.0 {
            *fill = wanted;
        }
    }
}

/// Bench presses walk the maker over; the standing bench wears the gold.
fn work_buttons(
    palette: Res<Palette>,
    mut bench: ResMut<Bench>,
    mut buttons: Query<(
        &Interaction,
        &BenchButton,
        &mut BorderColor,
        &mut BackgroundColor,
    )>,
) {
    for (interaction, button, _, _) in &buttons {
        if *interaction == Interaction::Pressed && *bench != button.0 {
            *bench = button.0;
        }
    }
    for (_, button, mut border, mut fill) in &mut buttons {
        let standing = *bench == button.0;
        *border = BorderColor::all(if standing {
            theme::accent(&palette)
        } else {
            theme::panel_border(&palette)
        });
        *fill = BackgroundColor(if standing {
            Color::srgb(0.075, 0.082, 0.102)
        } else {
            Color::BLACK.with_alpha(0.18)
        });
    }
}

/// The gear opens and closes the settings panel.
fn work_settings(
    gears: Query<&Interaction, (Changed<Interaction>, With<SettingsButton>)>,
    mut panels: Query<&mut Visibility, With<SettingsPanel>>,
) {
    let pressed = gears
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if !pressed {
        return;
    }
    for mut visibility in &mut panels {
        *visibility = if *visibility == Visibility::Hidden {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
