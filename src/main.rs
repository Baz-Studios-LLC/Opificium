//! THE OPIFICIUM — the maker's own bench.
//!
//! A standalone maker's bench. Buildings are authored here by hand out of boxes
//! on a lattice and exported as JSON; models are commissioned from a picture and
//! kept as GLB. The two programs share no code — the game exports its palette as
//! data so Opificium paints with true colours, and Opificium exports blueprints
//! and models the game reads at its leisure. See FORMATS.md for every file that
//! passes between them.
//!
//! The bench holds no game's content. What it works on is a PROJECT: one
//! game's own folder of palette, templates and work, living in that game's
//! repository. See `project`.

use bevy::asset::AssetApp;
use bevy::prelude::*;

mod bake;
mod builder;
mod camera;
mod gizmo;
mod kiln;
mod look;
mod menu;
mod model;
mod opening;
mod project;
mod rail;
mod rig;
mod stage;
mod terrain;

/// Which bench the maker stands at.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Default)]
pub enum Bench {
    /// Buildings: boxes, ramps and widgets on the ground grid.
    #[default]
    Builder,
    /// A model, looked at closely - and in time, rigged.
    Rig,
    /// The ground itself: a game's world, shaped by hand.
    ///
    /// Like the kiln, it earns its own bench by not being a part. What the other
    /// benches make is a THING that stands on the ground - boxes on a lattice, a
    /// model on the grid - and this one makes the ground they stand on. It has no
    /// lattice, because a hill does not snap; no ramp, because it wears the game's
    /// own biome colours by height and slope; and no shelf of parts, because there
    /// is only ever one world open. It is measured in kilometres where every other
    /// bench is measured in metres, which is reason enough on its own.
    Terrain,
    /// The kiln: an image in, a model out, by way of somebody else's machine.
    ///
    /// The odd one out, and it earns the place by being one. Every other bench
    /// MAKES a thing out of parts the bench already holds - boxes on a lattice, a
    /// pose on a body - and this one commissions a mesh from an image and keeps the
    /// file. What comes back cannot be painted from a ramp, cut on the lattice or
    /// written as boxes, so it is not a part and never becomes one: it is an asset
    /// the game loads whole. Better a bench of its own than a drawer in the
    /// builder pretending otherwise.
    Kiln,
}

fn main() {
    // `--bake` never opens a window, and never touches the recent list. A
    // drawing could otherwise only enter a game through somebody pressing a
    // button, which makes a floor plan a checked-in artefact rather than a
    // generated one: no build, and nobody who has the source without the bench
    // open, could rebuild a house from its blueprint.
    if let Some(ask) = bake::asked_for() {
        std::process::exit(bake::run(&ask));
    }
    // And the kiln, for the same reason: a button cannot be driven by a script or by
    // anybody checking that the machine at the other end still answers.
    if let Some(code) = kiln::from_the_command_line() {
        std::process::exit(code);
    }

    // WHICH GAME, before anything else. A path on argv or in the environment is
    // an instruction and is obeyed; a bench with nothing to go on asks, rather
    // than opening whichever game it happened to open last. Choosing reopens the
    // bench pointed at the answer, so `ask` never returns to here.
    //
    // The project has to be settled before a single plugin starts: the palette is read
    // while the plugins are built, out of whichever game's folder is open.
    let Some(road) = project::named_outright() else {
        opening::ask();
        return;
    };
    let opened = open_a_project(&road);
    let title = match &opened {
        Some(project) => format!("Opificium — {}", project.name),
        None => "Opificium".to_string(),
    };

    let mut bench = App::new();
    // The PROJECT as a second asset root, registered before `DefaultPlugins` because
    // that is when the asset plugin is built and sources cannot be added afterwards.
    //
    // A model the kiln commissions is kept in the project, which is a game's own
    // repository - not in the bench's `assets/`, where Bevy looks by default. Copying
    // it in would work from a source tree and fail in the shipped `.app`, which is
    // read-only and signed. So the bench learns a second place to look, and
    // `project://models/x.glb` means whatever game is open.
    if let Some(open) = &opened {
        bench.register_asset_source(
            "project",
            bevy::asset::io::AssetSourceBuilder::platform_default(
                &open.root.to_string_lossy(),
                None,
            ),
        );
    }
    bench
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title,
                resolution: bevy::window::WindowResolution::new(1440, 900),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<Bench>()
        .add_plugins((look::LookPlugin, camera::CameraPlugin, stage::StagePlugin))
        .add_plugins(menu::MenuPlugin)
        .add_plugins((rail::RailPlugin, builder::BuilderPlugin, gizmo::GizmoPlugin))
        .add_plugins(rig::RigPlugin)
        .add_plugins(kiln::KilnPlugin)
        .add_plugins(terrain::TerrainPlugin)
        // Which bench the maker opens at. A maker working on the rig for an
        // hour should not walk across the builder to reach it every time.
        .insert_resource(match std::env::var("OPIFICIUM_BENCH").as_deref() {
            Ok("rig") => Bench::Rig,
            Ok("kiln") => Bench::Kiln,
            Ok("terrain") => Bench::Terrain,
            _ => Bench::Builder,
        })
        .run();
}

/// Opens the project the bench was TOLD to open, and says so on the way past.
///
/// No choosing left to do by the time this runs: either a path was named outright
/// or `opening::ask` has already asked and reopened the bench with the answer on
/// argv. This only carries it out.
fn open_a_project(road: &std::path::Path) -> Option<project::Project> {
    match project::open(road) {
        Ok(project) => {
            info!("project: {} ({})", project.name, project.root.display());
            Some(project)
        }
        Err(why) => {
            // A bench standing in no project still draws - it takes every default
            // and paints in its own colours - which is a better greeting than
            // refusing to open at all.
            warn!("could not open {}: {why}", road.display());
            None
        }
    }
}
