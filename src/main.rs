//! THE OPIFICIUM — the maker's own bench.
//!
//! A standalone maker's bench: buildings and animations are authored here
//! by hand and exported as JSON for a game to take in. The two programs
//! share no code — the game exports its palette as data so Opificium
//! paints with true colours, and Opificium exports blueprints and clips
//! the game translates at its leisure. See FORMATS.md for every file that
//! passes between them.
//!
//! The bench holds no game's content. What it works on is a PROJECT: one
//! game's own folder of palette, bodies, templates and work, living in
//! that game's repository. See `project`.

use bevy::prelude::*;

mod project;
mod builder;
mod camera;
mod gizmo;
mod look;
mod rail;
mod rig;
mod stage;

/// Which bench the maker stands at.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Default)]
pub enum Bench {
    /// Buildings: boxes, ramps and widgets on the ground grid.
    #[default]
    Builder,
    /// Animation: the canonical body on its pedestal, and the timeline.
    Rig,
}

fn main() {
    // The project comes first, before a single plugin starts: the palette
    // and the bodies are both read during startup, and they are read out
    // of whichever game's folder is open.
    let opened = open_a_project();
    let title = match &opened {
        Some(project) => format!("Opificium — {}", project.name),
        None => "Opificium".to_string(),
    };

    App::new()
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
        .add_plugins((rail::RailPlugin, builder::BuilderPlugin, gizmo::GizmoPlugin))
        .add_plugins(rig::RigPlugin)
        // Which bench the maker opens at. A maker working on the rig for an
        // hour should not walk across the builder to reach it every time.
        .insert_resource(match std::env::var("OPIFICIUM_BENCH").as_deref() {
            Ok("rig") => Bench::Rig,
            _ => Bench::Builder,
        })
        .run();
}

/// Finds the project to work in, and says so on the way past.
///
/// In order: one named on the command line, then the last one worked in,
/// then - only on a bench that has never been opened before - a folder
/// picker. A maker who dismisses the picker still gets a working bench
/// standing in whatever folder it was started from, because refusing to
/// open at all would be a poor greeting.
fn open_a_project() -> Option<project::Project> {
    if let Some(road) = project::opening() {
        match project::open(&road) {
            Ok(project) => {
                info!("project: {} ({})", project.name, project.root.display());
                return Some(project);
            }
            Err(why) => warn!("could not open {}: {why}", road.display()),
        }
    }

    let picked = rfd::FileDialog::new()
        .set_title("Open a project folder")
        .pick_folder()?;
    match project::open(&picked) {
        Ok(project) => {
            info!("project: {} ({})", project.name, project.root.display());
            Some(project)
        }
        Err(why) => {
            warn!("could not open {}: {why}", picked.display());
            None
        }
    }
}
