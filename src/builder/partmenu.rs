//! A part on its own terms: the menu a right-click raises, and the deeds on it.

use super::*;

/// Something a right-click can do to the part under the cursor.
///
/// One entry today. The menu exists as much for the ones after it — Brett:
/// "that way we could add other things to the menu later" — and a menu is the
/// right home for the deeds that are neither a tool nor a key: rare enough not
/// to earn a letter, specific enough to belong to one part.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Deed {
    /// Break a made part into the parts it is made of.
    Ungroup,
    /// Say what a part IS - which decides what comes off with the roof, what
    /// the walls are made of, and what the village can walk through.
    Nature(&'static str),
    /// Bring a part back until it stops at the roof instead of coming through
    /// it.
    TrimToRoof,
    /// Make one thing of everything chosen.
    Group,
    /// What a flight is made of: its treads, and its rail.
    ///
    /// One deed for the pair rather than a toggle each. "Stone treads with a timber rail"
    /// is a thing a maker pictures whole - asking them to reach it by flipping two
    /// switches is asking them to hold the other one in their head while they press this
    /// one.
    StairsOf {
        treads_stone: bool,
        rail_stone: bool,
    },
    /// A rail or a trim in the other material.
    MadeOfStone(bool),
    /// What the village BUILDS this part out of - the game's word, not a colour.
    BuiltOf(&'static str),
    /// Teach this project a material it does not know yet.
    ANewMaterial,
    /// Paint the bars of this wall's windows dark, or leave them as timber.
    BarsIn(bool),
    /// How many panes this wall's windows are divided into, one way or the other.
    ///
    /// In PANES rather than in metres, because that is how a maker counts a window -
    /// Brett: "The three high two wide windows with the black crossbars need to be
    /// made" - and because a size in metres would land between two panes and round
    /// to whichever it happened to be nearer.
    Panes {
        /// Up the wall rather than across it.
        up: bool,
        count: i32,
    },
    /// Frame a wall, or take the framing off it again.
    Frame(bool),
    /// Raise a roof over a ceiling, sized to it - and say WHICH in the same press.
    ///
    /// It was two lines: GENERATE ROOF, and a toggle beside it that read as a
    /// second way to generate one and was really a setting. So a maker wanting a
    /// hip pressed the line that said "A HIPPED ROOF" and got a shorter ridge
    /// beam and no roof. Brett: "Generate roof should have a sub menu for gable
    /// or hipped... Its just confusing."
    ///
    /// The ceiling still carries which kind it wears, because it wears the RIDGE
    /// of that kind and a maker reads it before pressing anything. This sets it
    /// and raises it together, so the two can never disagree.
    RoofOf { hipped: bool },
    /// Open a drawer of the menu's own, alongside the line that names it.
    ///
    /// The menu answers several different questions about a part - what it IS, what it is
    /// made of, what may be done to it - and asking all of them at once made a list eleven
    /// lines long where five were the same question. Brett: "the right click menu could
    /// use sub catagories too. Like for the Part Of stuff."
    More(&'static str),
    /// Keep everything chosen as a PIECE, to bring into other works.
    ///
    /// Brett: "if I could save groups that I could bring into other builds."
    /// A group is one thing within a work; a piece is that same thing kept as a
    /// file, so a porch drawn on a longhouse can be set down on a tavern.
    KeepAsPiece,
}

impl Deed {
    /// Whether this line describes what the part already IS, so the menu can mark it.
    ///
    /// One place for the question, because two menus ask it - the menu itself and every
    /// drawer - and a drawer that marked nothing would leave a maker guessing which of
    /// four looks their stairs are wearing.
    pub(crate) fn is_standing(self, kind: &PartKind, wearing: &str, material: &str) -> bool {
        match self {
            Deed::Nature(nature) => nature == wearing,
            Deed::BuiltOf(word) => word == material,
            Deed::StairsOf {
                treads_stone,
                rail_stone,
            } => matches!(
                kind,
                PartKind::Stairs { stone, rail_stone: rail, .. }
                    if *stone == treads_stone && *rail == rail_stone
            ),
            // The window it would make, against the window that is there. Read off
            // the hole rather than off any number kept beside it, so a window
            // resized any other way still marks the line it is standing on.
            Deed::Panes { up, count } => match kind {
                PartKind::Wall { openings, .. } => openings
                    .iter()
                    .flatten()
                    .find(|hole| hole.what == Opening::Window)
                    .is_some_and(|hole| panes_in(if up { hole.high } else { hole.wide }) == count),
                PartKind::Window { wide, high } => {
                    panes_in(if up { *high } else { *wide }) == count
                }
                _ => false,
            },
            // Which ridge the ceiling is standing there wearing. Both lines are
            // actions, and the mark says which of the two the beam in front of the
            // maker is promising.
            Deed::RoofOf { hipped } => {
                matches!(kind, PartKind::Ceiling { hipped: wears, .. } if *wears == hipped)
            }
            _ => false,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Deed::Ungroup => "UNGROUP",
            // Short, because the drawer they hang in is already called PART OF.
            Deed::Nature("roof") => "THE ROOF",
            Deed::Nature("walls") => "THE WALLS",
            Deed::Nature("frame") => "THE FRAME",
            Deed::Nature("footing") => "THE FOOTING",
            Deed::Nature(_) => "FURNISHING",
            Deed::TrimToRoof => "TRIM TO THE ROOF",
            Deed::Group => "GROUP",
            // Short, because the drawer they hang in already says MADE OF.
            Deed::StairsOf {
                treads_stone: false,
                rail_stone: false,
            } => "TIMBER",
            Deed::StairsOf {
                treads_stone: true,
                rail_stone: true,
            } => "STONE",
            Deed::StairsOf {
                treads_stone: true,
                rail_stone: false,
            } => "STONE, TIMBER RAIL",
            Deed::StairsOf {
                treads_stone: false,
                rail_stone: true,
            } => "TIMBER, STONE RAIL",
            Deed::MadeOfStone(true) => "IN STONE",
            Deed::MadeOfStone(false) => "IN TIMBER",
            Deed::BuiltOf(word) => word,
            Deed::ANewMaterial => "+ ANOTHER...",
            // Just the number: the drawer they hang in says which way.
            Deed::Panes { count: 1, .. } => "1",
            Deed::Panes { count: 2, .. } => "2",
            Deed::Panes { count: 3, .. } => "3",
            Deed::Panes { .. } => "4",
            Deed::BarsIn(true) => "BARS IN BLACK",
            Deed::BarsIn(false) => "BARS IN TIMBER",
            Deed::Frame(true) => "ADD FRAMING",
            Deed::Frame(false) => "REMOVE FRAMING",
            // Short, because the drawer they hang in already says GENERATE ROOF.
            Deed::RoofOf { hipped: true } => "A HIPPED ROOF",
            Deed::RoofOf { hipped: false } => "A GABLE ROOF",
            Deed::More(group) => group,
            Deed::KeepAsPiece => "KEEP AS A PIECE",
        }
    }
}

/// Brings a part back along its own length until it stops at the roof.
///
/// Brett wanted the option rather than the rule, and was right to: "Somethings
/// like Chimneys I would want to clip through, thats why I would like an
/// option." A chimney's whole purpose is to come out the other side.
///
/// It moves the GEOMETRY rather than hiding pixels, and that is the whole
/// constraint. The bench's promise is that what is saved is what the village
/// raises, and the bake emits plain boxes - so a beam clipped only in the
/// picture would sit right here and stand proud of the roof out in the world,
/// which is a worse bug than the one being fixed because it cannot be seen from
/// the bench at all.
///
/// Each end is cast at the roof separately and only the end that meets one comes
/// in. The other stays exactly where the maker put it, because a part that
/// shortened from both ends would walk out of the joint it was seated in.
///
/// The end stays SQUARE. A mitre is what a carpenter would cut, and a mitre is
/// not a box - the baked format speaks boxes, wedges and ridges, and nothing
/// that is a box with one corner off. At a steep pitch that leaves a gap about
/// the beam's own thickness, which in a world built of boxes reads as ordinary
/// framing.
pub(crate) fn trim_to_roof(
    kind: &PartKind,
    record: &Placed,
    roofs: &[(Vec3, Vec3, Quat)],
) -> Option<(PartKind, Placed)> {
    let (long, rebuild) = length_of(kind)?;
    let spin = pose(record.yaw, record.tilt, record.flip);
    let along = spin * Vec3::X;
    let at = Vec3::from(record.at);
    let half = long * 0.5;

    // How far each end may reach before it is inside the roof.
    // A box the part's MIDDLE already lies inside is a part it is SEATED IN,
    // not one it is coming through, and trimming to it would be nonsense: a tie
    // beam across a gable end sits in that gable by design. Left in, such a box
    // reports a hit at no distance at all in both directions, the trim comes out
    // as nothing, and the deed silently declines - which is exactly what Brett
    // saw. His beam sits in its gable and pokes out through a slope; the slope
    // was two and three quarter metres along the very ray that had already
    // given up.
    let seated: Vec<&(Vec3, Vec3, Quat)> = roofs
        .iter()
        .filter(|(box_at, box_half, box_turn)| !point_in_box(at, *box_at, *box_half, *box_turn))
        .collect();

    // The part's own cross-section, so its CORNERS are cast and not its centre
    // line. A square beam meeting a slanted roof touches with a corner first, so
    // trimming to where the middle meets the slope leaves that corner standing
    // proud of it - a small stub above the roof, which is what Brett
    // photographed after the first trim worked.
    let mut low = Vec3::splat(f32::INFINITY);
    let mut high = Vec3::splat(f32::NEG_INFINITY);
    for Slab {
        at: offset, size, ..
    } in body_of(kind, None)
    {
        low = low.min(offset - size * 0.5);
        high = high.max(offset + size * 0.5);
    }
    let corners = [
        Vec3::new(0.0, low.y, low.z),
        Vec3::new(0.0, low.y, high.z),
        Vec3::new(0.0, high.y, low.z),
        Vec3::new(0.0, high.y, high.z),
    ];

    // Every corner is cast, and BOTH the nearest and the furthest are kept. The
    // near one is where the saw goes in and the far one is where it comes out -
    // and the difference between them is the cut. A square end can only stop at
    // the near one, which is why it stood off the slope by the beam's own
    // thickness; Brett: "get the angle of the roof and cut the beam at that
    // angle and delete the escess".
    let mut nearest = [half, half];
    let mut furthest = [0.0_f32, 0.0];
    let mut met = [false, false];
    for (end, way) in [(0usize, 1.0_f32), (1, -1.0)] {
        for corner in corners {
            let from = at + spin * corner;
            for (box_at, box_half, box_turn) in &seated {
                if let Some(hit) =
                    ray_meets_box(from, along * way, half, *box_at, *box_half, *box_turn)
                {
                    nearest[end] = nearest[end].min(hit);
                    furthest[end] = furthest[end].max(hit);
                    met[end] = true;
                }
            }
        }
        if !met[end] {
            furthest[end] = half;
        }
    }
    if !met[0] && !met[1] {
        return None;
    }
    // The beam reaches as far as the LAST corner to meet the roof; the cut takes
    // back the wedge between that and the first.
    let reach = furthest;
    let cut = [
        (furthest[0] - nearest[0]).max(0.0),
        (furthest[1] - nearest[1]).max(0.0),
    ];
    let trimmed = reach[0] + reach[1];
    if trimmed < 0.125 {
        return None;
    }
    let middle = at + along * (reach[0] - reach[1]) * 0.5;
    let lattice = |n: f32| (n * 16.0).round() / 16.0;
    // What this end was ALREADY cut back to, if a saw has been here before.
    let (had_high, had_low) = match *kind {
        PartKind::Beam(_, high, low) => (high, low),
        _ => (0.0, 0.0),
    };
    let made = match rebuild(lattice(trimmed).max(0.125)) {
        // A beam takes its cuts; anything else can only be shortened, since
        // nothing else has an end to cut.
        //
        // And it KEEPS the ones it has. The cast reads a beam's square envelope
        // - the box its slabs fill - so a beam already mitred to a slope reads
        // as meeting that slope everywhere at once, the wedge between first
        // corner and last comes out as nothing, and a second trim handed the
        // beam back with square ends at the same length it already had. Which
        // is exactly what a maker sees as the saw work being undone: Brett,
        // after tagging a trimmed beam as part of the roof, "it loses its trim
        // to roof angles". Taking the deeper of the two can only ever cut more.
        PartKind::Beam(long, ..) => PartKind::Beam(
            long,
            lattice(cut[0]).max(had_high),
            lattice(cut[1]).max(had_low),
        ),
        other => other,
    };
    let mut moved = record.clone();
    moved.part = part_name(&made);
    moved.at = middle.into();
    Some((made, moved))
}

/// The length a part is drawn to, if it has one, and the kind it becomes at
/// another length.
///
/// The `long` family: everything a maker draws by pulling it out to size along
/// its own X. Trimming is only meaningful for these — a part with no length has
/// no direction to come back along.
pub(crate) fn length_of(kind: &PartKind) -> Option<(f32, Box<dyn Fn(f32) -> PartKind>)> {
    match *kind {
        PartKind::Beam(long, high, low) => {
            Some((long, Box::new(move |n| PartKind::Beam(n, high, low))))
        }
        // A wall pulled longer keeps its height, its framing and its openings. A framed
        // one GAINS A BAY doing it; a plain one simply gets longer.
        PartKind::Wall {
            long,
            high,
            framed,
            openings,
        } => Some((
            long,
            Box::new(move |n| PartKind::Wall {
                long: n,
                high,
                framed,
                openings,
            }),
        )),
        PartKind::Ridge(long) => Some((long, Box::new(PartKind::Ridge))),
        // Everything else a maker can size along its own length. Brett: "Trim to
        // roof needs to be added to all the parts in the bench" - and the three
        // above were only the three that had come up. A part that can be made
        // shorter can be trimmed; a part that cannot has nothing for a saw to
        // take, which is every prop and every widget.
        PartKind::Seg { long, high, lift } => Some((
            long,
            Box::new(move |n| PartKind::Seg {
                long: n,
                high,
                lift,
            }),
        )),
        PartKind::Trim { long, stone } => {
            Some((long, Box::new(move |n| PartKind::Trim { long: n, stone })))
        }
        PartKind::Rail { long, hand, stone } => Some((
            long,
            Box::new(move |n| PartKind::Rail {
                long: n,
                hand,
                stone,
            }),
        )),
        PartKind::Gable {
            long,
            pitch,
            framed,
        } => Some((
            long,
            Box::new(move |n| PartKind::Gable {
                long: n,
                pitch,
                framed,
            }),
        )),
        PartKind::GableRoof(long, span, over, pitch) => Some((
            long,
            Box::new(move |n| PartKind::GableRoof(n, span, over, pitch)),
        )),
        PartKind::HipRoof(long, span, over, pitch, deck) => Some((
            long,
            Box::new(move |n| PartKind::HipRoof(n, span, over, pitch, deck)),
        )),
        // The flat ones are sized in two, and it is their X a trim comes back
        // along - the same axis every other part is cut on.
        PartKind::Ceiling {
            deep,
            hipped,
            across,
            ..
        } => Some((
            0.0,
            Box::new(move |n| PartKind::Ceiling {
                long: n,
                deep,
                hipped,
                across,
            }),
        )),
        PartKind::Floor(_, deep) => Some((
            body_of(kind, None)
                .iter()
                .map(|Slab { at, size, .. }| at.x.abs() + size.x * 0.5)
                .fold(0.0_f32, f32::max)
                * 2.0,
            Box::new(move |n| PartKind::Floor(n, deep)),
        )),
        PartKind::Foundation(_, deep, high) => Some((
            body_of(kind, None)
                .iter()
                .map(|Slab { at, size, .. }| at.x.abs() + size.x * 0.5)
                .fold(0.0_f32, f32::max)
                * 2.0,
            Box::new(move |n| PartKind::Foundation(n, deep, high)),
        )),
        PartKind::Roof(_, deep) => Some((
            body_of(kind, None)
                .iter()
                .map(|Slab { at, size, .. }| at.x.abs() + size.x * 0.5)
                .fold(0.0_f32, f32::max)
                * 2.0,
            Box::new(move |n| PartKind::Roof(n, deep)),
        )),
        _ => None,
    }
}

/// Whether a point is inside an oriented box.
pub(crate) fn point_in_box(p: Vec3, box_at: Vec3, box_half: Vec3, box_turn: Quat) -> bool {
    let local = box_turn.inverse() * (p - box_at);
    (0..3).all(|axis| local[axis].abs() <= box_half[axis])
}

/// Where a ray first meets an oriented box, if it does within `reach`.
///
/// The slab method, in the box's own frame - which is why the box may be turned
/// any way at all, and a roof's slopes are turned every way there is.
pub(crate) fn ray_meets_box(
    from: Vec3,
    along: Vec3,
    reach: f32,
    box_at: Vec3,
    box_half: Vec3,
    box_turn: Quat,
) -> Option<f32> {
    let inverse = box_turn.inverse();
    let origin = inverse * (from - box_at);
    let heading = inverse * along;
    let (mut near, mut far) = (0.0_f32, reach);
    for axis in 0..3 {
        let (o, d, h) = (origin[axis], heading[axis], box_half[axis]);
        if d.abs() < 1e-6 {
            if o.abs() > h {
                return None;
            }
            continue;
        }
        let (mut t0, mut t1) = ((-h - o) / d, (h - o) / d);
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }
        near = near.max(t0);
        far = far.min(t1);
        if near > far {
            return None;
        }
    }
    (near <= far && near >= 0.0).then_some(near)
}

/// The natures a maker can hand a part, in the order they are offered.
///
/// Brett's idea, and it takes a rule the bench was enforcing by TYPE and hands
/// it to the maker: "we could tag an item as a roof peice... then I dont need
/// special peices for the roof...any peice could be a roof piece." A plank laid
/// as a canopy is a roof, and until now only the parts the bench thought of as
/// roofs came off when H was pressed.
///
/// Nothing new had to be stored for it. A part already carries what it is — the
/// cutaway reads it, the game reads it to learn a building's wall and roof
/// cloth, and the walkable shell is what stands on `walls` or `frame`. It was
/// simply never something a maker could change after placing.
pub(crate) const NATURES: [&str; 5] = ["footing", "frame", "walls", "roof", "furnishing"];

/// What may be done to this part.
///
/// Ungrouping is offered only where the pieces have somewhere to GO. A gable
/// roof is two slopes and two ends, and the bench already has a part for each -
/// a roof panel and a gable - so breaking one up leaves four parts a maker can
/// go on working with. A door is jambs and a leaf and a latch, and the bench has
/// no part for a jamb, so breaking one up would leave nothing but a hole where a
/// door used to be.
pub(crate) fn deeds_for(kind: &PartKind) -> Vec<Deed> {
    // Only what the KIND can answer. Whether a part is also carrying marks is a
    // question about this bench rather than about the kind, so the menu asks
    // that one itself and adds the entry if the answer is yes - an UNGROUP that
    // would do nothing is worse than no UNGROUP, because it has to be tried
    // before a maker learns it is nothing.
    let mut deeds = match kind {
        PartKind::GableRoof(..) => vec![Deed::Ungroup],
        _ => Vec::new(),
    };
    // A CEILING takes a roof of its own size. Brett: "placing a roof is a PIA. Lets make
    // the way you place a roof the same as the way you place a floor... you right click it
    // to generate roof" - and "they should keep their ceiling too", so the ceiling stays
    // and the roof joins it.
    //
    // Offered on a floor, because a floor laid at the top of a wall IS the ceiling. One
    // flat part, and a line that puts a roof over it.
    if matches!(kind, PartKind::Ceiling { .. }) {
        deeds.push(Deed::More(ROOF_OVER));
    }
    // A wall can be framed or plain, offered as the thing it would BECOME. Brett:
    // "Walls should just have a right click to add the framing." It goes first because it
    // changes what the wall IS, where everything under it only changes how it looks.
    // A GABLE the same way, which is what Brett asked for in the same breath: "Walls
    // should just have a right click to add the framing, same with gables."
    if let PartKind::Wall { framed, .. } | PartKind::Gable { framed, .. } = kind {
        deeds.push(Deed::Frame(!framed));
    }
    // A flight's materials, in a drawer of their own: four looks rather than two toggles.
    if matches!(kind, PartKind::Stairs { .. }) {
        deeds.push(Deed::More(MADE_OF));
    }
    // And the parts that are simply one material or the other, offered as the thing they
    // would BECOME.
    if let PartKind::Rail { stone, .. } | PartKind::Trim { stone, .. } = kind {
        deeds.push(Deed::MadeOfStone(!stone));
    }
    // A framed wall's window bars, the same way - and only when the wall HAS a window,
    // since a line that would do nothing is worse than no line at all: it has to be tried
    // before a maker learns it is nothing.
    if let PartKind::Wall { openings, .. } = kind
        && let Some(window) = openings
            .iter()
            .flatten()
            .find(|hole| hole.what == Opening::Window)
    {
        deeds.push(Deed::BarsIn(!window.dark));
        // And how big it is, in panes, each way. Two drawers rather than a list of
        // shapes: a maker who wants one more pane up should not have to find the
        // line that says the width they already have.
        deeds.push(Deed::More(PANES_ACROSS));
        deeds.push(Deed::More(PANES_UP));
    }
    // Every part can be told what it is, and what it is made of - each behind one line.
    deeds.push(Deed::More(PART_OF));
    deeds.push(Deed::More(BUILT_OF));
    deeds
}

/// The drawer that says what a part is BUILT of.
///
/// Every part takes one, the way every part takes a nature: what a thing is made of is a
/// fact about it, not about which shelf it came off.
pub(crate) const BUILT_OF: &str = "MADE OF...";

/// The drawer a flight's materials hang in.
pub(crate) const MADE_OF: &str = "STONE OR TIMBER...";

/// The drawer the roofs hang in.
pub(crate) const ROOF_OVER: &str = "GENERATE ROOF...";

/// The two drawers a window's size hangs in, counted in panes.
pub(crate) const PANES_ACROSS: &str = "PANES ACROSS...";
pub(crate) const PANES_UP: &str = "PANES UP...";

/// What the PART OF drawer holds.
pub(crate) const PART_OF: &str = "PART OF...";

pub(crate) fn deeds_in(group: &str) -> Vec<Deed> {
    match group {
        PART_OF => NATURES.iter().map(|nature| Deed::Nature(nature)).collect(),
        ROOF_OVER => vec![
            Deed::RoofOf { hipped: false },
            Deed::RoofOf { hipped: true },
        ],
        PANES_ACROSS => (1..=MOST_PANES)
            .map(|count| Deed::Panes { up: false, count })
            .collect(),
        PANES_UP => (1..=MOST_PANES)
            .map(|count| Deed::Panes { up: true, count })
            .collect(),
        // The project's own words, and a way to add one. Brett: "We can have the basic
        // stuff like stone, wood, clay, and then there could be a plus where I could type
        // my own."
        BUILT_OF => {
            let mut deeds: Vec<Deed> = crate::project::materials()
                .into_iter()
                .map(|word| Deed::BuiltOf(crate::project::a_kept_word(&word)))
                .collect();
            deeds.push(Deed::ANewMaterial);
            deeds
        }
        MADE_OF => [(false, false), (true, true), (true, false), (false, true)]
            .into_iter()
            .map(|(treads_stone, rail_stone)| Deed::StairsOf {
                treads_stone,
                rail_stone,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// The menu itself. Which part raised it rides on each LINE, since that is
/// where it is read - keeping a second copy up here only invited the two to
/// disagree.
#[derive(Component)]
pub(crate) struct PartMenu;

/// A drawer standing open beside the line that names it.
///
/// A CHILD of that line, and placed at `left: 100%`, so the layout engine puts it against
/// the menu's right edge and level with its own line. Working the corner out by hand would
/// mean knowing how wide the menu came out, which depends on the longest word on it.
#[derive(Component)]
pub(crate) struct PartDrawer;

/// One line of it.
#[derive(Component)]
pub(crate) struct MenuLine {
    pub(crate) deed: Deed,
    pub(crate) part: Entity,
}

/// Right-click a part to raise its menu.
///
/// On the RELEASE, and only if the mouse barely moved: the right button already
/// orbits the camera, so a right DRAG has to stay an orbit and only a right
/// CLICK is a menu. Every three-dimensional tool resolves it this way and a
/// maker will not notice the rule at all, which is the point of it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn raise_part_menu(
    mut commands: Commands,
    fonts: Res<Fonts>,
    palette: Res<Palette>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<bevy::input::mouse::MouseMotion>,
    windows: Query<&Window>,
    hovered: Res<Hovered>,
    naming: Res<Naming>,
    hand: Res<Hand>,
    selected: Res<crate::gizmo::Selected>,
    placed: Query<(Entity, &Transform, &Placed), Without<Ghost>>,
    menus: Query<Entity, With<PartMenu>>,
    mut drift: Local<f32>,
) {
    let stirred: f32 = motion.read().map(|moved| moved.delta.length()).sum();
    if buttons.pressed(MouseButton::Right) {
        *drift += stirred;
    }
    if buttons.just_pressed(MouseButton::Right) {
        *drift = 0.0;
    }
    if !buttons.just_released(MouseButton::Right) {
        return;
    }
    let travelled = std::mem::take(&mut *drift);
    for menu in &menus {
        commands.entity(menu).despawn();
    }
    // Four pixels of slop: a hand resting on a mouse is never quite still, and
    // an orbit that happens to end where it started is still an orbit.
    if travelled > 4.0 || naming.0.is_some() || hand.kind.is_some() {
        return;
    }
    let Some(part) = hovered.grab else {
        return;
    };
    let Ok((_, _, record)) = placed.get(part) else {
        return;
    };
    let Some(kind) = kind_from_name(&record.part) else {
        return;
    };
    let mut deeds = deeds_for(&kind);
    // Grouping needs company: one part is already as gathered as it gets.
    if selected.0.len() > 1 {
        deeds.insert(0, Deed::Group);
        deeds.insert(1, Deed::KeepAsPiece);
    }
    // Only where there is a roof to trim to, and a length to trim.
    if length_of(&kind).is_some()
        && placed
            .iter()
            .any(|(_, _, other)| other.stage == "roof" && other.part != record.part)
    {
        deeds.insert(0, Deed::TrimToRoof);
    }
    // Carrying a mark is reason enough to offer it, whatever the part is.
    if !deeds.contains(&Deed::Ungroup)
        && placed
            .get(part)
            .is_ok_and(|(_, _, record)| record.group.is_some())
    {
        deeds.insert(0, Deed::Ungroup);
    }
    if !deeds.contains(&Deed::Ungroup) {
        let held: Vec<(Entity, Vec3, &Placed)> = placed
            .iter()
            .map(|(entity, at, record)| (entity, at.translation, record))
            .collect();
        let carrying = placed.get(part).is_ok_and(|(_, at, _)| {
            !carried_marks(
                part,
                at.translation,
                held.iter().map(|(e, at, record)| (*e, *at, *record)),
            )
            .is_empty()
        });
        if carrying {
            deeds.insert(0, Deed::Ungroup);
        }
    }
    if deeds.is_empty() {
        return;
    }
    let Some(at) = windows
        .iter()
        .next()
        .and_then(|window| window.cursor_position())
    else {
        return;
    };

    hang_the_part_menu(
        &mut commands,
        &fonts,
        &palette,
        at,
        part,
        &record.stage,
        &record.material,
        &Some(kind),
        deeds,
    );
}

/// Hangs a drawer open beside the line that named it.
///
/// A child of that LINE at `left: 100%`, so the layout engine sets it against the menu's
/// right edge and level with the line - no arithmetic, and nothing that has to know how
/// wide the menu came out.
#[allow(clippy::too_many_arguments)]
pub(crate) fn hang_a_drawer(
    commands: &mut Commands,
    fonts: &Fonts,
    palette: &Palette,
    line: Entity,
    part: Entity,
    wearing: &str,
    material: &str,
    kind: &Option<PartKind>,
    deeds: Vec<Deed>,
) {
    let drawer = commands
        .spawn((
            PartDrawer,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(100.0),
                // A hair up, so the drawer's first line sits level with the line that
                // opened it rather than a border below it.
                top: Val::Px(-4.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(3.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme::panel_bg()),
            BorderColor::all(theme::panel_border(palette)),
            // Above the menu it hangs off, which is above everything else.
            GlobalZIndex(61),
            ChildOf(line),
        ))
        .id();
    for deed in deeds {
        let standing = kind
            .as_ref()
            .is_some_and(|kind| deed.is_standing(kind, wearing, material));
        let row = commands
            .spawn((
                MenuLine { deed, part },
                Interaction::default(),
                Node {
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                ChildOf(drawer),
            ))
            .id();
        commands.spawn((
            Text::new(if standing {
                format!("- {}", deed.label())
            } else {
                format!("  {}", deed.label())
            }),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: crate::look::text_at(11.0),
                ..default()
            },
            TextColor(if standing {
                theme::accent(palette)
            } else {
                theme::text_dim(palette)
            }),
            ChildOf(row),
        ));
    }
}

/// Hangs the menu at a point, with these lines on it.
///
/// Its own function because the menu is hung TWICE: once when a maker right-clicks a
/// part, and again when they open one of its drawers - and a drawer that drew itself a
/// second way would be a second menu that happened to look similar.
#[allow(clippy::too_many_arguments)]
pub(crate) fn hang_the_part_menu(
    commands: &mut Commands,
    fonts: &Fonts,
    palette: &Palette,
    at: Vec2,
    part: Entity,
    wearing: &str,
    material: &str,
    kind: &Option<PartKind>,
    deeds: Vec<Deed>,
) {
    let menu = commands
        .spawn((
            PartMenu,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(at.x),
                top: Val::Px(at.y),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(3.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme::panel_bg()),
            BorderColor::all(theme::panel_border(&palette)),
            GlobalZIndex(60),
        ))
        .id();
    for deed in deeds {
        // The nature it already has is marked, so the menu answers "what is
        // this?" as well as offering to change it.
        let standing = kind
            .as_ref()
            .is_some_and(|kind| deed.is_standing(kind, wearing, material));
        let line = commands
            .spawn((
                MenuLine { deed, part },
                Interaction::default(),
                Node {
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                ChildOf(menu),
            ))
            .id();
        commands.spawn((
            Text::new(if standing {
                format!("- {}", deed.label())
            } else {
                format!("  {}", deed.label())
            }),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: crate::look::text_at(11.0),
                ..default()
            },
            TextColor(if standing {
                theme::accent(&palette)
            } else {
                theme::text_dim(&palette)
            }),
            ChildOf(line),
        ));
    }
}

/// Carries a chosen deed out, and shuts the menu on anything else.
#[allow(clippy::too_many_arguments)]
pub(crate) fn work_part_menu(
    mut commands: Commands,
    fonts: Res<Fonts>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    palette: Res<Palette>,
    selected: Res<crate::gizmo::Selected>,
    mut keeping: ResMut<PieceKept>,
    mut wants_naming: ResMut<PieceWantsAName>,
    mut naming: ResMut<Naming>,
    // Bundled: the ceiling is sixteen, and a window's size is the seventeenth
    // thing this menu can change.
    errands: (ResMut<MaterialFor>, ResMut<WindowPanes>),
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut lines: Query<(Entity, &MenuLine, &Interaction, &mut BackgroundColor)>,
    menus: Query<Entity, With<PartMenu>>,
    drawers: Query<Entity, With<PartDrawer>>,
    mut placed: Query<(Entity, &mut Transform, &mut Placed), Without<Ghost>>,
) {
    if menus.is_empty() {
        return;
    }
    let (mut material_for, mut panes) = errands;
    let mut chosen = None;
    let mut pressed_line = None;
    let mut over = false;
    for (entity, line, interaction, mut fill) in &mut lines {
        if *interaction != Interaction::None {
            over = true;
        }
        if *interaction == Interaction::Pressed {
            chosen = Some((line.deed, line.part));
            pressed_line = Some(entity);
        }
        let wanted = BackgroundColor(if *interaction == Interaction::None {
            Color::NONE
        } else {
            theme::accent(&palette).with_alpha(0.18)
        });
        if fill.0 != wanted.0 {
            *fill = wanted;
        }
    }
    // A click anywhere but on the menu closes it, which is what a menu does.
    let dismissed =
        keys.just_pressed(KeyCode::Escape) || (buttons.just_pressed(MouseButton::Left) && !over);
    if chosen.is_none() && !dismissed {
        return;
    }
    match chosen {
        Some((Deed::Ungroup, part)) => {
            // A group first: the commonest thing UNGROUP is asked to undo, now
            // that grouping exists at all.
            let holding = placed
                .get(part)
                .ok()
                .and_then(|(_, _, record)| record.group);
            if let Some(group) = holding {
                let kin: Vec<Entity> = placed
                    .iter()
                    .filter(|(_, _, record)| record.group == Some(group))
                    .map(|(entity, _, _)| entity)
                    .collect();
                for part in kin {
                    if let Ok((_, _, mut record)) = placed.get_mut(part) {
                        record.group = None;
                    }
                }
            }
            // Then the marks it carries, which any part may have; then the
            // part's own pieces, which only some parts can be broken into.
            let held: Vec<(Entity, Vec3, Placed)> = placed
                .iter()
                .map(|(entity, at, record)| (entity, at.translation, record.clone()))
                .collect();
            if let Some((_, owner_at, record)) = held.iter().find(|(e, ..)| *e == part) {
                let carried = carried_marks(
                    part,
                    *owner_at,
                    held.iter().map(|(e, at, record)| (*e, *at, record)),
                );
                for mark in &carried {
                    if let Ok((_, _, mut loosed)) = placed.get_mut(*mark) {
                        loosed.loose = true;
                    }
                }
                if let Some(kind) = kind_from_name(&record.part) {
                    let record = record.clone();
                    ungroup(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &palette,
                        &kind,
                        &record,
                        part,
                    );
                }
            }
        }
        Some((Deed::TrimToRoof, part)) => {
            // Every box the roof is made of, in world space, so the part's own
            // ends can be cast at them.
            let roofs: Vec<(Vec3, Vec3, Quat)> = placed
                .iter()
                .filter(|(other, _, record)| *other != part && record.stage == "roof")
                .filter_map(|(_, at, record)| {
                    kind_from_name(&record.part).map(|kind| (at, record, kind))
                })
                .flat_map(|(at, record, kind)| {
                    let spin = pose(record.yaw, record.tilt, record.flip);
                    body_of(&kind, None)
                        .into_iter()
                        .map(
                            move |Slab {
                                      at: offset,
                                      size,
                                      lean,
                                      cant,
                                      ..
                                  }| {
                                // A slab may lean inside its own part - a roof's
                                // slopes do nothing else - so its turn is the part's
                                // and then its own.
                                let turn = spin
                                    * Quat::from_rotation_z(cant)
                                    * Quat::from_rotation_x(lean);
                                (at.translation + spin * offset, size * 0.5, turn)
                            },
                        )
                        .collect::<Vec<_>>()
                })
                .collect();
            let trimmed = placed.get(part).ok().and_then(|(_, _, record)| {
                kind_from_name(&record.part).and_then(|kind| trim_to_roof(&kind, record, &roofs))
            });
            if let Some((made, moved)) = trimmed
                && let Ok((_, mut transform, mut record)) = placed.get_mut(part)
            {
                transform.translation = Vec3::from(moved.at);
                *record = moved.clone();
                commands.entity(part).despawn_related::<Children>();
                dress_part(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &palette,
                    &made,
                    &moved,
                    part,
                    false,
                );
            }
        }
        Some((Deed::RoofOf { hipped }, part)) => {
            // The ceiling takes the ridge it is about to raise, and is drawn again
            // wearing it: it is not the same shape in the two states - a gabled one
            // stops short at each end to leave room for its gables - and this used to
            // change the record and none of the drawing, so pressing it did nothing a
            // maker could see.
            let ceiling = placed.get_mut(part).ok().and_then(|(_, at, mut record)| {
                let PartKind::Ceiling {
                    long, deep, across, ..
                } = kind_from_name(&record.part)?
                else {
                    return None;
                };
                let made = PartKind::Ceiling {
                    long,
                    deep,
                    hipped,
                    across,
                };
                record.part = part_name(&made);
                Some((at.translation, record.clone(), made, long, deep, across))
            });
            if let Some((stands, ceiling, made, w, d, across)) = ceiling {
                commands.entity(part).despawn_related::<Children>();
                dress_part(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &palette,
                    &made,
                    &ceiling,
                    part,
                    false,
                );
                // The RIDGE RUNS THE LONG WAY, which is what a roof does and what a maker
                // would otherwise turn it by hand to get. A square ceiling picks either.
                // The ridge the ceiling has been WEARING - the long side unless a maker
                // pressed R - so what is raised is what the beam promised.
                let (long, span, turn) = if (w >= d) != across {
                    (w, d, 0.0)
                } else {
                    (d, w, std::f32::consts::FRAC_PI_2)
                };
                let raised = if hipped {
                    PartKind::HipRoof(long, span, ROOF_OVERHANG, ROOF_PITCH_DEGREES, HIP_DECK)
                } else {
                    PartKind::GableRoof(long, span, ROOF_OVERHANG, ROOF_PITCH_DEGREES)
                };
                let mut roof = ceiling.clone();
                roof.part = part_name(&raised);
                // THE SAME SEAT THE CEILING HAS, which is the wall top - not the ceiling's
                // top. Brett: "the gable should be flush with the edge of the ceiling and
                // replacing its atoms so that the gable sits flush ontop of the wall."
                //
                // A roof's eaves rest at its own nought, so lifting it by the ceiling's
                // thickness stood the gable a slab's depth above the wall it belongs on,
                // with the ceiling wedged between them. Rafters and ceiling joists share a
                // wall plate in a real building, and they share it here: the gable's first
                // atoms stand in the same band the ceiling's do, and replace them.
                roof.at = [stands.x, stands.y, stands.z];
                roof.yaw = ceiling.yaw + turn;
                roof.stage = "roof".to_string();
                spawn_part(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &palette,
                    &raised,
                    &roof,
                    false,
                );
                // NOT GROUPED, though the first cut of this grouped them. A part chosen
                // with others can only be moved, so a grouped roof wore no handles of its
                // own, and its UNGROUP spent its one press parting it from the ceiling
                // instead of breaking it into slopes and gables. Brett wanted both back.
                //
                // The ceiling still stands where it was, which is what keeping it meant.
                // Two parts that want to travel together can be gathered and grouped by
                // hand, which is what that pair of commands is for.
            }
        }
        Some((Deed::Frame(framed), part)) => {
            if let Ok((_, _, mut record)) = placed.get_mut(part)
                && let Some(made) = kind_from_name(&record.part).and_then(|kind| match kind {
                    // Its length, its height and its openings all stay. Framing decides
                    // how the wall is FILLED - bays and studs, or plain plaster - and a
                    // maker turning one into the other has not moved a door.
                    PartKind::Wall {
                        long,
                        high,
                        openings,
                        ..
                    } => Some(PartKind::Wall {
                        long,
                        high,
                        framed,
                        openings,
                    }),
                    // A gable keeps its length and its pitch, which are the two numbers
                    // that make it fit the roof it closes.
                    PartKind::Gable { long, pitch, .. } => Some(PartKind::Gable {
                        long,
                        pitch,
                        framed,
                    }),
                    _ => None,
                })
            {
                record.part = part_name(&made);
                let copy = record.clone();
                commands.entity(part).despawn_related::<Children>();
                dress_part(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &palette,
                    &made,
                    &copy,
                    part,
                    false,
                );
            }
        }
        Some((Deed::BarsIn(dark), part)) => {
            if let Ok((_, _, mut record)) = placed.get_mut(part)
                && let Some(PartKind::Wall {
                    framed,
                    long,
                    high,
                    mut openings,
                }) = kind_from_name(&record.part)
            {
                // EVERY window in the wall, because one wall with two windows in two
                // colours is not a thing anybody means, and the menu acts on the part.
                for hole in openings.iter_mut().flatten() {
                    if hole.what == Opening::Window {
                        hole.dark = dark;
                    }
                }
                let made = PartKind::Wall {
                    long,
                    high,
                    framed,
                    openings,
                };
                record.part = part_name(&made);
                let copy = record.clone();
                commands.entity(part).despawn_related::<Children>();
                dress_part(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &palette,
                    &made,
                    &copy,
                    part,
                    false,
                );
            }
        }
        Some((Deed::Panes { up, count }, part)) => {
            if let Ok((_, _, mut record)) = placed.get_mut(part)
                && let Some(PartKind::Wall {
                    framed,
                    long,
                    high,
                    mut openings,
                }) = kind_from_name(&record.part)
            {
                let atoms = panes_across(count);
                // EVERY window in the wall, the same way BARS IN BLACK paints every
                // one: the menu acts on the part, and a wall with two windows of two
                // sizes is a thing a maker builds deliberately, one wall at a time.
                for hole in openings.iter_mut().flatten() {
                    if hole.what != Opening::Window {
                        continue;
                    }
                    if up {
                        hole.high = atoms;
                    } else {
                        hole.wide = atoms;
                    }
                    // Remembered for the NEXT one. A townhall wants seven windows of
                    // one size, and sizing each of them in turn is six trips through
                    // this menu that a maker should not have to make.
                    *panes = WindowPanes {
                        across: panes_in(hole.wide),
                        up: panes_in(hole.high),
                    };
                }
                let made = PartKind::Wall {
                    long,
                    high,
                    framed,
                    openings,
                };
                record.part = part_name(&made);
                let copy = record.clone();
                commands.entity(part).despawn_related::<Children>();
                dress_part(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &palette,
                    &made,
                    &copy,
                    part,
                    false,
                );
            }
        }
        Some((Deed::BuiltOf(word), part)) => {
            if let Ok((_, _, mut record)) = placed.get_mut(part) {
                // NOTHING IS REBUILT. What a part is built of changes nothing a maker can
                // see - it is a fact carried into the bake, and the mark beside the line is
                // the whole of how it shows. Brett: "The color shouldn't have anything to
                // do with that. That's just what you painted in the palette."
                record.material = word.to_string();
            }
        }
        Some((Deed::ANewMaterial, part)) => {
            // The card the kinds use, for the reason the kinds use it: the word belongs to
            // the game and nothing here can check it. The part waits in `MaterialFor`,
            // since the card has one field and no room for which part asked.
            material_for.0 = Some(part);
            naming.0 = Some(String::new());
            naming.1 = NamingFor::AMaterial;
            raise_naming_card(&mut commands, &fonts, &palette, NamingFor::AMaterial, 0);
            return;
        }
        Some((
            Deed::StairsOf {
                treads_stone,
                rail_stone,
            },
            part,
        )) => {
            if let Ok((_, _, mut record)) = placed.get_mut(part)
                && let Some(PartKind::Stairs {
                    rise, wide, hand, ..
                }) = kind_from_name(&record.part)
            {
                let made = PartKind::Stairs {
                    rise,
                    wide,
                    stone: treads_stone,
                    rail_stone,
                    hand,
                };
                record.part = part_name(&made);
                let copy = record.clone();
                commands.entity(part).despawn_related::<Children>();
                dress_part(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &palette,
                    &made,
                    &copy,
                    part,
                    false,
                );
            }
        }
        // A RAIL or a TRIM is one material or the other, so it takes a line rather than a
        // drawer: two states is a toggle, and only a flight's four looks earned a drawer.
        Some((Deed::MadeOfStone(stone), part)) => {
            if let Ok((_, _, mut record)) = placed.get_mut(part) {
                let made = match kind_from_name(&record.part) {
                    Some(PartKind::Rail { long, hand, .. }) => {
                        Some(PartKind::Rail { long, hand, stone })
                    }
                    Some(PartKind::Trim { long, .. }) => Some(PartKind::Trim { long, stone }),
                    _ => None,
                };
                if let Some(made) = made {
                    record.part = part_name(&made);
                    let copy = record.clone();
                    commands.entity(part).despawn_related::<Children>();
                    dress_part(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &palette,
                        &made,
                        &copy,
                        part,
                        false,
                    );
                }
            }
        }
        Some((Deed::KeepAsPiece, _)) => {
            // The parts themselves ride in the resource; the card only asks
            // what to call them.
            let kept: Vec<Placed> = selected
                .iter()
                .filter_map(|part| placed.get(part).ok())
                .map(|(_, _, record)| record.clone())
                .collect();
            if kept.len() > 1 {
                keeping.0 = kept;
                wants_naming.0 = true;
            }
        }
        Some((Deed::Group, _)) => {
            let together = a_fresh_group(placed.iter().map(|(_, _, record)| record));
            let chosen: Vec<Entity> = selected.iter().collect();
            for part in chosen {
                if let Ok((_, _, mut record)) = placed.get_mut(part) {
                    record.group = Some(together);
                }
            }
        }
        Some((Deed::Nature(nature), part)) => {
            // Nothing to redraw: a part's nature changes what the bench and the
            // village make of it, not what it looks like.
            if let Ok((_, _, mut record)) = placed.get_mut(part) {
                record.stage = nature.to_string();
            }
        }
        // A DRAWER IS NOT A DEED. It stands open beside the line that names it and does
        // not touch the part, where every other line acts and closes. Returning here rather
        // than falling through is the whole of it - the teardown below closes the menu on
        // anything that WAS done.
        Some((Deed::More(group), part)) => {
            // Any drawer already open goes first, so a second press moves the drawer
            // rather than stacking one on another.
            for drawer in &drawers {
                commands.entity(drawer).despawn();
            }
            if let Some(line) = pressed_line {
                let (wearing, material, kind) = placed
                    .get(part)
                    .map(|(_, _, record)| {
                        (
                            record.stage.clone(),
                            record.material.clone(),
                            kind_from_name(&record.part),
                        )
                    })
                    .unwrap_or_default();
                hang_a_drawer(
                    &mut commands,
                    &fonts,
                    &palette,
                    line,
                    part,
                    &wearing,
                    &material,
                    &kind,
                    deeds_in(group),
                );
            }
            return;
        }
        None => {}
    }
    for menu in &menus {
        commands.entity(menu).despawn();
    }
}

/// Breaks a made part into the parts it is made of, standing exactly where its
/// own pieces stood — and GROUPED, so it is still one thing to move.
///
/// Which is where the two kinds of togetherness meet, and Brett asked how they
/// fit: a gable roof is a MADE part, parametric, a span and an overhang and a
/// pitch with handles that change those numbers and re-derive every piece from
/// them. A group is an ASSEMBLY: separate parts, each with its own record, moved
/// as one and edited freely. Neither is a lesser version of the other.
///
/// Ungrouping is the door between them, and it only opens one way. What was a
/// roof becomes four parts that can each be pulled, pitched, painted and
/// deleted - and can no longer be pitched TOGETHER, because the number they
/// shared is gone. Trading parameters for freedom is exactly what a maker is
/// asking for when they break something up, so the trade is the feature. Coming
/// back is what undo is for.
///
/// A gable roof is four things the bench can already draw: two slopes, which are
/// roof panels, and two ends, which are gables. So it comes apart into four
/// parts a maker can pull, pitch, paint and delete one at a time — which is the
/// whole point, and the reason a door cannot do this (see [`deeds_for`]).
///
/// The offsets are read off the same arithmetic that draws the roof, because a
/// piece that lands even slightly off is worse than no ungroup at all: a maker
/// would have to notice, and then correct something they did not move.
///
/// Undo needs no help here. The bench remembers whole states rather than deeds,
/// so four parts appearing and one leaving is one step back like anything else.
#[allow(clippy::too_many_arguments)]
/// What a part comes APART into: the pieces, as records, and nothing placed yet.
///
/// Split from `ungroup` so the answer can be checked without a world to put it in.
/// The deed needs meshes, materials and a palette; what it comes apart into needs
/// none of those, and that is the half worth testing - it is where a piece being
/// born already grouped went unnoticed.
pub(crate) fn pieces_of(kind: &PartKind, record: &Placed) -> Vec<(PartKind, Placed)> {
    let PartKind::GableRoof(long, span, over, degrees) = *kind else {
        return Vec::new();
    };
    let pitch = degrees.clamp(PITCH_LEAST, PITCH_MOST).to_radians();
    let half = span * 0.5;
    let rise = half * pitch.tan();
    let slope = (half * half + rise * rise).sqrt();
    let thick = 0.125;
    let at = Vec3::from(record.at);
    let spin = pose(record.yaw, record.tilt, record.flip);

    let mut born: Vec<(PartKind, Placed)> = Vec::new();
    for way in [-1.0_f32, 1.0] {
        // Where the slope's own slab sits inside the roof.
        let middle = Vec3::new(
            0.0,
            rise * 0.5 + thick * 0.5 - over * 0.5 * pitch.sin(),
            way * (half * 0.5 + over * 0.5 * pitch.cos()),
        );
        // A roof panel is drawn standing ON its origin - its slab is half a
        // thickness up - while the slope's offset is the slab's MIDDLE. So the
        // panel is seated half a thickness back down its own face, which is not
        // straight down once it is pitched.
        let lean = record.tilt + way * pitch;
        let face = pose(record.yaw, lean, record.flip) * Vec3::Y;
        let panel = PartKind::Roof(long + over * 2.0, slope + over);
        born.push((
            panel,
            Placed {
                part: part_name(&panel),
                at: (at + spin * middle - face * (thick * 0.5)).into(),
                yaw: record.yaw,
                tilt: lean,
                ramp: record.ramp.clone(),
                shade: record.shade,
                stage: record.stage.clone(),
                flip: record.flip,
                loose: false,
                material: String::new(),
                group: None,
            },
        ));

        // The ends. A gable is drawn with its width along its own X and the
        // roof's ends stand across the span, so each one is turned a quarter
        // circle to face down the building.
        let gable = PartKind::gable(span, degrees);
        born.push((
            gable,
            Placed {
                part: part_name(&gable),
                at: (at + spin * Vec3::new(way * (long * 0.5 - 0.125), 0.0, 0.0)).into(),
                yaw: record.yaw + std::f32::consts::FRAC_PI_2,
                tilt: record.tilt,
                ramp: record.ramp.clone(),
                shade: record.shade,
                stage: record.stage.clone(),
                flip: record.flip,
                loose: false,
                material: String::new(),
                group: None,
            },
        ));
    }

    born
}

/// Breaks a part into its own pieces, and leaves them FREE of one another.
pub(crate) fn ungroup(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
    kind: &PartKind,
    record: &Placed,
    part: Entity,
) {
    let born = pieces_of(kind, record);
    if born.is_empty() {
        return;
    }
    // FREE, not gathered under one number.
    //
    // They used to come out sharing a group, so a roof that was one thing to move
    // stayed one thing to move - and that quietly denied the deed its own purpose.
    // A click on a grouped part takes the whole group, which is what grouping
    // MEANS, so the pieces could not be painted, tilted or buried one at a time:
    // the only reasons anybody breaks a roof apart. Brett, having done it: "I went
    // to pain the gable and it is painting the entire rood instead of just the
    // gable."
    //
    // Worse, it took TWO presses to get here and the first one looked like nothing
    // had happened - the pieces stand exactly where the slabs stood, so the roof is
    // unchanged to the eye - and the second press only worked because UNGROUP does
    // a different job when it finds a group than when it does not.
    //
    // A maker who wants them moving together can say so: that is what GROUP is for.
    // "ungroup should o it all".
    for (kind, record) in born {
        spawn_part(commands, meshes, materials, palette, &kind, &record, false);
    }
    commands.entity(part).despawn();
}

// -------------------------------------------------------------- the palette
