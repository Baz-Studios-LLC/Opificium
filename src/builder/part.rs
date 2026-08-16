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
pub(crate) const PUNCHED: [&str; 3] = ["door-leaf", "door-double-leaf", "window"];

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
    /// How tall it is, in atoms.
    ///
    /// A WINDOW'S OWN, not the wall's. This was the wall's upper course and
    /// nothing else - a window WAS that course - so its height changed when the
    /// wall's did, its panes were counted off a number nobody chose, and a
    /// three-pane window could not be made at all. Brett, with the townhall
    /// drawn: "i want to uncouple the windows size and pane count from the wall
    /// height."
    ///
    /// A door still takes its kind's, since a door is as tall as a door.
    pub high: i32,
    /// How far its foot stands off the wall's own foot, in atoms.
    ///
    /// The other half of the same freedom: "I need to pick the size of the
    /// window and choose anywhere on the wall I want to put it."
    pub lift: i32,
}

/// Where an opening stands in a wall and how tall it is, in atoms off the
/// wall's own foot.
///
/// Not what a hole STORES - a hole carries these flat, because a size and a
/// place are two facts and a maker changes them one at a time. This is what
/// [`band_of`](super::band_of) ANSWERS with: the band a kind takes when nobody
/// has said otherwise, which is where a hole's two numbers come from at the one
/// moment it is made.
#[derive(Clone, Copy, PartialEq)]
pub struct Band {
    pub foot: i32,
    pub rise: i32,
}

impl PartKind {
    /// A plain wall of a length: the usual height, no framing, nothing in it.
    ///
    /// The shorthand for what used to be `Wall(long)`, kept because most of the bench
    /// wants exactly that and spelling four fields at every one of them would bury the
    /// two places that actually care.
    /// A plain gable of a length, at a pitch: no framing.
    ///
    /// The same shorthand as [`PartKind::wall`], and for the same reason - most of
    /// the bench wants exactly this, and the two places that care about framing
    /// should be the ones that have to say so.
    pub const fn gable(long: f32, pitch: f32) -> PartKind {
        PartKind::Gable {
            long,
            pitch,
            framed: false,
        }
    }

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
    ///
    /// Unused by the bench itself since the framed-wall shelf entries went: a hole punched
    /// into a wall carries the width the OPENING needs, which is the whole of how a double
    /// door came to frame a hole its leaves fit through. Kept because it is what a dozen
    /// tests mean by "an ordinary door", and because the next thing that wants a hole at
    /// its usual width will want exactly this.
    #[allow(dead_code)]
    pub fn plain(what: Opening, at: f32) -> Hole {
        Hole::usual(what, at, (WALL_HIGH / ATOM).round() as i32)
    }

    /// An opening at the width and band its kind takes in a wall this tall.
    ///
    /// The one place a hole is made from nothing but its kind, and the reason
    /// the wall's own courses still mean something: they are what a hole is
    /// GIVEN when nobody has said, rather than what it IS ever after.
    pub fn usual(what: Opening, at: f32, tall: i32) -> Hole {
        let band = band_of(what, tall);
        Hole {
            what,
            at,
            wide: usual_width(what),
            dark: false,
            high: band.rise,
            lift: band.foot,
        }
    }
}

/// The size of window the shelf hands out, in panes.
///
/// A maker sizes a window once and then places six more like it. Without this,
/// the shelf hands back a two-by-two every time and the whole townhall is sized
/// window by window through the menu - so the LAST size chosen is what WINDOW
/// means until it is chosen again.
///
/// Not where the size LIVES: that is on the part, and on the hole it punches.
/// This is only what the shelf reaches for, the way a brush remembers the colour
/// it was last dipped in.
#[derive(Resource, Clone, Copy)]
pub struct WindowPanes {
    pub across: i32,
    pub up: i32,
}

impl Default for WindowPanes {
    fn default() -> Self {
        WindowPanes { across: 2, up: 2 }
    }
}

impl WindowPanes {
    /// The window this many panes makes.
    pub fn window(self) -> PartKind {
        PartKind::Window {
            wide: panes_across(self.across),
            high: panes_across(self.up),
        }
    }
}

/// The kind a shelf button actually hands over.
///
/// Every entry is itself, except a WINDOW: the shelf's own is the two-by-two a
/// maker starts from, and what they want back is the one they last sized. Asked
/// in ONE place, because the button that arms the hand and the border that shows
/// which button is armed have to agree about what the button means - and a
/// border that disagreed would simply stop lighting up the moment a maker
/// resized a window.
pub fn from_the_shelf(kind: PartKind, panes: WindowPanes) -> PartKind {
    match kind {
        PartKind::Window { .. } => panes.window(),
        other => other,
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
    /// A WINDOW, of a size a maker chose: how wide and how tall, in atoms,
    /// always a whole number of panes each way.
    ///
    /// It was `Prop("window")` - one window, forever the same, taking its height
    /// from whatever wall it was aimed at and its panes from that. So a hall
    /// wanting three-pane windows over its door could not have them, and a
    /// cottage and a townhall were glazed to the same two-by-two whatever they
    /// were built of.
    ///
    /// The size is on the PART, so the thing in your hand is the size you are
    /// about to place, the ghost draws it, and picking one up brings its size
    /// back to hand. See [`panes_across`](super::panes_across).
    Window {
        wide: i32,
        high: i32,
    },
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
    Gable {
        long: f32,
        pitch: f32,
        /// Half-timbered: rake plates up both slopes, a plate along its foot and
        /// studs standing between, with the plaster set back behind them.
        ///
        /// A PROPERTY, like a wall's. Brett said so when framing became a
        /// right-click: "Walls should just have a right click to add the framing,
        /// same with gables." A framed gable is the same triangle with a
        /// different infill, not another kind of thing.
        framed: bool,
    },
    /// The same timber STOOD UP: a post on its own foot, as tall as it is drawn.
    ///
    /// Brett: "pole should be exactly like the beam only verticle" - and then, of
    /// the fixed-height corner post it replaces, "pole, corner is obsolete once we
    /// make the new pole." It was a prop: one height, forever, with no handle on
    /// it, so a post for a two-metre wall and a post for a tower were the same
    /// part and neither could be made.
    ///
    /// It grows from its FOOT, on the gold handle a foundation and a wall already
    /// wear, because that is what standing on something means.
    Pole(f32),
    /// A squared timber laid along its own length: the corner post's section,
    /// on its side and as long as it is drawn — and how far each end is cut
    /// back at an angle, nought for a square end.
    ///
    /// The cut is a RUN, not an angle: how far along the beam the saw travels
    /// while crossing its full height. That is the number the roof hands over —
    /// the difference between where the top corner meets the slope and where the
    /// bottom does — and it needs no trigonometry at either end.
    Beam(f32, f32, f32),
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
    /// A CEILING: the flat top of a room, and the roof it will raise.
    ///
    /// Its own part rather than a floor, though the slab is the same. Brett: "I think I
    /// would prefer to have a ceiling part separate... it could hold the roof type
    /// automatically and floors having generate roof is weird."
    ///
    /// Both halves of that are right. A ground floor being asked to raise a roof is a
    /// question about a thing that has no roof over it - and, more than tidiness, a
    /// ceiling REMEMBERS which roof it makes, which a floor has nowhere to put. So the
    /// kind is chosen on the ceiling, while it is still a rectangle a maker can drag
    /// about, and generating it is one press with nothing left to decide.
    Ceiling {
        long: f32,
        deep: f32,
        /// Hipped rather than gabled: slopes on all four sides.
        hipped: bool,
        /// The ridge laid ACROSS the short way instead of along the long one.
        ///
        /// A ridge runs the long side of a building nearly always, so that is the default
        /// and this is the exception - a cross wing whose gable faces the street wants its
        /// ridge the other way, and is the whole reason the choice exists.
        ///
        /// R flips this rather than turning the ceiling. Brett: "pressing R should only
        /// change the direction of the ridge beam, not rotate the ceiling itself" - and a
        /// rectangle spun a quarter is the same rectangle, so the key had nothing else
        /// worth doing.
        across: bool,
    },
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
    RidgeRun,
    /// A hip roof: four faces sloping in from the eaves to a flat deck.
    ///
    /// The same four numbers a gable roof takes - how long, how far across, how
    /// far the eaves reach past the walls, and the pitch - because it is the
    /// same roof with its ends hipped in rather than closed by a gable.
    HipRoof(f32, f32, f32, f32),
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
            | PartKind::RidgeRun => Some(1),
            _ => None,
        }
    }

    /// What a run becomes at the drawn size.
    ///
    /// The depth goes unread at the moment: every run left stretches along ONE axis, now
    /// that the whole roofs and the rectangles are raised from a ceiling or placed
    /// ready-made and pulled. The parameter stays because the two-axis path it belongs to
    /// still stands - a run that stretches both ways is a thing this could have again.
    pub fn run_made(&self, w: f32, _d: f32) -> PartKind {
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
            PartKind::RidgeRun => PartKind::Ridge(w),
            PartKind::SegRun { high, lift } => PartKind::Seg {
                long: w,
                high: *high,
                lift: *lift,
            },
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
    // PLACED WHOLE, then pulled - as the wall and the gable are. A beam was the
    // last of the stretch stubs a maker meets first, and the two-click gesture it
    // needed is not the one anything else on this shelf uses.
    structure("BEAM", PartKind::Beam(2.0, 0.0, 0.0), "frame"),
    structure(
        "CEILING",
        PartKind::Ceiling {
            long: 4.0,
            deep: 4.0,
            hipped: false,
            across: false,
        },
        "roof",
    ),
    structure("DOOR", PartKind::Prop("door"), "walls"),
    structure("DOOR, DOUBLE", PartKind::Prop("door-double"), "walls"),
    structure("DOORWAY", PartKind::Prop("doorway"), "walls"),
    structure("FLOOR", PartKind::Floor(2.0, 2.0), "footing"),
    structure(
        "FOUNDATION",
        PartKind::Foundation(2.0, 2.0, STEP_UP),
        "footing",
    ),
    // PLACED WHOLE, then pulled - the way a wall is. It was a run: a stub of a
    // ghost you set one end of and dragged the other out from, which is a
    // different gesture from everything else that has a length, and the one
    // Brett wanted rid of first: "the gables use the old stretch idea where you
    // have a tiny little ghost and stretch it. I like how the wall works where
    // you have say a 2m wall and you place it and then stretch it."
    structure("GABLE", PartKind::gable(2.0, ROOF_PITCH_DEGREES), "roof"),
    structure(
        "HEADER",
        PartKind::SegRun {
            high: 0.375,
            lift: 2.125,
        },
        "walls",
    ),
    structure("POLE", PartKind::Pole(WALL_HIGH), "frame"),
    structure("RAIL", PartKind::RailRun { stone: false }, "frame"),
    structure("RIDGE", PartKind::RidgeRun, "roof"),
    structure("ROOF", PartKind::Roof(2.2, 2.2), "roof"),
    structure(
        "SILL",
        PartKind::SegRun {
            high: 0.75,
            lift: 0.0,
        },
        "walls",
    ),
    structure(
        "STAIRS",
        PartKind::Stairs {
            rise: STEP_UP,
            wide: 1.25,
            stone: false,
            rail_stone: false,
            hand: RAIL_HIGH,
        },
        "footing",
    ),
    structure("TRIM", PartKind::TrimRun { stone: false }, "walls"),
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
    // The size the shelf hands out is only where a maker STARTS: two panes by
    // two, the ordinary cottage window. What they hand back is whatever they
    // last chose - see `WindowPanes`.
    structure(
        "WINDOW",
        PartKind::Window {
            wide: WINDOW_WIDE,
            high: WINDOW_WIDE,
        },
        "walls",
    ),
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
