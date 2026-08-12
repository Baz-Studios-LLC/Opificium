//! Carrying a work into the game: the card that asks what it is, and the writing.

use super::*;

pub(crate) fn bake_into_the_game(
    mut commands: Commands,
    bench: Res<Bench>,
    fonts: Res<Fonts>,
    palette: Res<Palette>,
    work_name: Res<WorkName>,
    mut naming: ResMut<Naming>,
    mut kind: ResMut<CarryingKind>,
    bakes: Query<&Interaction, (Changed<Interaction>, With<BakeButton>)>,
) {
    if *bench != Bench::Builder
        || naming.0.is_some()
        || !bakes
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
    {
        return;
    }
    let called = work_name.0.clone().unwrap_or_default();
    // The kind the name already suggests, if it suggests one - a maker who has
    // been naming their works `longhouse1` for a week should find the card
    // already pointing at the longhouse. LONGEST word first, or `longhouse1`
    // opens on "house".
    let known = crate::project::kinds();
    let mut guessed: Vec<usize> = (0..known.len()).collect();
    guessed.sort_by_key(|index| std::cmp::Reverse(known[*index].word.len()));
    kind.0 = guessed
        .into_iter()
        .find(|index| called.starts_with(&known[*index].word))
        .unwrap_or(0);
    naming.0 = Some(called);
    naming.1 = NamingFor::Carrying;
    raise_naming_card(&mut commands, &fonts, &palette, NamingFor::Carrying, kind.0);
}

/// Presses on the card's kinds, while it is up.
#[allow(clippy::too_many_arguments)]
pub(crate) fn choose_the_kind(
    mut commands: Commands,
    fonts: Res<Fonts>,
    palette: Res<Palette>,
    mut naming: ResMut<Naming>,
    mut kind: ResMut<CarryingKind>,
    mut held: ResMut<NameHeld>,
    cards: Query<Entity, With<NamingCard>>,
    buttons: Query<(&Interaction, &KindButton), Changed<Interaction>>,
    adding: Query<&Interaction, (Changed<Interaction>, With<NewKindButton>)>,
) {
    if naming.0.is_none() || naming.1 != NamingFor::Carrying {
        return;
    }
    // Asked for a kind the project does not know: the same card, holding the
    // work's name, asks for the word instead. `take_the_name` brings it back.
    if adding.iter().any(|touch| *touch == Interaction::Pressed) {
        held.0 = naming.0.clone().unwrap_or_default();
        naming.0 = Some(String::new());
        naming.1 = NamingFor::AKind;
        for card in &cards {
            commands.entity(card).despawn();
        }
        raise_naming_card(&mut commands, &fonts, &palette, NamingFor::AKind, kind.0);
        return;
    }
    let Some(chosen) = buttons
        .iter()
        .find(|(touch, _)| **touch == Interaction::Pressed)
        .map(|(_, button)| button.0)
    else {
        return;
    };
    if chosen == kind.0 {
        return;
    }
    kind.0 = chosen;
    // Redrawn rather than repainted: the card is a handful of nodes and this
    // happens once per press, so the marking has ONE place it is decided rather
    // than two that could come to disagree.
    for card in &cards {
        commands.entity(card).despawn();
    }
    raise_naming_card(&mut commands, &fonts, &palette, NamingFor::Carrying, chosen);
}

/// Writes the work into the game's own folder, as a building of a named kind.
pub(crate) fn carry_into_the_game(
    work: &Workbench,
    palette: &Palette,
    name: &str,
    kind: &str,
) -> Result<(usize, usize), String> {
    let (json, boxes, marks) = bake_a_work(work, palette, name);
    let json = if kind.is_empty() {
        json
    } else {
        json.replacen(
            "\"format\": 1,",
            &format!("\"format\": 1,\n  \"kind\": \"{kind}\","),
            1,
        )
    };
    let home = carried_home("buildings");
    std::fs::create_dir_all(&home).map_err(|why| format!("{}: {why}", home.display()))?;
    let out = home.join(format!("{name}.json"));
    std::fs::write(&out, json).map_err(|why| format!("{}: {why}", out.display()))?;
    info!("carried {name} in as a {kind}, at {}", out.display());
    Ok((boxes, marks))
}
