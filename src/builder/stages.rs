//! The cutaway, and walking the levels and phases of a work.

use super::*;

/// How much of the work is standing: all of it, the roof lifted off,
/// or the walls down as well - the dollhouse view, for furnishing.
#[derive(Resource, Default, Clone, Copy, PartialEq)]
pub enum Cutaway {
    #[default]
    Whole,
    RoofOff,
    WallsDown,
}

/// Kept as a resource of its own so the rail's button can read it.
#[derive(Resource, Default)]
pub struct RoofsLifted(pub Cutaway);

/// H lifts the roof off and sets it back - everything raised at the
/// roof stage goes with it, panels and ridge caps alike.
/// Delete takes away whatever is chosen, in whichever tool is in hand.
///
/// Being rid of a part used to mean going back to NORMAL, picking the part up,
/// and pressing escape to throw away what you were holding - three steps, one of
/// which is a mode change, to undo one placement. Brett asked for the obvious
/// thing instead: choose it and press delete.
///
/// BACKSPACE as well as Delete, and on this bench that is the important half:
/// the key labelled "delete" on a Mac keyboard IS backspace, and the forward
/// Delete these keyboards do not have is the one Bevy calls `Delete`.
///
/// It stands aside while anything is being TYPED. Backspace belongs to whoever
/// is taking letters - the name card, the dimensions box - and a part quietly
/// vanishing while a maker corrects a typo would be a bad way to learn that.
///
/// Nothing to do about undo: the bench remembers whole states, so a part
/// removed is one step back like anything else.
pub(crate) fn bury_the_chosen(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    bench: Res<Bench>,
    naming: Res<Naming>,
    dims: Res<DimsEntry>,
    hand: Res<Hand>,
    palette: Res<Palette>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut selected: ResMut<crate::gizmo::Selected>,
    parts: Query<(Entity, &Transform, &Placed, &Visibility), Without<Ghost>>,
) {
    if *bench != Bench::Builder || naming.0.is_some() || dims.0.is_some() {
        return;
    }
    // A full hand answers these keys already, by throwing away what it holds.
    if hand.kind.is_some() {
        return;
    }
    if !keys.just_pressed(KeyCode::Delete) && !keys.just_pressed(KeyCode::Backspace) {
        return;
    }
    // Everything chosen, not merely the first of them.
    let doomed: Vec<Entity> = selected.iter().collect();
    if doomed.is_empty() {
        return;
    }
    for chosen in doomed {
        if let Ok((_, chosen_at, _, _)) = parts.get(chosen) {
            // The marks it carries go with it. A door's routing mark left standing
            // in a wall with no door is worse than either: the village reads it and
            // sends people to walk through masonry.
            let carried = carried_marks(
                chosen,
                chosen_at.translation,
                parts
                    .iter()
                    .map(|(e, at, record, _)| (e, at.translation, record)),
            );
            for mark in carried {
                commands.entity(mark).despawn();
            }
            // A door taken out leaves the wall whole again. The older path through
            // X has always done this; a second way to remove a part that did not
            // would leave holes nobody could account for.
            heal_wall(
                &mut commands,
                &mut meshes,
                &mut materials,
                &palette,
                &parts,
                chosen,
            );
            commands.entity(chosen).despawn();
        }
    }
    selected.clear();
}

/// A request to show another step, or to add or drop one.
///
/// Held as a resource rather than done where it is asked for, because setting a
/// step out means despawning and respawning the whole work, and exactly one
/// place should be allowed to do that.
#[derive(Resource, Default)]
pub struct StageWish(pub Option<StageDeed>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StageDeed {
    /// Show this step.
    Show(usize),
    /// One more step, right after the one showing, holding a copy of it.
    ///
    /// A copy rather than an empty floor, because a step is what the building
    /// looks like at that moment and the next moment is nearly always this one
    /// with something added. There were two buttons for this, `+ COPY` and
    /// `+ BARE`, and Brett found what anybody would: "the copy and bare buttons
    /// are confusing". An empty step is a step with everything taken off it,
    /// which is a thing a maker can do with their hands.
    Add,
    /// Drop the step being shown.
    Drop,
    /// Remember this step, to put on another.
    Take,
    /// Put the remembered step here, in place of what stands.
    Put,
}

/// A step held aside, waiting to be put on another.
///
/// Brett: "I need a way to copy a stage and paste it on anotehr stage." `+ COPY`
/// only ever made a NEW step from the one showing, which is the wrong shape for
/// "make step three look like step two again" - there is no new step wanted, and
/// the one that needs changing already exists.
#[derive(Resource, Default)]
pub struct StageHeld(Option<Vec<Placed>>);

/// Sets out another step of the work.
///
/// The bench holds one step as entities and the rest as records, so this is the
/// only place a step changes hands: gather what is standing back into the step
/// it belongs to, then set the wanted one out. Doing it anywhere else would lose
/// whichever step the maker was on.
#[allow(clippy::too_many_arguments)]
pub(crate) fn turn_to_stage(
    mut commands: Commands,
    mut wish: ResMut<StageWish>,
    mut held: ResMut<StageHeld>,
    mut stages: ResMut<Stages>,
    mut history: ResMut<History>,
    palette: Res<Palette>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut selected: ResMut<crate::gizmo::Selected>,
    standing: Query<(Entity, &Placed), Without<Ghost>>,
) {
    let Some(deed) = wish.0.take() else {
        return;
    };
    // Whatever is on the bench belongs to the step it was drawn on.
    let showing = stages.showing.min(stages.count().saturating_sub(1));
    let gathered: Vec<Placed> = standing.iter().map(|(_, record)| record.clone()).collect();
    if let Some(slot) = stages.phases_mut().get_mut(showing) {
        *slot = gathered;
    }

    let wanted = match deed {
        StageDeed::Show(step) => step.min(stages.count() - 1),
        StageDeed::Take => {
            // Nothing moves: the step is remembered exactly as it was gathered a
            // moment ago, and the bench goes on showing it.
            held.0 = Some(stages.phases()[showing].clone());
            return;
        }
        StageDeed::Put => {
            let Some(kept) = held.0.clone() else {
                return;
            };
            // In PLACE of what stands, not beside it. Two steps merged would be
            // a step nobody drew, and the way to add to a step is to draw on it.
            stages.phases_mut()[showing] = kept;
            showing
        }
        StageDeed::Add => {
            // Beside the one showing, not at the end. A maker adding a step
            // while looking at step two means a step between two and three -
            // "we need to be able to add one stage at a time".
            let copy = stages.phases()[showing].clone();
            stages.phases_mut().insert(showing + 1, copy);
            showing + 1
        }
        StageDeed::Drop => {
            // Never the last one standing: a building with no steps is not a
            // building anybody can raise.
            if stages.count() <= 1 {
                return;
            }
            stages.phases_mut().remove(showing);
            showing.min(stages.count() - 1)
        }
    };

    for (part, _) in &standing {
        commands.entity(part).despawn();
    }
    selected.clear();
    for record in &stages.phases()[wanted].clone() {
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
    let travelled = wanted != showing;
    stages.showing = wanted;
    // Undo does not reach ACROSS a step. Every part on the bench has just been
    // swapped for another step's, and a history that let someone undo into that
    // would put one step's parts down on another - not a thing a maker could
    // have done by hand, and so not a thing undo should be able to do.
    //
    // But PUTTING a step over the one showing never leaves it, and replaces
    // what was there - which is exactly the kind of large, destructive, ordinary
    // edit undo exists for. Forgetting there would make a mis-aimed PUT the one
    // thing in this bench that cannot be taken back.
    if travelled {
        history.forget();
    }
}

/// How many steps a work rises in, and which step a part belongs to.
///
/// This is the GAME's rule, written out again here because the bench and the
/// game share no code — see FORMATS.md. It has to match exactly or the playback
/// is a lie, and a lie in a preview is worse than no preview: the maker would
/// trust it. The rule, including the awkward part:
///
///   footing 0, frame 1, walls 2, everything else 3 — unless the work has no
///   frame at all, in which case walls and the rest each move DOWN a step,
///   because a build with nothing to raise at step 1 never reaches step 3.
pub(crate) fn step_of(stage: &str, framed: bool) -> u8 {
    match (stage, framed) {
        ("footing", _) => 0,
        ("frame", _) => 1,
        ("walls", true) => 2,
        ("walls", false) => 1,
        (_, true) => 3,
        (_, false) => 2,
    }
}

pub(crate) fn lift_roofs(
    keys: Res<ButtonInput<KeyCode>>,
    bench: Res<Bench>,
    naming: Res<Naming>,
    dims: Res<DimsEntry>,
    mut lifted: ResMut<RoofsLifted>,
    mut parts: Query<(&Placed, &mut Visibility), Without<Ghost>>,
) {
    if *bench == Bench::Builder
        && naming.0.is_none()
        && dims.0.is_none()
        && keys.just_pressed(KeyCode::KeyH)
    {
        lifted.0 = match lifted.0 {
            Cutaway::Whole => Cutaway::RoofOff,
            Cutaway::RoofOff => Cutaway::WallsDown,
            Cutaway::WallsDown => Cutaway::Whole,
        };
    }
    // The cutaway decides what a BUILDING shows, and only while a maker is
    // standing at the building bench. It used to write every part's visibility
    // whichever bench was out, which put the whole work back on the stage the
    // instant the rig had put it away - Brett: "When you go to the rig it doesnt
    // clear the bench first." The stage says who is on show; this says which of
    // them the roof is hiding.
    if *bench != Bench::Builder {
        return;
    }
    for (record, mut visibility) in &mut parts {
        // What a part IS, and nothing else. This used to ask the part's KIND as
        // well - a gable roof was roof-ish whatever it had been told, a wall was
        // wall-ish - which was a sensible net while nobody could say otherwise
        // and became an override the moment they could. Brett: "any peice could
        // be a roof piece", and the other way about for walls. So the tag is the
        // only word: a plank tagged as roof comes off with the roof, and a roof
        // panel tagged as walls stays until the walls come down.
        //
        // The frame comes down with the walls. What WallsDown means is "show me
        // the ground it stands on", and a hall's posts left standing over bare
        // footings is not that.
        let roofish = record.stage == "roof";
        let wallish = record.stage == "walls" || record.stage == "frame";
        let cut = match lifted.0 {
            Cutaway::Whole => true,
            Cutaway::RoofOff => !roofish,
            Cutaway::WallsDown => !roofish && !wallish,
        };
        let showing = cut;
        let wanted = if showing {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
}
