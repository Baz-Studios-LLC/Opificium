//! THE RIG — a model, looked at closely.
//!
//! Pick a model the project holds, stand it on the stage, and walk around it. That is
//! the whole bench today.
//!
//! # Why it is called the rig
//!
//! Because of where it is going. Brett: "we will use that section to load a model, look
//! at it...and down the line add the ability to rig the model." Looking closely is the
//! first half of rigging - you cannot put a joint where you cannot see - so the viewer
//! is not a placeholder that rigging will replace, it is the part of rigging that works
//! already.
//!
//! # What was here before
//!
//! A body-and-clips bench: it posed a villager from `data/bodies/*.json` by dragging
//! limbs, keyed the pose on a timeline, and looped it back. That was built for one
//! game's animation and no longer needed, and it was the last thing in the bench that
//! knew one game's vocabulary by heart. Its clips went to `out/clips/*.baz` and were
//! never baked, so no game could read them and nothing downstream lost a file. It is in
//! the history if it is ever wanted.
//!
//! # A model is not a part
//!
//! Which is why this bench shows one and cannot edit it. A part is a name resolved into
//! boxes on the lattice and painted from a ramp; a model is arbitrary triangles wearing
//! their own materials. The builder's tools - the brush, the lattice, the widgets -
//! have nothing to grip. See `model`.

use std::path::PathBuf;

use bevy::prelude::*;

use crate::Bench;
use crate::look::{Fonts, Palette, theme};

/// What the bench is looking at.
#[derive(Resource, Default)]
pub struct Rig {
    /// The model standing on the stage, if any.
    standing: Option<PathBuf>,
}

/// The model standing on the stage.
#[derive(Component)]
struct OnTheStage;

/// The shelf down the right, holding the project's models.
#[derive(Component)]
struct ModelShelf;

/// The drawer the model buttons hang in, emptied and refilled as the folder changes.
#[derive(Component)]
struct ModelDrawer;

/// One model on the shelf.
#[derive(Component)]
struct ModelButton(PathBuf);

/// The line along the top saying what is standing and how big it is.
#[derive(Component)]
struct RigBar;

/// The words in it.
#[derive(Component)]
struct RigWord;

pub struct RigPlugin;

impl Plugin for RigPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Rig>()
            .add_systems(Startup, (hang_the_shelf, hang_the_top_bar))
            .add_systems(
                Update,
                (
                    fill_the_shelf,
                    take_a_model,
                    stand_the_model,
                    say_what_stands,
                    show_the_furniture,
                    stand_the_camera,
                ),
            );
    }
}

/// Hangs the shelf, empty. What goes on it is read off the folder, not baked in here.
fn hang_the_shelf(mut commands: Commands, fonts: Res<Fonts>, palette: Res<Palette>) {
    let shelf = commands
        .spawn((
            ModelShelf,
            // The builder's shelf exactly: same edge, same width, same border on the one
            // side it touches, because a bench whose panels each float somewhere near an
            // edge is three benches that happen to share a window.
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(crate::menu::BAR_HIGH),
                bottom: Val::Px(0.0),
                width: Val::Px(crate::look::PANEL_WIDE),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
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
    let drawer = crate::builder::drawer(&mut commands, &fonts, &palette, shelf, "THE MODEL", true);
    commands.entity(drawer).insert(ModelDrawer);
}

/// Fills the shelf from the folder, whenever the maker arrives at the bench.
///
/// Read on ARRIVAL rather than at startup, because the interesting model is usually one
/// the kiln made a minute ago - a shelf filled once when the window opened would never
/// show it, and the maker would have to relaunch to look at their own work.
fn fill_the_shelf(
    mut commands: Commands,
    bench: Res<Bench>,
    fonts: Res<Fonts>,
    palette: Res<Palette>,
    drawer: Query<Entity, With<ModelDrawer>>,
    standing: Query<Entity, With<ModelButton>>,
) {
    if !bench.is_changed() || *bench != Bench::Rig {
        return;
    }
    let Ok(drawer) = drawer.single() else {
        return;
    };
    for old in &standing {
        commands.entity(old).despawn();
    }

    let models = crate::model::all();
    if models.is_empty() {
        // An empty shelf that says why. A maker who has never fired the kiln is not
        // looking at a broken bench, they are looking at a folder with nothing in it.
        let note = commands
            .spawn((
                ModelButton(PathBuf::new()),
                Node {
                    padding: UiRect::axes(Val::Px(9.0), Val::Px(6.0)),
                    ..default()
                },
                ChildOf(drawer),
            ))
            .id();
        commands.spawn((
            Text::new("NOTHING YET - FIRE THE KILN"),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: crate::look::text_at(10.0),
                ..default()
            },
            TextColor(theme::text_dim(&palette).with_alpha(0.6)),
            ChildOf(note),
        ));
        return;
    }

    for road in models {
        let size = crate::model::bounds_of(&road);
        let button = commands
            .spawn((
                ModelButton(road.clone()),
                Interaction::default(),
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::axes(Val::Px(9.0), Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::BLACK.with_alpha(0.18)),
                BorderColor::all(theme::panel_border(&palette)),
                ChildOf(drawer),
            ))
            .id();
        commands.spawn((
            Text::new(crate::model::name_of(&road).to_uppercase()),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: crate::look::text_at(11.0),
                ..default()
            },
            TextColor(theme::text_dim(&palette)),
            ChildOf(button),
        ));
        // How tall the file itself says it is. Worth showing on the shelf rather than
        // only once it stands: a model kept at the wrong height is the mistake this
        // number catches, and catching it before standing it saves the walk.
        commands.spawn((
            Text::new(match size {
                // Its SHAPE, since its size is whatever a maker states. Two of these are
                // proportions and the third is the one they will set.
                Some(size) => format!(
                    "{:.2} x {:.2} x {:.2}",
                    size.wide(),
                    size.tall(),
                    size.deep()
                ),
                None => "UNREADABLE".to_string(),
            }),
            TextFont {
                font: fonts.text.clone().into(),
                font_size: crate::look::text_at(10.0),
                ..default()
            },
            TextColor(theme::text_dim(&palette).with_alpha(0.55)),
            ChildOf(button),
        ));
        commands.spawn((
            crate::rail::Word("Stand this model on the bench"),
            ChildOf(button),
        ));
    }
}

/// A press stands that model up.
fn take_a_model(
    bench: Res<Bench>,
    mut rig: ResMut<Rig>,
    chosen: Query<(&Interaction, &ModelButton), Changed<Interaction>>,
) {
    if *bench != Bench::Rig {
        return;
    }
    for (how, which) in &chosen {
        // `Changed` rather than a bare `Pressed`: an interaction stays pressed for as
        // long as the button is held, so a plain read fires every frame of one press.
        if *how == Interaction::Pressed && !which.0.as_os_str().is_empty() {
            rig.standing = Some(which.0.clone());
        }
    }
}

/// Stands the chosen model at the size its own file says.
///
/// No height to state here, unlike the kiln: a model the bench kept has its fit baked
/// into the file, so what the file says IS the answer. A viewer that scaled it to taste
/// would be showing something other than what a game will load, which is the one thing
/// this bench exists to show.
fn stand_the_model(
    mut commands: Commands,
    rig: Res<Rig>,
    assets: Res<AssetServer>,
    standing: Query<Entity, With<OnTheStage>>,
    mut showing: Local<Option<PathBuf>>,
) {
    if *showing == rig.standing {
        return;
    }
    showing.clone_from(&rig.standing);
    for old in &standing {
        commands.entity(old).despawn();
    }
    let Some(road) = rig.standing.clone() else {
        return;
    };
    info!("the rig stands {}", road.display());
    commands.spawn((
        OnTheStage,
        crate::stage::RigFurniture,
        crate::model::stand(&assets, &road, None),
    ));
}

/// Hangs the line along the top that says what is standing.
fn hang_the_top_bar(mut commands: Commands, fonts: Res<Fonts>, palette: Res<Palette>) {
    let centering = commands
        .spawn((
            RigBar,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(crate::menu::BAR_HIGH),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Visibility::Hidden,
        ))
        .id();
    let bar = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
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
            ChildOf(centering),
        ))
        .id();
    commands.spawn((
        RigWord,
        Text::new(""),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: crate::look::text_at(12.0),
            ..default()
        },
        TextColor(theme::accent(&palette)),
        ChildOf(bar),
    ));
}

/// Says what is standing, and how big it is.
fn say_what_stands(rig: Res<Rig>, mut words: Query<&mut Text, With<RigWord>>) {
    if !rig.is_changed() {
        return;
    }
    let said = match &rig.standing {
        Some(road) => {
            let name = crate::model::name_of(road).to_uppercase();
            match crate::model::bounds_of(road) {
                Some(size) => format!(
                    "{name}  -  {:.2} x {:.2} x {:.2}",
                    size.wide(),
                    size.tall(),
                    size.deep()
                ),
                None => name,
            }
        }
        None => "PICK A MODEL".to_string(),
    };
    for mut text in &mut words {
        **text = said.clone();
    }
}

/// The bench's own furniture, shown here and put away everywhere else.
fn show_the_furniture(
    bench: Res<Bench>,
    showing: Res<crate::look::Showing>,
    mut shelves: Query<&mut Visibility, With<ModelShelf>>,
    mut bars: Query<&mut Visibility, (With<RigBar>, Without<ModelShelf>)>,
) {
    if !bench.is_changed() && !showing.is_changed() {
        return;
    }
    let here = *bench == Bench::Rig;
    let how = |out: bool| {
        if out {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };
    for mut it in &mut shelves {
        *it = how(here && showing.wanted(crate::look::Tool::Shelf));
    }
    for mut it in &mut bars {
        *it = how(here && showing.wanted(crate::look::Tool::TopBar));
    }
}

/// The eye the bench opens at, and what it turns about.
///
/// Framed by the MODEL rather than set to a fixed distance, because models differ by
/// more than an order of magnitude - a fly is a fifth of a meter and a barn is ten - and
/// one distance that suits either shows the other as a dot or from inside. The eye backs
/// off to twice the model's height and looks at its middle.
///
/// Set on arrival and on every change of model, so walking to the bench is walking to
/// the same view rather than to wherever the camera was left.
fn stand_the_camera(
    bench: Res<Bench>,
    rig: Res<Rig>,
    mut eye: ResMut<crate::camera::OrbitRig>,
    mut center: ResMut<crate::camera::Center>,
) {
    if *bench != Bench::Rig || !(bench.is_changed() || rig.is_changed()) {
        return;
    }
    let tall = rig
        .standing
        .as_ref()
        .and_then(|road| crate::model::bounds_of(road))
        .map(|size| size.tall())
        .unwrap_or(2.0);
    let middle = Vec3::new(0.0, tall * 0.5, 0.0);
    center.0 = middle;
    eye.focus = middle;
    // Off the shoulder line rather than square on: a model seen dead ahead reads as a
    // silhouette, and three quarters shows depth for free.
    eye.yaw = -std::f32::consts::FRAC_PI_2 + 0.6;
    eye.pitch = 0.28;
    eye.distance = (tall * 2.4).clamp(0.5, 20.0);
}
