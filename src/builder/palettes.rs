//! SAVED PALETTES: the colours a maker used on one building, kept for the next.
//!
//! Brett: "I build a house, paint it and save just the colors that are used in that
//! specific house so that when I build another one I can quickly color it with the same
//! colors."
//!
//! A village is not painted from twenty-four ramps at five shades each. It is painted from
//! the eight or ten colours that make its houses look like they belong to one another, and
//! finding those again by eye across a hundred and twenty swatches is the work this saves.
//!
//! # Per project, because a palette is a game's own
//!
//! The colours name the game's own ramps, so a set saved in one game means nothing in
//! another - a `cloth-gold` that is not there paints nothing. They live in the project
//! beside its kinds, which is the same rule everything game-shaped on this bench follows:
//! the tool is universal, the vocabulary belongs to whoever is being served.
//!
//! # Harvested, not authored
//!
//! There is no palette editor here, and there should not be. A maker who has just finished
//! a house has already made every decision a palette holds; asking them to make them again
//! in a list is asking twice. So the set is READ OFF the finished building, and the only
//! thing left to say is what to call it.

use super::*;

/// The drawer the saved palettes hang in.
#[derive(Component)]
pub(crate) struct PaletteDrawer(pub(crate) Entity);

/// Whether that drawer is out of date - set when one is kept or dropped.
///
/// True to begin with, so the drawer fills itself on the first frame rather than needing
/// something to change before a maker can see what they saved last week.
#[derive(Resource)]
pub(crate) struct PalettesStale(pub(crate) bool);

impl Default for PalettesStale {
    fn default() -> Self {
        PalettesStale(true)
    }
}

/// The button that keeps the work's colours.
#[derive(Component)]
pub(crate) struct KeepColoursButton;

/// One saved palette's row, so a click can forget it.
#[derive(Component)]
pub(crate) struct DropPaletteButton(pub(crate) String);

/// One colour: a ramp of the game's, at a shade.
///
/// The ramp is a real name, never the bare swatch. Painting with the bare swatch strips a
/// part back to its own colours - it is the absence of a choice, and a set of colours has
/// nothing to remember about it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Colour {
    pub ramp: String,
    pub shade: f32,
}

/// A named set of them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedPalette {
    pub name: String,
    pub colours: Vec<Colour>,
}

/// The file, as it sits in the project.
#[derive(Serialize, Deserialize, Default)]
struct PalettesFile {
    #[serde(default)]
    format: u32,
    #[serde(default)]
    palettes: Vec<SavedPalette>,
}

/// Every palette this project has saved.
///
/// A missing file is no palettes rather than an error: a project that has never saved one
/// is the ordinary case, not a fault.
pub fn saved() -> Vec<SavedPalette> {
    let road = crate::project::saved_palettes();
    let Ok(text) = std::fs::read_to_string(&road) else {
        return Vec::new();
    };
    match serde_json::from_str::<PalettesFile>(&text) {
        Ok(file) => file.palettes,
        Err(why) => {
            // Said out loud rather than swallowed, the same as the kinds: an empty drawer
            // and a drawer whose file has a comma out of place look identical.
            warn!("{}: {why}", road.display());
            Vec::new()
        }
    }
}

/// Keeps a set under a name, replacing any set of that name.
///
/// REPLACING, because the name is usually the building's and a maker who paints a little
/// more and saves again means "this is what the longhouse is now" - not "here is a second
/// longhouse". Nothing is lost that they did not overwrite on purpose.
pub fn keep_a_palette(name: &str, colours: Vec<Colour>) -> Result<(), String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("a palette needs a name".to_string());
    }
    if colours.is_empty() {
        return Err("nothing on the bench is painted, so there are no colours to keep".to_string());
    }
    let mut known = saved();
    known.retain(|kept| !kept.name.eq_ignore_ascii_case(&name));
    known.push(SavedPalette { name, colours });
    known.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    write_them(&known)
}

/// Forgets one.
pub fn drop_a_palette(name: &str) -> Result<(), String> {
    let mut known = saved();
    known.retain(|kept| !kept.name.eq_ignore_ascii_case(name));
    write_them(&known)
}

fn write_them(all: &[SavedPalette]) -> Result<(), String> {
    let road = crate::project::saved_palettes();
    if let Some(under) = road.parent() {
        std::fs::create_dir_all(under).map_err(|why| format!("{}: {why}", under.display()))?;
    }
    let text = serde_json::to_string_pretty(&PalettesFile {
        format: 1,
        palettes: all.to_vec(),
    })
    .map_err(|why| format!("could not write the palettes: {why}"))?;
    std::fs::write(&road, format!("{text}\n")).map_err(|why| format!("{}: {why}", road.display()))
}

/// Every colour a work is painted with, once each, in the order they were first met.
///
/// THE WHOLE WORK, not the phase on the bench: a house is its footings, its frame, its
/// walls and its roof, and the colours of a roof are exactly the ones a maker forgets by
/// the time they build the next house. Every level too, since an upgrade is part of the
/// same building.
///
/// First-met order rather than sorted, because it is the order the building was painted in
/// and a maker recognises their own sequence. Sorting would put the shades of one ramp
/// together, which reads tidier and means less.
///
/// Only PAINTED colours. A part left alone wears whatever its own body says - a framed
/// wall's timbers and panels - and those were never chosen, so they are not what a maker
/// means by "the colours I used". They are also not a set that could be re-applied: there
/// is no brush stroke to repeat.
pub fn colours_in(work: &Workbench) -> Vec<Colour> {
    let mut found: Vec<Colour> = Vec::new();
    for level in levels_of(work) {
        for phase in &level.phases {
            for part in phase {
                let Some(ramp) = &part.ramp else {
                    continue;
                };
                if ramp.is_empty() {
                    continue;
                }
                if found.iter().any(|kept| same_colour(kept, ramp, part.shade)) {
                    continue;
                }
                found.push(Colour {
                    ramp: ramp.clone(),
                    shade: part.shade,
                });
            }
        }
    }
    found
}

/// Whether two colours are the same one.
///
/// Shades are compared LOOSELY, because they are floats that arrive both from swatches and
/// from the keys nudging between them: two strokes a maker considers the same colour can
/// differ in the last bit, and an exact comparison would keep both and show a palette with
/// a duplicate in it.
fn same_colour(kept: &Colour, ramp: &str, shade: f32) -> bool {
    kept.ramp == ramp && (kept.shade - shade).abs() < 0.001
}

/// Fills the drawer with the sets this project has saved.
///
/// Each set is its name and a row of its colours, and the colours are ordinary [`Swatch`]
/// entities - so clicking one arms it through the very system that arms every other swatch
/// on the panel. Nothing here has to know what arming means.
pub(crate) fn fill_the_palettes(
    mut commands: Commands,
    mut stale: ResMut<PalettesStale>,
    fonts: Res<Fonts>,
    palette: Res<Palette>,
    drawers: Query<&PaletteDrawer>,
) {
    if !stale.0 {
        return;
    }
    let Ok(drawer) = drawers.single() else {
        return;
    };
    stale.0 = false;
    commands.entity(drawer.0).despawn_related::<Children>();

    let kept = saved();
    if kept.is_empty() {
        // Says what the drawer is FOR while it is empty, rather than being a gap somebody
        // has to guess the meaning of.
        commands.spawn((
            Text::new("NO SAVED COLOURS YET"),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: crate::look::text_at(9.0),
                ..default()
            },
            TextColor(theme::text_dim(&palette).with_alpha(0.5)),
            ChildOf(drawer.0),
        ));
        return;
    }
    for set in kept {
        let row = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(2.0),
                    margin: UiRect::top(Val::Px(3.0)),
                    ..default()
                },
                ChildOf(drawer.0),
            ))
            .id();
        // The name, clickable to forget the set. A saved palette is cheap to make again
        // off any building that wears it, so forgetting one wants no ceremony.
        let label = commands
            .spawn((
                DropPaletteButton(set.name.clone()),
                Interaction::default(),
                Node {
                    width: Val::Px(74.0),
                    ..default()
                },
                ChildOf(row),
            ))
            .id();
        commands.spawn((
            Text::new(set.name.to_uppercase()),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: crate::look::text_at(9.0),
                ..default()
            },
            TextColor(theme::text_dim(&palette)),
            ChildOf(label),
        ));
        commands.spawn((
            crate::rail::Word("Click the name to forget this set"),
            ChildOf(label),
        ));
        for colour in &set.colours {
            commands.spawn((
                Swatch::of(&colour.ramp, colour.shade),
                Interaction::default(),
                Node {
                    width: Val::Px(20.0),
                    height: Val::Px(18.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(palette.shade(&colour.ramp, colour.shade)),
                BorderColor::all(Color::BLACK.with_alpha(0.35)),
                ChildOf(row),
            ));
        }
    }
}

/// A press on KEEP THESE COLOURS asks what to call them.
///
/// The field arrives holding the WORK's name, since "the longhouse colours" is what a maker
/// means nine times in ten, and enter takes it.
pub(crate) fn work_keep_colours(
    mut commands: Commands,
    fonts: Res<Fonts>,
    palette: Res<Palette>,
    mut naming: ResMut<Naming>,
    work_name: Res<WorkName>,
    kind: Res<CarryingKind>,
    asked: Query<&Interaction, (Changed<Interaction>, With<KeepColoursButton>)>,
) {
    // Not while another card is up: two things typing into one field is one of them losing.
    if naming.0.is_some() || !asked.iter().any(|touch| *touch == Interaction::Pressed) {
        return;
    }
    naming.0 = Some(work_name.0.clone().unwrap_or_default());
    naming.1 = NamingFor::APalette;
    raise_naming_card(&mut commands, &fonts, &palette, NamingFor::APalette, kind.0);
}

/// A click on a saved set's name forgets it.
pub(crate) fn work_drop_a_palette(
    mut stale: ResMut<PalettesStale>,
    asked: Query<(&Interaction, &DropPaletteButton), Changed<Interaction>>,
) {
    for (touch, which) in &asked {
        if *touch == Interaction::Pressed {
            match drop_a_palette(&which.0) {
                Ok(()) => {
                    info!("forgot the palette {}", which.0);
                    stale.0 = true;
                }
                Err(why) => warn!("could not forget {}: {why}", which.0),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn painted(ramp: Option<&str>, shade: f32) -> Placed {
        Placed {
            part: "wall".to_string(),
            at: [0.0, 0.0, 0.0],
            yaw: 0.0,
            tilt: 0.0,
            ramp: ramp.map(str::to_string),
            shade,
            stage: String::new(),
            flip: false,
            group: None,
            loose: false,
        }
    }

    /// The colours of a work are gathered once each, from every phase.
    #[test]
    fn it_gathers_what_was_painted() {
        let work = Workbench {
            levels: vec![Level {
                name: String::new(),
                phases: vec![
                    vec![painted(Some("wood"), 0.5), painted(Some("bone"), 0.75)],
                    // A later phase - the roof - carries colours of its own, and the
                    // duplicate from the phase before must not be kept twice.
                    vec![painted(Some("wood"), 0.5), painted(Some("cloth-red"), 0.25)],
                ],
            }],
            ..default()
        };
        let found = colours_in(&work);
        assert_eq!(
            found,
            vec![
                Colour {
                    ramp: "wood".into(),
                    shade: 0.5
                },
                Colour {
                    ramp: "bone".into(),
                    shade: 0.75
                },
                Colour {
                    ramp: "cloth-red".into(),
                    shade: 0.25
                },
            ],
            "wrong colours, wrong order, or a duplicate kept"
        );
    }

    /// Two shades of one ramp are two colours; a hair's difference is one.
    #[test]
    fn a_shade_makes_a_colour() {
        let work = Workbench {
            levels: vec![Level {
                name: String::new(),
                phases: vec![vec![
                    painted(Some("wood"), 0.25),
                    painted(Some("wood"), 0.75),
                    // The same colour arrived at by the keys rather than the swatch. An
                    // exact comparison would keep this and show a palette with what looks
                    // like a duplicate in it.
                    painted(Some("wood"), 0.750_02),
                ]],
            }],
            ..default()
        };
        assert_eq!(colours_in(&work).len(), 2);
    }

    /// An unpainted part contributes nothing.
    ///
    /// `ramp: None` is the absence of a choice - the part wears its own body's colours -
    /// and there is no brush stroke there to repeat on the next building.
    #[test]
    fn what_was_never_painted_is_not_a_colour() {
        let work = Workbench {
            levels: vec![Level {
                name: String::new(),
                phases: vec![vec![
                    painted(None, 0.5),
                    painted(Some(""), 0.5),
                    painted(Some("wood"), 0.5),
                ]],
            }],
            ..default()
        };
        assert_eq!(
            colours_in(&work),
            vec![Colour {
                ramp: "wood".into(),
                shade: 0.5
            }]
        );
    }

    /// An empty bench has no palette to keep, and says so rather than writing nothing.
    #[test]
    fn an_unpainted_work_cannot_be_kept() {
        assert!(keep_a_palette("longhouse", Vec::new()).is_err());
        assert!(
            keep_a_palette(
                "   ",
                vec![Colour {
                    ramp: "wood".into(),
                    shade: 0.5
                }]
            )
            .is_err(),
            "a nameless palette was kept"
        );
    }

    /// A work with no levels at all still reads: an old flat `.baz`, or a fresh bench.
    #[test]
    fn a_flat_work_still_reads() {
        let flat = Workbench {
            parts: vec![painted(Some("earth"), 0.5)],
            ..default()
        };
        assert_eq!(
            colours_in(&flat),
            vec![Colour {
                ramp: "earth".into(),
                shade: 0.5
            }]
        );
        assert!(colours_in(&Workbench::default()).is_empty());
    }
}
