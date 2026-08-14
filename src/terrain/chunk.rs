//! The ground, cut into squares and meshed a square at a time.
//!
//! A world is kilometres across and no machine will draw it whole, so it is
//! meshed in chunks around wherever the eye is and thrown away behind. Each
//! chunk is built on a background thread and arrives when it is ready; the old
//! mesh stays on screen until the new one lands, so ground being re-cut under a
//! brush never blinks out.
//!
//! # Why the chunks stitch
//!
//! Height and normal at any point depend ONLY on world position — never on
//! which chunk is asking — so two neighbours sampling the edge they share get
//! bit-identical answers. No crack, no seam in the lighting, and no skirts.

use bevy::asset::RenderAssetUsages;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future};

use crate::look::Palette;
use crate::terrain::edit::smoothstep;
use crate::terrain::ground::{Ground, World};

/// A chunk covers this many metres each way.
pub const CHUNK: f32 = 256.0;

/// Quads along a chunk's edge, giving a 4 m vertex grid over a 256 m chunk.
///
/// Sized so the WHOLE world can stand at once. A bench is for judging the shape
/// of a coastline against the shape of the one across the water, and a disc of
/// ground following the eye about cannot show you that. The arithmetic decides
/// the numbers: an 8 km world at 128 m chunks and a 2 m grid is 2,176 meshes and
/// some 440 MB, which is not a thing to hold; at 256 m and a 4 m grid it is 544
/// meshes and about a tenth of that.
///
/// 4 m is not a compromise, either - it is exactly the edit grid's own cell size
/// (see `edit::CELL`), so every hill a maker can sculpt is a hill the mesh can
/// show. What is lost is a little of the generated fine detail, which is scenery
/// rather than shape.
pub const QUADS: u32 = 64;

/// How many may be building at once, so opening a world does not queue five
/// hundred tasks in a single frame.
pub const AT_ONCE: usize = 24;

/// A piece of ground on the stage.
///
/// A marker and nothing more: which square it is lives in [`Standing`], and
/// keeping it in two places would only be two places to disagree.
#[derive(Component)]
pub struct Chunk;

/// A chunk whose mesh is still being built.
#[derive(Component)]
pub struct Building(Task<Mesh>);

/// Which chunks are standing, by grid square.
#[derive(Resource, Default)]
pub struct Standing {
    pub up: HashMap<IVec2, Entity>,
}

/// One material for all of it — every colour comes from the mesh, so there is no
/// reason to hold a material per chunk.
#[derive(Resource, Deref)]
pub struct GroundMaterial(pub Handle<StandardMaterial>);

/// How far above and below the waterline sand reaches, in METRES OF HEIGHT.
///
/// Full sand within the first, gone by the second. Keyed to height rather than
/// to distance along the ground on purpose: keyed to distance, a beach widens
/// with its own gradient, so making the coast shelve gently turned the entire
/// world's shoreline into a kilometre of sand.
const BEACH_FULL: f32 = 1.0;
const BEACH_GONE: f32 = 6.0;

/// The game's ramps, resolved to linear colour once and carried to the threads.
///
/// The bench paints the ground in whatever game is open — its water, its grass,
/// its stone — the same way every other bench paints with the game's palette.
/// Snapshotted rather than borrowed because the meshing happens off the main
/// thread, where a Bevy resource cannot follow.
#[derive(Clone)]
pub struct Colours {
    silt: Vec3,
    shallow: Vec3,
    sand: Vec3,
    dry: Vec3,
    lush: Vec3,
    forest: Vec3,
    rock: Vec3,
    alpine: Vec3,
    snow: Vec3,
}

impl Colours {
    pub fn from(palette: &Palette) -> Self {
        let of = |name: &str, step: f32| {
            let c = palette.shade(name, step).to_linear();
            Vec3::new(c.red, c.green, c.blue)
        };
        Self {
            silt: of("water", 0.1),
            shallow: of("water", 0.5),
            sand: of("sand", 0.62),
            dry: of("scrub", 0.6),
            lush: of("grass", 0.6),
            forest: of("foliage", 0.45),
            rock: of("stone", 0.5),
            alpine: of("stone", 0.75),
            snow: of("snow", 0.95),
        }
    }

    /// What the ground looks like at one point.
    ///
    /// Every change is eased, so one thing becomes another rather than drawing a
    /// contour line across the landscape.
    fn at(
        &self,
        height: f32,
        slope: f32,
        moisture: f32,
        character: f32,
        sea_level: f32,
    ) -> [f32; 4] {
        // Under water, by DEPTH: dark in the deep, lightening as it shallows.
        // Deliberately not sand - the beach is a separate band added below, and
        // running the sea floor to sand made every gradual shelf pale for
        // hundreds of metres.
        let depth = sea_level - height;
        let drowned = self.silt.lerp(self.shallow, smoothstep(45.0, 3.0, depth));

        // Growing things: how wet it is picks dry plain, then grass, then wood.
        let grass = self.dry.lerp(self.lush, smoothstep(0.25, 0.60, moisture));
        let green = grass.lerp(self.forest, smoothstep(0.58, 0.88, moisture));

        // Height strips it back to bare ground, then to snow.
        let bare = green.lerp(self.alpine, smoothstep(125.0, 190.0, height));
        let capped = bare.lerp(self.snow, smoothstep(175.0, 225.0, height));

        let mut colour = if height >= sea_level { capped } else { drowned };

        // The shoreline band: how close to the waterline this is, fading out
        // with height from both sides rather than ending at a line.
        let shoreline = 1.0 - smoothstep(BEACH_FULL, BEACH_GONE, (height - sea_level).abs());

        // What the band is MADE of is the point. Sand is not the default state
        // of a coast - it needs somewhere for sediment to settle, which means a
        // gentle shore, and it changes along the coast rather than being true of
        // the whole map. Where those do not hold, the sea meets rock instead.
        // A world with every continent outlined in sand reads as a drawing of a
        // map rather than as ground, which is exactly how it looked.
        let gentle = 1.0 - smoothstep(0.06, 0.22, slope);
        let sandy = shoreline * character * gentle;
        let stony = shoreline * (1.0 - character * gentle);

        colour = colour.lerp(self.rock, stony * 0.7);
        colour = colour.lerp(self.sand, sandy);

        // Steep ground is bare rock whatever else it would have been. This is
        // what makes a cliff read as stone instead of vertical lawn.
        colour = colour.lerp(self.rock, smoothstep(0.34, 0.62, slope));
        [colour.x, colour.y, colour.z, 1.0]
    }
}

/// Where a chunk's north-west corner sits.
pub fn corner(square: IVec2) -> Vec2 {
    Vec2::new(square.x as f32, square.y as f32) * CHUNK
}

/// Which square a place falls in.
pub fn square_at(place: Vec3) -> IVec2 {
    IVec2::new(
        (place.x / CHUNK).floor() as i32,
        (place.z / CHUNK).floor() as i32,
    )
}

/// Builds one chunk's mesh. Pure and thread-safe, so it runs off the frame.
pub fn build(world: &World, colours: &Colours, square: IVec2) -> Mesh {
    let quads = QUADS as usize;
    let side = quads + 1;
    let step = CHUNK / QUADS as f32;
    let origin = corner(square);

    let count = side * side;
    let mut places = Vec::with_capacity(count);
    let mut normals = Vec::with_capacity(count);
    let mut colour = Vec::with_capacity(count);
    let mut uvs = Vec::with_capacity(count);

    for iz in 0..side {
        for ix in 0..side {
            let local = Vec2::new(ix as f32 * step, iz as f32 * step);
            let at = origin + local;

            let height = world.height(at.x, at.y);
            // Half a cell: fine enough to catch what the mesh can show, coarse
            // enough not to amplify detail the vertices never sample.
            let normal = world.normal(at.x, at.y, step * 0.5);
            let slope = 1.0 - normal.y;
            let moisture = world.moisture(at.x, at.y);
            let character = world.shore_character(at.x, at.y);

            places.push([local.x, height, local.y]);
            normals.push([normal.x, normal.y, normal.z]);
            colour.push(colours.at(height, slope, moisture, character, 0.0));
            uvs.push([ix as f32 / quads as f32, iz as f32 / quads as f32]);
        }
    }

    // Two triangles a quad, wound anticlockwise seen from above so they face up
    // and survive backface culling.
    let mut indices = Vec::with_capacity(quads * quads * 6);
    for iz in 0..quads {
        for ix in 0..quads {
            let a = (iz * side + ix) as u32;
            let b = a + 1;
            let c = a + side as u32;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        // Drawn and never read back, and dropping the CPU copy after it is
        // uploaded keeps memory flat while hundreds stream past.
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, places)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colour)
    .with_inserted_indices(Indices::U32(indices))
}

/// Sets a chunk's mesh building on a background thread.
///
/// Used both when ground first comes into view and when the brush has cut it, so
/// a chunk arriving and a chunk being re-cut are one operation. In both cases
/// `collect` hangs the finished mesh on it, and the old one stays up meanwhile.
pub fn set_building(
    commands: &mut Commands,
    who: Entity,
    ground: &Ground,
    colours: &Colours,
    square: IVec2,
) {
    let world = ground.0.clone();
    let colours = colours.clone();
    let task = AsyncComputeTaskPool::get().spawn(async move { build(&world, &colours, square) });
    commands.entity(who).insert(Building(task));
}

/// Brings up the whole world, a few chunks a frame.
///
/// Nothing is ever taken down. The world stands entire once it has arrived,
/// because the reason to be at this bench is to see it entire - where the ranges
/// sit relative to the coast, whether that isthmus is walkable, how two
/// landmasses read against each other. A disc of ground following the eye about
/// answers none of that.
///
/// Filled a few at a time, nearest the eye first, so the ground you are looking
/// at is there while the far side is still arriving.
pub fn stream(
    mut commands: Commands,
    ground: Res<Ground>,
    colours: Res<GroundColours>,
    eye: Res<crate::camera::OrbitRig>,
    mut standing: ResMut<Standing>,
    building: Query<(), With<Building>>,
) {
    let mut afoot = building.iter().count();
    if afoot >= AT_ONCE {
        return;
    }

    let half = ground.half();
    let edge = IVec2::new(
        (half.x / CHUNK).ceil() as i32,
        (half.y / CHUNK).ceil() as i32,
    );

    let middle = square_at(eye.focus);
    let mut wanted: Vec<(i32, IVec2)> = Vec::new();
    for z in -edge.y..edge.y {
        for x in -edge.x..edge.x {
            let square = IVec2::new(x, z);
            if standing.up.contains_key(&square) {
                continue;
            }
            let away = square - middle;
            wanted.push((away.x * away.x + away.y * away.y, square));
        }
    }
    // Everything is up: the common case once a world has settled, and the reason
    // this costs nothing to run every frame.
    if wanted.is_empty() {
        return;
    }
    wanted.sort_by_key(|(away, _)| *away);

    for (_, square) in wanted {
        if afoot >= AT_ONCE {
            break;
        }
        // The entity exists the moment work starts, wearing its final place;
        // only the mesh arrives later. Recording it now is what stops the same
        // square being queued again next frame.
        let at = corner(square);
        let who = commands
            .spawn((Chunk, Transform::from_xyz(at.x, 0.0, at.y)))
            .id();
        set_building(&mut commands, who, &ground, &colours.0, square);
        standing.up.insert(square, who);
        afoot += 1;
    }
}

/// The palette, snapshotted, so the threads can have it.
#[derive(Resource)]
pub struct GroundColours(pub Colours);

/// Hangs finished meshes on the chunks waiting for them.
pub fn collect(
    mut commands: Commands,
    material: Res<GroundMaterial>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut waiting: Query<(Entity, &mut Building)>,
) {
    for (who, mut building) in &mut waiting {
        let Some(mesh) = block_on(future::poll_once(&mut building.0)) else {
            continue;
        };
        commands
            .entity(who)
            .remove::<Building>()
            .insert((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material.0.clone())));
    }
}

/// Sets every chunk overlapping a piece of ground building again.
///
/// The sculpting is read between cells, so a change reaches one cell past its
/// own bounds — hence the margin, without which chunk edges would drift apart
/// along the rim of a stroke.
pub fn recut(
    commands: &mut Commands,
    ground: &Ground,
    colours: &Colours,
    standing: &Standing,
    building: &Query<(), With<Building>>,
    patch: Rect,
) {
    let margin = crate::terrain::edit::CELL;
    let low = ((patch.min - margin) / CHUNK).floor().as_ivec2();
    let high = ((patch.max + margin) / CHUNK).floor().as_ivec2();

    for z in low.y..=high.y {
        for x in low.x..=high.x {
            let square = IVec2::new(x, z);
            let Some(&who) = standing.up.get(&square) else {
                continue;
            };
            // Already building: skip rather than queue a second. This is what
            // paces the painting — a chunk is re-cut as fast as it can be and no
            // faster, however many frames a stroke lasts.
            if building.contains(who) {
                continue;
            }
            set_building(commands, who, ground, colours, square);
        }
    }
}

/// Takes the whole world down, sea included, for when a maker leaves the bench
/// or opens a different world.
pub fn clear(commands: &mut Commands, standing: &mut Standing, sea: &Query<Entity, With<Sea>>) {
    for (_, who) in standing.up.drain() {
        commands.entity(who).despawn();
    }
    for who in sea {
        commands.entity(who).despawn();
    }
}

/// The sea, standing at the waterline.
#[derive(Component)]
pub struct Sea;

/// How far the tide carries the waterline up and down, in metres.
///
/// **Small.** The coast shelves over hundreds of metres, so the water's
/// horizontal travel is its vertical travel divided by a gradient of about a
/// tenth — every centimetre of tide is ten centimetres of beach. At half a metre
/// the sea was drawing back a good fifteen metres and stranding the shallows,
/// which reads as a lake emptying rather than as a shore.
const TIDE: f32 = 0.18;
/// How long a full tide takes, in seconds.
const TIDE_PERIOD: f32 = 20.0;

/// Quads along the sea's edge.
const SEA_QUADS: usize = 160;

/// Swell: how tall, how far apart, and how fast, in metres and seconds.
///
/// **The wavelengths are long because the mesh cannot hold short ones.** The sea
/// spans several times the world, so even at this many quads its vertices sit
/// well over a hundred metres apart, and a thirty-metre wave written onto that
/// grid does not come out as a wave — it comes out as noise, sampled at random
/// points along a curve nobody can see. Anything under about four vertices per
/// wavelength is a lie. What is left is long ocean swell, which is what you see
/// from any height worth looking at a coastline from.
const SWELL: [(f32, f32, f32); 2] = [(0.20, 1800.0, 24.0), (0.12, 900.0, 15.0)];

/// Lays the sea over the world.
///
/// Without it the dark ground of the sea FLOOR reads as the water itself, and
/// every coast looks like a cliff standing sixty metres above a dry basin -
/// which is not what the numbers say at all, and sends you tuning heights that
/// were never wrong. A surface at the waterline is the reference the whole
/// landscape is judged against.
///
/// Three times the world across, so the horizon past any coast is water rather
/// than the edge of a pane.
pub fn lay_the_sea(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
    half: Vec2,
) {
    let reach = half.max_element() * 6.0;
    commands.spawn((
        Sea,
        // A grid rather than a single quad, because its vertices are moved every
        // frame to carry the swell. Coarse: it holds a wave, not a coastline.
        Mesh3d(meshes.add(
            Plane3d::default()
                .mesh()
                .size(reach, reach)
                .subdivisions(SEA_QUADS as u32),
        )),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: palette.shade("water", 0.35).with_alpha(0.78),
            // Blended and smooth: you have to be able to read the shape of the
            // sea floor through it to know where the shallows are.
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.12,
            reflectance: 0.25,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

/// How high the sea stands at a point, at a moment.
///
/// The tide is the important half. Swell alone gives you a textured sheet; it is
/// the slow rise and fall of the whole surface that makes the water *approach
/// and recede*, because on a shelving coast a little vertical travel is a long
/// horizontal one. Shared with the game so the two agree about where the water
/// is — a shoreline that washes differently in the tool than in the game is a
/// shoreline you cannot judge.
pub fn sea_height(at: Vec2, seconds: f32) -> f32 {
    let tide = (seconds / TIDE_PERIOD * std::f32::consts::TAU).sin() * TIDE;
    let mut swell = 0.0;
    for (i, (height, length, period)) in SWELL.iter().enumerate() {
        // Each layer runs at its own angle, so they interfere rather than
        // marching in step - waves in lockstep read as corrugated iron.
        let angle = i as f32 * 2.1;
        let along = at.x * angle.cos() + at.y * angle.sin();
        let phase = along / length - seconds / period;
        swell += (phase * std::f32::consts::TAU).sin() * height;
    }
    tide + swell
}

/// Walks the sea's vertices to carry the swell and the tide.
pub fn move_the_sea(
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    sea: Query<&Mesh3d, With<Sea>>,
) {
    let seconds = time.elapsed_secs();
    for handle in &sea {
        let Some(mut mesh) = meshes.get_mut(&handle.0) else {
            continue;
        };
        let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(places)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        else {
            continue;
        };
        for place in places.iter_mut() {
            place[1] = sea_height(Vec2::new(place[0], place[2]), seconds);
        }
        // Normals are left flat on purpose. Recomputing them across a grid this
        // size every frame costs more than the lighting gains, and a broad
        // water surface reads off its colour and its silhouette against the
        // shore rather than off its shading.
    }
}

/// The ground's material: white, because every colour it wears comes from the
/// mesh, and matte, because a landscape with a sheen on it reads as plastic.
pub fn ground_material(materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.94,
        reflectance: 0.03,
        ..default()
    })
}
