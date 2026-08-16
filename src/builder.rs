//! The Builder's bench: the game's own parts, snapped together by hand.
//!
//! Nothing here is freeform. The shelf holds walls at the game's true
//! thickness, floors and roof panels at its true proportions, props the
//! god has authored, and the widget blocks that tell the game what a
//! place *does*. You build the toys; the legos come pre-measured.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Bench;
use crate::look::{Fonts, Palette, theme};
use crate::stage::BuilderFurniture;
// Bevy's prelude carries a `Clipboard` too, and a glob cannot choose between
// them. This says which one the bench means.
use history::Clipboard;

// One concern per file. `builder.rs` was twelve thousand lines and held all of
// them; what is left here is the bench itself - the record a part is saved as,
// the hand's own shape, and the plugin that hangs the rest together.
//
// Every child globs this module back in and is globbed back out of it, so a
// name means the same thing on either side of the split and no caller outside
// the bench had to change.
mod body;
mod brush;
mod carried;
mod carry;
mod chosen;
mod hand;
mod history;
mod measure;
mod naming;
mod open;
mod oven;
mod paint;
mod palettes;
mod part;
mod partmenu;
mod pieces;
mod shape;
mod shelf;
mod stages;
mod turn;
mod wall;
mod work;
pub(crate) use body::*;
pub(crate) use brush::*;
pub(crate) use carried::*;
pub(crate) use carry::*;
pub(crate) use chosen::*;
pub(crate) use hand::*;
pub(crate) use history::*;
pub(crate) use measure::*;
pub(crate) use naming::*;
pub(crate) use open::*;
pub(crate) use oven::*;
pub(crate) use paint::*;
pub(crate) use palettes::*;
pub(crate) use part::*;
pub(crate) use partmenu::*;
pub(crate) use pieces::*;
pub(crate) use shape::*;
pub(crate) use shelf::*;
pub(crate) use stages::*;
pub(crate) use turn::*;
pub(crate) use wall::*;
pub(crate) use work::*;

/// A placed part's record: everything the export needs to rebuild it.
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Placed {
    pub part: String,
    pub at: [f32; 3],
    pub yaw: f32,
    pub tilt: f32,
    pub ramp: Option<String>,
    pub shade: f32,
    /// The stage the village raises this in: footing, frame, walls, roof,
    /// furnishing - or "widget", which never becomes a box at all.
    #[serde(default)]
    pub stage: String,
    /// Mirrored: the body reflected across its own length, and any tilt
    /// leaning the other way - the far half of a gable, the other hand
    /// of an L.
    #[serde(default)]
    pub flip: bool,
    /// What the village BUILDS this out of: wood, stone, clay, or whatever else the
    /// game knows.
    ///
    /// Not its colour, which is `ramp` and `shade` above and is only what a maker painted.
    /// This is what the thing is made of, and it is the game's business what that costs -
    /// a stone wall is quarried and hauled where a timber one is felled.
    ///
    /// Empty means UNSAID, and unsaid is not "wood": a game that hears nothing may charge
    /// whatever it charges for a building whose parts were never specified, and that is a
    /// decision for the game rather than a default the bench smuggles in.
    #[serde(default)]
    pub material: String,

    /// Which group this part belongs to, if any.
    ///
    /// A plain number, shared by everything grouped together, and that is the
    /// whole mechanism: parts wearing one move, paint, delete and travel as one.
    /// It replaces a GUESS with a fact. A part used to carry the marks that
    /// happened to sit within a metre of it, nearest owner winning, which works
    /// until two beds are pushed together or a mark stands between a door and a
    /// wall. A group says which pieces belong to which and stops the bench
    /// inferring it.
    ///
    /// Flat, not nested. A group of groups doubles the question a click has to
    /// answer - the part, or the whole? - and nothing yet needs the second
    /// level.
    #[serde(default)]
    pub group: Option<u32>,

    /// A widget that has been cut loose from whatever it was standing in.
    ///
    /// A door arrives with its routing mark, a bed could arrive with its sleep
    /// mark, and Brett's rule is that they travel together: "they should stay
    /// grouped like the stretch gable roofs unless you right click and ungroup".
    /// So a mark belongs to the part it sits inside, and moving or burying that
    /// part takes the mark with it - until somebody says otherwise here.
    ///
    /// Defaulting to false means every mark in every saved work is bundled,
    /// which is what a maker who placed one inside a door meant by it.
    #[serde(default)]
    pub loose: bool,
}

/// Everything that moves when this part moves: itself, and whatever shares its
/// group.
///
/// A click on any one of a group takes them all, because that is what being
/// grouped means. An ungrouped part is its own company.
pub fn kin_of(part: Entity, records: &Query<(Entity, &Placed), Without<Ghost>>) -> Vec<Entity> {
    let Ok((_, record)) = records.get(part) else {
        return vec![part];
    };
    let Some(group) = record.group else {
        return vec![part];
    };
    records
        .iter()
        .filter(|(_, other)| other.group == Some(group))
        .map(|(entity, _)| entity)
        .collect()
}

/// A group number nothing else is using.
///
/// Taken over whatever records the caller has to hand, since the two callers
/// hold different shapes of query and neither should have to reshape itself to
/// ask a question about numbers.
pub fn a_fresh_group<'a>(records: impl Iterator<Item = &'a Placed>) -> u32 {
    records
        .filter_map(|record| record.group)
        .max()
        .map_or(1, |highest| highest + 1)
}

/// A number to draw a part's own dice from, taken from where it is going down and
/// how much is already standing.
///
/// A row of books is dealt a fresh hand every time one is set down - Brett: "could
/// we have the books be random colors when placed so every group doesnt look the
/// same?" - and the hand is remembered in the part's NAME rather than rolled again
/// at drawing time, so a shelf comes back the shelf it was every time the work is
/// reopened.
///
/// Where and how much, rather than a clock: the bench has no wall clock to ask, and
/// two rows set down in different places want different books whatever the hour.
pub fn a_fresh_seed(at: Vec3, standing: usize) -> u32 {
    let mix = |measure: f32| (measure * 64.0).round() as i64 as u32;
    mix(at.x)
        .wrapping_mul(2_246_822_519)
        .wrapping_add(mix(at.z).wrapping_mul(3_266_489_917))
        .wrapping_add(standing as u32)
        .wrapping_mul(668_265_263)
}

/// How far a mark may sit from a part and still be carried by it.
///
/// A metre: a double door's two marks stand half a metre either side of its
/// middle, and nothing else a maker places sits that close to a part it does
/// not belong to.
const CARRIES_WITHIN: f32 = 1.0;

/// The marks a part carries: its own, bundled with it.
///
/// Nearest owner wins, so a mark between two parts belongs to the one it is
/// actually in rather than to both. Loose marks belong to nobody.
pub(crate) fn carried_marks<'a>(
    owner: Entity,
    owner_at: Vec3,
    everything: impl Iterator<Item = (Entity, Vec3, &'a Placed)> + Clone,
) -> Vec<Entity> {
    let mut carried = Vec::new();
    for (mark, mark_at, record) in everything.clone() {
        if record.loose || record.stage != "widget" {
            continue;
        }
        let reach = mark_at.distance(owner_at);
        if reach > CARRIES_WITHIN {
            continue;
        }
        // Whoever is nearest. A mark inside a door and beside a wall belongs to
        // the door.
        let nearer = everything.clone().any(|(other, other_at, other_record)| {
            other != mark
                && other_record.stage != "widget"
                && other_at.distance(mark_at) < reach - 1e-4
        });
        if !nearer && owner != mark {
            carried.push(mark);
        }
    }
    carried
}

/// A part's turn: yaw, then tilt - which leans the other way when the
/// part is mirrored, so a pitched panel's twin completes the gable.
pub fn pose(yaw: f32, tilt: f32, flip: bool) -> Quat {
    Quat::from_rotation_y(yaw) * Quat::from_rotation_x(if flip { -tilt } else { tilt })
}

/// The ghost that follows the cursor while the hand is full.
#[derive(Component)]
pub struct Ghost;

/// The maker's hand: what it holds and how it holds it. Filled from the
/// shelf, or by picking a placed part back up with an empty hand.
#[derive(Resource, Default)]
pub struct Hand {
    pub kind: Option<PartKind>,
    /// A stretch wall's anchored start, once the first click lands.
    pub anchor: Option<Vec3>,
    /// Whether the held part is mirrored.
    pub flip: bool,
    pub stage: String,
    pub yaw: f32,
    pub tilt: f32,
    pub lift: f32,
    pub ramp: Option<String>,
    pub shade: f32,
}

impl Hand {
    fn filled(kind: PartKind, stage: String) -> Self {
        // A roof comes to hand already pitched: a flat panel is the
        // exception, not the starting point.
        let tilt = if matches!(kind, PartKind::Roof(..)) {
            ROOF_PITCH_DEGREES.to_radians()
        } else {
            0.0
        };
        Hand {
            kind: Some(kind),
            anchor: None,
            flip: false,
            tilt,
            stage,
            shade: 0.7,
            ..default()
        }
    }

    fn record(&self, at: Vec3) -> Option<Placed> {
        let kind = self.kind.as_ref()?;
        Some(Placed {
            part: part_name(kind),
            at: at.into(),
            yaw: self.yaw,
            tilt: self.tilt,
            ramp: self.ramp.clone(),
            shade: self.shade,
            stage: self.stage.clone(),
            flip: self.flip,
            loose: false,
            material: String::new(),
            group: None,
        })
    }
}

/// A shelf button holding one catalog entry.
#[derive(Component)]
pub(crate) struct ShelfButton(&'static CatalogEntry);

/// A shelf button holding one widget.
#[derive(Component)]
pub(crate) struct WidgetButton(&'static str);

/// The button that sweeps the bench bare.
#[derive(Component)]
pub(crate) struct ClearButton;

/// A drawer header: pressing it opens and closes the drawer body.
#[derive(Component)]
pub(crate) struct DrawerHeader {
    body: Entity,
    label: Entity,
    /// Owned, because a drawer's head is not always a word chosen when the bench
    /// was written: THE PROJECT's head is the name of the game that is open.
    name: String,
    open: bool,
}

/// The shelf panel itself, shown only at the Builder bench.
#[derive(Component)]
pub(crate) struct Shelf;

/// The export/save button.
#[derive(Component)]
pub(crate) struct SaveButton;

/// The IN THE GAME drawer's body, so the list can be hung again after a bake.
#[derive(Resource)]
pub(crate) struct CarriedDrawer(Entity);

/// Set whenever the list is out of date: at startup, after a bake, after a
/// removal.
#[derive(Resource)]
pub(crate) struct CarriedStale(bool);

impl Default for CarriedStale {
    fn default() -> Self {
        CarriedStale(true)
    }
}

/// One drawing already carried into the game.
#[derive(Component, Clone)]
pub(crate) struct CarriedRow {
    path: std::path::PathBuf,
    name: String,
}

/// The card that asks before a drawing is taken back out of the game.
#[derive(Component)]
pub(crate) struct RemovalCard;

/// Its two answers.
#[derive(Component)]
pub(crate) struct RemovalYes(CarriedRow);

#[derive(Component)]
pub(crate) struct RemovalNo;

/// A button that asks the desktop for a work to open.
///
/// Brett wanted `.baz` files associated with the bench so a double click opened
/// them, and then found the shorter way himself: "what if we just had a open
/// file button?" - which needs nothing of the operating system, works the same
/// on both, and can open a work kept anywhere rather than only the ones in the
/// bench's own folder.
#[derive(Component)]
pub(crate) struct OpenWorkButton;

/// A work the maker has chosen from the desktop's own file window, waiting to
/// be set out on the bench.
#[derive(Resource, Default)]
pub(crate) struct WorkWanted(pub Option<std::path::PathBuf>);

/// The save button's label, so it can say what just happened.
#[derive(Component)]
pub(crate) struct SaveLabel;

/// The name this work goes by, once it has been given one. Saving again
/// updates the same file instead of scattering copies.
#[derive(Resource, Default)]
pub struct WorkName(pub Option<String>);

/// A label speaking a passing word; it returns to its old text at `until`.
#[derive(Component)]
pub(crate) struct PassingWord {
    back: &'static str,
    until: f32,
}

/// The part waiting for a material this project has never heard of.
///
/// The naming card asks for the word and the part it belongs to is not on the card, so it
/// waits here - the same shape as `NameHeld`, which holds a work's name while a kind is
/// being typed on the same field.
#[derive(Resource, Default)]
pub struct MaterialFor(pub Option<Entity>);

/// The name being typed for an export, while the naming card is up.
/// While this is Some, every other key on the bench holds its tongue.
#[derive(Resource, Default)]
pub struct Naming(pub Option<String>, pub NamingFor);

/// What the name being typed is FOR.
///
/// The same card asks both questions, because they are the same question asked
/// of two different places: what shall this be called where it is kept, and what
/// shall it be called where it is raised.
#[derive(Default, Clone, Copy, PartialEq)]
pub enum NamingFor {
    /// Keeping the work on the bench, as a `.baz`.
    #[default]
    Keeping,
    /// Carrying it into the game, as a building of a named kind.
    Carrying,
    /// Keeping what is chosen as a piece, to bring into other works.
    AsAPiece,
    /// Naming the colours the work on the bench is painted with, to paint another
    /// building the same way later.
    APalette,
    /// Naming a material this project does not know yet.
    ///
    /// Raised from a part's own menu and ending there: the word is added to the project
    /// and given to the part that asked for it.
    AMaterial,
    /// Naming a kind of building this project does not know yet.
    ///
    /// Raised FROM the carrying card and returning to it, so the bake a maker was
    /// halfway through is still there when they come back with a new word.
    AKind,
}

/// The work's own name, held while a kind is being named on the same card.
///
/// The card has one text field and two things to type into it, one after the
/// other. Without this the work's name is what the maker typed for the kind.
#[derive(Resource, Default)]
pub struct NameHeld(String);

/// A button that asks for a kind this project does not know yet.
#[derive(Component)]
pub(crate) struct NewKindButton;

/// Which kind the card is offering, as an index into [`crate::project::kinds`].
/// Which kind the card is offering, while it is up.
#[derive(Resource, Default)]
pub struct CarryingKind(pub usize);

/// One kind on the card.
#[derive(Component)]
pub(crate) struct KindButton(usize);

/// The naming card's root, for tearing it down.
#[derive(Component)]
pub(crate) struct NamingCard;

/// The text inside the card that shows the name as it is typed.
#[derive(Component)]
pub(crate) struct NameText;

/// The card's own buttons, for those who would rather click.
#[derive(Component)]
pub(crate) struct NamingSave;

#[derive(Component)]
pub(crate) struct NamingCancel;

/// F walks between face snapping and plain ground placement; G cycles
/// the grid interval through the powers of the atom.
fn toggle_snap_mode(
    keys: Res<ButtonInput<KeyCode>>,
    bench: Res<Bench>,
    naming: Res<Naming>,
    dims: Res<DimsEntry>,
    mut mode: ResMut<SnapMode>,
    mut grid: ResMut<SnapGrid>,
    // The word hangs inside its own little ground, so the text is the child.
    words: Query<&Children, With<SnapModeText>>,
    mut labels: Query<&mut Text>,
) {
    if *bench == Bench::Builder && naming.0.is_none() && dims.0.is_none() {
        if keys.just_pressed(KeyCode::KeyF) {
            mode.face = !mode.face;
        }
        if keys.just_pressed(KeyCode::KeyG) {
            grid.0 = match grid.0 {
                1 => 2,
                2 => 4,
                4 => 8,
                8 => 16,
                _ => 1,
            };
        }
    }
    let word = format!(
        "face snap - {} (F) / grid - {} (G)",
        if mode.face { "on" } else { "off" },
        grid.0
    );
    for kids in &words {
        for kid in kids.iter() {
            if let Ok(mut label) = labels.get_mut(kid)
                && label.0 != word
            {
                *label = Text::new(word.clone());
            }
        }
    }
}

pub struct BuilderPlugin;

impl Plugin for BuilderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Hand>()
            .init_resource::<Stages>()
            .init_resource::<StageWish>()
            .init_resource::<StageHeld>()
            .init_resource::<WorkWanted>()
            .init_resource::<CarryingKind>()
            .init_resource::<CarriedStale>()
            .init_resource::<PieceKept>()
            .init_resource::<PieceWantsAName>()
            .init_resource::<PieceInHand>()
            .init_resource::<PiecesStale>()
            .init_resource::<PalettesStale>()
            .init_resource::<MaterialFor>()
            .init_resource::<Brush>()
            .init_resource::<Naming>()
            .init_resource::<NameHeld>()
            .init_resource::<Hovered>()
            .init_resource::<WorkName>()
            .init_resource::<SnapMode>()
            .init_resource::<DimsEntry>()
            .init_resource::<History>()
            .init_resource::<SnapGrid>()
            .init_resource::<Clipboard>()
            .init_resource::<RoofsLifted>()
            .init_resource::<WindowPanes>()
            .init_resource::<DoorAs>()
            .add_systems(
                Startup,
                (raise_shelf, raise_palette).after(crate::rail::raise_rail),
            )
            // Two chains, because a tuple of systems stops at twenty:
            // the hand's work, then the bench's bookkeeping after it.
            .add_systems(
                Update,
                (
                    show_shelf,
                    work_drawers,
                    work_shelf,
                    open_or_clear,
                    steer_hand,
                    // The painting tools and the part menu as one group: this
                    // tuple is at Bevy's own limit for how many systems it will
                    // take in a row, and a nested tuple counts as one.
                    (
                        paint_the_work,
                        work_palette,
                        fill_the_palettes,
                        work_keep_colours,
                        work_drop_a_palette,
                        raise_part_menu,
                        turn_to_stage,
                        bury_the_chosen,
                    ),
                    toggle_snap_mode,
                    disarm_on_mode,
                    turn_part,
                    tilt_part,
                    turn_the_work,
                    reflow_openings,
                    lift_roofs,
                    copy_and_paste,
                    mirror_part,
                    feel_ahead,
                    light_the_chosen,
                    hang_the_chosen,
                    move_ghost,
                    // The menu acts AFTER the grab has had its look, and takes
                    // the ordering from this chain rather than asking for it: an
                    // `after` inside an already-chained tuple states the
                    // opposite of what the chain states, which Bevy cannot solve
                    // and will not start with.
                    //
                    // The order matters because a click on the menu is the
                    // menu's business, and the grab knows that - it steps aside
                    // for any click landing on interface. But choosing a line
                    // despawns the menu that same frame, so with the menu first
                    // the grab looked for interface under the cursor, found it
                    // already gone, and took the click for the world: it picked
                    // the roof up as it came apart.
                    (place_grab_remove, work_part_menu).chain(),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    save_workbench,
                    pick_a_work,
                    bake_into_the_game,
                    choose_the_kind,
                    // The pieces as one group: a tuple of systems stops at
                    // twenty, and a nested tuple counts as one.
                    (
                        hang_the_carried,
                        hang_the_pieces,
                        name_the_piece,
                        // The one that puts a piece IN THE HAND. It was written
                        // and never added to the schedule, so a piece could be
                        // kept and listed and never picked up - the whole
                        // feature, compiling cleanly and doing nothing.
                        wield_a_piece,
                    ),
                    take_one_back_out,
                    take_the_name,
                    dims_panel,
                    recall,
                    remember,
                    settle_words,
                )
                    .chain()
                    .after(place_grab_remove),
            );
    }
}

/// A mark's word, kept alive for a `PartKind` to hold.
///
/// ANY word, declared or not. What the project declares is what the shelf offers
/// and what colour a block wears - never what a saved work is allowed to contain.
/// A drawing opened in a game that has not declared its marks keeps every one of
/// them, and saving it keeps them still.
fn a_word(said: &str) -> &'static str {
    crate::project::widgets()
        .iter()
        .find(|mark| mark.word == said)
        .map_or_else(|| crate::project::a_kept_word(said), |mark| mark.word)
}

pub fn part_name(kind: &PartKind) -> String {
    match kind {
        PartKind::Seg { long, high, lift } => format!("wallseg-{long}x{high}@{lift}"),
        PartKind::Trim { long, stone } => {
            if *stone {
                format!("trimstone-{long}")
            } else {
                format!("trim-{long}")
            }
        }
        PartKind::Wall {
            long,
            high,
            framed,
            openings,
        } => {
            // Every opening's kind and where along the wall it sits, so a wall
            // with two windows in it is a different part from the same wall
            // with one.
            // ONE spelling for every wall: how long, how high, framed or not, and what it
            // makes room for. There were two before, because a framed wall was a different
            // kind of thing - `wall-4` for the plain one and `framed-4x2.5` for the other.
            let mut name = format!("wall-{long}x{high}");
            if *framed {
                name.push_str("xf");
            }
            // What this wall would have given a hole of each kind, which is what
            // a name is allowed to leave unsaid.
            let tall = (high / ATOM).round().max((PLATE_TALL * 3 + 8) as f32) as i32;
            for hole in openings.iter().flatten() {
                let letter = match hole.what {
                    Opening::Door => 'd',
                    Opening::Window => 'w',
                };
                name.push_str(&format!("x{letter}{}", hole.at));
                // Only when it is NOT the usual width for its kind. Every framed
                // wall drawn before openings had widths of their own writes exactly
                // the name it always wrote, and reads back the same.
                if hole.wide != usual_width(hole.what) {
                    name.push_str(&format!("@{}", hole.wide));
                }
                // And only when it is not the band this wall would have given
                // it. Same rule, same reason: a door, which is always a door's
                // height on the floor, is spelled the way it always was, and so
                // is every wall drawn before a window had a size of its own.
                let usual = band_of(hole.what, tall);
                if hole.lift != usual.foot || hole.high != usual.rise {
                    name.push_str(&format!("+{}+{}", hole.lift, hole.high));
                }
                // Only when the bars are dark. A wall drawn before they could be writes
                // the name it always wrote, and reads back the same.
                if hole.dark {
                    name.push('!');
                }
            }
            name
        }
        // In ATOMS, which is what it is measured in - and always a whole number
        // of panes, so `window-18x27` is two across and three up.
        PartKind::Window { wide, high } => format!("window-{wide}x{high}"),
        PartKind::Gable {
            long,
            pitch,
            framed,
        } => {
            // The framing is a word of its own on the end, exactly as a wall's is,
            // so every gable drawn before this reads and writes what it always did.
            let mut name = format!("gable-{long}x{pitch}");
            if *framed {
                name.push_str("xf");
            }
            name
        }
        // A BARE post spells itself the way it always did, so every work ever
        // drawn opens unchanged; the knees are a word on the end.
        PartKind::Pole { high, knees } => match knees {
            Knee::Bare => format!("pole-{high}"),
            Knee::One => format!("pole-{high}+brace"),
            Knee::Both => format!("pole-{high}+braces"),
            Knee::Corner => format!("pole-{high}+corner"),
            Knee::Three => format!("pole-{high}+braces3"),
            Knee::All => format!("pole-{high}+braces4"),
        },
        PartKind::Clock(wide) => format!("clock-{wide}"),
        PartKind::Table(long, deep) => format!("table-{long}x{deep}"),
        PartKind::Books(seed) => format!("books-{seed}"),
        // The four corners of one square, each spelled as what it IS. The three
        // that existed as props keep reading under their old names as well.
        PartKind::Door { double, leaf } => match (double, leaf) {
            (false, Leaf::Plain) => "door".to_string(),
            (true, Leaf::Plain) => "door-double".to_string(),
            (false, Leaf::Barn) => "door-barn".to_string(),
            (true, Leaf::Barn) => "door-barn-double".to_string(),
            (false, Leaf::Gone) => "doorway".to_string(),
            (true, Leaf::Gone) => "doorway-double".to_string(),
        },
        PartKind::Beam(long, high, low) => format!("beam-{long}x{high}x{low}"),
        PartKind::Ridge(long) => format!("ridge-{long}"),
        PartKind::Chimney(drop) => format!("chimney-{drop}"),
        PartKind::Rail { long, hand, stone } => {
            format!("rail-{long}x{hand}x{}", if *stone { "s" } else { "w" })
        }
        PartKind::RailRun { stone } => format!("railrun-{}", if *stone { "s" } else { "w" }),
        PartKind::Stairs {
            rise,
            wide,
            stone,
            rail_stone,
            hand,
        } => {
            // Two letters for two materials: the treads, then the rail. The
            // older spellings said one thing about a flight and are still read.
            let say = |stone: &bool| if *stone { "s" } else { "w" };
            format!(
                "stairs-{rise}x{wide}x{}{}x{hand}",
                say(stone),
                say(rail_stone)
            )
        }
        PartKind::GableRoof(long, span, over, pitch) => {
            format!("gableroof-{long}x{span}x{over}x{pitch}")
        }
        PartKind::HipRoof(long, span, over, pitch, deck) => {
            format!("hiproof-{long}x{span}x{over}x{pitch}x{deck}")
        }
        PartKind::RoofPlan(w, d) => format!("roofplan-{w}x{d}"),
        PartKind::Floor(w, d) => format!("floor-{w}x{d}"),
        PartKind::Ceiling {
            long,
            deep,
            hipped,
            across,
        } => {
            let mut name = format!("ceiling-{long}x{deep}");
            if *hipped {
                name.push_str("xh");
            }
            if *across {
                name.push_str("xa");
            }
            name
        }
        PartKind::Foundation(w, d, high) => format!("foundation-{w}x{d}x{high}"),
        PartKind::Roof(w, d) => format!("roof-{w}x{d}"),
        PartKind::TrimRun { .. } | PartKind::SegRun { .. } | PartKind::RidgeRun => {
            "run".to_string()
        }
        PartKind::Prop(name) => format!("prop:{name}"),
        PartKind::Widget(name) => format!("widget:{name}"),
        // The size LAST, so the word may hold hyphens of its own - a game with
        // `pallet-timber` and `pallet-stone` spells them out and this still reads
        // back only the three numbers on the end.
        PartKind::Area {
            word,
            long,
            deep,
            high,
        } => format!("area:{word}-{long}x{deep}x{high}"),
    }
}

pub fn kind_from_name(name: &str) -> Option<PartKind> {
    if let Some(rest) = name.strip_prefix("wall-") {
        let mut parts = rest.split('x');
        let long = parts.next()?.parse().ok()?;
        let high = parts
            .next()
            .and_then(|n| n.parse().ok())
            .unwrap_or(WALL_HIGH);
        let mut framed = false;
        let mut openings = [None; MOST_OPENINGS];
        let mut slots = openings.iter_mut();
        for said in parts {
            // The framing is a word of its own among the openings.
            if said == "f" {
                framed = true;
                continue;
            }
            let Some(slot) = slots.next() else { break };
            // `d0.5`, `d0.5@36`, `w0.5+18+16`: a width only when it is not the
            // usual one, and a band only when it is not the one its kind takes.
            let (said, dark) = match said.strip_suffix('!') {
                Some(said) => (said, true),
                None => (said, false),
            };
            // The band comes off the end first, since what is left of the two
            // numbers before it is what the older spellings said and no more.
            let (rest, band) = match said[1..].split_once('+') {
                Some((rest, band)) => {
                    let mut two = band.split('+').map(|n| n.parse::<i32>());
                    match (two.next(), two.next()) {
                        (Some(Ok(foot)), Some(Ok(rise))) => (rest, Some(Band { foot, rise })),
                        _ => (rest, None),
                    }
                }
                None => (&said[1..], None),
            };
            // A name that says no band means the band this wall gives - which is
            // the one place the courses still speak, and the reason every wall
            // drawn before a window had a size of its own comes back unchanged.
            let tall = (high / ATOM).round().max((PLATE_TALL * 3 + 8) as f32) as i32;
            let (where_at, wide) = match rest.split_once('@') {
                Some((at, wide)) => (at, wide.parse::<i32>().ok()),
                None => (rest, None),
            };
            let Ok(at) = where_at.parse() else { continue };
            let what = match said.as_bytes().first() {
                Some(b'd') => Some(Opening::Door),
                Some(b'w') => Some(Opening::Window),
                _ => None,
            };
            *slot = what.map(|what| {
                let usual = band.unwrap_or_else(|| band_of(what, tall));
                Hole {
                    what,
                    at,
                    wide: wide.unwrap_or(usual_width(what)),
                    dark,
                    high: usual.rise,
                    lift: usual.foot,
                }
            });
        }
        return Some(PartKind::Wall {
            long,
            high,
            framed,
            openings,
        });
    }
    if let Some(rest) = name.strip_prefix("window-") {
        let mut parts = rest.split('x');
        let wide = parts.next()?.parse().ok()?;
        let high = parts.next().and_then(|n| n.parse().ok()).unwrap_or(wide);
        return Some(PartKind::Window { wide, high });
    }
    if let Some(rest) = name.strip_prefix("hiproof-") {
        let mut parts = rest.split('x');
        let long = parts.next()?.parse().ok()?;
        let span = parts.next()?.parse().ok()?;
        let over = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0.25);
        let pitch = parts
            .next()
            .and_then(|n| n.parse().ok())
            .unwrap_or(ROOF_PITCH_DEGREES);
        // And the deck, which a hip saved before it could be closed did not carry:
        // it opens at the half every hip on this bench was drawn with.
        let deck = parts
            .next()
            .and_then(|n| n.parse().ok())
            .unwrap_or(HIP_DECK);
        return Some(PartKind::HipRoof(long, span, over, pitch, deck));
    }
    if let Some(rest) = name.strip_prefix("gableroof-") {
        // Four numbers now: the building it covers, the overhang, and the
        // pitch. Three is a roof from before the pitch could be pulled and two
        // from before the eaves could; both still open, at the pitch every roof
        // in the world had when they were drawn.
        let mut parts = rest.split('x');
        let long = parts.next()?.parse().ok()?;
        let span = parts.next()?.parse().ok()?;
        let over = parts.next().and_then(|o| o.parse().ok()).unwrap_or(0.25);
        let pitch = parts
            .next()
            .and_then(|p| p.parse().ok())
            .unwrap_or(ROOF_PITCH_DEGREES);
        return Some(PartKind::GableRoof(long, span, over, pitch));
    }
    if let Some(rest) = name.strip_prefix("chimney-") {
        return rest.parse::<f32>().ok().map(PartKind::Chimney);
    }
    if name == "prop:chimney" {
        // The first chimneys, from before the shaft could reach.
        return Some(PartKind::Chimney(0.0));
    }
    // The doors, new spellings and old: `prop:door` and its two fellows were the
    // three parts this one replaced, and a saved work full of them opens unchanged.
    if let Some(door) = match name {
        "door" | "prop:door" => Some((false, Leaf::Plain)),
        "door-double" | "prop:door-double" => Some((true, Leaf::Plain)),
        "door-barn" => Some((false, Leaf::Barn)),
        "door-barn-double" => Some((true, Leaf::Barn)),
        "doorway" | "prop:doorway" => Some((false, Leaf::Gone)),
        "doorway-double" => Some((true, Leaf::Gone)),
        _ => None,
    } {
        return Some(PartKind::Door {
            double: door.0,
            leaf: door.1,
        });
    }
    if let Some(rest) = name.strip_prefix("books-") {
        return rest.parse::<u32>().ok().map(PartKind::Books);
    }
    if name == "prop:books" {
        // The rows drawn before one carried a seed, at the colours they all had.
        return Some(PartKind::Books(0));
    }
    if let Some(rest) = name.strip_prefix("table-") {
        return sides_of(rest).map(|(long, deep)| PartKind::Table(long, deep));
    }
    if name == "prop:table" {
        // Every table drawn before one could be pulled longer, at the size they all
        // were.
        return Some(PartKind::Table(1.5, 0.875));
    }
    if let Some(rest) = name.strip_prefix("clock-") {
        return rest.parse::<f32>().ok().map(PartKind::Clock);
    }
    if let Some(rest) = name.strip_prefix("pole-") {
        // Longest first, or `+braces` would swallow nothing and `+brace` would
        // never be reached. A BARE post still spells itself the way it always
        // did, so every work already drawn opens unchanged.
        let (high, knees) = [
            ("+braces4", Knee::All),
            ("+braces3", Knee::Three),
            ("+corner", Knee::Corner),
            ("+braces", Knee::Both),
            ("+brace", Knee::One),
        ]
        .into_iter()
        .find_map(|(said, knees)| rest.strip_suffix(said).map(|high| (high, knees)))
        .unwrap_or((rest, Knee::Bare));
        return high
            .parse::<f32>()
            .ok()
            .map(|high| PartKind::Pole { high, knees });
    }
    if name == "prop:pole" {
        // The corner post, from before it had a height of its own. At the height
        // it always had, which is a wall's - it was drawn to stand beside one.
        return Some(PartKind::Pole {
            high: WALL_HIGH,
            knees: Knee::Bare,
        });
    }
    if let Some(rest) = name.strip_prefix("rail-") {
        let mut parts = rest.split('x');
        let long = parts.next()?.parse().ok()?;
        let hand = parts
            .next()
            .and_then(|n| n.parse().ok())
            .unwrap_or(RAIL_HIGH);
        let stone = parts.next().is_some_and(|word| word.starts_with('s'));
        return Some(PartKind::Rail { long, hand, stone });
    }
    // Stone first: `stairs-` is a prefix of nothing else, but `stairsstone-`
    // begins with `stairs` and would be read as a timber flight called `stone…`.
    for (word, was_stone) in [("stairsstone-", true), ("stairs-", false)] {
        if let Some(rest) = name.strip_prefix(word) {
            // What it climbs, how wide it is, and two letters for the two
            // materials. A flight from before a maker could widen them opens at
            // the width they all had; one from before the rail could differ
            // wears the same material throughout, which is what it looked like.
            let mut parts = rest.split('x');
            let rise = parts.next()?.parse().ok()?;
            let wide = parts.next().and_then(|n| n.parse().ok()).unwrap_or(1.25);
            let cloth = parts.next().unwrap_or("");
            let letter =
                |at: usize, was: bool| cloth.chars().nth(at).map_or(was, |letter| letter == 's');
            // And the rail's own height, on the end. A flight from before the
            // rail could be raised opens at the height they all had.
            let hand = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0.875);
            return Some(PartKind::Stairs {
                rise,
                wide,
                stone: letter(0, was_stone),
                rail_stone: letter(1, was_stone),
                hand,
            });
        }
    }
    if let Some(rest) = name.strip_prefix("ridge-") {
        return rest.parse::<f32>().ok().map(PartKind::Ridge);
    }
    if let Some(rest) = name.strip_prefix("beam-") {
        // Three numbers: its length and the cut at each end. One is a beam from
        // before ends could be cut, and opens square at both.
        let mut parts = rest.split('x');
        let long = parts.next()?.parse().ok()?;
        let high = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0.0);
        let low = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0.0);
        return Some(PartKind::Beam(long, high, low));
    }
    if let Some(rest) = name.strip_prefix("gable-") {
        // Two numbers now, its width and its pitch. One is a gable from before
        // gables had a pitch of their own, and opens at the one they all had.
        let mut parts = rest.split('x');
        let long = parts.next()?.parse().ok()?;
        let pitch = parts
            .next()
            .and_then(|p| p.parse().ok())
            .unwrap_or(ROOF_PITCH_DEGREES);
        // And the framing, if it says so.
        let framed = parts.next() == Some("f");
        return Some(PartKind::Gable {
            long,
            pitch,
            framed,
        });
    }
    if let Some(rest) = name.strip_prefix("trimstone-") {
        return rest
            .parse::<f32>()
            .ok()
            .map(|long| PartKind::Trim { long, stone: true });
    }
    if let Some(rest) = name.strip_prefix("trim-") {
        return rest
            .parse::<f32>()
            .ok()
            .map(|long| PartKind::Trim { long, stone: false });
    }
    if let Some(rest) = name.strip_prefix("floor-") {
        return sides_of(rest).map(|(w, d)| PartKind::Floor(w, d));
    }
    if let Some(rest) = name.strip_prefix("ceiling-") {
        let across = rest.ends_with("xa");
        let rest = rest.strip_suffix("xa").unwrap_or(rest);
        let hipped = rest.ends_with("xh");
        let rest = rest.strip_suffix("xh").unwrap_or(rest);
        return sides_of(rest).map(|(long, deep)| PartKind::Ceiling {
            long,
            deep,
            hipped,
            across,
        });
    }
    if let Some(rest) = name.strip_prefix("foundation-") {
        // Three numbers now. A pad from before it could be raised opens at the
        // height every pad used to have.
        let mut parts = rest.split('x');
        let w = parts.next()?.parse().ok()?;
        let d = parts.next()?.parse().ok()?;
        let high = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0.375);
        return Some(PartKind::Foundation(w, d, high));
    }
    if let Some(rest) = name.strip_prefix("roof-") {
        return sides_of(rest).map(|(w, d)| PartKind::Roof(w, d));
    }
    if let Some(rest) = name.strip_prefix("wallseg-") {
        let (long, rest) = rest.split_once('x')?;
        let (high, lift) = rest.split_once('@')?;
        return Some(PartKind::Seg {
            long: long.parse().ok()?,
            high: high.parse().ok()?,
            lift: lift.parse().ok()?,
        });
    }
    if let Some(wanted) = name.strip_prefix("prop:") {
        // What the bench makes for itself, before what it offers: see `PUNCHED`.
        if let Some(made) = PUNCHED.iter().find(|made| **made == wanted) {
            return Some(PartKind::Prop(made));
        }
        return STRUCTURE
            .iter()
            .chain(FURNITURE)
            .chain(DECOR)
            .find_map(|e| match e.kind {
                PartKind::Prop(p) if p == wanted => Some(e.kind),
                _ => None,
            });
    }
    if let Some(widget) = name.strip_prefix("widget:") {
        return Some(PartKind::Widget(a_word(widget)));
    }
    // A MARKED VOLUME, whose size is its own and not the project's: a maker drags
    // a pallet to the room they meant it to have, and reopening the work has to
    // give back that room rather than whatever the project declares today.
    if let Some(rest) = name.strip_prefix("area:") {
        let (word, measures) = rest.rsplit_once('-')?;
        let mut said = measures.split('x').filter_map(|n| n.parse::<f32>().ok());
        let (long, deep, high) = (said.next()?, said.next()?, said.next()?);
        return Some(PartKind::Area {
            word: a_word(word),
            long,
            deep,
            high,
        });
    }
    match name {
        // Legacy names from before the primitives learned their sizes.
        "floor" => Some(PartKind::Floor(2.0, 2.0)),
        "roof" => Some(PartKind::Roof(2.2, 2.2)),
        "prop:foundation" => Some(PartKind::Foundation(2.0, 2.0, STEP_UP)),
        "prop:trim" => Some(PartKind::Trim {
            long: 2.0,
            stone: false,
        }),
        "prop:trim-stone" => Some(PartKind::Trim {
            long: 2.0,
            stone: true,
        }),
        _ => None,
    }
}

/// Splits "3x4.5" into its two sides.
fn sides_of(text: &str) -> Option<(f32, f32)> {
    let (w, d) = text.split_once('x')?;
    Some((w.parse().ok()?, d.parse().ok()?))
}

/// Spawns a part's boxes under one root. Widgets go translucent.
fn spawn_part(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
    kind: &PartKind,
    record: &Placed,
    ghostly: bool,
) -> Entity {
    let root = commands
        .spawn((
            record.clone(),
            Transform::from_translation(Vec3::from(record.at)).with_rotation(pose(
                record.yaw,
                record.tilt,
                record.flip,
            )),
            Visibility::default(),
            BuilderFurniture,
        ))
        .id();
    if ghostly {
        commands.entity(root).insert(Ghost);
    }
    dress_part(
        commands, meshes, materials, palette, kind, record, root, ghostly,
    );
    root
}

/// The figures a piece of furniture comes with: where a body will lie
/// or sit on it, and which way round. They arrive as REAL widgets set
/// down beside the furniture, not as decoration - so an unwanted one is
/// picked up and thrown away like anything else, and a bed with no
/// sleeper on it means exactly that.
///
/// Every one of these pieces faces its own +Z - the back of a chair, the
/// pillow of a bed - and a widget's nose is its +X, so the figure always
/// turns a quarter behind the furniture that carries it.
pub fn companions(kind: &PartKind) -> Vec<(&'static str, Vec3)> {
    // Mattress top: where a sleeper's back actually rests.
    const LIE: f32 = 0.53125;
    match kind {
        PartKind::Prop("bed") => vec![("sleep", Vec3::new(0.0, LIE, 0.0))],
        PartKind::Prop("bed-double") => vec![
            ("sleep", Vec3::new(-0.5, LIE, 0.0)),
            ("sleep", Vec3::new(0.5, LIE, 0.0)),
        ],
        PartKind::Prop("chair" | "stool") => vec![("sit", Vec3::ZERO)],
        PartKind::Prop("bench") => vec![
            ("sit", Vec3::new(-0.4375, 0.0, 0.0)),
            ("sit", Vec3::new(0.4375, 0.0, 0.0)),
        ],
        // A cushion sits higher than a plank, so the sitter rides up
        // with it, and forward a touch so the knees clear the plinth.
        PartKind::Prop("couch") => vec![
            ("sit", Vec3::new(-0.4375, 0.09375, 0.0)),
            ("sit", Vec3::new(0.4375, 0.09375, 0.0)),
        ],
        _ => vec![],
    }
}

/// The PARTS a piece of furniture brings with it, as kind, offset and turn.
///
/// Brett, of the conference table: "Can we have it as a group with chairs and sit
/// widgets already there when you place it?" A council's board is not a board - it
/// is a board and the seats round it - and setting eight chairs by hand and then
/// gathering them is work a maker should not be doing twice for every hall.
///
/// The chairs bring their OWN sit marks, because a chair already knows it is for
/// sitting on: see `companions`. Nothing here has to say so twice.
pub fn company_of(kind: &PartKind) -> Vec<(PartKind, Vec3, f32)> {
    match kind {
        // THE CLERK'S OWN CHAIR, on the side the drawers open and the back panel
        // hides - which is the side somebody sits at. Brett: "The desk should have
        // a chair with a widget too."
        PartKind::Prop("desk") => vec![(
            PartKind::Prop("chair"),
            Vec3::new(0.0, 0.0, ATOM * 10.0),
            std::f32::consts::PI,
        )],
        PartKind::Table(long, deep) => {
            // A seat every three quarters of a metre, which is elbow room, and at
            // least one a side however short the board is.
            let seats = ((long / 0.75).round() as i32).clamp(1, 10);
            let mut company = Vec::new();
            for step in 0..seats {
                // Spread evenly, each in the middle of its own share of the board.
                let along = long * ((step as f32 + 0.5) / seats as f32 - 0.5);
                for side in [-1.0f32, 1.0] {
                    company.push((
                        PartKind::Prop("chair"),
                        Vec3::new(
                            on_the_lattice(along),
                            0.0,
                            side * on_the_lattice(deep * 0.5 + 0.25),
                        ),
                        // Facing the board it is drawn up to: a chair's back is
                        // behind it, so the far side turns right round.
                        if side < 0.0 {
                            0.0
                        } else {
                            std::f32::consts::PI
                        },
                    ));
                }
            }
            company
        }
        _ => vec![],
    }
}

/// Sets down the company a part brings, grouped with it so it travels as one.
///
/// Returns the group they share, for the part itself to wear.
#[allow(clippy::too_many_arguments)]
pub fn seat_the_company(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
    kind: &PartKind,
    record: &Placed,
    group: u32,
) {
    let turn = pose(record.yaw, record.tilt, record.flip);
    for (piece, offset, facing) in company_of(kind) {
        let offset = if record.flip {
            Vec3::new(-offset.x, offset.y, offset.z)
        } else {
            offset
        };
        let at = Vec3::from(record.at) + turn * offset;
        let seated = Placed {
            part: part_name(&piece),
            at: at.into(),
            yaw: record.yaw + facing,
            tilt: 0.0,
            ramp: record.ramp.clone(),
            shade: record.shade,
            stage: record.stage.clone(),
            flip: false,
            loose: false,
            material: String::new(),
            group: Some(group),
        };
        spawn_part(commands, meshes, materials, palette, &piece, &seated, false);
        // And whatever IT implies in turn - a chair's own sitting place.
        seat_the_figures(commands, meshes, materials, palette, &piece, &seated);
    }
}

/// Sets down the figures a piece of furniture implies, alongside it.
#[allow(clippy::too_many_arguments)]
pub fn seat_the_figures(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
    kind: &PartKind,
    record: &Placed,
) {
    let turn = pose(record.yaw, record.tilt, record.flip);
    for (what, offset) in companions(kind) {
        let widget = PartKind::Widget(what);
        let offset = if record.flip {
            Vec3::new(-offset.x, offset.y, offset.z)
        } else {
            offset
        };
        let at = Vec3::from(record.at) + turn * offset;
        let mark = Placed {
            part: part_name(&widget),
            at: at.into(),
            yaw: record.yaw - std::f32::consts::FRAC_PI_2,
            tilt: 0.0,
            ramp: None,
            shade: 0.7,
            stage: "widget".to_string(),
            flip: false,
            loose: false,
            material: String::new(),
            group: None,
        };
        spawn_part(commands, meshes, materials, palette, &widget, &mark, false);
    }
}

/// Dresses an existing root in a part's boxes - the resize handles use
/// this to rebuild a body in place without disturbing the entity.
#[allow(clippy::too_many_arguments)]
pub fn dress_part(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
    kind: &PartKind,
    record: &Placed,
    root: Entity,
    ghostly: bool,
) {
    let translucent = ghostly || is_a_mark(kind);
    let repaint = record.ramp.as_deref().map(|r| (r, record.shade));
    for Slab {
        mut at,
        size,
        ramp,
        shade,
        clarity,
        shape,
        mut lean,
        cant,
        cut,
    } in body_of(kind, repaint)
    {
        // Mirrored: the body reflects across its own length, and any
        // lean of its own leans the other way.
        if record.flip {
            at.x = -at.x;
            lean = -lean;
        }
        let mut color = palette.shade(&ramp, shade);
        let see_through = translucent || clarity < 1.0;
        if see_through {
            color = color.with_alpha(if ghostly {
                0.45
            } else if is_a_mark(kind) {
                0.55
            } else {
                clarity
            });
        }
        commands.spawn((
            Mesh3d(match shape {
                Shape::Wedge => meshes.add(wedge_mesh(false)),
                Shape::Ridge => meshes.add(wedge_mesh(true)),
                Shape::Hip(top_x, top_z) => meshes.add(hip_mesh(top_x, top_z)),
                // A square-ended box is the common case and Bevy's own cuboid
                // is the cheapest way to say it.
                Shape::Box if cut == Vec2::ZERO => meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                Shape::Box => meshes.add(cut_mesh(cut.x / size.x, cut.y / size.x)),
            }),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.95,
                reflectance: 0.03,
                alpha_mode: if see_through {
                    AlphaMode::Blend
                } else {
                    AlphaMode::Opaque
                },
                ..default()
            })),
            Transform::from_translation(at)
                .with_rotation(Quat::from_rotation_z(cant) * Quat::from_rotation_x(lean))
                .with_scale(size),
            ChildOf(root),
        ));
    }
}

/// Rebuilds the ghost from the hand's current state.
fn dress_ghost(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
    hand: &Hand,
    ghosts: &Query<Entity, With<Ghost>>,
) {
    for ghost in ghosts {
        commands.entity(ghost).despawn();
    }
    if let Some(kind) = hand.kind
        && let Some(record) = hand.record(Vec3::new(0.0, hand.lift, 0.0))
    {
        spawn_part(commands, meshes, materials, palette, &kind, &record, true);
    }
}

/// What a piece is on disk, and in the hand: a cluster of parts with their own
/// little world, whose middle is where a maker points.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Piece {
    format: u32,
    kind: String,
    name: String,
    parts: Vec<Placed>,
}

/// The parts a maker has just asked to keep, waiting for a name.
#[derive(Resource, Default)]
pub struct PieceKept(pub Vec<Placed>);

/// Whether the card should go up for them.
#[derive(Resource, Default)]
pub struct PieceWantsAName(pub bool);

/// The piece the hand is holding, if it holds one.
#[derive(Resource, Default)]
pub struct PieceInHand {
    parts: Vec<Placed>,
    name: String,
    /// Quarter turns, the way every other placement turns.
    yaw: f32,
}

/// A ghost box belonging to the held piece.
#[derive(Component)]
pub(crate) struct PieceGhost;

/// A button that takes a kept piece into the hand.
#[derive(Component)]
pub(crate) struct PieceButton(std::path::PathBuf);

/// Where pieces are kept.
fn pieces_home() -> std::path::PathBuf {
    bench_home().join("out/pieces")
}

/// Centres a cluster on its own middle, so a piece is set down where the cursor
/// is rather than wherever it happened to be drawn.
///
/// Middle on the ground, FOOT in the air: a porch put down at head height would
/// be a puzzle. The lowest part of a piece is what meets the ground the maker
/// is pointing at.
fn piece_from(parts: &[Placed]) -> Vec<Placed> {
    let mut low = Vec3::splat(f32::INFINITY);
    let mut high = Vec3::splat(f32::NEG_INFINITY);
    for record in parts {
        low = low.min(Vec3::from(record.at));
        high = high.max(Vec3::from(record.at));
    }
    if !low.x.is_finite() {
        return parts.to_vec();
    }
    let middle = Vec3::new((low.x + high.x) * 0.5, low.y, (low.z + high.z) * 0.5);
    parts
        .iter()
        .map(|record| {
            let mut moved = record.clone();
            moved.at = (Vec3::from(record.at) - middle).into();
            moved
        })
        .collect()
}

/// A full hand places on click. An empty hand picks a placed part back up.
/// X removes what the cursor touches either way.
#[allow(clippy::too_many_arguments)]
pub(crate) fn place_grab_remove(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    snap_grid: Res<SnapGrid>,
    bench: Res<Bench>,
    naming: Res<Naming>,
    hovered: Res<Hovered>,
    // Bundled: Bevy's parameter ceiling is sixteen, and this system
    // presses it.
    gizmo: (
        Res<crate::gizmo::GizmoHot>,
        Res<crate::gizmo::Selected>,
        Res<crate::gizmo::ToolMode>,
    ),
    mut hand: ResMut<Hand>,
    palette: Res<Palette>,
    ghosts: Query<Entity, With<Ghost>>,
    ghost_spots: Query<(&Transform, &Placed), With<Ghost>>,
    placed: Query<(Entity, &Transform, &Placed, &Visibility), Without<Ghost>>,
    hovers: Query<&Interaction>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (gizmo_hot, selected, tool) = gizmo;
    if *bench != Bench::Builder
        || naming.0.is_some()
        || gizmo_hot.0
        || !selected.is_empty()
        || *tool != crate::gizmo::ToolMode::Normal
    {
        return;
    }
    // A click that lands on UI is the UI's business.
    let over_ui = hovers
        .iter()
        .any(|interaction| *interaction != Interaction::None);

    // A SHIFT-click gathers rather than picks up - but only with an EMPTY hand. Somebody
    // holding a wall is placing walls, not choosing them, and shift there means "and
    // another one" like it does everywhere else in the bench.
    if held_shift(&keys) && hand.kind.is_none() && buttons.just_pressed(MouseButton::Left) {
        return;
    }
    // ONE PART PER PICK, unless the maker says otherwise. Brett: "if you place one down
    // there shouldnt be another ghost in your hand unless you hold shift while placing
    // it". The hand used to keep whatever it held for ever, so setting down a foundation
    // left another foundation stuck to the cursor and the next click somewhere harmless
    // put down a second one.
    //
    // SHIFT, which is where the rest of "and another" lives - and where Brett reached for
    // it. The fine snap moved to ALT to make room; see `held_shift` and `held_fine`.
    let keep_holding = held_shift(&keys);
    if buttons.just_pressed(MouseButton::Left) && !over_ui {
        if let Some(kind) = hand.kind {
            // A stretch tool: the first click sets the anchor where the
            // stub stands; the next makes the drawn part real. A wall run
            // chains - the far end becomes the next anchor - while rects
            // rest after each one.
            if kind.run_axes().is_some() {
                if hand.anchor.is_none() {
                    if let Some((ghost_at, _)) = ghost_spots.iter().next() {
                        hand.anchor = Some(ghost_at.translation);
                    }
                } else if let Some((ghost_at, drawn)) = ghost_spots.iter().next()
                    && let Some(made) = kind_from_name(&drawn.part)
                {
                    spawn_part(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &palette,
                        &made,
                        drawn,
                        false,
                    );
                    // A run CHAINS while it is held: the far end becomes the next
                    // anchor, which is how a line of wall is drawn in one go. Letting go
                    // of it is putting the tool down.
                    hand.anchor = if kind.run_axes() == Some(1) && keep_holding {
                        hand.anchor
                            .map(|anchor| ghost_at.translation * 2.0 - anchor)
                    } else {
                        None
                    };
                    if !keep_holding {
                        hand.kind = None;
                    }
                }
                return;
            }
            // Doors and windows would rather punch through a wall than
            // stand alone: if one lands on a wall, the wall parts around
            // the opening and the frame settles in.
            let opening = opening_of(&kind);
            let punched = if let Some(opens) = opening
                && let Some((ghost_at, _)) = ghost_spots.iter().next()
            {
                punch_wall(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &palette,
                    &placed,
                    hovered.build.map(|hit| (hit.entity, hit.point, hit.normal)),
                    ghost_at.translation,
                    opens,
                    &hand,
                    snap_step(held_fine(&keys), snap_grid.0),
                )
            } else {
                false
            };
            // Setting down (a punch already set the frame itself).
            if !punched
                && let Some((ghost_at, _)) = ghost_spots.iter().next()
                && let Some(record) = hand.record(ghost_at.translation)
            {
                // WHAT COMES WITH IT, grouped with it: a table arrives with the
                // chairs drawn up to it, and the lot moves as one thing.
                let mut record = record;
                // AND WHAT IS DEALT AFRESH: a row of books gets its own hand of
                // colours, so no two shelves in a village are the same shelf.
                let kind = match kind {
                    PartKind::Books(_) => {
                        PartKind::Books(a_fresh_seed(Vec3::from(record.at), placed.iter().count()))
                    }
                    other => other,
                };
                record.part = part_name(&kind);
                let company = !company_of(&kind).is_empty();
                let group = a_fresh_group(placed.iter().map(|(_, _, standing, _)| standing));
                if company {
                    record.group = Some(group);
                }
                spawn_part(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &palette,
                    &kind,
                    &record,
                    false,
                );
                seat_the_figures(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &palette,
                    &kind,
                    &record,
                );
                if company {
                    seat_the_company(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &palette,
                        &kind,
                        &record,
                        group,
                    );
                }
            }
            // Down it goes, and the hand is empty - a door punched into a wall counts,
            // since that is what setting a door down means.
            if !keep_holding {
                hand.kind = None;
                hand.anchor = None;
            }
        } else if let Some(grabbed) = hovered.grab
            && let Ok((_, transform, record, _)) = placed.get(grabbed)
            && let Some(kind) = kind_from_name(&record.part)
        {
            // A WINDOW IS NOT A PART, so grabbing one means asking the wall which of its
            // openings the cursor was on and taking that one out. The wall stays where it
            // is and closes over the hole; the window comes to hand as the thing that was
            // put there, ready to be set down somewhere better.
            if let Some(hit) = hovered.build
                && let Some(slot) =
                    opening_under(&kind, transform.translation, record.yaw, hit.point)
                && let PartKind::Wall {
                    long,
                    high,
                    framed,
                    mut openings,
                } = kind
            {
                let taken = openings[slot].take();
                let made = PartKind::Wall {
                    long,
                    high,
                    framed,
                    openings,
                };
                let mut healed = record.clone();
                healed.part = part_name(&made);
                commands.entity(grabbed).despawn_related::<Children>();
                dress_part(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &palette,
                    &made,
                    &healed,
                    grabbed,
                    false,
                );
                commands.entity(grabbed).insert(healed);
                // A WINDOW COMES BACK AT ITS OWN SIZE. It is the size that makes it
                // the window it is now, so a hand that dropped it back at the shelf's
                // two-by-two would have quietly resized every window a maker ever
                // moved.
                *hand = Hand {
                    kind: Some(match taken {
                        Some(hole) if hole.what == Opening::Window => PartKind::Window {
                            wide: hole.wide,
                            high: hole.high,
                        },
                        // A DOOR comes back as the door it was, read off the hole
                        // it left: how tall says whether it was a barn's, how wide
                        // says whether it was a pair.
                        Some(hole) => {
                            let leaf = if hole.high >= BARN_HIGH {
                                Leaf::Barn
                            } else {
                                Leaf::Plain
                            };
                            PartKind::Door {
                                double: hole.wide > door_clear(leaf, false).0,
                                leaf,
                            }
                        }
                        None => PartKind::Window {
                            wide: WINDOW_WIDE,
                            high: WINDOW_WIDE,
                        },
                    }),
                    stage: record.stage.clone(),
                    yaw: record.yaw,
                    ..Hand::default()
                };
                dress_ghost(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &palette,
                    &hand,
                    &ghosts,
                );
                return;
            }
            // An opening picked up closes the wall behind it.
            heal_wall(
                &mut commands,
                &mut meshes,
                &mut materials,
                &palette,
                &placed,
                grabbed,
            );
            // Picking back up: the part leaves the floor and rides the
            // cursor again with its paint and turn intact. Only the height
            // ABOVE its old support comes along - the new resting place
            // supplies its own.
            let beneath = support_height(
                &placed,
                &footprint_samples(&kind, transform.translation, record.yaw),
                is_structure(&kind),
                Some(grabbed),
            );
            *hand = Hand {
                kind: Some(kind),
                anchor: None,
                flip: record.flip,
                stage: record.stage.clone(),
                yaw: record.yaw,
                tilt: record.tilt,
                lift: (transform.translation.y - beneath).max(0.0),
                ramp: record.ramp.clone(),
                shade: record.shade,
            };
            commands.entity(grabbed).despawn();
            dress_ghost(
                &mut commands,
                &mut meshes,
                &mut materials,
                &palette,
                &hand,
                &ghosts,
            );
        }
    }

    if (keys.just_pressed(KeyCode::KeyX)
        || keys.just_pressed(KeyCode::Delete)
        || keys.just_pressed(KeyCode::Backspace))
        && let Some(doomed) = hovered.grab
        && placed.contains(doomed)
    {
        // A removed opening leaves the wall whole again.
        heal_wall(
            &mut commands,
            &mut meshes,
            &mut materials,
            &palette,
            &placed,
            doomed,
        );
        commands.entity(doomed).despawn();
    }
}

#[cfg(test)]
mod bake_tests;
#[cfg(test)]
mod colours;
#[cfg(test)]
mod levels;
#[cfg(test)]
mod roof_tests;
