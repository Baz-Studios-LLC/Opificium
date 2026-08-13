//! The panel that says what is chosen, and the light on it.

use super::*;

/// The panel that says what is chosen, and offers to make one thing of it.
#[derive(Component)]
pub(crate) struct ChosenPanel;

/// What a part is CALLED, for a maker reading a list of them.
///
/// A record's own word is a shape and a measurement - `wallseg-3x2@0` - which
/// is the right thing to write in a file and the wrong thing to read in a list.
pub(crate) fn spoken_of(kind: &PartKind) -> String {
    match kind {
        PartKind::Wall(long) => format!("WALL, {long:.2}M"),
        PartKind::Seg { long, .. } => format!("WALL PIECE, {long:.2}M"),
        PartKind::Trim { long, stone } => {
            format!("{} TRIM, {long:.2}M", if *stone { "STONE" } else { "WOOD" })
        }
        PartKind::Gable(long, pitch) => format!("GABLE, {long:.2}M AT {pitch:.0}"),
        PartKind::Beam(long, high, low) => {
            if *high > 0.0 || *low > 0.0 {
                format!("BEAM, {long:.2}M, MITRED")
            } else {
                format!("BEAM, {long:.2}M")
            }
        }
        PartKind::Ridge(long) => format!("RIDGE, {long:.2}M"),
        PartKind::Chimney(drop) => format!("CHIMNEY, {drop:.2}M"),
        PartKind::GableRoof(long, span, _, pitch) => {
            format!("GABLE ROOF, {long:.2} X {span:.2}M AT {pitch:.0}")
        }
        PartKind::RoofPlan(w, d) => format!("ROOF PLAN, {w:.2} X {d:.2}M"),
        PartKind::Floor(w, d) => format!("FLOOR, {w:.2} X {d:.2}M"),
        PartKind::Foundation(w, d, high) => {
            format!("FOUNDATION, {w:.2} X {d:.2}M, {high:.2} TALL")
        }
        PartKind::Roof(w, d) => format!("ROOF PANEL, {w:.2} X {d:.2}M"),
        PartKind::Prop(what) | PartKind::Widget(what) => what.to_uppercase(),
        other => part_name(other).to_uppercase(),
    }
}

/// Hangs the list of what is chosen, and hides it when nothing is.
///
/// Brett: "Maybe a popup window that shows the group pieces listed would be
/// great too." A group is otherwise a thing a maker has to remember making -
/// the parts look no different, and the only way to ask what is in one was to
/// drag it and watch what followed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn hang_the_chosen(
    mut commands: Commands,
    bench: Res<Bench>,
    fonts: Res<Fonts>,
    palette: Res<Palette>,
    selected: Res<crate::gizmo::Selected>,
    panels: Query<Entity, With<ChosenPanel>>,
    records: Query<&Placed, Without<Ghost>>,
    mut hung: Local<usize>,
) {
    let count = if *bench == Bench::Builder {
        selected.0.len()
    } else {
        0
    };
    // Hung again when the count changes, or when the choice does - a swap of
    // one part for another leaves the count alone and the list wrong.
    if !selected.is_changed() && count == *hung && !bench.is_changed() {
        return;
    }
    *hung = count;
    for panel in &panels {
        commands.entity(panel).despawn();
    }
    if count < 2 {
        // One part is a part, not a group. The dimension readout already says
        // everything there is to say about it.
        return;
    }
    let grouped = selected
        .iter()
        .filter_map(|part| records.get(part).ok())
        .filter(|record| record.group.is_some())
        .count();

    let panel = commands
        .spawn((
            ChosenPanel,
            Node {
                position_type: PositionType::Absolute,
                // Upper right of the VIEW, clear of the shelf that owns the
                // right edge: the panel is about the thing under the cursor, and
                // the cursor is out here rather than over on the rail.
                right: Val::Px(222.0),
                top: Val::Px(10.0),
                width: Val::Px(232.0),
                max_height: Val::Px(420.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(theme::panel_bg()),
            BorderColor::all(theme::accent(&palette).with_alpha(0.5)),
            crate::look::Scrollable,
            bevy::ui::ScrollPosition::default(),
            GlobalZIndex(20),
        ))
        .id();
    commands.spawn((
        Text::new(if grouped == count {
            format!("A GROUP OF {count}")
        } else if grouped > 0 {
            format!("{count} CHOSEN - {grouped} OF THEM GROUPED")
        } else {
            format!("{count} CHOSEN")
        }),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: crate::look::text_at(12.0),
            ..default()
        },
        TextColor(theme::accent(&palette)),
        ChildOf(panel),
    ));
    let mut said: Vec<String> = selected
        .iter()
        .filter_map(|part| records.get(part).ok())
        .map(|record| {
            kind_from_name(&record.part)
                .map(|kind| spoken_of(&kind))
                .unwrap_or_else(|| record.part.to_uppercase())
        })
        .collect();
    said.sort();
    // Alike things counted rather than repeated: eight identical wall pieces
    // are one line saying eight, not eight lines a maker has to count.
    let mut runs: Vec<(String, usize)> = Vec::new();
    for word in said {
        match runs.last_mut() {
            Some((had, times)) if *had == word => *times += 1,
            _ => runs.push((word, 1)),
        }
    }
    for (word, times) in runs {
        commands.spawn((
            Text::new(if times > 1 {
                format!("{times} x {word}")
            } else {
                word
            }),
            TextFont {
                font: fonts.text.clone().into(),
                font_size: crate::look::text_at(11.0),
                ..default()
            },
            TextColor(theme::text_dim(&palette)),
            TextLayout::new(
                bevy::text::Justify::Left,
                bevy::text::LineBreak::WordBoundary,
            ),
            ChildOf(panel),
        ));
    }
    commands.spawn((
        Text::new(if grouped == count {
            "right-click to ungroup"
        } else {
            "right-click one of them to group"
        }),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: crate::look::text_at(10.0),
            ..default()
        },
        TextColor(theme::text_dim(&palette).with_alpha(0.7)),
        Node {
            margin: UiRect::top(Val::Px(5.0)),
            ..default()
        },
        ChildOf(panel),
    ));
}

/// What a part is wearing: nothing, the hand's own light, or the mark of being
/// chosen.
///
/// ONE system owns the glow. Hovering and choosing both used to write it, which
/// meant moving the cursor off a chosen part put its mark out - so shift-clicking
/// a second thing appeared to unchoose the first. Brett: "holding shift and
/// clicking multiple things doesnt highlight the multiple things".
#[allow(clippy::too_many_arguments)]
pub(crate) fn light_the_chosen(
    bench: Res<Bench>,
    hand: Res<Hand>,
    tool: Res<crate::gizmo::ToolMode>,
    hovered: Res<Hovered>,
    selected: Res<crate::gizmo::Selected>,
    placed: Query<Entity, (With<Placed>, Without<Ghost>)>,
    children: Query<&Children>,
    slabs: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut lit: Local<std::collections::HashMap<Entity, LinearRgba>>,
) {
    if *bench != Bench::Builder {
        return;
    }
    // The chosen wear a brighter mark than the hovered, and go on wearing it
    // wherever the cursor wanders.
    const CHOSEN: LinearRgba = LinearRgba::new(0.30, 0.23, 0.07, 1.0);
    const UNDER_HAND: LinearRgba = LinearRgba::new(0.14, 0.11, 0.04, 1.0);
    let touchable = hand.kind.is_none() || *tool != crate::gizmo::ToolMode::Normal;
    for part in &placed {
        let wanted = if selected.holds(part) {
            CHOSEN
        } else if hovered.grab == Some(part) && touchable {
            UNDER_HAND
        } else {
            LinearRgba::BLACK
        };
        // Written only when it changes: a hundred parts times their boxes, every
        // frame, is a lot of asset writes to say nothing.
        if lit.get(&part).copied() == Some(wanted) {
            continue;
        }
        lit.insert(part, wanted);
        let Ok(kids) = children.get(part) else {
            continue;
        };
        for &kid in kids {
            if let Ok(handle) = slabs.get(kid)
                && let Some(mut material) = materials.get_mut(&handle.0)
            {
                material.emissive = wanted;
            }
        }
    }
    lit.retain(|part, _| placed.contains(*part));
}
