//! The palette panel: every ramp the game paints with, and the brush.

use super::*;

/// The palette panel, shown only while painting.
#[derive(Component)]
pub(crate) struct PalettePanel;

/// The big square at the head of the palette: the colour now armed.
#[derive(Component)]
pub(crate) struct BrushFace;

/// One colour a maker can arm. `ramp` empty is the bare swatch: painting with
/// it strips a part back to its own colours.
#[derive(Component, Clone)]
pub(crate) struct Swatch {
    ramp: Option<String>,
    shade: f32,
}

/// The shades a swatch row offers, which are the shades the keys step through —
/// so a colour picked with the mouse can be nudged with `-` and `=` and land on
/// another swatch rather than between two of them.
pub(crate) const SWATCHES: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

/// Builds the palette once, standing hidden until the paint tool is chosen.
///
/// Brett: "a palet comes up on the screen. You have an armed color and click a
/// part to paint it." Which is the right shape and better than what the keys
/// alone could do — walking `[` and `]` through twenty-four ramps is guessing at
/// a colour, and a palette is looking at one. The keys stay for nudging a shade
/// once the eye is close.
pub(crate) fn raise_palette(mut commands: Commands, fonts: Res<Fonts>, palette: Res<Palette>) {
    let panel = commands
        .spawn((
            PalettePanel,
            // In the SHELF's place, not beside it. The shelf holds what a
            // building is made of and this holds what it is coloured with, and a
            // maker who is painting is not placing - so one panel stands at a
            // time and neither has to be squeezed to make room for the other.
            // (The left edge is spoken for: the key rail lives there.)
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(crate::menu::BAR_HIGH),
                bottom: Val::Px(0.0),
                // The shelf's own width: the two share an edge and only one
                // stands at a time, so a differing width would make the panel
                // jump as a maker changes tool.
                width: Val::Px(crate::look::PANEL_WIDE),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(2.0),
                border: UiRect::left(Val::Px(1.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(theme::panel_bg()),
            BorderColor::all(theme::panel_border(&palette)),
            // The other half of scrolling, and the other way round from the
            // rail's fault: this one clipped its children and had nothing to
            // tell it where to. Overflow alone is a pane that hides what will
            // not fit; `Scrollable` is what lets the wheel reach it.
            crate::look::Scrollable,
            bevy::ui::ScrollPosition::default(),
            Visibility::Hidden,
        ))
        .id();
    commands.spawn((
        Text::new("THE PALETTE"),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: FontSize::Px(13.0),
            ..default()
        },
        TextColor(theme::accent(&palette)),
        Node {
            margin: UiRect::bottom(Val::Px(6.0)),
            ..default()
        },
        ChildOf(panel),
    ));

    // The armed colour, large, at the head of the panel. A swatch ringed in gold
    // says which one is armed but says it in the size of a swatch; this says
    // what is actually on the brush, at a size worth glancing at.
    commands.spawn((
        BrushFace,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(44.0),
            border: UiRect::all(Val::Px(1.0)),
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(palette.shade("wood", 0.5)),
        BorderColor::all(theme::accent(&palette)),
        ChildOf(panel),
    ));

    // The bare swatch first, because stripping a part is the one stroke a maker
    // cannot reach any other way once a part has been painted.
    let bare = commands
        .spawn((
            Swatch {
                ramp: None,
                shade: 0.5,
            },
            Interaction::default(),
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                margin: UiRect::bottom(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.18)),
            BorderColor::all(theme::panel_border(&palette)),
            ChildOf(panel),
        ))
        .id();
    commands.spawn((
        Text::new("BARE - its own colours"),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: FontSize::Px(10.0),
            ..default()
        },
        TextColor(theme::text_dim(&palette)),
        ChildOf(bare),
    ));

    let names: Vec<String> = palette.names().map(|n| n.to_string()).collect();
    for name in names {
        let row = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(2.0),
                    ..default()
                },
                ChildOf(panel),
            ))
            .id();
        commands.spawn((
            Text::new(name.clone()),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: FontSize::Px(9.0),
                ..default()
            },
            TextColor(theme::text_dim(&palette)),
            Node {
                width: Val::Px(74.0),
                ..default()
            },
            ChildOf(row),
        ));
        for shade in SWATCHES {
            commands.spawn((
                Swatch {
                    ramp: Some(name.clone()),
                    shade,
                },
                Interaction::default(),
                Node {
                    width: Val::Px(20.0),
                    height: Val::Px(18.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(palette.shade(&name, shade)),
                BorderColor::all(Color::BLACK.with_alpha(0.35)),
                ChildOf(row),
            ));
        }
    }
}

/// Shows the palette while painting, arms whatever is clicked, and rings the
/// armed colour in gold.
#[allow(clippy::too_many_arguments)]
pub(crate) fn work_palette(
    palette: Res<Palette>,
    mode: Res<crate::gizmo::ToolMode>,
    hovered: Res<Hovered>,
    placed: Query<&Placed, Without<Ghost>>,
    mut brush: ResMut<Brush>,
    mut panels: Query<&mut Visibility, With<PalettePanel>>,
    mut face: Query<&mut BackgroundColor, With<BrushFace>>,
    mut swatches: Query<(&Swatch, &Interaction, &mut BorderColor)>,
) {
    let painting = *mode == crate::gizmo::ToolMode::Paint;
    for mut visibility in &mut panels {
        let wanted = if painting {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
    if !painting {
        return;
    }
    for (swatch, interaction, _) in &swatches {
        if *interaction == Interaction::Pressed {
            brush.ramp = swatch.ramp.clone();
            brush.shade = swatch.shade;
        }
    }
    // The brush's own face. An empty brush strips rather than paints, and shows
    // as the panel's own dark rather than as a colour it does not have.
    let showing = match brush.ramp.as_deref() {
        Some(ramp) => palette.shade(ramp, brush.shade),
        None => Color::BLACK.with_alpha(0.30),
    };
    for mut fill in &mut face {
        if fill.0 != showing {
            *fill = BackgroundColor(showing);
        }
    }

    // What the part under the cursor is wearing, so the palette can point at
    // it. Brett's idea, and better than the eyedropper he first reached for:
    // there is no tool to arm and no modifier to hold, and once the swatch has
    // lit up, clicking it is the whole of picking the colour up. It also tells a
    // maker where in the ramps a colour they liked actually lives, which an
    // eyedropper never would.
    let worn = hovered
        .grab
        .and_then(|part| placed.get(part).ok())
        .map(|record| (record.ramp.clone(), record.shade));

    for (swatch, _, mut border) in &mut swatches {
        let same_as = |ramp: &Option<String>, shade: f32| {
            swatch.ramp.as_deref() == ramp.as_deref()
                // The bare swatch stands for a part with no paint at all, and a
                // part like that has a shade the maker never chose.
                && (swatch.ramp.is_none()
                    // Half a step, because a shade set before the swatches
                    // existed - or nudged from an odd starting point - can sit
                    // between two of them, and the nearer one is the honest
                    // answer.
                    || (swatch.shade - shade).abs() < 0.13)
        };
        let armed = same_as(&brush.ramp, brush.shade);
        let shown = worn
            .as_ref()
            .is_some_and(|(ramp, shade)| same_as(ramp, *shade));
        let dress = BorderColor::all(match (armed, shown) {
            // Armed wins the gold: what the next click will lay down matters
            // more than what the cursor happens to be over.
            (true, _) => theme::accent(&palette),
            (false, true) => Color::WHITE.with_alpha(0.85),
            (false, false) => Color::BLACK.with_alpha(0.35),
        });
        if *border != dress {
            *border = dress;
        }
    }
}

// ---------------------------------------------------------------- the shelf
