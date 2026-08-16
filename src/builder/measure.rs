//! The lattice, and the arithmetic a framed wall is solved with.

use super::*;

/// The pitch a roof arms at, in DEGREES, before anybody pulls on it.
///
/// The unit is in the name because it changed. This was radians until roofs
/// carried their own pitch, and two other readers went on treating it as
/// radians afterwards: a gable took the tangent of thirty RADIANS and came out
/// inverted and eleven times too tall, and a roof panel arrived in the hand
/// tilted two hundred and seventy-nine degrees. Both compiled perfectly, because
/// an angle has no type. A name is the only thing that would have stopped it.
///
/// Thirty was the bench's ONE pitch for every roof of every building - forty-five
/// stood too tall over a village of this scale and the houses read as steeples.
/// It is still the right place to start, and it is no longer the only place: a
/// roof carries its own pitch now and the gold handle at its ridge sets it, which
/// is the difference between a village that reads modern and one that does not.
/// Steep is medieval; shallow is not.
pub(crate) const ROOF_PITCH_DEGREES: f32 = 30.0;

/// What a roof's pitch may be pulled to, in degrees, and the step it moves in.
/// Ten is nearly flat and sixty is a steeple; two and a half is fine enough to
/// tune by eye and coarse enough that two roofs meant to match will match.
pub(crate) const PITCH_LEAST: f32 = 10.0;
pub(crate) const PITCH_MOST: f32 = 60.0;
pub const PITCH_STEP: f32 = 2.5;

/// The pitch a roof arms at has to be one a handle can reach, or a roof would
/// draw itself at an angle its own tool could not return it to. Checked when the
/// bench is BUILT rather than when it is tested, because there is no arrangement
/// of these three numbers worth compiling that fails it.
const _: () = assert!(PITCH_LEAST <= ROOF_PITCH_DEGREES && ROOF_PITCH_DEGREES <= PITCH_MOST);

/// Opificium's own measurements - the source of truth now; the game
/// conforms to these when its buildings are replaced. A quarter-metre
/// wall on a quarter-metre grid means centrelines always land on snaps.
pub(crate) const WALL_THICK: f32 = 0.25;

/// How thick a floor is, and so a ceiling.
///
/// Named because a roof generated over a ceiling has to seat on the ceiling's TOP, and a
/// literal 0.125 at that one call site would be a number nobody could connect to the slab
/// it came from.
pub(crate) const FLOOR_THICK: f32 = 0.125;

/// How far a sill stands proud of the wall's face, on each side.
///
/// One atom. A sill sheds water, which it cannot do flush - and on the bench it is the
/// line that tells you where a window is from across the room. Brett asked for it on both
/// framed and plain walls, which is the point: it is part of the WINDOW, and the window is
/// one window.
pub(crate) const SILL_PROUD: f32 = ATOM;

/// How thick a gable end is, standing at the end of a roof.
///
/// A wall's thickness, because that is what a gable IS - the wall carried on up to the
/// ridge. Named rather than written twice: the roof draws its gables this thick and the
/// ceiling shrinks by exactly this much to make room for them, and two 0.25s that have to
/// agree are two that will not.
pub(crate) const GABLE_THICK: f32 = WALL_THICK;

/// How far a generated roof reaches past what it covers.
///
/// A hand's breadth, the same as a roof drawn out by hand starts with - the gold handles
/// pull it further without moving the gables.
pub(crate) const ROOF_OVERHANG: f32 = 0.25;

/// How far a piece that is MEANT to butt against another is drawn past its own
/// measure, so the two lap instead of meeting exactly. A sixty-fourth at each
/// end. See the wall segment in `body_of`.
/// The lattice everything is drawn on: a sixteenth of a metre, one ATOM.
///
/// Every measurement in a part is a whole number of these. Brett's rule, and the
/// one that makes the bench's seams close by themselves - a sixteenth is a power
/// of two, so parts that meet on the lattice agree on the edge exactly.
pub const ATOM: f32 = 0.0625;

/// The railing's own measurements, shared by the stairs and the flat rail so a
/// landing carries straight on from a flight.
/// Four atoms, not three, and the reason is the whole point of the lattice.
///
/// A three-atom post is an ODD number of atoms, so its centre falls half an atom
/// off the grid - and the flight's rail line, which is measured in from the
/// tread edge by a post's half-width, landed at seven and a half atoms. A flat
/// rail placed on the grid could not reach it however carefully it was set down.
/// Brett: "the new railing looks great but it doesn't line up with the railing
/// post, I painted it blue to be able to see the difference." An even post puts
/// the line at a whole seven, and the two meet.
pub(crate) const RAIL_POST: f32 = ATOM * 4.0;
pub(crate) const RAIL_THICK: f32 = ATOM * 2.0;
pub(crate) const RAIL_HIGH: f32 = ATOM * 14.0;
/// About a pace between balusters, and a whole number of atoms.
pub(crate) const RAIL_GAP: f32 = ATOM * 8.0;

/// A baluster is TWO atoms square, not one.
///
/// An odd number of atoms cannot have both its faces on the lattice while its
/// centre is on it too - a one-atom post centred on an atom has its sides on
/// half-atoms - and it is the FACES that decide whether two parts meet.
pub(crate) const RAIL_PIN: f32 = ATOM * 2.0;

/// How high a flight climbs when it comes off the shelf, and how tall a footing
/// stands when it does.
///
/// One number for both, so a platform and a flight meet without being measured
/// against each other by hand - Brett: "Can we make the two foundation pieces
/// that we have default to the same height as the default stairs?" Twelve atoms
/// is four treads of three, which is what the rhythm makes of it anyway.
pub(crate) const STEP_UP: f32 = ATOM * 12.0;

/// How finely a hand may place: points per metre, from the grid G sets, and
/// always whole atoms while shift is held.
///
/// One function because two things have to agree about it. The ghost shows where an
/// opening will land and the punch decides where it does; a step computed twice is
/// a ghost that lies.
pub(crate) fn snap_step(fine: bool, grid: i32) -> f32 {
    if fine {
        16.0
    } else {
        16.0 / grid.max(1) as f32
    }
}

/// The nearest whole atom to a measurement.
///
/// Everything a hand pulls goes through here, so a part cannot be left between
/// two atoms by a drag - and a part drawn before the rule existed comes back
/// onto the lattice the first time anybody touches it.
pub fn on_the_lattice(measure: f32) -> f32 {
    (measure / ATOM).round() * ATOM
}
pub(crate) const WALL_HIGH: f32 = 2.5;

// The framing, in whole atoms.
//
// Integers, and that is the point rather than an implementation detail. A bay
// edge reached by adding stud widths and a bay edge reached by dividing the
// span have to be the SAME NUMBER, or the panel between them is a hair too
// wide and the seam shows. Added and divided as `i32` they cannot help but
// agree; turned into metres once at the end, they land on the lattice by
// construction rather than by being snapped back onto it afterwards.

/// A corner post's width. Visibly heavier than a stud - that contrast is most
/// of what makes a wall read as framed rather than striped.
pub(crate) const POST_WIDE: i32 = 4;
/// A stud's width.
pub(crate) const STUD_WIDE: i32 = 2;
/// How deep the sill and head plates are, measured up the wall.
pub(crate) const PLATE_TALL: i32 = 3;
/// What a bay wants to be. The solver divides each clear span into whole bays
/// as near this as it can, so no bay is ever a runt.
pub(crate) const BAY_WANTED: i32 = 14;
/// How far the panel between the timbers sits behind their faces.
pub(crate) const INFILL_SET: i32 = 1;

/// A doorway's clear width and height.
pub(crate) const DOOR_WIDE: i32 = 16;
pub(crate) const DOOR_HIGH: i32 = 32;

/// A window's clear width. It takes its HEIGHT from the wall rather than from a
/// number here: it fills the upper course, so the rail is its sill and the head
/// plate is its lintel, and a window that is a fixed height would leave a
/// second beam lying against one of them.
pub(crate) const WINDOW_WIDE: i32 = 18;
/// How heavy the timber down an opening's side is.
///
/// A DOOR's is as heavy as a corner post: it is a big hole and the wall over it
/// has to be carried, and a stud's worth of timber under that lintel reads as a
/// mistake. A WINDOW's is a stud, because it is not carrying much and because a
/// frame twice the weight of every other upright in the wall reads as a
/// different kind of timber altogether - which is what it looked like.
///
/// The window grew by exactly what its jambs gave up, so the hole and its frame
/// together take the same room they always did.
pub(crate) fn jamb_of(what: Opening) -> i32 {
    match what {
        Opening::Door => POST_WIDE,
        Opening::Window => STUD_WIDE,
    }
}

/// The widest a jamb can be, for the room an opening needs reserved.
pub(crate) const JAMB_WIDE: i32 = POST_WIDE;

/// Where a wall of this height puts its courses: the clear band's foot and
/// height, the rail's foot, and the upper course's foot and height.
///
/// Shared, because the openings have to be placed against the same courses the
/// framing lays - a window that fills the upper course has to know where that
/// course is, and working it out twice is working it out twice.
pub(crate) fn courses_of(tall: i32) -> (i32, i32, i32, i32, i32) {
    let inner_foot = PLATE_TALL;
    let inner_tall = tall - PLATE_TALL * 2;
    let low_tall = ((inner_tall - PLATE_TALL) * 2 / 5).max(4);
    let rail_foot = inner_foot + low_tall;
    let high_foot = rail_foot + PLATE_TALL;
    let high_tall = (tall - PLATE_TALL - high_foot).max(0);
    (inner_foot, low_tall, rail_foot, high_foot, high_tall)
}

/// How many doors and windows one framed wall may hold.
pub const MOST_OPENINGS: usize = 4;

/// The band a window's GHOST is drawn in: the one an ordinary wall gives it.
///
/// A ghost cannot know which wall it is over until it is over one, so it is
/// drawn for the ordinary wall and LIFTED to where it will land. Both halves ask
/// here - the body that draws it and the hand that lifts it - because a ghost
/// drawn off one number and moved by another is a ghost that lies about where it
/// is going.
pub(crate) fn ghost_band() -> Band {
    band_of(Opening::Window, (WALL_HIGH / ATOM).round() as i32)
}

/// How much timber goes over an opening's head, given the room between its head
/// and the head plate.
///
/// A lintel is a TIMBER, not a filler. It was whatever was left, which is the
/// same thing while every opening reaches within a plate or two of the top - and
/// becomes a post half the height of the wall the moment a window is set low in
/// one. So: a plate's worth, unless what is left is small enough that framing it
/// would be a strip nobody would build, in which case it takes the lot.
///
/// The wall ABOVE that is ordinary wall and gets ordinary framing, which is the
/// other half of this and lives in [`body_of`](crate::builder::body_of).
pub(crate) fn lintel_of(over: i32) -> i32 {
    if over <= PLATE_TALL * 2 {
        over
    } else {
        PLATE_TALL
    }
}

/// The lowest a window's foot may be set, in atoms.
///
/// Its sill hangs a plate's worth below it and the wall's own sill plate takes
/// the plate above the floor, so two plates is where the two stop fighting for
/// the same atoms.
pub(crate) const LOWEST_SILL: i32 = PLATE_TALL * 2;

/// Where a kind of opening stands in a wall of this height when nobody has said
/// otherwise.
///
/// Also what a NAME leaves out, exactly as [`usual_width`] is: a hole standing
/// where its kind would stand writes no band at all, so every wall drawn before
/// a window could be lifted reads back byte for byte the same.
///
/// A HAND asks this too, and must: the window it lifts off the course keeps the
/// size the course gave it, so sliding one up a wall changes where it is and
/// nothing else.
pub(crate) fn band_of(what: Opening, tall: i32) -> Band {
    let (_, _, _, high_foot, high_tall) = courses_of(tall);
    let (rise, foot) = match what {
        // A door reaches the FLOOR. It stood on the sill plate before, which put
        // its head two atoms above the leaf hung in it - the leaf is two metres
        // from the ground, and the hole was two metres from the top of the plate
        // - so the door sat low in its own opening with daylight over it.
        // Nothing crosses a doorway: the plate is laid in pieces around it,
        // which is what you walk through.
        Opening::Door => (DOOR_HIGH, 0),
        // A window FILLS the upper course, from the rail up. It used to sit
        // INSIDE that course with a sill of its own under it, landing directly
        // against the rail - two beams touching read as one thick beam, and the
        // window looked squeezed. Reaching the course's own foot instead, the
        // rail IS its sill, one plate thick like every other horizontal in the
        // wall.
        Opening::Window => (high_tall, high_foot),
    };
    Band {
        foot,
        // A plate's worth of lintel over it, always: the timber that carries the
        // wall across the hole. The head plate is not that timber - it carries
        // the roof.
        rise: rise.min(tall - PLATE_TALL - foot - PLATE_TALL),
    }
}

/// A window bar's thickness - the mullion standing up it and the transom lying
/// across. Half a stud: these divide the light rather than carry the wall, and
/// a bar as heavy as a stud reads as a wall with a small hole either side of it
/// rather than as a window.
pub(crate) const BAR_WIDE: i32 = 1;

/// How big a pane of glass wants to be, in atoms.
///
/// The same idea as [`BAY_WANTED`] and for the same reason: a pane is a SIZE, not a
/// fraction of a window. A taller window gains another row of panes rather than three
/// taller ones, exactly as a longer wall gains a bay rather than wider ones - so every
/// window in a village is glazed alike whatever wall it stands in.
///
/// Nine atoms lands the ordinary 2.5 m cottage wall on two panes by two, and a hall's
/// three-metre wall on two by three, which is what a townhall's windows look like.
pub(crate) const PANE_WANTED: i32 = 9;

/// How many panes fit across a span of atoms - the rule that divides a window.
pub(crate) fn panes_in(across: i32) -> i32 {
    (across as f32 / PANE_WANTED as f32).round().max(1.0) as i32
}

/// And back: the atoms a whole number of panes wants.
///
/// The two are exact inverses by construction, which is the point. A maker asks
/// for three panes up and gets a window that divides into three - not a window
/// of some height that happens to round to three, which is what choosing a size
/// in metres would have meant. Brett counts his windows in panes: "The three
/// high two wide windows with the black crossbars need to be made."
pub(crate) fn panes_across(panes: i32) -> i32 {
    panes.clamp(1, MOST_PANES) * PANE_WANTED
}

/// How much of its run a hip roof's flat top keeps when nobody has said.
///
/// A half, which is what every hip on this bench had while the deck was implied
/// - the slope ran in by half the shorter half-extent - so a roof saved before
/// the number existed opens at the shape it was drawn in.
pub(crate) const HIP_DECK: f32 = 0.5;

/// The most panes a window is offered either way.
///
/// Four is a shopfront. Past that a maker wants two windows, and a drawer with
/// twelve lines in it is a drawer nobody reads.
pub(crate) const MOST_PANES: i32 = 4;

/// A span of atoms in `n` parts that sum to exactly the span.
///
/// Integer division leaves a remainder of up to `n - 1` atoms. Dropping it
/// would leave a visible runt at one end of the wall, so it is spread an atom
/// at a time across the leading parts: the widest and narrowest bay then differ
/// by one atom - four millimetres, invisible - and the parts still tile the
/// span exactly. Straight out of Opificium's `Len::divide`, which is the piece
/// that makes a wall gain a bay cleanly instead of gaining a seam.
/// Where an opening actually sits in a wall of this size: its left edge, its
/// width, its foot and its height, all in atoms.
///
/// Lifted out so the solver and the test that checks the wall for holes are
/// asking the same question of the same arithmetic. A test that worked the
/// clamp out for itself would agree with the solver right up until one of them
/// was changed.
pub(crate) fn openings_at(
    span: i32,
    tall: i32,
    openings: &[Option<Hole>; MOST_OPENINGS],
) -> Vec<(Opening, i32, i32, i32, i32, bool)> {
    let mut holes: Vec<(Opening, i32, i32, i32, i32, bool)> = Vec::new();
    for hole in openings.iter().flatten().copied() {
        let (what, at) = (hole.what, hole.at);
        // THE HOLE'S OWN, all four of them. The wall decides nothing about a
        // window any more - it hands one a band at the moment it is made and has
        // no further say, which is what lets a townhall's windows be three panes
        // tall in a wall whose courses would have given them two.
        let (foot, rise, wide) = (hole.lift, hole.high, hole.wide);
        let room = span - POST_WIDE - JAMB_WIDE - wide;
        if room < POST_WIDE + JAMB_WIDE {
            continue;
        }
        let middle = (span as f32 * 0.5 + at / ATOM).round() as i32;
        let from = (middle - wide / 2).clamp(POST_WIDE + JAMB_WIDE, room);
        // Inside the wall, whoever asked for it: never through the floor, never
        // into the head plate. A band its kind chose is inside these already -
        // this is the clamp that catches a band a HAND chose on a wall that has
        // since been made shorter.
        let foot = foot.clamp(0, (tall - PLATE_TALL).max(0));
        let rise = rise.min(tall - PLATE_TALL - foot);
        // Two openings that overlap would frame each other's jambs and leave a
        // bay of no width between them. The later one simply does not fit.
        if holes.iter().any(|(theirs, hx, hw, hy, hh, _)| {
            from < hx + hw + jamb_of(*theirs)
                && from + wide + jamb_of(what) > *hx
                && foot < hy + hh
                && foot + rise > *hy
        }) {
            continue;
        }
        holes.push((what, from, wide, foot, rise, hole.dark));
    }
    holes.sort_by_key(|(_, from, ..)| *from);
    holes
}

pub(crate) fn into_bays(span: i32, bays: i32) -> Vec<i32> {
    if bays <= 0 {
        return Vec::new();
    }
    let base = span.div_euclid(bays);
    let mut over = span.rem_euclid(bays);
    (0..bays)
        .map(|_| {
            let extra = if over > 0 {
                over -= 1;
                1
            } else {
                0
            };
            base + extra
        })
        .collect()
}
