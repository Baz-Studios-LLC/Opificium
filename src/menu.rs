//! THE MENU BAR — every command the bench has, in one place along the top.
//!
//! The bench grew four places to put a command and no place that showed them
//! all: four modes on the top bar, five steps on the bottom one, five glyphs in
//! the rail's foot, and a drawer of projects. Brett: "I feel like this project is
//! building large enough we could use a file bar like this."
//!
//! # Drawn rather than native
//!
//! This is the bench's own bar, in the bench's own dress, and not the system menu
//! bar beside the apple. Bevy has no menu of any kind - there is nothing in
//! `bevy_winit` or `bevy_window` to hang one on - so the native bar would mean
//! AppKit through `objc2`, a channel to carry a click from AppKit's thread into
//! the schedule, and all of it macOS-only while the bench also ships for Windows.
//! Blender, Unity and Godot all draw their own for the same reasons.
//!
//! # How a menu item reaches the thing it does
//!
//! Two ways, and the first one is the reason this module is short.
//!
//! Five of these commands already have a glyph on the rail: save, open, bake,
//! clear, and the keys. Those items wear THAT GLYPH'S OWN marker component - a
//! `SaveButton` on a menu line is a `SaveButton` - so the system that already
//! answers the glyph answers the menu too, without knowing a menu exists. No
//! second path to keep in step with the first.
//!
//! The rest are either a resource this module can simply set - which bench, which
//! way the eye looks, the cutaway, the grid - or one of the four that live inside
//! a system reading the keyboard, and those go through [`MenuWish`].

use bevy::prelude::*;
use bevy::text::FontSize;

use crate::look::{Fonts, Palette, theme};

/// How tall the bar stands.
///
/// Every panel that hangs from the window's top edge is held off by exactly this,
/// so the bar is the ceiling of the whole bench rather than something lying over
/// it. There are six of them: the rail, the shelf, the palette, the body shelf,
/// and the two top bars.
pub const BAR_HIGH: f32 = 26.0;

/// The width every title takes, whatever its word.
///
/// One width for all of them, the way the mode buttons do it - a row whose items
/// step in and out as the eye runs along them reads as unrelated things - and it
/// is also what lets a menu's panel know where to stand without measuring
/// anything: the fifth menu opens under the fifth width.
const TITLE_WIDE: f32 = 76.0;

/// How far the first title sits from the window's edge.
const LEAD: f32 = 10.0;

/// Above every panel, below the tooltip. The naming card is 50 and the tooltip is
/// 200; a menu that opened behind the shelf would be a menu that does not open.
const ABOVE: i32 = 100;

/// What a menu item does when it is chosen.
///
/// Only the ones that need a name here. The five that repeat a glyph the rail
/// already has carry no deed at all - see the module's own word on that.
#[derive(Component, Clone, Copy, PartialEq)]
pub enum MenuDeed {
    Undo,
    Redo,
    Copy,
    Paste,
    Bench(crate::Bench),
    /// Where the eye stands: the drafting angles, the same six the number row
    /// gives.
    Look {
        yaw: f32,
        pitch: f32,
    },
    Cutaway,
    Grid,
    FaceSnap,
    Quit,
}

impl MenuDeed {
    /// Whether this means anything only at the building bench.
    ///
    /// Kept with the DEED rather than marked on each line in the table, because
    /// it is a fact about the command and not about where it was written down -
    /// and the systems that answer these all guard on the same thing, so a line
    /// that looks live at the rig bench is a line that silently does nothing.
    ///
    /// FILE is not here: the save, the open and the sweep all answer at the rig
    /// bench too, where they keep and open CLIPS instead of buildings.
    fn only_at_the_builder(&self) -> bool {
        matches!(
            self,
            MenuDeed::Undo
                | MenuDeed::Redo
                | MenuDeed::Copy
                | MenuDeed::Paste
                | MenuDeed::Cutaway
                | MenuDeed::Grid
                | MenuDeed::FaceSnap
        )
    }
}

/// The deed a menu item was chosen for, for the four that cannot be reached any
/// other way.
///
/// Undo, redo, copy and paste all live inside systems that read the keyboard, and
/// a menu is a second way of asking for the same thing. Set here, read there,
/// which is one line in each of them.
///
/// The alternative was for the menu to SYNTHESISE the keypress - press `Z` into
/// `ButtonInput` and let the system that already listens for it answer. It would
/// have worked, and it would have made four systems that currently depend on
/// nothing depend on running after this one, in a schedule where the ordering
/// bugs already found were all of that kind.
#[derive(Resource, Default)]
pub struct MenuWish(pub Option<MenuDeed>);

impl MenuWish {
    /// Takes the wish if it is this one. A deed is answered once.
    pub fn taken(&mut self, deed: MenuDeed) -> bool {
        if self.0 == Some(deed) {
            self.0 = None;
            return true;
        }
        false
    }
}

/// Which glyph on the rail a menu line stands in for.
///
/// The line wears that glyph's marker, so this says only which one to hang.
#[derive(Clone, Copy)]
enum Glyph {
    Save,
    OpenWork,
    Bake,
    Clear,
    OpenProject,
    Keys,
}

/// One line of a menu.
enum Line {
    /// A command of this module's own, and the key that does the same thing.
    Deed(&'static str, &'static str, MenuDeed),
    /// One of the rail's glyphs, reached by wearing its marker.
    Same(&'static str, &'static str, Glyph),
    /// A rule between groups of lines.
    Rule,
}

/// The bar, as a table. Adding a command is adding a line here.
const MENUS: &[(&str, &[Line])] = &[
    (
        "FILE",
        &[
            Line::Same("NEW", "", Glyph::Clear),
            Line::Same("OPEN WORK...", "", Glyph::OpenWork),
            Line::Same("SAVE", "", Glyph::Save),
            Line::Rule,
            // The one that carries a building out to the game, and the word that
            // says what that means: it is the only command here whose name is a
            // term of art.
            Line::Same("BAKE INTO THE GAME...", "", Glyph::Bake),
            Line::Rule,
            Line::Same("OPEN A GAME...", "", Glyph::OpenProject),
            Line::Rule,
            Line::Deed("QUIT", "", MenuDeed::Quit),
        ],
    ),
    (
        "EDIT",
        &[
            Line::Deed("UNDO", "cmd Z", MenuDeed::Undo),
            Line::Deed("REDO", "cmd Y", MenuDeed::Redo),
            Line::Rule,
            Line::Deed("COPY", "cmd C", MenuDeed::Copy),
            Line::Deed("PASTE", "cmd V", MenuDeed::Paste),
        ],
    ),
    (
        "VIEW",
        &[
            // The same six the number row gives, and the same numbers: a menu
            // that showed a seventh angle would be a seventh angle nobody could
            // reach by key.
            Line::Deed(
                "FRONT",
                "1",
                MenuDeed::Look {
                    yaw: 0.0,
                    pitch: 0.32,
                },
            ),
            Line::Deed(
                "RIGHT",
                "2",
                MenuDeed::Look {
                    yaw: std::f32::consts::FRAC_PI_2,
                    pitch: 0.32,
                },
            ),
            Line::Deed(
                "BACK",
                "3",
                MenuDeed::Look {
                    yaw: std::f32::consts::PI,
                    pitch: 0.32,
                },
            ),
            Line::Deed(
                "LEFT",
                "4",
                MenuDeed::Look {
                    yaw: -std::f32::consts::FRAC_PI_2,
                    pitch: 0.32,
                },
            ),
            Line::Deed(
                "OVERHEAD",
                "5",
                MenuDeed::Look {
                    yaw: 0.0,
                    pitch: 1.55,
                },
            ),
            Line::Deed(
                "THE PERCH",
                "6",
                MenuDeed::Look {
                    yaw: 0.6,
                    pitch: 0.7,
                },
            ),
            Line::Rule,
            Line::Deed("CUTAWAY", "H", MenuDeed::Cutaway),
            Line::Deed("GRID INTERVAL", "G", MenuDeed::Grid),
            Line::Deed("FACE SNAP", "F", MenuDeed::FaceSnap),
        ],
    ),
    (
        "BENCH",
        &[
            Line::Deed("THE BUILDER", "", MenuDeed::Bench(crate::Bench::Builder)),
            Line::Deed("THE RIG", "", MenuDeed::Bench(crate::Bench::Rig)),
            Line::Deed("THE KILN", "", MenuDeed::Bench(crate::Bench::Kiln)),
        ],
    ),
    ("HELP", &[Line::Same("THE KEYS", "", Glyph::Keys)]),
];

/// The strip itself.
#[derive(Component)]
struct MenuBar;

/// One word on the strip, and which menu it opens.
#[derive(Component)]
struct MenuTitle(usize);

/// One menu's panel of lines.
#[derive(Component)]
struct MenuPanel(usize);

/// One line in a panel.
///
/// Worn by every line, deed-carrying or glyph-wearing alike, because two things
/// need to speak about "a line" without caring which kind it is: the hover light,
/// and the question of whether a press landed inside a menu. Without it the light
/// would query every interactive node in the bench that has a colour, and repaint
/// the shelf.
#[derive(Component)]
struct MenuRow;

/// Which menu stands open, if any.
#[derive(Resource, Default)]
struct OpenMenu(Option<usize>);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuWish>()
            .init_resource::<OpenMenu>()
            .add_systems(Startup, hang_the_bar)
            .add_systems(
                Update,
                (
                    work_the_titles,
                    show_the_menus,
                    light_the_lines,
                    dim_what_does_not_apply,
                    work_the_lines,
                    close_on_a_glyph,
                )
                    .chain(),
            );
    }
}

fn hang_the_bar(mut commands: Commands, fonts: Res<Fonts>, palette: Res<Palette>) {
    let bar = commands
        .spawn((
            MenuBar,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(BAR_HIGH),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::left(Val::Px(LEAD)),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme::panel_bg()),
            BorderColor::all(theme::panel_border(&palette)),
            GlobalZIndex(ABOVE),
        ))
        .id();

    for (which, (name, lines)) in MENUS.iter().enumerate() {
        let title = commands
            .spawn((
                MenuTitle(which),
                Interaction::default(),
                Node {
                    width: Val::Px(TITLE_WIDE),
                    height: Val::Px(BAR_HIGH),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
                ChildOf(bar),
            ))
            .id();
        commands.spawn((
            Text::new(*name),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: FontSize::Px(12.0),
                ..default()
            },
            TextColor(theme::accent(&palette)),
            ChildOf(title),
        ));

        // The panel, built now and hidden, rather than raised on the press. A
        // menu is the same menu every time it opens, and one built on every press
        // is one that spends a frame not being there.
        let panel = commands
            .spawn((
                MenuPanel(which),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(LEAD + which as f32 * TITLE_WIDE),
                    top: Val::Px(BAR_HIGH),
                    min_width: Val::Px(224.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::axes(Val::Px(0.0), Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(theme::panel_bg()),
                BorderColor::all(theme::panel_border(&palette)),
                GlobalZIndex(ABOVE + 1),
            ))
            .id();

        for line in lines.iter() {
            let (label, cap) = match line {
                Line::Rule => {
                    commands.spawn((
                        Node {
                            height: Val::Px(1.0),
                            margin: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(theme::panel_border(palette.as_ref())),
                        ChildOf(panel),
                    ));
                    continue;
                }
                Line::Deed(label, cap, _) | Line::Same(label, cap, _) => (label, cap),
            };
            let row = commands
                .spawn((
                    MenuRow,
                    Interaction::default(),
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(18.0),
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    ChildOf(panel),
                ))
                .id();
            commands.spawn((
                Text::new(*label),
                TextFont {
                    font: fonts.display.clone().into(),
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(theme::text_dim(&palette)),
                ChildOf(row),
            ));
            // The key that does the same thing, in the machine's own face, the way
            // every menu in the world writes it. This is most of why a menu bar is
            // worth having: the gear's panel lists the keys and cannot DO any of
            // them, and a maker learns a shortcut from the menu they were already
            // reaching for.
            if !cap.is_empty() {
                commands.spawn((
                    Text::new(*cap),
                    TextFont {
                        font_size: FontSize::Px(10.0),
                        ..default()
                    },
                    TextColor(theme::text_dim(&palette).with_alpha(0.55)),
                    ChildOf(row),
                ));
            }
            // And what makes the line act. Either a deed of this module's own, or
            // the very component the rail's own glyph wears.
            match line {
                Line::Deed(_, _, deed) => {
                    commands.entity(row).insert(*deed);
                }
                Line::Same(_, _, glyph) => {
                    let row = commands.entity(row).id();
                    wear_the_glyph(&mut commands, row, *glyph);
                }
                Line::Rule => {}
            }
        }
    }
}

/// Hangs the marker the rail's own glyph wears, so the system that answers that
/// glyph answers this line too.
fn wear_the_glyph(commands: &mut Commands, row: Entity, glyph: Glyph) {
    let mut row = commands.entity(row);
    match glyph {
        Glyph::Save => {
            row.insert(crate::builder::SaveButton);
        }
        Glyph::OpenWork => {
            row.insert(crate::builder::OpenWorkButton);
        }
        Glyph::Bake => {
            row.insert(crate::builder::BakeButton);
        }
        Glyph::Clear => {
            row.insert(crate::builder::ClearButton);
        }
        Glyph::OpenProject => {
            row.insert(crate::rail::OpenProjectButton);
        }
        Glyph::Keys => {
            row.insert(crate::rail::SettingsButton);
        }
    };
}

/// A press on a title opens its menu; a press anywhere else closes them all.
///
/// And while one stands open, crossing another title walks to it without a second
/// press, which is what every menu bar does and what a hand expects.
fn work_the_titles(
    buttons: Res<ButtonInput<MouseButton>>,
    titles: Query<(&Interaction, &MenuTitle)>,
    lines: Query<&Interaction, With<MenuRow>>,
    mut open: ResMut<OpenMenu>,
) {
    let under = titles
        .iter()
        .find(|(touch, _)| **touch != Interaction::None)
        .map(|(_, title)| title.0);

    if buttons.just_pressed(MouseButton::Left) {
        match under {
            // The same title again closes it: a menu bar's titles are latches.
            Some(which) if open.0 == Some(which) => open.0 = None,
            Some(which) => open.0 = Some(which),
            None => {
                // A press on anything that is not a title and not inside a menu
                // puts the menu away - including a press on the bench itself,
                // which is the commonest way anybody closes one.
                let inside = lines.iter().any(|touch| *touch != Interaction::None);
                if !inside {
                    open.0 = None;
                }
            }
        }
        return;
    }
    // Walking the bar with one open.
    if let (Some(which), Some(_)) = (under, open.0)
        && open.0 != Some(which)
    {
        open.0 = Some(which);
    }
}

fn show_the_menus(
    open: Res<OpenMenu>,
    mut panels: Query<(&MenuPanel, &mut Node)>,
    mut titles: Query<(&MenuTitle, &mut BackgroundColor)>,
) {
    if !open.is_changed() {
        return;
    }
    for (panel, mut node) in &mut panels {
        let wanted = if open.0 == Some(panel.0) {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != wanted {
            node.display = wanted;
        }
    }
    // The open title stays lit while its menu is down, so the eye can see which
    // of them it is looking at the inside of.
    for (title, mut fill) in &mut titles {
        let wanted = if open.0 == Some(title.0) {
            Color::srgb(0.075, 0.082, 0.102)
        } else {
            Color::NONE
        };
        if fill.0 != wanted {
            *fill = BackgroundColor(wanted);
        }
    }
}

/// A line that means nothing where the maker is standing goes grey, and stops
/// answering.
///
/// Greyed rather than removed: a menu whose lines come and go is a menu you cannot
/// learn, and the command has not gone anywhere - the maker has.
fn dim_what_does_not_apply(
    bench: Res<crate::Bench>,
    palette: Res<Palette>,
    rows: Query<(&MenuDeed, &Children)>,
    mut words: Query<&mut TextColor>,
) {
    if !bench.is_changed() {
        return;
    }
    for (deed, kids) in &rows {
        let live = !deed.only_at_the_builder() || *bench == crate::Bench::Builder;
        let wanted = if live {
            theme::text_dim(&palette)
        } else {
            theme::text_dim(&palette).with_alpha(0.3)
        };
        for kid in kids.iter() {
            if let Ok(mut dye) = words.get_mut(kid)
                && dye.0 != wanted
            {
                *dye = TextColor(wanted);
            }
        }
    }
}

/// The line under the hand wears the gold.
fn light_the_lines(
    palette: Res<Palette>,
    mut lines: Query<(&Interaction, &mut BackgroundColor), With<MenuRow>>,
) {
    for (touch, mut fill) in &mut lines {
        let wanted = if *touch == Interaction::None {
            Color::NONE
        } else {
            theme::accent(&palette).with_alpha(0.14)
        };
        if fill.0 != wanted {
            *fill = BackgroundColor(wanted);
        }
    }
}

/// Carries out a chosen line, and puts the menu away.
///
/// The five that wear a glyph's marker are not here at all: their own systems saw
/// the press. This is the rest.
#[allow(clippy::too_many_arguments)]
fn work_the_lines(
    mut wish: ResMut<MenuWish>,
    mut open: ResMut<OpenMenu>,
    mut leaving: MessageWriter<AppExit>,
    mut bench: ResMut<crate::Bench>,
    mut eye: ResMut<crate::camera::OrbitRig>,
    mut lifted: ResMut<crate::builder::RoofsLifted>,
    mut grid: ResMut<crate::builder::SnapGrid>,
    mut snap: ResMut<crate::builder::SnapMode>,
    chosen: Query<(&Interaction, &MenuDeed), Changed<Interaction>>,
) {
    for (touch, deed) in &chosen {
        if *touch != Interaction::Pressed {
            continue;
        }
        // Greyed above, and inert here: the dimming is what a maker SEES and this
        // is what makes it true.
        if deed.only_at_the_builder() && *bench != crate::Bench::Builder {
            continue;
        }
        match deed {
            // The four that live inside a keyboard-reading system.
            MenuDeed::Undo | MenuDeed::Redo | MenuDeed::Copy | MenuDeed::Paste => {
                wish.0 = Some(*deed);
            }
            MenuDeed::Bench(which) => *bench = *which,
            MenuDeed::Look { yaw, pitch } => {
                eye.yaw = *yaw;
                eye.pitch = *pitch;
            }
            MenuDeed::Cutaway => {
                lifted.0 = match lifted.0 {
                    crate::builder::Cutaway::Whole => crate::builder::Cutaway::RoofOff,
                    crate::builder::Cutaway::RoofOff => crate::builder::Cutaway::WallsDown,
                    crate::builder::Cutaway::WallsDown => crate::builder::Cutaway::Whole,
                };
            }
            MenuDeed::Grid => {
                grid.0 = match grid.0 {
                    1 => 2,
                    2 => 4,
                    4 => 8,
                    8 => 16,
                    _ => 1,
                };
            }
            MenuDeed::FaceSnap => snap.face = !snap.face,
            MenuDeed::Quit => {
                leaving.write(AppExit::Success);
            }
        }
        // Chosen is chosen: the menu goes away, the way a menu does.
        open.0 = None;
    }
}

/// A menu line that stands for one of the rail's glyphs closes the menu too.
///
/// It cannot do it in `work_the_lines`, because those lines carry no `MenuDeed` -
/// that is the whole trick of them - so the closing is watched for here instead.
fn close_on_a_glyph(
    mut open: ResMut<OpenMenu>,
    pressed: Query<
        &Interaction,
        (
            Changed<Interaction>,
            Or<(
                With<crate::builder::SaveButton>,
                With<crate::builder::OpenWorkButton>,
                With<crate::builder::BakeButton>,
                With<crate::builder::ClearButton>,
            )>,
        ),
    >,
) {
    if open.0.is_none() {
        return;
    }
    if pressed.iter().any(|touch| *touch == Interaction::Pressed) {
        open.0 = None;
    }
}
