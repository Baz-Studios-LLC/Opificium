//! The stage: floor, grid, axis marks and lights.

use bevy::prelude::*;

use crate::Bench;
use crate::look::Palette;

/// The grid spans this many metres each way from the origin.
const REACH: f32 = 14.0;

/// Stage furniture that belongs to the builder's bench.
#[derive(Component)]
pub struct BuilderFurniture;

/// Stage furniture that belongs to the rig bench: the model standing on it.
#[derive(Component)]
pub struct RigFurniture;

/// The measuring post: one band per decimetre, a brighter one per metre.
#[derive(Component)]
pub struct Ruler;

/// Stage furniture that belongs to the kiln: the model standing on it.
#[derive(Component)]
pub struct KilnFurniture;

pub struct StagePlugin;

impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, dress_stage)
            .add_systems(Update, (follow_bench, follow_the_ruler));
    }
}

fn dress_stage(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<Palette>,
) {
    // Workshop light: a keyed sun and a soft fill, no weather, no hour.
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.75, 0.78, 0.85),
        brightness: 220.0,
        ..default()
    });
    // The key light stands over the maker's shoulder - the working perch
    // looks from +X +Z - so the faces you are looking at are the lit
    // ones and the shadows fall away behind the work.
    commands.spawn((
        DirectionalLight {
            illuminance: 9_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(9.0, 12.0, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    // The fill: a cooler, dimmer light from the far side, casting no
    // shadows of its own, so the backs of things read instead of going
    // to pitch.
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.82, 0.86, 1.0),
            illuminance: 3_200.0,
            ..default()
        },
        Transform::from_xyz(-8.0, 6.0, -9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let matte = |materials: &mut Assets<StandardMaterial>, color: Color| {
        materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.95,
            reflectance: 0.03,
            ..default()
        })
    };

    // The bench floor: near-black, so authored colour reads true against it.
    let floor = matte(&mut materials, Color::srgb(0.05, 0.055, 0.065));
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(floor),
        Transform::from_xyz(0.0, -0.5, 0.0).with_scale(Vec3::new(
            REACH * 2.0 + 8.0,
            1.0,
            REACH * 2.0 + 8.0,
        )),
    ));

    // The metre grid, drawn in dim bone. Every fourth line stands brighter,
    // which is enough structure to count cells by eye.
    let faint = matte(&mut materials, palette.shade("bone", 0.3).with_alpha(1.0));
    let strong = matte(&mut materials, palette.shade("bone", 0.55));
    let span = REACH * 2.0;
    let lines = (REACH * 2.0) as i32 + 1;
    for i in 0..lines {
        let at = -REACH + i as f32;
        let bold = (at as i32) % 4 == 0;
        let material = if bold { strong.clone() } else { faint.clone() };
        let lift = if bold { 0.012 } else { 0.008 };
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(at, lift, 0.0).with_scale(Vec3::new(0.024, 0.016, span)),
        ));
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, lift, at).with_scale(Vec3::new(span, 0.016, 0.024)),
        ));
    }

    // The plot centre, marked with a gold cross: the origin every exported
    // part is measured from. Buildings need not be centred perfectly - the
    // god recentres on import - but the mark keeps the eye honest.
    for (sx, sz) in [(1.4, 0.05), (0.05, 1.4)] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(gold_center(&mut materials, &palette)),
            Transform::from_xyz(0.0, 0.02, 0.0).with_scale(Vec3::new(sx, 0.02, sz)),
        ));
    }

    // The door mark: buildings face the village with their local +X, so the
    // +X edge of the grid wears a gold sill. What you build toward the gold
    // is what the village sees first.
    let gold = matte(&mut materials, palette.shade("cloth-gold", 0.85));
    commands.spawn((
        BuilderFurniture,
        Mesh3d(cube.clone()),
        MeshMaterial3d(gold.clone()),
        Transform::from_xyz(REACH + 0.6, 0.05, 0.0).with_scale(Vec3::new(0.12, 0.1, 3.0)),
    ));

    raise_the_ruler(&mut commands, &mut meshes, &mut materials, &palette);

    // No pedestal at the rig bench. It had one when it held a body, and a model stands
    // on the GRID instead: the whole point of looking at a model here is judging its
    // true size, and a plinth a fifth of a metre high makes every reading off the metre
    // lines wrong by a fifth of a metre. `RigFurniture` is worn by the model itself now.
}

/// How tall the post stands, in metres. Two is a doorway, which is as much as anything
/// on this bench has ever needed and still short enough to see past.
const POST: i32 = 20;

/// Raises the measuring post.
///
/// Bands of a decimetre each, alternating, with every metre wearing a wider gold collar -
/// the same trick the floor grid uses, and for the same reason: you can count a stack of
/// ten by eye without counting at all.
///
/// It stands OFF to one side rather than at the origin, because a post through the middle
/// of the model is a post you cannot see past and the model cannot be judged around. Half
/// a metre out on both axes puts it clear of anything small and against the corner of
/// anything large.
fn raise_the_ruler(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
) {
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let matte = |materials: &mut Assets<StandardMaterial>, color: Color| {
        materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.95,
            reflectance: 0.03,
            ..default()
        })
    };
    let pale = matte(materials, palette.shade("bone", 0.75));
    let dark = matte(materials, palette.shade("bone", 0.28));
    let gold = matte(materials, palette.shade("cloth-gold", 0.8));
    let (at_x, at_z) = (0.5, 0.5);
    for band in 0..POST {
        let metre = band % 10 == 0;
        let material = if metre {
            gold.clone()
        } else if band % 2 == 0 {
            pale.clone()
        } else {
            dark.clone()
        };
        // A metre's collar stands proud of the post, so it reads as a mark rather than as
        // one more stripe among ten.
        let thick = if metre { 0.075 } else { 0.05 };
        commands.spawn((
            Ruler,
            Mesh3d(cube.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(at_x, band as f32 * 0.1 + 0.05, at_z)
                .with_scale(Vec3::new(thick, 0.1, thick)),
            Visibility::Hidden,
        ));
    }
}

fn gold_center(
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: palette.shade("cloth-gold", 0.7),
        perceptual_roughness: 0.95,
        reflectance: 0.03,
        ..default()
    })
}

/// Each bench keeps its own furniture on stage and the other's put away.
/// The ruler stands wherever the maker asked for it, at any bench - a thing to measure with
/// is not one bench's property, and a building wants measuring as much as a model does.
fn follow_the_ruler(
    showing: Res<crate::look::Showing>,
    mut posts: Query<&mut Visibility, With<Ruler>>,
) {
    if !showing.is_changed() {
        return;
    }
    let out = showing.wanted(crate::look::Tool::Ruler);
    for mut it in &mut posts {
        *it = if out {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn follow_bench(
    bench: Res<Bench>,
    mut builder: Query<&mut Visibility, (With<BuilderFurniture>, Without<RigFurniture>)>,
    mut rig: Query<&mut Visibility, With<RigFurniture>>,
    // Disjoint from BOTH the others, spelled out. Two `&mut Visibility` queries that
    // merely exclude the rig are not provably disjoint from each other - nothing says
    // an entity cannot wear two furniture markers - and Bevy refuses to run a system
    // whose parameters might overlap. It refused this one on the first launch.
    mut kiln: Query<
        &mut Visibility,
        (
            With<KilnFurniture>,
            Without<RigFurniture>,
            Without<BuilderFurniture>,
        ),
    >,
) {
    if !bench.is_changed() {
        return;
    }
    for mut visibility in &mut builder {
        *visibility = if *bench == Bench::Builder {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for mut visibility in &mut rig {
        *visibility = if *bench == Bench::Rig {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for mut visibility in &mut kiln {
        *visibility = if *bench == Bench::Kiln {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
