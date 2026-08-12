//! Opening a work from the disk, and sweeping the bench bare.

use super::*;

/// Template presses sweep the bench and set out the ready-made start; the
/// clear button just sweeps.
#[allow(clippy::too_many_arguments)]
/// Asks the desktop for a work to open.
///
/// The dialog is the system's own, and it stops the world while it is up - which
/// is what a modal dialog is. `NonSendMarker` is what keeps this system on the
/// main thread, where a Mac insists its panels be raised; without it Bevy is
/// free to run it on a worker and the panel is a crash rather than a window.
pub(crate) fn pick_a_work(
    _main_thread: bevy::ecs::system::NonSendMarker,
    bench: Res<Bench>,
    mut wanted: ResMut<WorkWanted>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<OpenWorkButton>)>,
) {
    if *bench != Bench::Builder {
        return;
    }
    if !buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        return;
    }
    let home = works_home();
    let _ = std::fs::create_dir_all(&home);
    let home = opening_home();
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Open a work")
        .add_filter("Opificium works", &[WORK_KIND])
        .set_directory(&home)
        .pick_file()
    {
        wanted.0 = Some(path);
    }
}

/// Opens a saved work onto a cleared bench, or simply clears it.
pub(crate) fn open_or_clear(
    mut commands: Commands,
    mut stages: ResMut<Stages>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<Palette>,
    bench: Res<Bench>,
    mut chosen: ResMut<WorkWanted>,
    clears: Query<&Interaction, (Changed<Interaction>, With<ClearButton>)>,
    standing: Query<(Entity, &Placed), Without<Ghost>>,
    mut work_name: ResMut<WorkName>,
    mut held: ResMut<StageHeld>,
) {
    if *bench != Bench::Builder {
        return;
    }
    let wanted = chosen.0.take();
    let sweeping = clears
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if wanted.is_none() && !sweeping {
        return;
    }
    // Kept before anything is taken away, and only when there is something to
    // keep. A sweep now empties every level and phase rather than the one on the
    // stage, and undo cannot reach the ones that were never standing - so the
    // whole work goes to the project's own `workbench.baz` on the way out. The
    // same insurance a project switch takes.
    if sweeping
        && let Some(kept) = keep_the_bench(
            &stages,
            standing.iter().map(|(_, record)| record),
            work_name.0.as_deref(),
        )
    {
        info!("swept, and kept what was there at {}", kept.display());
    }
    for (part, _) in &standing {
        commands.entity(part).despawn();
    }
    let Some(path) = wanted else {
        // EVERYTHING, which is what starting a new building means. Despawning the
        // standing parts only emptied the phase on the stage; every other phase and
        // every other level went on existing as records, so switching back brought
        // them out again and a save wrote them out. Brett: "sweeping the bench
        // should sweep everystage and everything. Like starting a new building,
        // right now it only clears its current stage."
        *stages = Stages::default();
        // And the phase held aside for PUT, or a sweep would leave one drawing in
        // the bench's hand to be pasted onto the empty work.
        held.0 = None;
        work_name.0 = None;
        return;
    };
    // A loaded work carries its own name into the bench.
    work_name.0 = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string());
    match std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<Workbench>(&text).ok())
    {
        Some(bench) => {
            // A work from before stages becomes stages on the way in, by the
            // rule the game used to infer them with - so an old building opens
            // with exactly the steps the village was already raising it in.
            // All three shapes a work has ever had: levels, phases without
            // levels, and one flat list from before either existed. A maker's
            // buildings are not something to lose to a format change.
            let levels = if !bench.levels.is_empty() {
                bench.levels
            } else {
                vec![Level {
                    name: String::new(),
                    phases: if bench.stages.is_empty() {
                        stages_from_flat(&bench.parts)
                    } else {
                        bench.stages
                    },
                }]
            };
            let drawings = levels[0].phases.clone();
            // The LAST step: a maker opening a building wants the building,
            // not its footings.
            let showing = drawings.len().saturating_sub(1);
            let count = drawings[showing].len();
            for record in &drawings[showing] {
                if let Some(kind) = kind_from_name(&record.part) {
                    spawn_part(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &palette,
                        &kind,
                        record,
                        false,
                    );
                }
            }
            let steps = drawings.len();
            *stages = Stages {
                levels,
                level: 0,
                showing,
            };
            info!(
                "set out {}: {count} parts, step {} of {steps}",
                path.display(),
                showing + 1
            );
        }
        None => warn!("nothing readable at {}", path.display()),
    }
}

// ---------------------------------------------------------------- the hand
