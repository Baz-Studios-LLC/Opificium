//! What a part is built from: one slab, and the four shapes one can be cut to.

use super::*;

/// One piece of a part's body: offset from the part origin, size, ramp,
/// shade, how much of the world shows through it (1.0 = none), whether it is
/// a wedge rather than a box - a triangular prism, for the honest slopes a
/// gable wants - and two angles.
///
/// The angles are the piece's own, and they are different rotations for
/// different jobs. LEAN turns it about its length, which is how a stair's rail
/// climbs beside the flight. CANT turns it about the part's thickness, which
/// swings it WITHIN the face of what it belongs to - and that is the one a
/// diagonal brace in a wall needs, because a brace stays in the wall's plane
/// and simply lies across it.
pub(crate) struct Slab {
    /// Offset from the part's origin.
    pub(crate) at: Vec3,
    pub(crate) size: Vec3,
    pub(crate) ramp: String,
    pub(crate) shade: f32,
    /// How much of the world shows through it. 1.0 is none.
    pub(crate) clarity: f32,
    pub(crate) shape: Shape,
    /// Turned about its own length - how a stair's rail climbs beside the
    /// flight. This takes a piece OUT of the face it belongs to.
    pub(crate) lean: f32,
    /// Turned about the part's thickness, which swings it WITHIN that face.
    /// What a diagonal brace in a wall needs.
    pub(crate) cant: f32,
    /// How far the top face is cut back at each end, as a RUN in the piece's
    /// own units: the distance the saw travels along it while crossing its full
    /// height. `x` is the -X end, `y` is the +X end, and nought is square.
    ///
    /// A run rather than an angle because a run is the number everything else
    /// already has - a roof hands over the difference between where its slope
    /// meets the top of a beam and where it meets the bottom - and no caller
    /// needs trigonometry to say what it wants.
    pub(crate) cut: Vec2,
}

/// What a piece of a body is cut from.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Shape {
    /// The plain box, which is most of everything.
    Box,
    /// A gable's prism: the triangle stands across the part's length.
    Wedge,
    /// A ridge cap's prism: the triangle stands ACROSS the part, which
    /// runs lengthwise under it, apex up.
    Ridge,
    /// A truncated pyramid: four faces sloping in from the box's foot to a
    /// smaller flat top. A hip roof with a deck on it.
    ///
    /// The two numbers are what fraction of the box's own half-extents the top
    /// face keeps, along X and along Z. They differ because a roof is rarely
    /// square and the slope should run in the same distance on every side.
    ///
    /// This has to be a SHAPE rather than an arrangement of slabs, because a
    /// slab can lean about X and no other axis - which is what a gable roof's
    /// two slopes use, and why a gable roof has only two. Four of them needs the
    /// slope carried in the mesh.
    Hip(f32, f32),
}

/// A truncated pyramid, in a unit box: four sloping faces and a flat top.
///
/// The hip roof. Brett: "a square roof that slopes on all four sides and it flat
/// on top." Built like every other shape here - unit-sized, so the slab's own
/// dimensions give it its pitch and its footprint - and the two fractions say
/// how much of the box the flat top keeps.
pub(crate) fn hip_mesh(top_x: f32, top_z: f32) -> Mesh {
    let (tx, tz) = (top_x.clamp(0.0, 1.0) * 0.5, top_z.clamp(0.0, 1.0) * 0.5);
    // Eight corners: the eave rectangle at the bottom, the deck at the top.
    let foot = [
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(-0.5, -0.5, 0.5),
    ];
    let deck = [
        Vec3::new(-tx, 0.5, -tz),
        Vec3::new(tx, 0.5, -tz),
        Vec3::new(tx, 0.5, tz),
        Vec3::new(-tx, 0.5, tz),
    ];
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut face = |corners: [Vec3; 4]| {
        let first = positions.len() as u32;
        // Flat-shaded, like every other face in this world: one normal for the
        // whole face, worked out from its own corners.
        let normal = (corners[1] - corners[0])
            .cross(corners[2] - corners[0])
            .normalize_or(Vec3::Y);
        for corner in corners {
            positions.push(corner.to_array());
            normals.push(normal.to_array());
        }
        indices.extend([first, first + 1, first + 2, first, first + 2, first + 3]);
    };
    // The deck, then the four slopes, each from an eave edge up to the deck edge
    // above it. Wound so every face looks outward.
    // The deck, wound so it looks UP. Wound the other way it is culled, and what
    // a maker sees through the hole is the underside of the roof - which reads
    // as a sunken tray rather than a missing face. Brett: "the hip roof needs the
    // flat part on top."
    face([deck[3], deck[2], deck[1], deck[0]]);
    face([foot[0], deck[0], deck[1], foot[1]]);
    face([foot[1], deck[1], deck[2], foot[2]]);
    face([foot[2], deck[2], deck[3], foot[3]]);
    face([foot[3], deck[3], deck[0], foot[0]]);
    // And the underside, looking DOWN, so a roof seen from below is not a hole.
    face([foot[1], foot[2], foot[3], foot[0]]);
    let uvs: Vec<[f32; 2]> = positions.iter().map(|_| [0.0, 0.0]).collect();
    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
}

/// A right-angle prism: a box with one end cut clean through at an angle.
///
/// The shape a saw makes. A wedge is a GABLE's prism - two slopes meeting at a
/// peak - and there was nothing in the bench for the far commoner cut, so a beam
/// meeting a roof had to stop square and stand off it. Brett: "There has to be a
/// way to cut the end of the beam at an angle. Keeping it square wont work."
///
/// Built in a unit box like every other shape, so its angle is whatever the
/// slab's own proportions make it: a mitre one long and one high is
/// forty-five degrees, and squashing it flatter or steeper is what sizing it
/// does. The full-height end stands at -X and the cut falls away to +X.
/// A unit box with its top or bottom face cut back at each end.
///
/// `low` and `high` are RUNS as fractions of the box's own length: how far along
/// it the saw travels while crossing its full height, at the -X end and the +X
/// end. Nought is a square end. A POSITIVE run cuts the top face back, a
/// NEGATIVE one cuts the bottom.
///
/// One shape for every angled end there is. There used to be two - a mitre and
/// its mirror - because a beam can be cut at one end or the other, and neither
/// could do both at once. A run of one takes a face all the way to the far
/// corner, which IS the old full mitre, so nothing that could be drawn before
/// has stopped being drawable.
///
/// The signs are what make a brace possible. Cut the top at one end and the
/// bottom at the other and the two ends come out PARALLEL - a parallelogram,
/// which is what a diagonal brace is, since both of its ends meet horizontal
/// timber.
pub(crate) fn cut_mesh(low: f32, high: f32) -> Mesh {
    let (low, high) = (low.clamp(-1.0, 1.0), high.clamp(-1.0, 1.0));
    // A run may be NEGATIVE, and that is what lets a brace exist. A positive
    // run cuts the top face back; a negative one cuts the bottom. Cut the top at
    // one end and the bottom at the other by the same amount and the two ends
    // come out PARALLEL - a parallelogram, which is what a diagonal brace is,
    // because both of its ends meet horizontal timber. With the top cut at both
    // ends the ends converge instead, and a brace would sit in its bay like a
    // wedge.
    let inset = |run: f32| (run.max(0.0), (-run).max(0.0));
    let (top_low, foot_low) = inset(low);
    let (top_high, foot_high) = inset(high);
    // Cuts that would cross each other leave a face inside out; share the
    // length between them instead.
    let share = |a: f32, b: f32| {
        if a + b > 1.0 {
            (a / (a + b), b / (a + b))
        } else {
            (a, b)
        }
    };
    let (top_low, top_high) = share(top_low, top_high);
    let (foot_low, foot_high) = share(foot_low, foot_high);

    let (ta, tb) = (-0.5 + top_low, 0.5 - top_high);
    let (fa, fb) = (-0.5 + foot_low, 0.5 - foot_high);
    let top_peak = tb <= ta;
    let foot_peak = fb <= fa;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Normals from the corners themselves, and wound to face outward.
    //
    // The shape is convex and centred on the origin, so a face's own middle
    // points the way that face does - which settles both the normal's sign and
    // the winding without anyone having to reason about which end is cut. The
    // slanted ends are where doing it by hand goes wrong, and they are the
    // whole point of this mesh.
    let mut face = |corners: &[Vec3]| {
        if corners.len() < 3 {
            return;
        }
        let middle = corners.iter().copied().sum::<Vec3>() / corners.len() as f32;
        let Some(normal) = (corners[1] - corners[0])
            .cross(corners[2] - corners[0])
            .try_normalize()
        else {
            return;
        };
        let (normal, flip) = if normal.dot(middle) < 0.0 {
            (-normal, true)
        } else {
            (normal, false)
        };
        let first = positions.len() as u32;
        let mut corners = corners.to_vec();
        if flip {
            corners.reverse();
        }
        for corner in &corners {
            positions.push(corner.to_array());
            normals.push(normal.to_array());
        }
        for step in 1..(corners.len() as u32 - 1) {
            indices.extend_from_slice(&[first, first + step, first + step + 1]);
        }
    };

    let at = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);

    // The underside and the top, each shortened by whatever was cut from it.
    if !foot_peak {
        face(&[
            at(fa, -0.5, -0.5),
            at(fb, -0.5, -0.5),
            at(fb, -0.5, 0.5),
            at(fa, -0.5, 0.5),
        ]);
    }
    if !top_peak {
        face(&[
            at(ta, 0.5, -0.5),
            at(tb, 0.5, -0.5),
            at(tb, 0.5, 0.5),
            at(ta, 0.5, 0.5),
        ]);
    }
    // The sides, walked round in order so a collapsed edge simply drops out.
    for z in [-0.5f32, 0.5] {
        let mut corners = vec![at(fa, -0.5, z)];
        if !foot_peak {
            corners.push(at(fb, -0.5, z));
        }
        corners.push(at(tb, 0.5, z));
        if !top_peak {
            corners.push(at(ta, 0.5, z));
        }
        face(&corners);
    }
    // And the ends themselves, square where nothing was cut and leaning where
    // something was.
    face(&[
        at(fa, -0.5, -0.5),
        at(fa, -0.5, 0.5),
        at(ta, 0.5, 0.5),
        at(ta, 0.5, -0.5),
    ]);
    face(&[
        at(fb, -0.5, -0.5),
        at(fb, -0.5, 0.5),
        at(tb, 0.5, 0.5),
        at(tb, 0.5, -0.5),
    ]);
    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
}

pub(crate) fn wedge_mesh(lengthwise: bool) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut face = |corners: &[[f32; 3]], normal: [f32; 3]| {
        let first = positions.len() as u32;
        for corner in corners {
            positions.push(*corner);
            normals.push(normal);
        }
        for step in 1..(corners.len() as u32 - 1) {
            indices.extend_from_slice(&[first, first + step, first + step + 1]);
        }
    };
    let slope = (2.0f32 / 5.0f32.sqrt(), 1.0 / 5.0f32.sqrt());
    // The two triangular faces.
    face(
        &[[-0.5, -0.5, 0.5], [0.5, -0.5, 0.5], [0.0, 0.5, 0.5]],
        [0.0, 0.0, 1.0],
    );
    face(
        &[[0.5, -0.5, -0.5], [-0.5, -0.5, -0.5], [0.0, 0.5, -0.5]],
        [0.0, 0.0, -1.0],
    );
    // The floor.
    face(
        &[
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, -0.5, -0.5],
            [-0.5, -0.5, -0.5],
        ],
        [0.0, -1.0, 0.0],
    );
    // The two slopes.
    face(
        &[
            [-0.5, -0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [0.0, 0.5, 0.5],
            [0.0, 0.5, -0.5],
        ],
        [-slope.0, slope.1, 0.0],
    );
    face(
        &[
            [0.5, -0.5, 0.5],
            [0.5, -0.5, -0.5],
            [0.0, 0.5, -0.5],
            [0.0, 0.5, 0.5],
        ],
        [slope.0, slope.1, 0.0],
    );
    // A ridge cap is the same prism turned a quarter: the triangle
    // stands across the part and the length runs under the apex.
    if lengthwise {
        for corner in &mut positions {
            *corner = [corner[2], corner[1], corner[0]];
        }
        for normal in &mut normals {
            *normal = [normal[2], normal[1], normal[0]];
        }
        for triangle in indices.chunks_mut(3) {
            triangle.swap(1, 2);
        }
    }
    let uvs: Vec<[f32; 2]> = positions.iter().map(|_| [0.0, 0.0]).collect();
    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
}
