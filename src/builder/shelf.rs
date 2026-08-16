//! The shelf: the drawers of parts a hand can take.

use super::*;

pub(crate) fn raise_shelf(mut commands: Commands, fonts: Res<Fonts>, palette: Res<Palette>) {
    let shelf = commands
        .spawn((
            Shelf,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(crate::menu::BAR_HIGH),
                bottom: Val::Px(0.0),
                // Wide enough for the longest thing on it - ROOF, GABLE,
                // STRETCH - on one line. A shelf that wraps its own names is a
                // shelf a maker reads twice.
                width: Val::Px(crate::look::PANEL_WIDE),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(3.0),
                border: UiRect::left(Val::Px(1.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(theme::panel_bg()),
            BorderColor::all(theme::panel_border(&palette)),
            crate::look::Scrollable,
            bevy::ui::ScrollPosition::default(),
        ))
        .id();

    // WHAT THE HAND WILL DO, over the stage rather than filed on the shelf.
    //
    // It lived at the top of the shelf in eleven-point dim bone, which is where a
    // maker never looked: it answers a question - where will this land? - that is
    // asked with the eyes on the work. Brett: "this is small and hard to read, I
    // think it should be bigger and maybe in the upper lefthand of the view area?"
    //
    // The upper left of the VIEW, which begins where the rail ends - see
    // `look::PANEL_WIDE` - and below the menu bar. The mode buttons are centred, so
    // this corner is empty at every window width.
    let word = commands
        .spawn((
            SnapModeText,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(crate::look::PANEL_WIDE + 14.0),
                top: Val::Px(crate::menu::BAR_HIGH + 10.0),
                padding: UiRect::axes(Val::Px(9.0), Val::Px(4.0)),
                ..default()
            },
            // Its own faint ground. The stage is near-black and reads fine, but a
            // building is not - a pale panel behind pale text is text nobody can
            // read, and the whole point of moving this was reading it.
            BackgroundColor(Color::BLACK.with_alpha(0.35)),
        ))
        .id();
    commands.spawn((
        Text::new("face snap - on (F)"),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: crate::look::text_at(15.0),
            ..default()
        },
        TextColor(theme::text(&palette).with_alpha(0.9)),
        ChildOf(word),
    ));

    // The drawers of parts.
    for (name, entries, open) in [
        ("STRUCTURE", STRUCTURE, true),
        ("FURNITURE", FURNITURE, false),
        ("DECOR", DECOR, false),
    ] {
        let body = drawer(&mut commands, &fonts, &palette, shelf, name, open);
        for entry in entries {
            let button = plain_button(&mut commands, &palette, body);
            commands.entity(button).insert(ShelfButton(entry));
            button_label(&mut commands, &fonts, &palette, button, entry.label);
        }
    }
    // What the game will raise: the drawings already carried in, so a maker can
    // see them and be rid of one. Brett: "Is there a way I can see those houses
    // and delete one or more of them?" - and the folder they live in is under
    // Application Support, which is not a place anybody browses by accident.
    let carried = drawer(&mut commands, &fonts, &palette, shelf, "IN THE GAME", false);
    commands.insert_resource(CarriedDrawer(carried));

    // Pieces: whole clusters kept for any work, not just this one.
    let pieces = drawer(&mut commands, &fonts, &palette, shelf, "PIECES", false);
    commands.insert_resource(PieceDrawer(pieces));

    let widgets = drawer(&mut commands, &fonts, &palette, shelf, "WIDGETS", false);
    for mark in crate::project::widgets() {
        let button = plain_button(&mut commands, &palette, widgets);
        commands.entity(button).insert(WidgetButton(mark.word));
        button_label(
            &mut commands,
            &fonts,
            &palette,
            button,
            Box::leak(mark.word.to_uppercase().into_boxed_str()),
        );
    }
}

/// A drawer: a header that opens and closes, and the body under it.
/// Returns the body, ready for buttons.
pub(crate) fn drawer(
    commands: &mut Commands,
    fonts: &Fonts,
    palette: &Palette,
    shelf: Entity,
    name: impl Into<String>,
    open: bool,
) -> Entity {
    let name = name.into();
    let header = commands
        .spawn((
            Interaction::default(),
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                padding: UiRect::axes(Val::Px(2.0), Val::Px(2.0)),
                ..default()
            },
            ChildOf(shelf),
        ))
        .id();
    let label = commands
        .spawn((
            Text::new(format!("{} {}", name, if open { "-" } else { "+" })),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: crate::look::text_at(13.0),
                ..default()
            },
            TextColor(theme::accent(palette)),
            ChildOf(header),
        ))
        .id();
    let body = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                display: if open { Display::Flex } else { Display::None },
                ..default()
            },
            ChildOf(shelf),
        ))
        .id();
    commands.entity(header).insert(DrawerHeader {
        body,
        label,
        name,
        open,
    });
    body
}

/// A saved-work row: the load button, and the small x that buries it.
pub(crate) fn plain_button(commands: &mut Commands, palette: &Palette, parent: Entity) -> Entity {
    commands
        .spawn((
            Interaction::default(),
            Node {
                padding: UiRect::axes(Val::Px(9.0), Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.18)),
            BorderColor::all(theme::panel_border(palette)),
            ChildOf(parent),
        ))
        .id()
}

/// A word on a button. Returns the text so a caller can say more about how it
/// should break — most should not, since breaking at spaces is what reading is.
pub(crate) fn button_label(
    commands: &mut Commands,
    fonts: &Fonts,
    palette: &Palette,
    button: Entity,
    label: &'static str,
) -> Entity {
    commands
        .spawn((
            Text::new(label),
            TextFont {
                font: fonts.text.clone().into(),
                font_size: crate::look::text_at(12.0),
                ..default()
            },
            TextColor(theme::text_dim(palette)),
            ChildOf(button),
        ))
        .id()
}

/// The shelf belongs to the Builder bench alone.
/// Which of the two right-hand panels is standing.
///
/// The shelf and the palette share an edge, and this is the ONE place that says
/// which is on it: a second system reaching for either one would be two writers
/// on one value, and they would take turns hiding each other's panel.
pub(crate) fn show_shelf(
    bench: Res<Bench>,
    mode: Res<crate::gizmo::ToolMode>,
    showing: Res<crate::look::Showing>,
    mut shelves: Query<&mut Visibility, With<Shelf>>,
    mut words: Query<&mut Visibility, (With<SnapModeText>, Without<Shelf>)>,
) {
    if !bench.is_changed() && !mode.is_changed() && !showing.is_changed() {
        return;
    }
    // The snap word follows the BENCH alone, not the tool. It used to ride on the
    // shelf and so vanished whenever the colours came out - but F and G work in
    // every mode of the building bench, and a line that blinks off while its keys
    // still work reads as a fault.
    for mut visibility in &mut words {
        *visibility = if *bench == Bench::Builder {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    // Painting is not placing: the parts go away while the colours are out.
    let standing = *bench == Bench::Builder
        && *mode != crate::gizmo::ToolMode::Paint
        && showing.wanted(crate::look::Tool::Shelf);
    for mut visibility in &mut shelves {
        *visibility = if standing {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Drawer headers open and close their bodies.
pub(crate) fn work_drawers(
    mut headers: Query<(&mut DrawerHeader, &Interaction), Changed<Interaction>>,
    mut nodes: Query<&mut Node>,
    mut labels: Query<&mut Text>,
) {
    for (mut header, interaction) in &mut headers {
        if *interaction != Interaction::Pressed {
            continue;
        }
        header.open = !header.open;
        if let Ok(mut node) = nodes.get_mut(header.body) {
            node.display = if header.open {
                Display::Flex
            } else {
                Display::None
            };
        }
        if let Ok(mut text) = labels.get_mut(header.label) {
            *text = Text::new(format!(
                "{} {}",
                header.name,
                if header.open { "-" } else { "+" }
            ));
        }
    }
}

/// Shelf presses fill the hand; the armed entry wears the gold.
#[allow(clippy::too_many_arguments)]
pub(crate) fn work_shelf(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<Palette>,
    mut hand: ResMut<Hand>,
    panes: Res<WindowPanes>,
    doors: Res<DoorAs>,
    mut tool: ResMut<crate::gizmo::ToolMode>,
    mut parts: Query<(&Interaction, &ShelfButton, &mut BorderColor), Without<WidgetButton>>,
    mut widgets: Query<(&Interaction, &WidgetButton, &mut BorderColor), Without<ShelfButton>>,
    ghosts: Query<Entity, With<Ghost>>,
) {
    let mut rearmed = false;
    for (interaction, button, _) in &parts {
        let want = from_the_shelf(button.0.kind, *panes, *doors);
        if *interaction == Interaction::Pressed && hand.kind != Some(want) {
            *hand = Hand::filled(want, button.0.stage.to_string());
            rearmed = true;
        }
    }
    for (interaction, button, _) in &widgets {
        let kind = a_mark(button.0);
        if *interaction == Interaction::Pressed && hand.kind != Some(kind) {
            *hand = Hand::filled(kind, "widget".to_string());
            rearmed = true;
        }
    }
    if rearmed {
        *tool = crate::gizmo::ToolMode::Normal;
        dress_ghost(
            &mut commands,
            &mut meshes,
            &mut materials,
            &palette,
            &hand,
            &ghosts,
        );
    }
    for (_, button, mut border) in &mut parts {
        // Through the same door the press went through, so a resized WINDOW still
        // lights its own button up.
        let want = from_the_shelf(button.0.kind, *panes, *doors);
        dress_shelf_border(&palette, hand.kind == Some(want), &mut border);
    }
    for (_, button, mut border) in &mut widgets {
        dress_shelf_border(&palette, hand.kind == Some(a_mark(button.0)), &mut border);
    }
}

pub(crate) fn dress_shelf_border(palette: &Palette, standing: bool, border: &mut BorderColor) {
    let dress = BorderColor::all(if standing {
        theme::accent(palette)
    } else {
        theme::panel_border(palette)
    });
    if *border != dress {
        *border = dress;
    }
}
