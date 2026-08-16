//! What a part IS: the kinds, and the shelves they are offered from.

use super::*;

/// What a framed wall makes room for.
///
/// NOT a hole cut in anything. An opening is a region the panels decline to
/// fill and that gathers its own timber around itself - jambs either side, a
/// lintel over, and for a window a sill under. That is why every window in a
/// real half-timbered wall sits inside its own little frame, and it is why
/// there is no boolean anywhere in here.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Opening {
    /// Floor to head height, for walking through.
    Door,
    /// A hole in the upper course with a sill under it.
    Window,
}

/// Parts the bench MAKES rather than offers: the leaf a punch hangs in an
/// opening it has just framed.
///
/// Not on any shelf, because nobody places a bare door leaf by hand - a punch
/// creates them, and only when the wall it went into drew the frame itself. But
/// they must still be READABLE, which is what this list is for: `kind_from_name`
/// resolves a prop by searching the shelves, so a part on no shelf came back as
/// nothing at all. It drew once when punched and then vanished at the next respawn
/// - a phase change, a save and reopen - and never reached the game, because the
/// bake skips a name it cannot read. Every framed-wall door in every drawing was
/// losing its leaf in silence.
pub(crate) const PUNCHED: [&str; 2] = ["door-leaf", "door-double-leaf"];

/// One opening in a framed wall: what kind, where along it, and how wide.
///
/// The width used to be implied by the kind - a door was `DOOR_WIDE` and that was
/// that - so a double door punched into a framed wall got a single door's hole and
/// its leaves stood over solid timber. Brett: "double doors dont work when placing
/// them on framed walls."
///
/// In ATOMS, because a wall is solved in atoms: it is the CLEAR span, the hole a
/// leaf has to fit through, and the jambs gather outside it.
#[derive(Clone, Copy, PartialEq)]
pub struct Hole {
    pub what: Opening,
    /// How far along the wall from its middle, in metres.
    pub at: f32,
    pub wide: i32,
    /// Whether this window's bars are painted dark rather than left as timber.
    ///
    /// On the WINDOW rather than on the wall, because that is what wears them - and
    /// because a wall is spelled out as a name, so a field here costs seven places that
    /// build a hole where one on the wall would have cost twenty-two.
    ///
    /// A door has bars in nothing and ignores it.
    pub dark: bool,
}

impl PartKind {
    /// A plain wall of a length: the usual height, no framing, nothing in it.
    ///
    /// The shorthand for what used to be `Wall(long)`, kept because most of the bench
    /// wants exactly that and spelling four fields at every one of them would bury the
    /// two places that actually care.
    pub const fn wall(long: f32) -> PartKind {
        PartKind::Wall {
            long,
            high: WALL_HIGH,
            framed: false,
            openings: [None; MOST_OPENINGS],
        }
    }
}

impl Hole {
    /// An opening at the width its kind usually takes.
    pub const fn plain(what: Opening, at: f32) -> Hole {
        Hole {
            what,
            at,
            wide: usual_width(what),
            dark: false,
        }
    }
}

/// The width a kind of opening takes when nobody says otherwise.
///
/// Also what a NAME leaves out: a hole of the usual width writes no width at all,
/// so every framed wall drawn before this reads back byte for byte the same.
pub const fn usual_width(what: Opening) -> i32 {
    match what {
        Opening::Door => DOOR_WIDE,
        Opening::Window => WINDOW_WIDE,
    }
}

/// What a shelf entry stands for.
#[derive(Clone, Copy, PartialEq)]
pub enum PartKind {
    /// A WALL: how long, how high, whether it is framed, and what it makes room for.
    ///
    /// One part where there were two. A framed wall used to be its own species, which is
    /// why a window was two different things - openings belonged to the framed kind, and a
    /// plain wall had to be cut into `Seg` leftovers around a hole instead. Brett: "Walls
    /// should just have a right click to add the framing."
    ///
    /// So framing is a PROPERTY, like a flight's rail being stone or timber. A plain wall
    /// and a half-timbered one are the same wall with the same openings in the same places;
    /// the flag decides whether the infill is bays and studs or plain plaster.
    Wall {
        long: f32,
        high: f32,
        /// Half-timbered: the bench solves sill, plates, posts and studs for it, and it
        /// GAINS A BAY when pulled longer rather than stretching what is there.
        framed: bool,
        /// What it leaves holes for, and how far along it each one sits.
        ///
        /// A fixed array because a `PartKind` is `Copy` and is spelled out as a name - a
        /// wall is its measurements, and a list that could grow without bound could not be
        /// either. Four is more openings than one wall of a house has ever wanted.
        openings: [Option<Hole>; MOST_OPENINGS],
    },
    /// A piece of wall left standing around an opening: the sides of a
    /// doorway, the header above it, the sill strip under a window.
    Seg {
        long: f32,
        high: f32,
        lift: f32,
    },
    Trim {
        long: f32,
        stone: bool,
    },
    /// The stepped triangle that closes a pitched roof's end: courses of
    /// wall narrowing to a peak at the roof's own thirty degrees.
    Gable(f32, f32),
    /// A squared timber laid along its own length: the corner post's section,
    /// on its side and as long as it is drawn — and how far each end is cut
    /// back at an angle, nought for a square end.
    ///
    /// The cut is a RUN, not an angle: how far along the beam the saw travels
    /// while crossing its full height. That is the number the roof hands over —
    /// the difference between where the top corner meets the slope and where the
    /// bottom does — and it needs no trigonometry at either end.
    Beam(f32, f32, f32),
    BeamRun,
    /// The cap that hides the seam where two slopes meet.
    Ridge(f32),
    /// A chimney stack: the number is how far its shaft reaches DOWN
    /// from where it stands, so it can be buried in a roof's slope or
    /// run all the way to the hearth below.
    Chimney(f32),
    /// A wooden flight with a rail on each side, climbing by the height given.
    ///
    /// It climbs along its own +Z rather than +X, which is the one thing about
    /// it a maker might notice and only if they were looking for it: a slab can
    /// lean about X and no other axis, so a rail that RUNS at the stair's own
    /// angle - which is what Brett asked for, "We can use angles for the railing
    /// now that we have angles right?" - has to climb the axis that leaning
    /// answers to. Turn it with R like anything else.
    ///
    /// One part in two materials, the way TRIM and TRIM, STONE are one part:
    /// the flight is the same flight, and stone or timber is a fact about it
    /// rather than a different thing to draw.
    ///
    /// The TREADS and the RAIL answer separately, because a stone stair with a
    /// timber handrail is a real building and so is its opposite - Brett: "What
    /// if i wanted a stone railing on a wooden step or vice versa?" The shelf
    /// offers the two matching pairs, which are what a maker reaches for most,
    /// and the right-click menu changes the rail on a flight already standing.
    Stairs {
        rise: f32,
        wide: f32,
        stone: bool,
        rail_stone: bool,
        /// How high the rail stands above each tread.
        hand: f32,
    },
    /// A handrail on the flat: posts at each end, a rail between them, and
    /// balusters under it - the stair's own railing, run along level ground.
    ///
    /// Brett: "can you make a stratchable railing that lines up with this post
    /// from the stair railing? It would be great to continue it on a flat
    /// surface." So every measurement of it is the flight's: the same post, the
    /// same rail, the same balusters, and the same height above what it stands
    /// on - a landing at the top of a flight carries straight on.
    Rail {
        long: f32,
        hand: f32,
        stone: bool,
    },
    RailRun {
        stone: bool,
    },
    /// A ridge pole: a round log along the spine, the older way of
    /// closing a roof.
    /// A whole gable roof: both slopes and the ridge between them, drawn
    /// once over the walls instead of lined up slope by slope.
    GableRoof(f32, f32, f32, f32),
    /// What a whole roof looks like while it is being sized: the ground
    /// it will cover, with a gold line down the way the ridge will run.
    /// It is never placed - the record beneath it names the roof.
    RoofPlan(f32, f32),
    Floor(f32, f32),
    /// A stone pad: how wide, how deep, and how TALL.
    ///
    /// The height is a third number because Brett wanted footings that answer to
    /// the ground: "I would like the foundation to be able to be stretched in
    /// the other axis as well, so i could make taller foundations if I wanted."
    /// A pad set into a slope has to reach the ground somewhere.
    Foundation(f32, f32, f32),
    Roof(f32, f32),
    /// The stretch tools: anchored with one click, drawn to size, set
    /// with the next. They exist only in the hand - what they place are
    /// the plain kinds above at the drawn size.
    TrimRun {
        stone: bool,
    },
    /// A wall piece drawn at a fixed height and lift: the header that
    /// spans above an opening, the sill that fills below one.
    SegRun {
        high: f32,
        lift: f32,
    },
    GableRun,
    RidgeRun,
    GableRoofRun,
    /// A hip roof: four faces sloping in from the eaves to a flat deck.
    ///
    /// The same four numbers a gable roof takes - how long, how far across, how
    /// far the eaves reach past the walls, and the pitch - because it is the
    /// same roof with its ends hipped in rather than closed by a gable.
    HipRoof(f32, f32, f32, f32),
    HipRoofRun,
    RoofRun,
    Prop(&'static str),
    Widget(&'static str),
}

impl PartKind {
    /// The runs stretch along one axis; the rect runs stretch two.
    pub fn run_axes(&self) -> Option<u8> {
        match self {
            PartKind::TrimRun { .. }
            | PartKind::RailRun { .. }
            | PartKind::SegRun { .. }
            | PartKind::GableRun
            | PartKind::RidgeRun
            | PartKind::BeamRun => Some(1),
            PartKind::RoofRun | PartKind::GableRoofRun | PartKind::HipRoofRun => Some(2),
            _ => None,
        }
    }

    /// What a run becomes at the drawn size.
    pub fn run_made(&self, w: f32, d: f32) -> PartKind {
        match self {
            PartKind::TrimRun { stone } => PartKind::Trim {
                long: w,
                stone: *stone,
            },
            PartKind::RailRun { stone } => PartKind::Rail {
                long: w,
                hand: RAIL_HIGH,
                stone: *stone,
            },
            PartKind::GableRun => PartKind::Gable(w, ROOF_PITCH_DEGREES),
            PartKind::RidgeRun => PartKind::Ridge(w),
            PartKind::BeamRun => PartKind::Beam(w, 0.0, 0.0),
            // A hand's breadth of overhang to begin with; the gold
            // handles pull it further without moving the gables.
            PartKind::GableRoofRun => PartKind::GableRoof(w, d, 0.25, ROOF_PITCH_DEGREES),
            PartKind::HipRoofRun => PartKind::HipRoof(w, d, 0.25, ROOF_PITCH_DEGREES),
            PartKind::SegRun { high, lift } => PartKind::Seg {
                long: w,
                high: *high,
                lift: *lift,
            },
            PartKind::RoofRun => PartKind::Roof(w, d),
            other => *other,
        }
    }
}

/// A shelf entry: the name it wears, what it places, and the stage the
/// village raises it in.
pub struct CatalogEntry {
    pub label: &'static str,
    pub kind: PartKind,
    pub stage: &'static str,
}

pub(crate) const fn structure(
    label: &'static str,
    kind: PartKind,
    stage: &'static str,
) -> CatalogEntry {
    CatalogEntry { label, kind, stage }
}

pub(crate) const fn prop(label: &'static str, name: &'static str) -> CatalogEntry {
    CatalogEntry {
        label,
        kind: PartKind::Prop(name),
        stage: "furnishing",
    }
}

/// The shelf's drawers: each section opens and closes on its header.
pub const STRUCTURE: &[CatalogEntry] = &[
    structure("BEAM, STRETCH", PartKind::BeamRun, "frame"),
    structure("DOOR", PartKind::Prop("door"), "walls"),
    structure("DOOR, DOUBLE", PartKind::Prop("door-double"), "walls"),
    structure("DOORWAY", PartKind::Prop("doorway"), "walls"),
    structure("FLOOR", PartKind::Floor(2.0, 2.0), "footing"),
    structure(
        "FOUNDATION",
        PartKind::Foundation(2.0, 2.0, STEP_UP),
        "footing",
    ),
    structure("GABLE, STRETCH", PartKind::GableRun, "roof"),
    structure(
        "HEADER, STRETCH",
        PartKind::SegRun {
            high: 0.375,
            lift: 2.125,
        },
        "walls",
    ),
    structure("POLE, CORNER", PartKind::Prop("pole"), "frame"),
    structure(
        "RAIL, STONE, STRETCH",
        PartKind::RailRun { stone: true },
        "frame",
    ),
    structure("RAIL, STRETCH", PartKind::RailRun { stone: false }, "frame"),
    structure("RIDGE, STRETCH", PartKind::RidgeRun, "roof"),
    structure("ROOF, GABLE, STRETCH", PartKind::GableRoofRun, "roof"),
    structure("ROOF, HIP, STRETCH", PartKind::HipRoofRun, "roof"),
    structure("ROOF, PANEL", PartKind::Roof(2.2, 2.2), "roof"),
    structure("ROOF, STRETCH", PartKind::RoofRun, "roof"),
    structure(
        "SILL, STRETCH",
        PartKind::SegRun {
            high: 0.75,
            lift: 0.0,
        },
        "walls",
    ),
    // One noun for the family, the material after it - the way TRIM, STONE and
    // TRIM sit together. Two words for one thing, "stairs" and "steps", meant
    // knowing which of them we had happened to use.
    structure(
        "STAIRS, STONE",
        PartKind::Stairs {
            rise: STEP_UP,
            wide: 1.25,
            stone: true,
            rail_stone: true,
            hand: RAIL_HIGH,
        },
        "footing",
    ),
    structure(
        "STAIRS, WOOD",
        PartKind::Stairs {
            rise: STEP_UP,
            wide: 1.25,
            stone: false,
            rail_stone: false,
            hand: RAIL_HIGH,
        },
        "footing",
    ),
    structure(
        "TRIM, STONE, STRETCH",
        PartKind::TrimRun { stone: true },
        "walls",
    ),
    structure("TRIM, STRETCH", PartKind::TrimRun { stone: false }, "walls"),
    structure(
        "WALL",
        PartKind::Wall {
            long: 2.0,
            high: WALL_HIGH,
            framed: false,
            openings: [None; MOST_OPENINGS],
        },
        "walls",
    ),
    structure(
        "WALL, FRAMED",
        PartKind::Wall {
            framed: true,
            long: 3.0,
            high: WALL_HIGH,
            openings: [None; MOST_OPENINGS],
        },
        "walls",
    ),
    structure(
        "WALL, FRAMED, DOOR",
        PartKind::Wall {
            framed: true,
            long: 3.0,
            high: WALL_HIGH,
            openings: [Some(Hole::plain(Opening::Door, 0.0)), None, None, None],
        },
        "walls",
    ),
    structure(
        "WALL, FRAMED, WINDOW",
        PartKind::Wall {
            framed: true,
            long: 3.0,
            high: WALL_HIGH,
            openings: [Some(Hole::plain(Opening::Window, 0.0)), None, None, None],
        },
        "walls",
    ),
    structure("WINDOW", PartKind::Prop("window"), "walls"),
];

pub const FURNITURE: &[CatalogEntry] = &[
    prop("BED", "bed"),
    prop("BED, DOUBLE", "bed-double"),
    prop("BENCH", "bench"),
    prop("CHAIR", "chair"),
    prop("CHEST", "chest"),
    CatalogEntry {
        label: "CHIMNEY",
        kind: PartKind::Chimney(1.75),
        stage: "roof",
    },
    prop("COUCH", "couch"),
    prop("CRADLE", "cradle"),
    prop("CUPBOARD", "cupboard"),
    prop("HEARTH", "hearth"),
    prop("SHELVES", "shelves"),
    prop("STOOL", "stool"),
    prop("TABLE", "table"),
    prop("TABLE, SIDE", "side-table"),
    prop("WARDROBE", "wardrobe"),
];

pub const DECOR: &[CatalogEntry] = &[
    prop("ANVIL", "anvil"),
    prop("BARREL", "barrel"),
    prop("BASKET", "basket"),
    prop("CRATE", "crate"),
    prop("FENCE", "fence"),
    prop("LADDER", "ladder"),
    prop("LOOM", "loom"),
    prop("MANNEQUIN", "mannequin"),
    prop("PLANTER", "planter"),
    prop("POT, COOKING", "pot"),
    prop("RUG", "rug"),
    prop("SACK", "sack"),
    prop("STAND, CANDLE", "candle"),
    prop("TROUGH", "trough"),
    prop("WOODPILE", "woodpile"),
];
