//! THE TERRAIN — a game's world, shaped by hand.
//!
//! Open a game and its world is here: continents read off the map image it
//! keeps, ground generated the way that game generates it, and a brush to push
//! the ground about. What a maker sculpts is written back to the game's own
//! folder as `edits.bin`, and the game adds it to the same ground when it loads.
//! The two programs share no code. See FORMATS.md.
//!
//! # Why it is a bench of its own
//!
//! Everything else on the bench MAKES a thing that stands on the ground — boxes
//! on a lattice, a model on the grid. This makes the ground. It has no lattice,
//! because a hill does not snap to one; no ramp picker, because it wears the
//! game's biome colours by height and slope rather than by choice; and no shelf
//! of parts, because a game has exactly one world. It is measured in kilometres
//! where every other bench is measured in metres, which on its own is enough.
//!
//! # It is opened late
//!
//! Reading a map image, cleaning it, and sweeping it for distance-from-coast is
//! a second's work, and a maker who only ever draws buildings should never pay
//! it. Nothing here happens until somebody walks to this bench.

mod chunk;
pub mod edit;
mod forest;
mod ground;
mod opened;
mod settle;
mod shelf;
mod tree;

use bevy::prelude::*;

use crate::Bench;
use chunk::{Building, GroundColours, GroundMaterial, Standing};
use edit::{Brushing, Sculpt, Stamp};
use ground::{Ground, World};

/// How far the brush reaches from the eye, in metres.
const REACH: f32 = 4_000.0;
/// How far the ray steps while hunting the ground. Short enough not to tunnel
/// through a ridge, long enough that a miss is hundreds of samples and not
/// thousands.
const STEP: f32 = 2.5;

pub const MIN_RADIUS: f32 = 4.0;
pub const MAX_RADIUS: f32 = 600.0;
/// The radius changes by a PROPORTION per notch, so it feels the same whether
/// you are shaping a mound or a mountain range.
const RADIUS_STEP: f32 = 1.15;

pub const MIN_STRENGTH: f32 = 2.0;
pub const MAX_STRENGTH: f32 = 150.0;
const STRENGTH_STEP: f32 = 1.25;

/// What the brush is doing, and where.
#[derive(Resource)]
pub struct Brush {
    pub radius: f32,
    /// Metres a second, for the tools that push.
    pub strength: f32,
    pub how: Brushing,
    /// Where it rests on the ground, if it is on the ground at all.
    pub on: Option<Vec3>,
    /// The height a levelling stroke started at, so the whole stroke levels to
    /// one plane instead of chasing the ground as it moves.
    target: f32,
    /// Whether a stroke is open, for grouping the undo.
    stroking: bool,
    /// Where a two-point tool was first clicked, waiting for its second.
    pub pending: Option<Vec3>,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            radius: 60.0,
            strength: 25.0,
            how: Brushing::Raise,
            on: None,
            target: 0.0,
            stroking: false,
            pending: None,
        }
    }
}

/// Said back to the maker when something happened that leaves no mark.
#[derive(Resource, Default)]
pub struct Said(pub String, pub f32);

impl Said {
    pub fn that(&mut self, what: impl Into<String>) {
        self.0 = what.into();
        self.1 = 2.4;
    }
}

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Brush>()
            .init_resource::<Standing>()
            .init_resource::<Said>()
            .add_plugins(shelf::ShelfPlugin)
            .add_systems(Startup, ready_the_material)
            .add_systems(
                Update,
                (
                    arrive_or_leave,
                    // Choosing a world has to work when there is no world, so it
                    // cannot sit behind the condition that one is open.
                    take_a_world.run_if(standing_here),
                    (
                        chunk::stream,
                        chunk::collect,
                        chunk::move_the_sea,
                        aim,
                        adjust,
                        paint,
                        history,
                        keep,
                        draw_the_brush,
                        draw_the_sites,
                    )
                        .chain()
                        .run_if(at_the_terrain),
                    fade_what_was_said,
                )
                    .chain(),
            );
    }
}

fn standing_here(bench: Res<Bench>) -> bool {
    *bench == Bench::Terrain
}

fn at_the_terrain(bench: Res<Bench>, ground: Option<Res<Ground>>) -> bool {
    *bench == Bench::Terrain && ground.is_some()
}

fn ready_the_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<crate::look::Palette>,
) {
    commands.insert_resource(GroundMaterial(chunk::ground_material(&mut materials)));
    // The game's own ramps, resolved once. Every chunk is painted from these.
    commands.insert_resource(GroundColours(chunk::Colours::from(&palette)));
}

/// Opens the world the first time somebody walks here, and clears the stage on
/// the way out.
fn arrive_or_leave(
    mut commands: Commands,
    bench: Res<Bench>,
    ground: Option<Res<Ground>>,
    mut standing: ResMut<Standing>,
    mut rig: ResMut<crate::camera::OrbitRig>,
    // The bench eye only. The gizmo's overlay camera also wears `Camera3d`, and
    // reaching its projection from here would be this bench quietly changing how
    // another one draws its handles.
    mut eyes: Query<&mut Projection, (With<Camera3d>, Without<crate::gizmo::GizmoCamera>)>,
    mut said: ResMut<Said>,
    sea: Query<Entity, With<chunk::Sea>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<crate::look::Palette>,
    mut brush: ResMut<Brush>,
) {
    if !bench.is_changed() {
        return;
    }

    // How far the eye can see. A building is metres away and a coastline is
    // kilometres, and one far plane cannot serve both: at a thousand metres the
    // world is cut off mid-continent, and at thirty thousand a model loses its
    // depth precision to z-fighting. So it moves with the bench.
    let terrain = *bench == Bench::Terrain;
    for mut projection in &mut eyes {
        *projection = Projection::from(PerspectiveProjection {
            near: if terrain { 1.0 } else { 0.1 },
            far: if terrain { 30_000.0 } else { 1_000.0 },
            ..default()
        });
    }

    if !terrain {
        // The ground goes away with the maker. Hundreds of chunk meshes are a
        // great deal to leave standing behind a bench nobody is at, and putting
        // them back is a second's streaming.
        chunk::clear(&mut commands, &mut standing, &sea);
        // And so does a half-placed ramp. Coming back to a first point set
        // before you left is a click away from cutting a ramp to somewhere you
        // have forgotten choosing.
        brush.pending = None;
        return;
    }

    // Back to whichever world was open last, if it is still there. A maker who
    // spent yesterday on a coastline should be standing on it today without
    // going looking for it.
    if ground.is_none() {
        match opened::expected() {
            Some(folder) => stand_on(
                &mut commands,
                &folder,
                &mut rig,
                &mut said,
                &mut meshes,
                &mut materials,
                &palette,
            ),
            None => said.that("No world open yet - OPEN A WORLD on the shelf"),
        }
    }
}

/// Opens a world, lays the sea over it, and puts the eye somewhere useful.
#[allow(clippy::too_many_arguments)] // opening a world genuinely needs all of it
fn stand_on(
    commands: &mut Commands,
    folder: &std::path::Path,
    rig: &mut crate::camera::OrbitRig,
    said: &mut Said,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    palette: &crate::look::Palette,
) {
    let world = World::open(folder);
    let half = world.half();
    chunk::lay_the_sea(commands, meshes, materials, palette, half);

    // Far enough back to hold the WHOLE world, since the whole of it stands.
    // At Bevy's 45-degree view a widescreen window shows about 1.4 times the
    // distance across, so the world's full width divided by that frames it with
    // a little sea to spare.
    //
    // Angled rather than straight down: a plan view tells you where the coast is
    // and nothing about the shape of the ground, which is what a maker is here
    // to judge.
    rig.focus = Vec3::ZERO;
    rig.distance = (half.x * 2.0 / 1.4).clamp(300.0, 5_900.0);
    rig.pitch = 1.05;

    if world.has_map() {
        opened::remember(folder);
        said.that(format!("Opened {}", opened::called(folder)));
    } else {
        said.that("No heightmap.png in that folder");
    }
    commands.insert_resource(Ground(std::sync::Arc::new(world)));
}

/// A press on the shelf's OPEN A WORLD asks for one.
#[allow(clippy::too_many_arguments)] // opening a world genuinely needs all of it
#[allow(clippy::too_many_arguments)] // opening a world touches all of this
fn take_a_world(
    mut commands: Commands,
    pressed: Query<&Interaction, (Changed<Interaction>, With<shelf::OpenWorld>)>,
    ground: Option<Res<Ground>>,
    mut standing: ResMut<Standing>,
    mut rig: ResMut<crate::camera::OrbitRig>,
    mut said: ResMut<Said>,
    sea: Query<Entity, With<chunk::Sea>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<crate::look::Palette>,
    mut brush: ResMut<Brush>,
) {
    if !pressed.iter().any(|touch| *touch == Interaction::Pressed) {
        return;
    }
    // Start the dialog where the last world was, so swapping between two of them
    // is not two walks across the disk.
    let from = ground.as_ref().map(|ground| ground.folder().to_path_buf());
    let Some(folder) = opened::ask(from.as_deref()) else {
        return;
    };
    if from.as_deref() == Some(folder.as_path()) {
        return;
    }

    // The old world's ground goes with it. Chunks meshed from one world standing
    // in another would be scenery from somewhere else — and a ramp begun on one
    // coastline must not finish on another's.
    chunk::clear(&mut commands, &mut standing, &sea);
    brush.pending = None;
    stand_on(
        &mut commands,
        &folder,
        &mut rig,
        &mut said,
        &mut meshes,
        &mut materials,
        &palette,
    );
}

/// The stage: the window minus whatever furniture is standing on it.
///
/// A brush must not reach through the rail or the shelf. Asked of the same
/// `Showing` the panels themselves read, so a maker who puts the shelf away gets
/// the ground underneath it back.
fn on_the_stage(window: &Window, showing: &crate::look::Showing, at: Vec2) -> bool {
    let wide = crate::look::PANEL_WIDE;
    let left = if showing.wanted(crate::look::Tool::Rail) {
        wide
    } else {
        0.0
    };
    let right = window.width()
        - if showing.wanted(crate::look::Tool::Shelf) {
            wide
        } else {
            0.0
        };
    at.x > left && at.x < right && at.y > crate::menu::BAR_HIGH
}

/// Where the ground is under the pointer.
///
/// Marched until the ray goes under the ground, then halved sixteen times to
/// find the crossing. Cheaper than colliding against the chunk meshes, and it
/// works on ground that has not been meshed yet.
fn ground_under(world: &World, from: Vec3, along: Vec3) -> Option<Vec3> {
    let mut behind = from;
    let mut gone = 0.0;

    while gone < REACH {
        gone += STEP;
        let at = from + along * gone;
        if at.y <= world.height(at.x, at.z) {
            // `over` is always above ground and `under` always below, so the
            // surface is caught between them and this closes on it.
            let (mut over, mut under) = (behind, at);
            for _ in 0..16 {
                let middle = (over + under) * 0.5;
                if middle.y <= world.height(middle.x, middle.z) {
                    under = middle;
                } else {
                    over = middle;
                }
            }
            return Some(under);
        }
        behind = at;
    }
    None
}

fn aim(
    ground: Res<Ground>,
    showing: Res<crate::look::Showing>,
    windows: Query<&Window>,
    // The BENCH eye, explicitly. There are two cameras - the gizmo's overlay
    // rides along with this one - and asking for "the camera" gets neither,
    // because `single` refuses when there is more than one. That is exactly
    // what happened: the brush was never given a ray and no tool did anything.
    eyes: Query<(&Camera, &GlobalTransform), (With<Camera3d>, Without<crate::gizmo::GizmoCamera>)>,
    mut brush: ResMut<Brush>,
) {
    brush.on = None;
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(at) = window.cursor_position() else {
        return;
    };
    if !on_the_stage(window, &showing, at) {
        return;
    }
    let Ok((camera, eye)) = eyes.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(eye, at) else {
        return;
    };
    brush.on = ground_under(&ground.0, ray.origin, *ray.direction);
}

fn adjust(keys: Res<ButtonInput<KeyCode>>, mut brush: ResMut<Brush>) {
    // The brackets size the brush, because that is what a maker changes
    // constantly. The wheel is deliberately left alone: it zooms at every bench,
    // and a tool that quietly redefined it would be one you have to remember you
    // are standing at.
    if keys.just_pressed(KeyCode::BracketRight) {
        brush.radius = (brush.radius * RADIUS_STEP).min(MAX_RADIUS);
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        brush.radius = (brush.radius / RADIUS_STEP).max(MIN_RADIUS);
    }

    // Strength moves far less often, so it gets the further keys.
    if keys.just_pressed(KeyCode::Equal) {
        brush.strength = (brush.strength * STRENGTH_STEP).min(MAX_STRENGTH);
    }
    if keys.just_pressed(KeyCode::Minus) {
        brush.strength = (brush.strength / STRENGTH_STEP).max(MIN_STRENGTH);
    }

    // The tools are on the number row. At every other bench that row holds the
    // camera's snapped views, and here it does not: a maker at this bench
    // reaches for a tool a hundred times an hour and for a drafting angle
    // twice. The views are still there on Alt, which is the same modifier that
    // turns the eye — at this bench, Alt means you are talking to the camera.
    //
    // Guarded on the modifiers so Ctrl+Z takes a stroke back without also
    // changing which tool is in hand on the way past.
    if keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::AltLeft,
        KeyCode::AltRight,
        // Shift is the camera here, and Shift+1..6 are its drafting angles.
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
    ]) {
        return;
    }
    const KEYS: [KeyCode; 8] = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
    ];
    for (key, how) in KEYS.iter().zip(Brushing::ALL) {
        if keys.just_pressed(*key) && brush.how != how {
            // A half-placed ramp belongs to the tool that started it. Carrying
            // it across to another tool and back would arm a stale first point
            // from minutes ago, and the next click would cut to it.
            brush.pending = None;
            brush.how = how;
        }
    }
}

fn paint(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    ground: Res<Ground>,
    colours: Res<GroundColours>,
    standing: Res<Standing>,
    building: Query<(), With<Building>>,
    mut brush: ResMut<Brush>,
    mut said: ResMut<Said>,
) {
    // The left button lays the tool down and the right takes it back, so raising
    // and lowering are one gesture rather than a mode to switch. The right
    // button also orbits the eye at every other bench — here the ground has it,
    // and the camera stands off while the pointer is over the stage.
    // Shift is the camera at this bench, so while it is down the brush stands
    // off entirely: Shift+drag turns the eye and must not also cut the ground.
    let steering = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    // Two-point tools are CLICKED, not dragged: one press sets the start, the
    // next lays the thing between them. Dragging a ramp would mean holding the
    // button across the length of a hillside and never being able to see where
    // the far end was going to land.
    if brush.how.is_two_point() {
        if steering {
            return;
        }
        // The right button abandons a half-placed one, which is the only way out
        // of having clicked the wrong start.
        if buttons.just_pressed(MouseButton::Right) {
            if brush.pending.take().is_some() {
                said.that("Ramp abandoned");
            }
            return;
        }
        if !buttons.just_pressed(MouseButton::Left) {
            return;
        }
        let Some(hit) = brush.on else {
            return;
        };
        let Some(from) = brush.pending.take() else {
            brush.pending = Some(hit);
            said.that("Ramp started - click the far end");
            return;
        };

        let width = brush.radius;
        let patch = {
            let Ok(mut sculpt) = ground.sculpt().write() else {
                return;
            };
            let under = |p: Vec2| ground.made_height(p.x, p.y);
            sculpt.begin_stroke();
            let patch = sculpt.ramp(from, hit, width, &under);
            sculpt.end_stroke();
            patch
        };
        chunk::recut(&mut commands, &ground, &colours.0, &standing, &building, patch);
        said.that(format!(
            "Ramp laid, {:.0} m and {:+.0} m of climb",
            Vec2::new(from.x, from.z).distance(Vec2::new(hit.x, hit.z)),
            hit.y - from.y
        ));
        return;
    }
    let backwards = !steering && buttons.pressed(MouseButton::Right) && brush.on.is_some();
    let painting =
        (!steering && buttons.pressed(MouseButton::Left) && brush.on.is_some()) || backwards;

    // The undo group opens and closes around the whole drag, so a stroke lasting
    // two hundred frames comes back in one press.
    if painting && !brush.stroking {
        if let Ok(mut sculpt) = ground.sculpt().write() {
            sculpt.begin_stroke();
        }
        brush.stroking = true;
        brush.target = brush.on.map_or(0.0, |on| on.y);
    } else if !painting && brush.stroking {
        if let Ok(mut sculpt) = ground.sculpt().write() {
            sculpt.end_stroke();
        }
        brush.stroking = false;
    }

    let Some(on) = brush.on.filter(|_| painting) else {
        return;
    };

    let how = match (brush.how, backwards) {
        (Brushing::Raise, true) => Brushing::Lower,
        (Brushing::Lower, true) => Brushing::Raise,
        (how, _) => how,
    };
    let amount = how.rate(brush.strength, time.delta_secs());

    let patch = {
        let Ok(mut sculpt) = ground.sculpt().write() else {
            return;
        };
        // Reads the generator directly and never back through the sculpting —
        // that would deadlock against the very lock held here.
        let under = |p: Vec2| ground.made_height(p.x, p.y);
        sculpt.apply(&Stamp {
            centre: Vec2::new(on.x, on.z),
            radius: brush.radius,
            how,
            amount,
            target: brush.target,
            under: &under,
        })
    };

    chunk::recut(&mut commands, &ground, &colours.0, &standing, &building, patch);
}

fn history(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    ground: Res<Ground>,
    colours: Res<GroundColours>,
    standing: Res<Standing>,
    building: Query<(), With<Building>>,
    mut said: ResMut<Said>,
) {
    if !keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight, KeyCode::SuperLeft]) {
        return;
    }
    let shifted = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    // Both conventions for redo, since neither costs anything to answer.
    let forward = keys.just_pressed(KeyCode::KeyY) || (shifted && keys.just_pressed(KeyCode::KeyZ));
    let back = !shifted && keys.just_pressed(KeyCode::KeyZ);
    if !back && !forward {
        return;
    }

    let patch = {
        let Ok(mut sculpt) = ground.sculpt().write() else {
            return;
        };
        if back { sculpt.undo() } else { sculpt.redo() }
    };

    match patch {
        Some(patch) => {
            chunk::recut(&mut commands, &ground, &colours.0, &standing, &building, patch);
            said.that(if back { "Taken back" } else { "Put back" });
        }
        None => said.that(if back { "Nothing to take back" } else { "Nothing to put back" }),
    }
}

/// Writes the sculpted ground into the game's own folder.
fn keep(keys: Res<ButtonInput<KeyCode>>, ground: Res<Ground>, mut said: ResMut<Said>) {
    let held = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight, KeyCode::SuperLeft]);
    if !held || !keys.just_pressed(KeyCode::KeyS) {
        return;
    }
    let folder = ground.folder().to_path_buf();
    let Ok(mut sculpt) = ground.sculpt().write() else {
        return;
    };
    match sculpt.save(&folder) {
        Ok(road) => {
            info!("sculpted ground kept: {}", road.display());
            said.that(format!("Kept {} cells", sculpt.sculpted_cells()));
        }
        Err(why) => {
            error!("could not keep the sculpted ground: {why}");
            said.that(format!("Could not keep it: {why}"));
        }
    }
}

fn draw_the_brush(
    mut gizmos: Gizmos,
    ground: Res<Ground>,
    palette: Res<crate::look::Palette>,
    brush: Res<Brush>,
) {
    let Some(on) = brush.on else {
        return;
    };

    // A ring of short lines taken at ground height rather than a flat circle, so
    // on a slope it wraps the hill and shows what the stroke will actually cover.
    const AROUND: usize = 72;
    let colour = shelf::tool_colour(brush.how, &palette);
    let point = |i: usize, radius: f32| {
        let angle = i as f32 / AROUND as f32 * std::f32::consts::TAU;
        let x = on.x + angle.cos() * radius;
        let z = on.z + angle.sin() * radius;
        Vec3::new(x, ground.height(x, z) + brush.radius * 0.01, z)
    };
    let mut ring = |radius: f32, colour: Color| {
        let mut behind = point(0, radius);
        for i in 1..=AROUND {
            let next = point(i, radius);
            gizmos.line(behind, next, colour);
            behind = next;
        }
    };

    ring(brush.radius, colour);
    // The path tool has a flat bed to seven tenths of its radius. Showing that
    // inner edge is the difference between laying a road and guessing at one.
    if brush.how == Brushing::Path {
        ring(brush.radius * 0.7, colour.with_alpha(0.4));
    }
    // A mast at the middle, so the brush is findable when the ring falls behind
    // a rise.
    gizmos.line(on, on + Vec3::Y * brush.radius * 0.12, colour);

    // A ramp half-placed: show where it would go before it is cut rather than
    // after. The solid line is the grade the ramp would hold; the faint one
    // drops to the ground under it, so the gap between them is how much earth
    // the ramp moves and which way.
    if let Some(from) = brush.pending {
        const STEPS: usize = 48;
        gizmos.line(from, from + Vec3::Y * 14.0, colour);

        // The bed is as wide as the brush, so the edges are drawn too: a centre
        // line says where the ramp goes and nothing about what it eats, and the
        // width is the whole question when threading one between two hills.
        let run = Vec3::new(on.x - from.x, 0.0, on.z - from.z);
        let side = if run.length_squared() > 1.0 {
            Vec3::new(-run.z, 0.0, run.x).normalize() * brush.radius
        } else {
            Vec3::ZERO
        };

        let mut behind = (from, from - side, from + side);
        for i in 1..=STEPS {
            let at = from.lerp(on, i as f32 / STEPS as f32);
            let (left, right) = (at - side, at + side);
            gizmos.line(behind.0, at, colour);
            gizmos.line(behind.1, left, colour.with_alpha(0.5));
            gizmos.line(behind.2, right, colour.with_alpha(0.5));
            // A dropper to the ground under the grade. The gap between the two
            // is how much earth this moves, and which way — cut where the line
            // is under the ground, fill where it is above.
            gizmos.line(
                Vec3::new(at.x, ground.height(at.x, at.z), at.z),
                at,
                colour.with_alpha(0.22),
            );
            behind = (at, left, right);
        }
    }
}

/// Rings the ground levelled for each town, so a maker can see where the places
/// are before anything is built on them.
///
/// Cities in gold, towns in bone — the same distinction the plan makes, drawn
/// rather than described. Without this the levelling is invisible until you
/// happen to fly over a suspiciously flat field.
fn draw_the_sites(mut gizmos: Gizmos, ground: Res<Ground>, palette: Res<crate::look::Palette>) {
    const AROUND: usize = 40;
    let city = palette.shade("cloth-gold", 0.9);
    let town = palette.shade("bone", 0.8);

    for site in ground.sites() {
        let colour = if site.city { city } else { town };
        let point = |i: usize| {
            let angle = i as f32 / AROUND as f32 * std::f32::consts::TAU;
            let x = site.at.x + angle.cos() * site.radius;
            let z = site.at.y + angle.sin() * site.radius;
            Vec3::new(x, ground.height(x, z) + 1.5, z)
        };
        let mut behind = point(0);
        for i in 1..=AROUND {
            let next = point(i);
            gizmos.line(behind, next, colour);
            behind = next;
        }
        // A mast, so a place is findable from across the map.
        let middle = Vec3::new(site.at.x, site.height, site.at.y);
        let tall = if site.city { 90.0 } else { 45.0 };
        gizmos.line(middle, middle + Vec3::Y * tall, colour);
    }
}

fn fade_what_was_said(time: Res<Time>, mut said: ResMut<Said>) {
    if said.1 > 0.0 {
        said.1 = (said.1 - time.delta_secs()).max(0.0);
    }
}

/// What the shelf needs to know without borrowing the world.
pub fn tally(sculpt: &Sculpt) -> (usize, bool, bool, bool) {
    (
        sculpt.sculpted_cells(),
        sculpt.unsaved,
        sculpt.can_undo(),
        sculpt.can_redo(),
    )
}
