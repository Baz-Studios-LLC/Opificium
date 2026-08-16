//! The tool modes and the fine hand.
//!
//! A mode bar rides the top of the window, the way the big 3D programs
//! do it: NORMAL places and grabs, MOVE puts translate arrows on what
//! you click, RESIZE puts end-handles on any sized primitive and drags
//! its dimensions in quarter-metre steps. Tab walks the modes.

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::Bench;
use crate::builder::{self, Ghost, Hovered, Naming, PartKind, Placed};
use crate::look::Palette;

/// Which tool the mouse is.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ToolMode {
    #[default]
    Normal,
    Move,
    Resize,
    /// Colour what is already standing: clicking a part paints it with the
    /// brush rather than merely selecting it.
    Paint,
}

/// The part currently wearing handles.
#[derive(Resource, Default)]
pub struct Selected(pub Vec<Entity>);

impl Selected {
    /// The one part chosen, when exactly one is.
    ///
    /// Handles ask this rather than "the first of them", and the difference is
    /// the whole rule: a single part wears its OWN handles - a gable roof keeps
    /// its eaves and its ridge - while several together can only be moved, since
    /// stretching six things at once has no meaning to invent.
    pub fn one(&self) -> Option<Entity> {
        (self.0.len() == 1).then(|| self.0[0])
    }

    /// The part a menu or a measure speaks about: the first chosen.
    pub fn lead(&self) -> Option<Entity> {
        self.0.first().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn holds(&self, part: Entity) -> bool {
        self.0.contains(&part)
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// In or out - a shift-click on something already chosen lets it go.
    pub fn toggle(&mut self, part: Entity) {
        if let Some(at) = self.0.iter().position(|held| *held == part) {
            self.0.remove(at);
        } else {
            self.0.push(part);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.0.iter().copied()
    }
}

/// A drag in progress.
pub struct DragState {
    dir: Vec3,
    t0: f32,
    start_at: Vec3,
    grip: Grip,
}

#[derive(Clone, Copy)]
pub(crate) enum Grip {
    /// Slide the whole part along the handle's direction.
    Slide,
    /// Pull one end of a sized primitive: which local axis, and the
    /// dimensions when the grip closed. The handle's own direction
    /// already points out of the pulled end.
    Size {
        on_x: bool,
        w0: f32,
        d0: f32,
        /// What the part actually MEASURED when the handle was taken hold of.
        was: Vec2,
    },
    /// Pull a whole roof's eaves out past the walls, leaving the gables
    /// where the building is.
    Over { o0: f32 },
    /// Pull a roof's ridge up or down: how tall it stood when the grip closed.
    /// The eaves do not move, so a roof rises where it stands.
    ///
    /// A HEIGHT rather than an angle, because that is what the hand is pulling
    /// and because the two roofs answer it differently - a gable steepens, a hip
    /// closes its deck and then steepens.
    Pitch { high0: f32 },
    /// Raise or lower a flight's handrail. The treads do not move.
    Rail { h0: f32 },
    /// Raise or lower a pad: how tall the stone stands.
    Rise { h0: f32 },
}

#[derive(Resource, Default)]
pub struct GizmoDrag(Option<DragState>);

/// True while the cursor rides a handle or a drag is live.
#[derive(Resource, Default)]
pub struct GizmoHot(pub bool);

#[derive(Component)]
struct GizmoRoot;

/// A handle: its world direction, its dye, and what gripping it does.
#[derive(Component)]
struct Handle {
    dir: Vec3,
    ramp: &'static str,
    grip: Grip,
}

/// Visible to the crate so other benches can say "the bench eye, not this one".
/// The overlay camera rides along with the real one and must never be mistaken
/// for it: aiming a cursor ray through it, or setting its projection, quietly
/// breaks whatever asked.
#[derive(Component)]
pub(crate) struct GizmoCamera;

const ARROW_LAYER: usize = 1;

pub struct GizmoPlugin;

impl Plugin for GizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolMode>()
            .init_resource::<Selected>()
            .init_resource::<GizmoDrag>()
            .init_resource::<GizmoHot>()
            .add_systems(Startup, raise_gizmo_camera)
            .add_systems(
                Update,
                (walk_modes, select_part, dress_gizmo, work_gizmo)
                    .chain()
                    // ONE CLICK LETS GO AND PICKS UP. The choice is read by the grab -
                    // it stands aside while anything is chosen - so letting go has to
                    // happen first, or the click that drops a selection is spent doing
                    // only that and the maker clicks twice for one pickup. Brett: "When
                    // I placed them I couldnt pick them bqck up."
                    //
                    // And after the feel, because what is under the cursor is what this
                    // chooses: reading it before would choose from the frame before.
                    .after(builder::feel_ahead)
                    .before(builder::place_grab_remove)
                    // Moving, resizing and painting are things done to a
                    // BUILDING. Brett, on finding he could stretch a villager's
                    // head: "I shouldnt be able to stretch or resize any part of
                    // them." The rig bench has one tool and it is the hand.
                    .run_if(|bench: Res<crate::Bench>| *bench == crate::Bench::Builder),
            )
            .add_systems(Update, ride_along)
            .add_systems(
                Update,
                // And the modes go back to NORMAL on the way out, so a maker who
                // left the builder in RESIZE does not come back to a bench that
                // has been in RESIZE all the while they were somewhere else.
                |bench: Res<crate::Bench>,
                 mut mode: ResMut<ToolMode>,
                 mut chosen: ResMut<Selected>| {
                    if bench.is_changed() && *bench != crate::Bench::Builder {
                        *mode = ToolMode::Normal;
                        chosen.clear();
                    }
                },
            );
    }
}

/// The overlay camera: same eye as the bench camera, drawing only the
/// arrow layer, after everything, onto a cleared depth buffer. The UI
/// rides it too, so panels stay above the arrows.
fn raise_gizmo_camera(mut commands: Commands) {
    commands.spawn((
        GizmoCamera,
        Camera3d::default(),
        Camera {
            order: 1,
            clear_color: bevy::camera::ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
        RenderLayers::layer(ARROW_LAYER),
        Transform::default(),
    ));
}

fn ride_along(
    bench_camera: Query<&Transform, (With<Camera3d>, Without<GizmoCamera>)>,
    mut overlay: Query<&mut Transform, With<GizmoCamera>>,
) {
    let Ok(eye) = bench_camera.single() else {
        return;
    };
    for mut camera in &mut overlay {
        *camera = *eye;
    }
}

/// Tab walks the modes; returning to NORMAL drops the selection.
fn walk_modes(
    keys: Res<ButtonInput<KeyCode>>,
    bench: Res<Bench>,
    naming: Res<Naming>,
    dims: Res<builder::DimsEntry>,
    mut mode: ResMut<ToolMode>,
    mut selected: ResMut<Selected>,
) {
    if *bench != Bench::Builder || naming.0.is_some() || dims.0.is_some() {
        return;
    }
    if keys.just_pressed(KeyCode::Tab) {
        *mode = match *mode {
            ToolMode::Normal => ToolMode::Move,
            ToolMode::Move => ToolMode::Resize,
            ToolMode::Resize => ToolMode::Paint,
            ToolMode::Paint => ToolMode::Normal,
        };
    }
    if mode.is_changed() && *mode == ToolMode::Normal {
        selected.clear();
    }
}

/// In MOVE and RESIZE, a left click selects what the cursor touches;
/// escape and vanishing deselect. NORMAL keeps no selection at all.
#[allow(clippy::too_many_arguments)]
fn select_part(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mode: Res<ToolMode>,
    hot: Res<GizmoHot>,
    bench: Res<Bench>,
    naming: Res<Naming>,
    dims: Res<builder::DimsEntry>,
    hovered: Res<Hovered>,
    parts: Query<(), With<Placed>>,
    records: Query<(Entity, &Placed), Without<Ghost>>,
    mut selected: ResMut<Selected>,
) {
    if dims.0.is_some() {
        return;
    }
    if *bench != Bench::Builder || naming.0.is_some() {
        selected.clear();
        return;
    }
    // NORMAL is placement: a click there picks a part up, and always did. But
    // SHIFT-click is free in that mode, and grouping is the one thing a maker
    // had to leave the mode to do - Brett: "What about shift clicking in normal
    // mode? Should that work too? Right now it just picks the piece up." So
    // shift gathers here as well, and a plain click lets go of what was gathered
    // before picking anything up.
    let gathering = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if *mode == ToolMode::Normal && !gathering {
        if buttons.just_pressed(MouseButton::Left) {
            selected.clear();
        }
        return;
    }
    // A part that has gone - buried, or left behind on another step - stops
    // being chosen without taking the rest of the choice with it.
    selected.0.retain(|part| parts.contains(*part));
    // A click on NOTHING lets go. Without it a choice could only be dropped with the
    // escape key, and everything a click does in the builder - placing, picking up,
    // punching - stands aside while one is held: a maker who had chosen something
    // earlier found their next click doing nothing at all and no way to tell why.
    // Brett: "When I placed them I couldnt pick them bqck up."
    if buttons.just_pressed(MouseButton::Left)
        && !hot.0
        && hovered.grab.is_none()
        && !gathering
        && !selected.is_empty()
    {
        selected.clear();
    }
    if buttons.just_pressed(MouseButton::Left)
        && !hot.0
        && let Some(touched) = hovered.grab
    {
        // Shift adds and takes away; a plain click starts afresh. Clicking a
        // part that belongs to a group takes the whole group, which is what
        // being grouped MEANS - see `builder::kin_of`.
        // The part that BROUGHT the group leads it. A table's chairs are the
        // table's own, and it is the table a maker means when they take hold of
        // one - so it goes first, and the handles, the menu and the measure all
        // speak about it. Without this the lead was whichever member the query
        // happened to reach first.
        let mut kin = builder::kin_of(touched, &records);
        kin.sort_by_key(|part| {
            records
                .get(*part)
                .ok()
                .and_then(|(_, standing)| builder::kind_from_name(&standing.part))
                .is_none_or(|kind| builder::company_of(&kind).is_empty())
        });
        if gathering {
            for part in kin {
                selected.toggle(part);
            }
        } else {
            selected.clear();
            for part in kin {
                selected.toggle(part);
            }
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        selected.clear();
    }
}

/// The handles a part deserves in the standing mode: direction, offset
/// of the handle's FOOT from the part's origin, dye, grip.
/// What a part becomes when the gold handle takes it to a new height.
///
/// The ANSWER to `stands_at` below, and a function so that the two can be asked
/// the same question in a test. This pair is where the bench keeps hurting
/// itself: the handle was once offered on every wall and answered only for
/// framed ones, so it appeared on a plain wall and did nothing when pulled.
pub(crate) fn risen(kind: PartKind, high: f32) -> Option<PartKind> {
    match kind {
        PartKind::Pole { knees, .. } => Some(PartKind::Pole { high, knees }),
        PartKind::Area {
            word, long, deep, ..
        } => Some(PartKind::Area {
            word,
            long,
            deep,
            high,
        }),
        PartKind::Foundation(w, d, _) => Some(PartKind::Foundation(w, d, high)),
        // ANY wall, framed or not: every wall has a height.
        PartKind::Wall {
            long,
            framed,
            openings,
            ..
        } => Some(PartKind::Wall {
            long,
            high,
            framed,
            openings,
        }),
        _ => None,
    }
}

/// How tall a part stands, when that is a thing it can be told.
///
/// The OFFER: a gold handle appears exactly where this answers.
pub(crate) fn stands_at(kind: &PartKind) -> Option<f32> {
    match kind {
        PartKind::Foundation(_, _, high) => Some(*high),
        // A framed wall wants this more than a pad does: its length is one of the
        // two numbers it solves from and its height is the other. Pulled taller it
        // re-solves rather than stretching, so the plates stay plates.
        PartKind::Wall { high, .. } => Some(*high),
        // A POST wears this one alone. It is a beam stood up, and how tall it
        // stands is the only number it has.
        PartKind::Pole { high, .. } => Some(*high),
        // HOW HIGH A STACK MAY GROW, on the same gold handle a pad rises by -
        // which is what it is: a volume standing on its own foot.
        PartKind::Area { high, .. } => Some(*high),
        _ => None,
    }
}

/// How tall a roof or a gable stands at its ridge.
///
/// The OFFER for the gold handle at the apex: it is placed here, and the drag
/// starts from here. A hip's height is its RUN against its pitch and not its
/// half-span, so a handle placed by the gable's arithmetic hung in the air above
/// a hip - which is half of why pulling it did nothing.
pub(crate) fn apex_of(kind: &PartKind) -> f32 {
    builder::body_of(kind, None)
        .iter()
        .map(|builder::Slab { at, size, .. }| at.y + size.y * 0.5)
        .fold(0.0f32, f32::max)
}

/// What a roof becomes when its ridge is pulled to a height.
///
/// The ANSWER, and the reason this is a function: the handle was offered on BOTH
/// roofs and answered for the gable alone, so a hip wore a gold arrow that did
/// nothing whatever when pulled. Brett: "the hip roof has a yellow resize handle
/// that should pull up but it doesnt do anything."
///
/// A GABLE steepens about its eaves. A HIP closes its deck first - the slopes run
/// further in as it rises, until they meet in a ridge, or in a point when the
/// roof is square - and only then does it steepen. Brett again, and it is the
/// better gesture: "What if pulling that up increased the roof height until it
/// made a point?"
pub(crate) fn pitched(kind: PartKind, rise: f32) -> Option<PartKind> {
    let rise = rise.max(0.02);
    // Snapped in DEGREES rather than at the ridge, so two roofs meant to match
    // match exactly however differently they were dragged.
    let angle = |across: f32| {
        let half = (across * 0.5).max(0.125);
        (((rise / half).atan().to_degrees() / builder::PITCH_STEP).round() * builder::PITCH_STEP)
            .clamp(builder::PITCH_LEAST, builder::PITCH_MOST)
    };
    match kind {
        PartKind::GableRoof(long, span, over, _) => {
            Some(PartKind::GableRoof(long, span, over, angle(span)))
        }
        PartKind::Gable { long, framed, .. } => Some(PartKind::Gable {
            long,
            pitch: angle(long),
            framed,
        }),
        PartKind::HipRoof(long, span, over, pitch, _) => {
            // How far the slopes may run before they meet, and how tall the roof
            // is when they do. Past that there is no deck left to close and the
            // pull becomes a pitch, the way a gable's always is.
            let reach = (long * 0.5 + over)
                .min(span * 0.5 + over)
                .max(builder::ATOM);
            let shut = reach * pitch.to_radians().tan();
            if rise >= shut {
                let steeper = (((rise / reach).atan().to_degrees() / builder::PITCH_STEP).round()
                    * builder::PITCH_STEP)
                    .clamp(builder::PITCH_LEAST, builder::PITCH_MOST);
                return Some(PartKind::HipRoof(long, span, over, steeper, 0.0));
            }
            // The RUN lands on the lattice, and the deck is what is left of it -
            // so a hip's own faces meet the grid the way every other part's do.
            let run = builder::on_the_lattice(rise / pitch.to_radians().tan().max(1e-3))
                .clamp(builder::ATOM, reach);
            Some(PartKind::HipRoof(
                long,
                span,
                over,
                pitch,
                1.0 - run / reach,
            ))
        }
        _ => None,
    }
}

/// What a CHOICE wears, which is not always what one part wears.
///
/// ONE part wears its own handles for the standing mode. SEVERAL wear the MOVE
/// handles and nothing else, in every mode but the brush - `Selected::one` has
/// said so all along ("several together can only be moved, since stretching six
/// things at once has no meaning to invent"), and the slide already DOES it,
/// carrying every other chosen part the same distance.
///
/// Only the handles were never hung. They asked for the one part, so a group of
/// four had nothing to take hold of - in any mode, the standing one included.
/// Brett: "When I group a lot of items into a group, I should be able to move
/// the group as one piece right?", and "currently Normal mode ignores the group
/// as well."
pub(crate) fn handles_for_choice(
    mode: ToolMode,
    count: usize,
    record: &Placed,
) -> Vec<(Vec3, Vec3, &'static str, Grip)> {
    // SOME GROUPS CAN BE STRETCHED, and the flag is the group itself: a part that
    // BRINGS its company owns the group it made, so the group is one thing rather
    // than several a maker gathered. Brett, having dragged a table's handle and
    // pulled the board out from under its own chairs: "Can we flag that SOME groups
    // can be stretched?"
    //
    // Sizing a gathered six has no meaning to invent. Sizing a council's board is
    // exactly what a maker means, and its chairs follow it - see the size grip.
    let owns_its_company = builder::kind_from_name(&record.part)
        .is_some_and(|kind| !builder::company_of(&kind).is_empty());
    if count > 1 && !owns_its_company {
        return match mode {
            // Painting wears no handles whatever is chosen: the part IS the
            // handle, and a shaft in the way is something to click by mistake.
            ToolMode::Paint => Vec::new(),
            _ => handles_for(ToolMode::Move, record),
        };
    }
    handles_for(mode, record)
}

fn handles_for(mode: ToolMode, record: &Placed) -> Vec<(Vec3, Vec3, &'static str, Grip)> {
    // The part's TRUE pose, tilt and mirror included: a pitched panel's
    // handles must run up its own slope, or dragging one swings the far
    // edge instead of lengthening the roof, and a 45 looks like it bends.
    let spin = builder::pose(record.yaw, record.tilt, record.flip);
    match mode {
        // Painting wears no handles: the part IS the handle, and a shaft in the
        // way would only be something to click by mistake.
        ToolMode::Paint => Vec::new(),
        ToolMode::Move => vec![
            (Vec3::X, Vec3::ZERO, "cloth-red", Grip::Slide),
            (Vec3::Y, Vec3::ZERO, "cloth-gold", Grip::Slide),
            (Vec3::Z, Vec3::ZERO, "cloth-blue", Grip::Slide),
        ],
        ToolMode::Resize => {
            let standing = builder::kind_from_name(&record.part);
            // What the part MEASURES right now, kept with the grip so the mover
            // can tell how much it truly grew.
            let was = standing
                .as_ref()
                .map(builder::extent_of)
                .unwrap_or(Vec2::ZERO);
            let sized = standing.and_then(|kind| match kind {
                PartKind::Wall { long, .. } => Some((long, 0.0, false)),
                // A framed wall sizes along its length like any other wall.
                // Its height is the gold handle below - and this match is what
                // decides whether that one appears at all, because a part that
                // is not sized returns NO handles, gold included.
                // The pieces a punch leaves are walls too, and stretch
                // like them - only their height and lift stay put.
                PartKind::Seg { long, .. } => Some((long, 0.0, false)),
                PartKind::Trim { long, .. } => Some((long, 0.0, false)),
                PartKind::Rail { long, .. } => Some((long, 0.0, false)),
                PartKind::Gable { long, .. } => Some((long, 0.0, false)),
                PartKind::Beam(long, ..) => Some((long, 0.0, false)),
                // The chimney sizes its own reach downward.
                PartKind::Chimney(drop) => Some((drop, 0.0, false)),
                // A flight wears both handles. Across is its WIDTH, which is a
                // real measurement of it. Along is its RUN - and a longer run is
                // a taller flight, because the treads are even and the count is
                // what changes, so pulling it out really is climbing higher.
                PartKind::Stairs { rise, wide, .. } => {
                    let (steps, _, tread) = builder::stair_rhythm(rise);
                    Some((wide, steps as f32 * tread, true))
                }
                PartKind::Ridge(long) => Some((long, 0.0, false)),
                // A dial is square, so its one handle sizes the whole face.
                PartKind::Clock(wide) => Some((wide, 0.0, false)),
                // A table is sized both ways: a council's board is long AND wide.
                PartKind::Table(long, deep) => Some((long, deep, true)),
                PartKind::Floor(w, d) => Some((w, d, true)),
                // A MARKED VOLUME is dragged to the room it means, both ways.
                PartKind::Area { long, deep, .. } => Some((long, deep, true)),
                PartKind::Ceiling { long, deep, .. } => Some((long, deep, true)),
                PartKind::Foundation(w, d, _) => Some((w, d, true)),
                PartKind::Roof(w, d) => Some((w, d, true)),
                PartKind::GableRoof(w, d, _, _) => Some((w, d, true)),
                PartKind::HipRoof(w, d, ..) => Some((w, d, true)),
                _ => None,
            });
            // A part sized along its own footprint wears the red pair, and a
            // rectangle the blue one as well. A part that is only TALL wears
            // neither - it is not sized flat at all - and used to leave here with
            // no handles whatever, gold included, because this was the gate for
            // the lot.
            let mut handles = Vec::new();
            if let Some((w, d, both)) = sized {
                for end in [-1.0f32, 1.0] {
                    let dir = spin * (Vec3::X * end);
                    handles.push((
                        dir,
                        dir * (w * 0.5),
                        "cloth-red",
                        Grip::Size {
                            on_x: true,
                            w0: w,
                            d0: d,
                            was,
                        },
                    ));
                }
                if both {
                    for end in [-1.0f32, 1.0] {
                        let dir = spin * (Vec3::Z * end);
                        handles.push((
                            dir,
                            dir * (d * 0.5),
                            "cloth-blue",
                            Grip::Size {
                                on_x: false,
                                w0: w,
                                d0: d,
                                was,
                            },
                        ));
                    }
                }
            }
            // A pad carries one more, in gold: how TALL it stands. Both of the
            // red-and-blue pair are spoken for by its footprint, and a footing
            // that cannot be raised cannot reach the ground on a slope.
            let stands = builder::kind_from_name(&record.part).and_then(|kind| stands_at(&kind));
            if let Some(high) = stands {
                handles.push((
                    spin * Vec3::Y,
                    spin * Vec3::new(0.0, high, 0.0),
                    "cloth-gold",
                    Grip::Rise { h0: high },
                ));
            }
            // A flat rail carries the same gold handle a flight's does, and for
            // the same reason: both of the red-and-blue pair are spoken for, and
            // a landing has to meet the flight it continues.
            if let Some(PartKind::Rail { hand, .. }) = builder::kind_from_name(&record.part) {
                handles.push((
                    spin * Vec3::Y,
                    spin * Vec3::new(0.0, hand, 0.0),
                    "cloth-gold",
                    Grip::Rail { h0: hand },
                ));
            }
            // A flight carries one more, in gold: the rail's own height, since
            // both of the red-and-blue pair are spoken for by its width and its
            // run. Brett: "maybe we can add a handle for rail height?"
            if let Some(PartKind::Stairs { rise, hand, .. }) = builder::kind_from_name(&record.part)
            {
                handles.push((
                    spin * Vec3::Y,
                    spin * Vec3::new(0.0, rise + hand, 0.0),
                    "cloth-gold",
                    Grip::Rail { h0: hand },
                ));
            }
            // A whole roof carries two more, in gold: the eaves, which
            // reach out past the walls without taking the gables with
            // them.
            // Both roofs wear the gold pair: a hip has eaves and a pitch just
            // as a gable roof does, and Brett asked for the lot - "it needs
            // resize handles for everything too".
            if let Some((long, span, over, kind)) = match builder::kind_from_name(&record.part) {
                Some(
                    kind @ (PartKind::GableRoof(long, span, over, _)
                    | PartKind::HipRoof(long, span, over, ..)),
                ) => Some((long, span, over, kind)),
                _ => None,
            } {
                // Stood off along the ridge, not straight out past the blue
                // ones. A handle is a shaft one and a third long with a head on
                // the end, and these sat on the SAME LINE as the depth handles
                // with only the overhang between the two feet - so the gold lay
                // inside the blue for three quarters of its length and took
                // every click meant for it. Brett: "these yellow and blue
                // handles overlap so i cant use the blue handles."
                //
                // A quarter of the roof's length to one side, and never less
                // than a stride, so a short roof separates them too.
                let aside = spin * Vec3::X * (long * 0.25).max(0.9);
                for end in [-1.0f32, 1.0] {
                    let dir = spin * (Vec3::Z * end);
                    handles.push((
                        dir,
                        dir * (span * 0.5 + over + 0.35) + aside,
                        "cloth-gold",
                        Grip::Over { o0: over },
                    ));
                }
                // And one at the ridge, straight up. On a GABLE ROOF it is the
                // pitch: pull the ridge and the roof steepens about its eaves,
                // which stay on the walls where they were set. On a HIP it closes
                // the deck first and steepens after - see `pitched`.
                //
                // Standing at the roof's OWN apex, read off the part rather than
                // worked out here: a hip's height is its run against its pitch, not
                // its half-span, so a handle placed by the gable's arithmetic hung
                // in the air above a hip instead of on it.
                let rise = apex_of(&kind);
                handles.push((
                    spin * Vec3::Y,
                    spin * (Vec3::Y * (rise + 0.4)),
                    "cloth-gold",
                    Grip::Pitch { high0: rise },
                ));
            }
            // A gable is pulled by its peak the same way, so the wall under a
            // steepened roof can be steepened to meet it.
            if let Some(kind @ PartKind::Gable { .. }) = builder::kind_from_name(&record.part) {
                let rise = apex_of(&kind);
                handles.push((
                    spin * Vec3::Y,
                    spin * (Vec3::Y * (rise + 0.4)),
                    "cloth-gold",
                    Grip::Pitch { high0: rise },
                ));
            }
            handles
        }
        ToolMode::Normal => Vec::new(),
    }
}

/// Raises, moves and retires the handles as selection, mode and the
/// part's own size change.
#[allow(clippy::too_many_arguments)]
fn dress_gizmo(
    mut commands: Commands,
    selected: Res<Selected>,
    mode: Res<ToolMode>,
    palette: Res<Palette>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    parts: Query<(&Transform, &Placed), Without<GizmoRoot>>,
    roots: Query<Entity, With<GizmoRoot>>,
    mut stamp: Local<(Option<Entity>, ToolMode, String, usize)>,
) {
    // ONE part wears its own handles for the standing mode. SEVERAL wear the MOVE
    // handles and nothing else, at the middle of the lot.
    //
    // Which is what `Selected::one` has said all along - "several together can
    // only be moved, since stretching six things at once has no meaning to
    // invent" - and what the slide already DOES, carrying every other chosen part
    // the same distance. Only the handles were never hung: they asked for the one
    // part, so a group of four had nothing to take hold of and could not be moved
    // at all. Brett: "When I group a lot of items into a group, I should be able
    // to move the group as one piece right?"
    let lead = selected.lead().and_then(|part| parts.get(part).ok());
    let count = selected.iter().filter(|part| parts.contains(*part)).count();
    let fresh = (
        selected.lead(),
        *mode,
        lead.map(|(_, record)| record.part.clone())
            .unwrap_or_default(),
        count,
    );
    // The middle of everything chosen, so a group's arrows stand among the parts
    // rather than on whichever of them happened to be clicked first.
    let seat = || {
        let held: Vec<Vec3> = selected
            .iter()
            .filter_map(|part| parts.get(part).ok())
            .map(|(at, _)| at.translation)
            .collect();
        (!held.is_empty()).then(|| held.iter().sum::<Vec3>() / held.len() as f32)
    };
    if *stamp == fresh {
        if let Some(at) = seat() {
            for root in &roots {
                commands
                    .entity(root)
                    .insert(Transform::from_translation(at));
            }
        }
        return;
    }
    *stamp = fresh;
    for root in &roots {
        commands.entity(root).despawn();
    }
    let (Some((_, record)), Some(at)) = (lead, seat()) else {
        return;
    };
    let wanted = handles_for_choice(*mode, count, record);
    if wanted.is_empty() {
        return;
    }
    let root = commands
        .spawn((
            GizmoRoot,
            Transform::from_translation(at),
            Visibility::default(),
        ))
        .id();
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    for (dir, foot, ramp, grip) in wanted {
        let material = materials.add(StandardMaterial {
            base_color: palette.shade(ramp, 0.85),
            unlit: true,
            ..default()
        });
        let handle = commands
            .spawn((
                Handle { dir, ramp, grip },
                Transform::from_translation(foot),
                Visibility::default(),
                ChildOf(root),
            ))
            .id();
        // The shaft runs out of the foot along the direction, the head
        // caps it. Everything on the arrow layer.
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(dir * 0.65)
                .with_scale(Vec3::splat(0.05) + dir.abs() * (1.3 - 0.05)),
            RenderLayers::layer(ARROW_LAYER),
            ChildOf(handle),
        ));
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(dir * 1.35).with_scale(Vec3::splat(0.14)),
            RenderLayers::layer(ARROW_LAYER),
            ChildOf(handle),
        ));
    }
}

fn cursor_ray(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform), (With<Camera3d>, Without<GizmoCamera>)>,
) -> Option<Ray3d> {
    let window = windows.iter().next()?;
    let cursor = window.cursor_position()?;
    let (camera, camera_at) = cameras.iter().next()?;
    camera.viewport_to_world(camera_at, cursor).ok()
}

/// The parameter along `axis` (through `origin`) closest to the ray:
/// t = (e - b·f)/(1 - b²).
fn along_axis(ray: &Ray3d, origin: Vec3, axis: Vec3) -> Option<f32> {
    let toward = Vec3::from(ray.direction);
    let b = axis.dot(toward);
    let denominator = 1.0 - b * b;
    if denominator.abs() < 1e-4 {
        return None;
    }
    let w = ray.origin - origin;
    Some((w.dot(axis) - b * w.dot(toward)) / denominator)
}

fn ray_reach(ray: &Ray3d, point: Vec3) -> f32 {
    (point - ray.origin).dot(Vec3::from(ray.direction))
}

/// Dragging a handle: slides in MOVE (5cm steps), re-dimensions in
/// RESIZE (25cm steps, the far end standing still). Every change lands
/// in the part's record, and a resized body is rebuilt on the spot.
#[allow(clippy::too_many_arguments)]
fn work_gizmo(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    selected: Res<Selected>,
    mut drag: ResMut<GizmoDrag>,
    mut hot: ResMut<GizmoHot>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), (With<Camera3d>, Without<GizmoCamera>)>,
    handles: Query<(Entity, &Handle, &GlobalTransform)>,
    children: Query<&Children>,
    dyes: Query<&MeshMaterial3d<StandardMaterial>>,
    palette: Res<Palette>,
    grid: Res<builder::SnapGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut parts: Query<(Entity, &mut Transform, &mut Placed), Without<Handle>>,
) {
    // Handles hang on ONE part; a set of parts is moved by its own handles at
    // the middle, which are the move handles and no others.
    let Some(part) = selected.one().or_else(|| selected.lead()) else {
        drag.0 = None;
        hot.0 = false;
        return;
    };
    let Some(ray) = cursor_ray(&windows, &cameras) else {
        return;
    };

    // Which handle the cursor rides.
    let mut touched: Option<(Vec3, Grip)> = None;
    for (_, handle, at) in &handles {
        let origin = at.translation();
        let Some(t) = along_axis(&ray, origin, handle.dir) else {
            continue;
        };
        if !(-0.1..=1.5).contains(&t) {
            continue;
        }
        let on_axis = origin + handle.dir * t;
        let miss =
            (ray.origin + Vec3::from(ray.direction) * ray_reach(&ray, on_axis) - on_axis).length();
        if miss < 0.18 {
            touched = Some((handle.dir, handle.grip));
        }
    }
    hot.0 = touched.is_some() || drag.0.is_some();

    // The ridden handle brightens; a live drag keeps its own lit.
    let lit_dir = drag
        .0
        .as_ref()
        .map(|state| state.dir)
        .or(touched.map(|(dir, _)| dir));
    for (entity, handle, _) in &handles {
        let lit = lit_dir == Some(handle.dir);
        if let Ok(kids) = children.get(entity) {
            for &kid in kids {
                if let Ok(dye) = dyes.get(kid)
                    && let Some(mut material) = materials.get_mut(&dye.0)
                {
                    let wanted = palette.shade(handle.ramp, if lit { 1.0 } else { 0.85 });
                    if material.base_color != wanted {
                        material.base_color = wanted;
                    }
                }
            }
        }
    }

    if buttons.just_pressed(MouseButton::Left)
        && let Some((dir, grip)) = touched
        && let Ok((_, transform, _)) = parts.get_mut(part)
        && let Some(t0) = along_axis(&ray, transform.translation, dir)
    {
        drag.0 = Some(DragState {
            dir,
            t0,
            start_at: transform.translation,
            grip,
        });
    }
    if !buttons.pressed(MouseButton::Left) {
        drag.0 = None;
        return;
    }
    let Some(state) = drag.0.as_ref() else {
        return;
    };
    let Ok((_, mut transform, mut record)) = parts.get_mut(part) else {
        return;
    };
    let Some(t) = along_axis(&ray, state.start_at, state.dir) else {
        return;
    };
    // Every pull lands on the grid the bench is set to, which is what G
    // changes. It used to be whole sixteenths always - "resizing is fine work
    // by nature", which is true of most resizing and not of a maker who has
    // just asked for quarter metres because they are laying out a building.
    //
    // Placing a part already worked this way. A handle that did not meant the
    // same part could be put down on a quarter and then dragged off it.
    let per = 16.0 / grid.0.max(1) as f32;
    let step_of = |pull: f32| (pull * per).round() / per;

    match state.grip {
        Grip::Slide => {
            let step = step_of(t - state.t0);
            let was = transform.translation;
            transform.translation = state.start_at + state.dir * step;
            record.at = transform.translation.into();
            let moved = transform.translation - was;

            // Everything else chosen goes the same distance. The handle hangs on
            // one part, but a choice of several moves as one thing - that is
            // what choosing several is FOR.
            if moved.length_squared() > 0.0 {
                let others: Vec<Entity> = selected.iter().filter(|held| *held != part).collect();
                for other in others {
                    if let Ok((_, mut at, mut record)) = parts.get_mut(other) {
                        at.translation += moved;
                        record.at = at.translation.into();
                    }
                }
            }

            // And the marks EVERY moved part carries travel with it. A door slid
            // along a wall that left its routing mark behind would put the
            // village's doorway where the door used to be - and a table dragged
            // with its chairs left every sitting place standing in the old room,
            // because only the handle's own part was asked what it carried.
            if moved.length_squared() > 0.0 {
                let held: Vec<(Entity, Vec3, builder::Placed)> = parts
                    .iter()
                    .map(|(entity, at, record)| (entity, at.translation, record.clone()))
                    .collect();
                let mut carried: Vec<Entity> = Vec::new();
                for (owner, before) in std::iter::once((part, was)).chain(
                    held.iter()
                        .filter(|(entity, ..)| *entity != part && selected.holds(*entity))
                        // Where it WAS, which is where its marks still are.
                        .map(|(entity, at, _)| (*entity, *at - moved)),
                ) {
                    for mark in builder::carried_marks(
                        owner,
                        before,
                        held.iter().map(|(e, at, record)| (*e, *at, record)),
                    ) {
                        if !carried.contains(&mark) {
                            carried.push(mark);
                        }
                    }
                }
                for mark in carried {
                    if let Ok((_, mut mark_at, mut mark_record)) = parts.get_mut(mark) {
                        mark_at.translation += moved;
                        mark_record.at = mark_at.translation.into();
                    }
                }
            }
        }
        Grip::Pitch { high0 } => {
            // The drag is a ridge HEIGHT: pulling a ridge is what a roof looks
            // like being raised, and what it does with the height is the roof's
            // own business - see `pitched`.
            let Some(made) = builder::kind_from_name(&record.part)
                .and_then(|kind| pitched(kind, high0 + (t - state.t0)))
                .filter(|made| builder::part_name(made) != record.part)
            else {
                return;
            };
            record.part = builder::part_name(&made);
            commands.entity(part).despawn_related::<Children>();
            builder::dress_part(
                &mut commands,
                &mut meshes,
                &mut materials,
                &palette,
                &made,
                &record,
                part,
                false,
            );
        }
        Grip::Rise { h0 } => {
            // On the grid, like every other pull, and never thinner than one
            // atom: a pad of no height is a pad nobody can see or click.
            let pull = step_of(t - state.t0);
            let high = (h0 + pull).clamp(0.0625, 8.0);
            let Some(made) = builder::kind_from_name(&record.part)
                .and_then(|kind| risen(kind, high))
                .filter(|made| builder::part_name(made) != record.part)
            else {
                return;
            };
            record.part = builder::part_name(&made);
            // The part does NOT move. A pad's box is drawn from its origin
            // upward - the origin IS the underside - so growing it already grows
            // it upward and away from the ground it rests on.
            //
            // It used to be lifted by half the growth, on the assumption that
            // the box was centred on the origin like most parts. That put the
            // pad half an atom into the air for every atom it gained, and half
            // an atom is exactly what the lattice cannot have: Brett, "A
            // foundation on the ground when I stretch it up it seems to get off
            // the atom grid."
            commands.entity(part).despawn_related::<Children>();
            builder::dress_part(
                &mut commands,
                &mut meshes,
                &mut materials,
                &palette,
                &made,
                &record,
                part,
                false,
            );
        }
        Grip::Rail { h0 } => {
            // Whole atoms, and never below a step's own height or above a
            // chest. The result is snapped rather than the pull: see `Grip::Rise`.
            let pull = t - state.t0;
            let hand = builder::on_the_lattice(h0 + pull).clamp(0.375, 2.0);
            // The same handle serves a flight's rail and a flat one's.
            let made = match builder::kind_from_name(&record.part) {
                Some(PartKind::Stairs {
                    rise,
                    wide,
                    stone,
                    rail_stone,
                    hand: was,
                }) => {
                    if (hand - was).abs() < 1e-4 {
                        return;
                    }
                    PartKind::Stairs {
                        rise,
                        wide,
                        stone,
                        rail_stone,
                        hand,
                    }
                }
                Some(PartKind::Rail {
                    long,
                    hand: was,
                    stone,
                }) => {
                    if (hand - was).abs() < 1e-4 {
                        return;
                    }
                    PartKind::Rail { long, hand, stone }
                }
                _ => return,
            };
            record.part = builder::part_name(&made);
            commands.entity(part).despawn_related::<Children>();
            builder::dress_part(
                &mut commands,
                &mut meshes,
                &mut materials,
                &palette,
                &made,
                &record,
                part,
                false,
            );
        }
        Grip::Over { o0 } => {
            // The eaves reach out on the grid; the walls beneath and the
            // gables at the ends do not move at all.
            let pull = step_of(t - state.t0);
            let over = (o0 + pull).clamp(0.0, 3.0);
            // Both roofs reach their eaves out the same way.
            let made = match builder::kind_from_name(&record.part) {
                Some(PartKind::GableRoof(long, span, was, pitch)) => {
                    if (over - was).abs() < 1e-4 {
                        return;
                    }
                    PartKind::GableRoof(long, span, over, pitch)
                }
                Some(PartKind::HipRoof(long, span, was, pitch, deck)) => {
                    if (over - was).abs() < 1e-4 {
                        return;
                    }
                    PartKind::HipRoof(long, span, over, pitch, deck)
                }
                _ => return,
            };
            record.part = builder::part_name(&made);
            commands.entity(part).despawn_related::<Children>();
            builder::dress_part(
                &mut commands,
                &mut meshes,
                &mut materials,
                &palette,
                &made,
                &record,
                part,
                false,
            );
        }
        Grip::Size { on_x, w0, d0, was } => {
            // Pulling outward along the handle grows the dimension; the
            // far end stands still, so the centre walks half the growth.
            let pull = step_of(t - state.t0);
            let Some(kind) = builder::kind_from_name(&record.part) else {
                return;
            };
            let (w, d) = if on_x {
                ((w0 + pull).max(0.25), d0)
            } else {
                (w0, (d0 + pull).max(0.25))
            };
            // Measured rather than asked for: see `builder::extent_of`.
            let grown = 0.0;
            let Some(made) = sized(kind, w, d, grown) else {
                return;
            };
            let fresh = builder::part_name(&made);
            if fresh == record.part {
                return;
            }
            // The part keeps the end the maker is NOT pulling: it moves by half
            // of however much it truly grew, which for a part whose handle asks
            // for something other than its own width - a chimney's drop, a
            // flight's rise - is nothing at all.
            let now = builder::extent_of(&made);
            let truly = if on_x { now.x - was.x } else { now.y - was.y };
            let _ = grown;
            transform.translation = state.start_at + state.dir * (truly * 0.5);
            record.at = transform.translation.into();
            record.part = fresh;
            // The body is rebuilt in place; the entity, and with it the
            // selection, stands.
            commands.entity(part).despawn_related::<Children>();
            builder::dress_part(
                &mut commands,
                &mut meshes,
                &mut materials,
                &palette,
                &made,
                &record,
                part,
                false,
            );
            // AND ITS COMPANY FOLLOWS IT. A table pulled longer wants its chairs
            // spread down the new board and another pair on the end of it -
            // otherwise the board is dragged out from under them, which is what
            // Brett saw. The old company goes, marks and all, and is seated
            // afresh from the size the table is now.
            // Its own record, taken before the query is read again: the borrow the
            // handle holds on this part ends here.
            let seated = record.clone();
            if let Some(group) = seated.group
                && !builder::company_of(&made).is_empty()
            {
                let held: Vec<(Entity, Vec3, builder::Placed)> = parts
                    .iter()
                    .map(|(entity, at, record)| (entity, at.translation, record.clone()))
                    .collect();
                for (entity, at, standing) in &held {
                    if *entity == part || standing.group != Some(group) {
                        continue;
                    }
                    // Whatever it was carrying goes with it: a chair's own
                    // sitting place is no use once the chair has gone.
                    for mark in builder::carried_marks(
                        *entity,
                        *at,
                        held.iter().map(|(e, at, record)| (*e, *at, record)),
                    ) {
                        commands.entity(mark).despawn();
                    }
                    commands.entity(*entity).despawn();
                }
                builder::seat_the_company(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &palette,
                    &made,
                    &seated,
                    group,
                );
            }
        }
    }
}

/// WHAT A PART BECOMES when a red or blue handle drags it to a new footprint.
///
/// The ANSWER to the flat-sizing offer, lifted out of the drag so it can be ASKED.
/// It lived inside `work_gizmo`, which meant the only way to find out what a
/// ceiling did when it was dragged past square was to drag one - and what it did
/// was swing its ridge. A rule nothing can question is a rule nobody checks.
pub(crate) fn sized(kind: PartKind, w: f32, d: f32, grown: f32) -> Option<PartKind> {
    let _ = grown;
    Some(match kind {
        // A wall keeps everything it is; only its length is being pulled.
        PartKind::Wall {
            high,
            framed,
            openings,
            ..
        } => PartKind::Wall {
            long: w,
            high,
            framed,
            openings,
        },
        PartKind::Seg { high, lift, .. } => PartKind::Seg {
            long: w,
            high,
            lift,
        },
        PartKind::Trim { stone, .. } => PartKind::Trim { long: w, stone },
        PartKind::Rail { hand, stone, .. } => PartKind::Rail {
            long: w,
            hand,
            stone,
        },
        PartKind::Gable { pitch, framed, .. } => PartKind::Gable {
            long: w,
            pitch,
            framed,
        },
        PartKind::Beam(_, high, low) => PartKind::Beam(w, high, low),
        PartKind::Chimney(_) => PartKind::Chimney(w.max(0.0)),
        PartKind::Stairs {
            stone,
            rail_stone,
            hand,
            ..
        } => {
            let (_, riser, tread) = builder::stair_rhythm(0.0);
            let steps = (d / tread).round().clamp(2.0, 24.0);
            PartKind::Stairs {
                rise: steps * riser,
                wide: w.max(0.375),
                stone,
                rail_stone,
                hand,
            }
        }
        PartKind::Ridge(_) => PartKind::Ridge(w),
        PartKind::Clock(_) => PartKind::Clock(w),
        PartKind::Table(..) => PartKind::Table(w, d),
        PartKind::Floor(..) => PartKind::Floor(w, d),
        PartKind::Area { word, high, .. } => PartKind::Area {
            word,
            long: w,
            deep: d,
            high,
        },
        // Pulled to a new size, and it keeps the roof it was going to raise -
        // AND THE WAY IT WAS GOING TO RAISE IT.
        //
        // `across` is a flip of "the long side" rather than a direction, so
        // dragging a ceiling past square swapped which side was long and swung
        // the ridge with it, without a maker touching it. Brett: "Sometimes
        // when resizing a ceiling it auto changes the ridge. It should never
        // auto change. i can manually change it with R." The flag is worked
        // out again at the new sides so the beam stays where it was put.
        PartKind::Ceiling {
            long,
            deep,
            hipped,
            across,
        } => PartKind::Ceiling {
            long: w,
            deep: d,
            hipped,
            across: builder::ridge_across_for(w, d, builder::ridge_along_x(long, deep, across)),
        },
        PartKind::Foundation(_, _, high) => PartKind::Foundation(w, d, high),
        PartKind::Roof(..) => PartKind::Roof(w, d),
        PartKind::GableRoof(_, _, over, pitch) => PartKind::GableRoof(w, d, over, pitch),
        PartKind::HipRoof(_, _, over, pitch, deck) => PartKind::HipRoof(w, d, over, pitch, deck),
        // A part with no flat footprint is not dragged this way at all.
        _ => return None,
    })
}
