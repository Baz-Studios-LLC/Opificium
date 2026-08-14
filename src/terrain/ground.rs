//! The ground under the brush: a game's world, worked out the same way the game
//! works it out.
//!
//! # Why the recipe arrives as data
//!
//! The bench and the game must agree about the ground EXACTLY. A maker sculpts
//! offsets — how far the ground moved — and the game adds those to whatever it
//! generated itself. If the two disagree about what was underneath by so much as
//! a metre, every hill a maker placed sits at the wrong height in the game, and
//! nothing on screen says why.
//!
//! So the numbers are not written here. They arrive in `world.json`, exported by
//! the game, exactly as the palette does — see FORMATS.md. A game that has not
//! written one gets the defaults below and a line in the log saying so, which is
//! the ordinary case for a world being started from nothing rather than the
//! broken one.
//!
//! # What the map image is, and is not
//!
//! A **grayscale heightmap** carries real elevation. A **coloured political map**
//! does not: its brightness is region fill colours and means nothing as terrain.
//! It says where the continents are and nothing else. Which one it has is worked
//! out on sight, and said in the log.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use bevy::log::{info, warn};
use bevy::prelude::*;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use serde::Deserialize;

use crate::terrain::edit::{Sculpt, smoothstep};
use crate::terrain::settle::Settlements;

/// What the map image is called, inside the project's world folder.
pub const HEIGHTMAP: &str = "heightmap.png";
/// What the recipe is called, beside it.
pub const RECIPE: &str = "world.json";

/// How often the character of the coast changes, in cycles per metre. Low, so a
/// beach runs the better part of a kilometre before giving way to rock.
///
/// Not in the recipe: this decides how the ground is COLOURED, and colour has no
/// bearing on the offsets a maker sculpts. The game keeps the same number so the
/// two look alike, but nothing breaks if they drift.
const SHORE_FREQ: f64 = 0.000_6;

/// Everything about a world that the game and the bench must agree on.
///
/// Every field has a default, so a project with no file is still a world.
#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Recipe {
    /// How wide the world is, east to west, in metres. The one scale knob; the
    /// north-south extent comes from the map image's proportions.
    pub width: f32,
    /// Seeds every noise layer. Fixed, so the world is the same on every machine.
    pub seed: u32,
    /// How much bluer than red a pixel must be to be water, in 0-255 units.
    pub sea_blue_margin: i16,
    /// Brightness counting as the waterline, for grayscale maps only.
    pub sea_threshold: f32,
    /// Cleaning applied to the land mask, in map pixels.
    pub clean_radius: usize,
    pub clean_passes: usize,
    /// Smallest land blob kept, in map pixels.
    pub min_island_pixels: usize,
    /// Where the coastline fade begins, as a fraction of centre-to-border.
    pub coast_fade_start: f32,
    /// Land at the shoreline, and how far it climbs to the deep interior.
    pub coast_height: f32,
    pub inland_rise: f32,
    /// How far inland, in metres, the land takes to rise from the waterline to
    /// `coast_height`, and how far out to sea the floor takes to fall to
    /// `ocean_depth`.
    ///
    /// These exist because a coast has to shelve. Without them the whole drop
    /// from land to sea floor happens across the width of the mask's blur - a
    /// few metres - and no vertex grid can draw that: neighbouring columns land
    /// on opposite sides of it and the shoreline comes out as a picket fence of
    /// vertical slats. Spread over a beach and a shelf, the change per cell is
    /// small and the coast reads as a coast.
    pub beach_width: f32,
    pub shelf_width: f32,
    /// Distance from the coast, in metres, counting as fully inland.
    pub inland_full: f32,
    /// How deep the sea floor sits below the waterline.
    pub ocean_depth: f32,
    /// Relief a grayscale map's brightness is worth. Unused on a coloured map.
    pub base_elevation: f32,
    /// The mountains.
    pub range_elevation: f32,

    /// The one great mountain: how high it stands above the ground it sits on,
    /// and how far out its foot reaches.
    ///
    /// The world is otherwise deliberately gentle — plains and hills you walk
    /// over rather than around. That makes ONE massif worth more than a map full
    /// of them: it is visible from most of the continent, it is the thing you
    /// navigate by, and it is somewhere you decide to go. `range_elevation`
    /// stays low so this reads as the exception it is.
    ///
    /// Set `massif_height` to 0 for a world with no such landmark.
    pub massif_height: f32,
    pub massif_radius: f32,
    pub range_freq: f64,
    pub range_presence_freq: f64,
    pub range_presence_cutoff: f32,
    pub range_inland_start: f32,
    pub range_inland_full: f32,
    /// Fine detail, and the wander applied to the coastline.
    pub detail_elevation: f32,
    pub detail_freq: f64,
    pub warp_strength: f32,
    pub warp_freq: f64,
    /// How much of the world is rugged, and how much is level.
    ///
    /// A very low-frequency field, thresholded: below `rugged_low` the ground is
    /// plain — flat enough for forest, farmland and walking — and only above
    /// `rugged_high` does it get the full detail and the mountains. Without this
    /// every square metre of the map was equally lumpy, which leaves nowhere for
    /// anything to happen.
    pub rugged_freq: f64,
    pub rugged_low: f32,
    pub rugged_high: f32,
    /// Ruggedness left in the flattest country, so plains still read as ground
    /// rather than as a table.
    pub plains_relief: f32,

    // ----------------------------------------------------------- settlements
    /// How many of each kind of place gets ground levelled for it.
    pub cities: usize,
    pub towns: usize,
    /// How far the level ground reaches at each, in metres.
    pub city_radius: f32,
    pub town_radius: f32,
    /// How far apart they must stand.
    pub city_spacing: f32,
    pub town_spacing: f32,
    /// How far the levelling eases back into the surrounding land.
    pub site_skirt: f32,
    /// A site must be at least this far inland, below this height, and on ground
    /// no steeper than this.
    pub site_min_inland: f32,
    pub site_max_height: f32,
    pub site_max_slope: f32,
    /// Half-width of the graded road between sites, and its shoulders.
    pub road_width: f32,
    pub road_skirt: f32,

    // --------------------------------------------------------------- trees
    /// Metres between the slots a tree may stand in.
    ///
    /// The single knob for how thick a forest is, and the one to reach for
    /// first: trees go up as the SQUARE of this, so 14 m is a quarter the trees
    /// of 7 m. Wide enough that a wood reads as a wood and you can walk through
    /// it; narrow enough that it is not an orchard.
    pub tree_spacing: f32,
    /// Height at which trees give out. Below `massif_height`, so the great
    /// mountain stands bare above its own treeline.
    pub treeline: f32,
    /// How much bigger or smaller than grown a planted tree may be, so a stand
    /// has young trees and old ones in it.
    pub tree_scale_low: f32,
    pub tree_scale_high: f32,

    /// Damp all generated relief to nothing, leaving only sculpted ground. The
    /// shape-checking mode: the only thing to see is the outline of the coasts.
    pub flat: bool,
}

impl Default for Recipe {
    fn default() -> Self {
        Self {
            width: 8192.0,
            seed: 20_260_813,
            sea_blue_margin: 40,
            sea_threshold: 0.74,
            clean_radius: 3,
            clean_passes: 2,
            min_island_pixels: 900,
            coast_fade_start: 0.95,
            coast_height: 16.0,
            inland_rise: 28.0,
            beach_width: 90.0,
            shelf_width: 600.0,
            inland_full: 620.0,
            ocean_depth: 60.0,
            base_elevation: 110.0,
            range_elevation: 52.0,
            massif_height: 340.0,
            massif_radius: 950.0,
            tree_spacing: 14.0,
            treeline: 150.0,
            tree_scale_low: 0.75,
            tree_scale_high: 1.35,
            range_freq: 0.000_42,
            range_presence_freq: 0.000_35,
            range_presence_cutoff: 0.45,
            range_inland_start: 0.25,
            range_inland_full: 0.70,
            detail_elevation: 8.0,
            detail_freq: 0.009,
            warp_strength: 26.0,
            warp_freq: 0.004,
            rugged_freq: 0.000_25,
            rugged_low: 0.38,
            rugged_high: 0.72,
            plains_relief: 0.12,
            cities: 6,
            towns: 14,
            city_radius: 190.0,
            town_radius: 95.0,
            city_spacing: 1_100.0,
            town_spacing: 420.0,
            site_skirt: 140.0,
            site_min_inland: 70.0,
            site_max_height: 130.0,
            site_max_slope: 0.13,
            road_width: 9.0,
            road_skirt: 26.0,
            flat: false,
        }
    }
}

impl Recipe {
    /// Reads the recipe beside a world, or takes every default.
    pub fn read(folder: &Path) -> Self {
        let road = folder.join(RECIPE);
        match std::fs::read_to_string(&road) {
            Ok(text) => match serde_json::from_str::<Recipe>(&text) {
                Ok(recipe) => {
                    info!("world recipe: {}", road.display());
                    recipe
                }
                Err(why) => {
                    warn!("{}: {why} - taking the bench's own numbers", road.display());
                    Recipe::default()
                }
            },
            Err(_) => {
                info!(
                    "no {RECIPE} in {} - taking the bench's own numbers. A game that \
                     generates its ground differently exports one; see FORMATS.md",
                    folder.display()
                );
                Recipe::default()
            }
        }
    }
}

/// The world, shared with the threads that mesh it.
#[derive(Resource, Clone, Deref)]
pub struct Ground(pub Arc<World>);

pub struct World {
    /// The folder this world was opened from, and where its sculpting is kept.
    folder: PathBuf,
    map: Option<MapImage>,
    map_carries_elevation: bool,
    recipe: Recipe,
    /// Half-extents in metres. X east-west, Y north-south.
    half: Vec2,
    ranges: Fbm<Perlin>,
    presence: Fbm<Perlin>,
    /// Which stretches of coast are sand and which are rock.
    shores: Fbm<Perlin>,
    /// Which country is rugged and which is level.
    rugged: Fbm<Perlin>,
    detail: Fbm<Perlin>,
    warp_x: Perlin,
    warp_z: Perlin,
    /// The ground a maker put there.
    ///
    /// Behind a lock because chunks are meshed on background threads while the
    /// brush writes on the main one. Reads are short; writes happen only on the
    /// frames somebody is sculpting.
    sculpt: RwLock<Sculpt>,
    /// Ground levelled for towns, and the roads graded between them.
    settlements: Settlements,
    /// Where the one great mountain stands, if this world has one.
    massif: Option<Vec2>,
    /// Woods a maker painted in or cleared away.
    ///
    /// Behind a lock for the same reason the sculpting is: chunks are planted on
    /// background threads while the brush writes on the main one.
    forest: RwLock<crate::terrain::forest::Painted>,
}

impl World {
    /// Opens the world kept in a folder: its map, its recipe, and whatever has
    /// already been sculpted on it.
    pub fn open(folder: &Path) -> Self {
        let recipe = Recipe::read(folder);
        let map = MapImage::load(folder, &recipe);

        // The image decides the world's proportions; the recipe decides its
        // scale. A 2:1 map 8192 m across is 4096 m deep.
        let aspect = map.as_ref().map_or(2.0, MapImage::aspect);
        let half = Vec2::new(recipe.width * 0.5, recipe.width / aspect * 0.5);
        let map_carries_elevation = map.as_ref().is_some_and(|m| m.carries_elevation);
        let seed = recipe.seed;

        let mut world = Self {
            folder: folder.to_path_buf(),
            map,
            map_carries_elevation,
            half,
            // Two octaves, and the crest raised to a modest power later. NOT
            // ridged multifractal, and NOT squared: that combination creases at
            // every zero crossing and turns a mountain range into a row of
            // isolated teeth across the whole map. It has been tried twice.
            ranges: Fbm::<Perlin>::new(seed)
                .set_octaves(2)
                .set_frequency(1.0)
                .set_persistence(0.45),
            presence: Fbm::<Perlin>::new(seed.wrapping_add(7))
                .set_octaves(2)
                .set_frequency(1.0)
                .set_persistence(0.5),
            shores: Fbm::<Perlin>::new(seed.wrapping_add(9))
                .set_octaves(2)
                .set_frequency(1.0)
                .set_persistence(0.5),
            rugged: Fbm::<Perlin>::new(seed.wrapping_add(13))
                .set_octaves(2)
                .set_frequency(1.0)
                .set_persistence(0.5),
            detail: Fbm::<Perlin>::new(seed.wrapping_add(1))
                .set_octaves(4)
                .set_frequency(1.0),
            warp_x: Perlin::new(seed.wrapping_add(3)),
            warp_z: Perlin::new(seed.wrapping_add(4)),
            sculpt: RwLock::new(Sculpt::load(folder, half, seed.wrapping_add(11))),
            settlements: Settlements::nowhere(),
            massif: None,
            forest: RwLock::new(crate::terrain::forest::Painted::load(folder, half)),
            recipe,
        };

        // The great mountain goes in the heartland — the point furthest from any
        // sea. Found rather than chosen, so redrawing the map moves it to the
        // new map's interior instead of stranding it in a bay. Placed before the
        // towns, so their ground is judged against a world that already has it
        // and none of them ends up levelled onto its flank.
        world.massif = world.map.as_ref().and_then(|map| {
            (world.recipe.massif_height > 0.0).then(|| {
                let (u, v) = map.deepest_inland();
                let at = Vec2::new(
                    (u - 0.5) * half.x * 2.0,
                    (v - 0.5) * half.y * 2.0,
                );
                info!("the great mountain stands at {:.0}, {:.0}", at.x, at.y);
                at
            })
        });

        // Planned after the rest of the world exists, because choosing where a
        // town goes means asking how high and how steep the ground is there —
        // and answered with `raw_height`, which knows nothing of settlements, so
        // this is never reading back its own output.
        world.settlements = Settlements::plan(
            &world.recipe,
            half,
            &|at| world.raw_height(at.x, at.y),
            &|at| world.shore_metres(at.x, at.y),
        );
        info!(
            "planned {} places and {} roads between them",
            world.settlements.sites().len(),
            world.settlements.roads_len()
        );
        world
    }

    /// The woods a maker painted, for the brush to read and write.
    pub fn forest(&self) -> &RwLock<crate::terrain::forest::Painted> {
        &self.forest
    }

    /// Every tree standing in a patch of ground.
    ///
    /// Worked out from the ground and the painted layer rather than looked up in
    /// a list, so a chunk can be planted on its own, on any thread, in any
    /// order, and the game planting the same patch gets the same trees. Nothing
    /// about a tree is stored anywhere.
    pub fn trees_in(&self, low: Vec2, high: Vec2) -> Vec<crate::terrain::forest::Planted> {
        use crate::terrain::forest;

        let r = &self.recipe;
        let step = r.tree_spacing.max(1.0);
        let painted = self.forest.read().ok();

        // Slots are a world-wide lattice, not a per-chunk one, so a tree does
        // not move when the chunk boundaries around it change.
        let first = (low / step).floor().as_ivec2();
        let last = (high / step).ceil().as_ivec2();

        let mut standing = Vec::new();
        for slot_z in first.y..=last.y {
            for slot_x in first.x..=last.x {
                // Jittered off the lattice, or the wood comes out in rows.
                let jitter = Vec2::new(
                    forest::chance(slot_x, slot_z, 1) - 0.5,
                    forest::chance(slot_x, slot_z, 2) - 0.5,
                ) * step
                    * 0.85;
                let at = Vec2::new(slot_x as f32 * step, slot_z as f32 * step) + jitter;
                if at.x < low.x || at.x >= high.x || at.y < low.y || at.y >= high.y {
                    continue;
                }

                let shore = self.shore_metres(at.x, at.y);
                if shore < 25.0 {
                    continue;
                }
                let height = self.height(at.x, at.y);
                let slope = 1.0 - self.normal(at.x, at.y, step * 0.5).y;
                let levelled = self
                    .settlements
                    .level(at, r)
                    .map(|(_, weight)| weight)
                    .unwrap_or(0.0);

                let natural = forest::natural_density(
                    self.moisture(at.x, at.y),
                    height,
                    slope,
                    shore,
                    levelled,
                    r.treeline,
                );
                let painted_here = painted.as_ref().map_or(0.0, |p| p.at(at.x, at.y));
                let density = forest::density(natural, painted_here);
                if density <= 0.0 || forest::chance(slot_x, slot_z, 3) > density {
                    continue;
                }

                standing.push(forest::Planted {
                    at: Vec3::new(at.x, height, at.y),
                    variety: (forest::chance(slot_x, slot_z, 4) * crate::terrain::tree::VARIETIES
                        as f32) as usize
                        % crate::terrain::tree::VARIETIES,
                    turn: forest::chance(slot_x, slot_z, 5) * std::f32::consts::TAU,
                    scale: r.tree_scale_low
                        + (r.tree_scale_high - r.tree_scale_low)
                            * forest::chance(slot_x, slot_z, 6),
                });
            }
        }
        standing
    }

    /// Where the towns are, for anything that wants to draw or find them.
    pub fn sites(&self) -> &[crate::terrain::settle::Site] {
        self.settlements.sites()
    }

    /// The folder this world was opened from.
    pub fn folder(&self) -> &Path {
        &self.folder
    }

    pub fn half(&self) -> Vec2 {
        self.half
    }

    pub fn has_map(&self) -> bool {
        self.map.is_some()
    }

    /// The ground a maker put there, for the brush to read and write.
    pub fn sculpt(&self) -> &RwLock<Sculpt> {
        &self.sculpt
    }

    /// How high the ground is at a world position, sculpting included. Below
    /// zero is sea floor. The answer for anything that cares where the ground
    /// actually is.
    pub fn height(&self, x: f32, z: f32) -> f32 {
        let made = self.made_height(x, z);
        match self.sculpt.read() {
            Ok(sculpt) => made + sculpt.at(x, z),
            // A poisoned lock means a stroke panicked. The generated world is
            // still perfectly good, so keep drawing it rather than taking the
            // bench down.
            Err(_) => made,
        }
    }

    /// How high the ground is with the sculpting left out.
    ///
    /// The levelling brushes need this: what offset they want to write depends
    /// on what the ground was doing underneath, and they run while holding the
    /// lock over the sculpting, so they must not read back through it.
    pub fn made_height(&self, x: f32, z: f32) -> f32 {
        let h = self.raw_height(x, z);
        // Towns stand on level ground and roads are graded between them, so the
        // last word on height belongs to whatever has been levelled here.
        match self.settlements.level(Vec2::new(x, z), &self.recipe) {
            Some((target, pull)) => h + (target - h) * pull,
            None => h,
        }
    }

    /// The generated ground before any of it is levelled for people.
    ///
    /// Separate because planning where a town goes has to ask how high and how
    /// steep the ground is there, and asking `made_height` would be reading back
    /// the levelling it is in the middle of deciding.
    pub fn raw_height(&self, x: f32, z: f32) -> f32 {
        // Nudge the lookup by a low-frequency field. Without it a coastline
        // traced off an image reads as a run of straight pixel edges; with it,
        // the shore wanders the way a real one does.
        let (wx, wz) = self.warp(x, z);
        let r = &self.recipe;

        // Distance to the coast, positive inland and negative out to sea. The
        // whole landscape is built on this one number.
        let shore = self.shore_metres(wx, wz);

        // The coast shelves BOTH ways, each at its own rate: the land climbs a
        // beach's width to reach the shoreline height, and the floor falls a
        // shelf's width to reach the depths. They meet at zero, which is the
        // waterline. Anything faster than this cannot be drawn - neighbouring
        // vertices end up on opposite sides of the drop and the shore comes out
        // as a fence of vertical slats.
        let mut h = if shore >= 0.0 {
            r.coast_height * smoothstep(0.0, r.beach_width, shore)
        } else {
            -r.ocean_depth * smoothstep(0.0, r.shelf_width, -shore)
        };

        if r.flat {
            return h;
        }

        // 0 at the waterline and 1 once properly ashore. Everything generated is
        // masked by it, so nothing pokes out of the sea near a beach.
        let coast = smoothstep(0.0, r.beach_width, shore);
        if coast <= 0.0 {
            return h;
        }

        // How far inland, as 0 at the shore to 1 in the deep interior. This is
        // what makes the geography read as geography: plains by the water,
        // uplands behind them, mountains in the middle.
        let inland = (shore / r.inland_full).clamp(0.0, 1.0);
        h += smoothstep(0.0, 0.85, inland) * r.inland_rise * coast;

        if self.map_carries_elevation {
            h += self.map_elevation(wx, wz) * r.base_elevation * coast;
        }

        h += self.massif_height(wx, wz) * coast;

        // How rugged this country is, 0 plain to 1 mountainous. Mountains and
        // fine detail are both scaled by it, so most of the world is level
        // enough to walk, farm and put a forest on, and the rough ground is
        // somewhere in particular rather than everywhere at once.
        let rugged = self.ruggedness(wx, wz);

        h += self.range_height(wx, wz, inland) * coast * rugged;

        let d = self
            .detail
            .get([wx as f64 * r.detail_freq, wz as f64 * r.detail_freq]) as f32;
        h + d * r.detail_elevation * coast * (r.plains_relief + (1.0 - r.plains_relief) * rugged)
    }

    /// How rugged the country is here: 0 level plain, 1 full relief.
    fn ruggedness(&self, x: f32, z: f32) -> f32 {
        let r = &self.recipe;
        let n = self
            .rugged
            .get([x as f64 * r.rugged_freq, z as f64 * r.rugged_freq]) as f32
            * 0.5
            + 0.5;
        smoothstep(r.rugged_low, r.rugged_high, n)
    }

    /// What the one great mountain adds.
    ///
    /// A broad shoulder easing up to a peak, not a cone: the falloff is raised
    /// to a power so the foot spreads and the summit is the small part, which is
    /// how a massif reads from a distance. The ridge field warps it so the
    /// flanks have spurs and gullies rather than being a smooth dome, and the
    /// warp is scaled by height so the foot stays walkable while the top is
    /// broken up.
    fn massif_height(&self, x: f32, z: f32) -> f32 {
        let r = &self.recipe;
        let Some(peak) = self.massif else {
            return 0.0;
        };
        if r.massif_height <= 0.0 {
            return 0.0;
        }

        let away = peak.distance(Vec2::new(x, z));
        if away >= r.massif_radius {
            return 0.0;
        }

        let rise = smoothstep(r.massif_radius, 0.0, away).powf(1.9);
        let ridge = self
            .ranges
            .get([x as f64 * r.range_freq * 3.0, z as f64 * r.range_freq * 3.0])
            as f32;
        rise * r.massif_height * (1.0 + ridge * 0.22 * rise)
    }

    /// What the mountains add.
    ///
    /// Three things must agree before any mountain exists: a very low-frequency
    /// PRESENCE field, hard-thresholded, so ranges occupy a few regions rather
    /// than being the map's texture; distance INLAND, because a range rising
    /// straight out of the sea reads as a mistake; and a RIDGE line, where
    /// `1 - |noise|` creases into a crest that runs for kilometres.
    fn range_height(&self, x: f32, z: f32, inland: f32) -> f32 {
        let r = &self.recipe;
        let allowed = smoothstep(r.range_inland_start, r.range_inland_full, inland);
        if allowed <= 0.0 {
            return 0.0;
        }

        let presence = self
            .presence
            .get([x as f64 * r.range_presence_freq, z as f64 * r.range_presence_freq])
            as f32
            * 0.5
            + 0.5;
        let presence = smoothstep(r.range_presence_cutoff, 1.0, presence);
        if presence <= 0.0 {
            return 0.0;
        }

        let n = self.ranges.get([x as f64 * r.range_freq, z as f64 * r.range_freq]) as f32;
        let crest = (1.0 - n.abs()).clamp(0.0, 1.0).powf(1.7);

        crest * presence * allowed * r.range_elevation
    }

    /// The surface normal, from differences either side.
    ///
    /// Worked out from the height field rather than from mesh triangles on
    /// purpose: it depends only on world position, so two neighbouring chunks
    /// derive IDENTICAL normals along the edge they share and stitch together
    /// with no seam in the lighting.
    pub fn normal(&self, x: f32, z: f32, step: f32) -> Vec3 {
        let dx = self.height(x + step, z) - self.height(x - step, z);
        let dz = self.height(x, z + step) - self.height(x, z - step);
        Vec3::new(-dx, 2.0 * step, -dz).normalize()
    }

    /// How wet it is, 0 arid to 1 lush. Colours the ground.
    pub fn moisture(&self, x: f32, z: f32) -> f32 {
        let m = self.detail.get([x as f64 * 0.000_9, z as f64 * 0.000_9]) as f32;
        (m * 0.5 + 0.5).clamp(0.0, 1.0)
    }

    /// What KIND of coast this stretch is: 0 rock, 1 sand.
    ///
    /// Sand is not the default state of a shoreline. A coast is beach where the
    /// sea has somewhere to put sediment and rock where it has not, and which it
    /// is changes along the coast rather than being true of the whole map. A
    /// world where every continent is outlined in sand reads as a drawing of a
    /// map rather than as ground.
    ///
    /// Low frequency, so a beach runs for the better part of a kilometre and
    /// then gives way, instead of speckling.
    pub fn shore_character(&self, x: f32, z: f32) -> f32 {
        let n = self
            .shores
            .get([x as f64 * SHORE_FREQ, z as f64 * SHORE_FREQ]) as f32
            * 0.5
            + 0.5;
        smoothstep(0.40, 0.62, n)
    }

    fn warp(&self, x: f32, z: f32) -> (f32, f32) {
        let r = &self.recipe;
        let (u, v) = (x as f64 * r.warp_freq, z as f64 * r.warp_freq);
        (
            x + self.warp_x.get([u, v]) as f32 * r.warp_strength,
            z + self.warp_z.get([u, v]) as f32 * r.warp_strength,
        )
    }

    /// Image space from world space: 0..1 from the west edge and from the north.
    fn map_uv(&self, x: f32, z: f32) -> (f32, f32) {
        (
            (x + self.half.x) / (self.half.x * 2.0),
            (z + self.half.y) / (self.half.y * 2.0),
        )
    }

    /// Pulls land under water at the very edge of the world.
    ///
    /// The world ends in WATER, never a wall, and that has to hold whatever the
    /// source image shows at its own margins — a screenshot's toolbar lives
    /// exactly there. Kept tight to the border so it trims furniture rather than
    /// real coastline.
    fn border_fade(&self, x: f32, z: f32) -> f32 {
        let away = (x.abs() / self.half.x).max(z.abs() / self.half.y);
        smoothstep(1.0, self.recipe.coast_fade_start, away)
    }

    fn map_elevation(&self, x: f32, z: f32) -> f32 {
        match &self.map {
            Some(map) => {
                let (u, v) = self.map_uv(x, z);
                map.elevation(u, v)
            }
            None => 0.0,
        }
    }

    /// Distance to the coast in metres: **positive inland, negative out to sea**.
    ///
    /// The one number the whole landscape is built on. It crosses zero exactly
    /// at the shoreline and changes smoothly through it, which is what lets the
    /// land rise and the sea floor fall at their own separate rates instead of
    /// meeting at a cliff.
    pub fn shore_metres(&self, x: f32, z: f32) -> f32 {
        let Some(map) = &self.map else {
            // Nothing to shape: open sea everywhere.
            return -self.recipe.shelf_width;
        };
        let (u, v) = self.map_uv(x, z);
        let per_pixel = self.half.x * 2.0 / map.wide as f32;
        let shore = (map.inland_pixels(u, v) - map.offshore_pixels(u, v)) * per_pixel;

        // The world ends in water, whatever the image shows at its own margins -
        // a screenshot's toolbar lives exactly there. Carried far out to sea
        // rather than merely lowered, so the border is ocean and not a shelf.
        let fade = self.border_fade(x, z);
        shore * fade - (1.0 - fade) * self.recipe.shelf_width
    }
}

// ------------------------------------------------------------------ the image

pub struct MapImage {
    wide: usize,
    deep: usize,
    /// Normalised brightness, row-major, north row first.
    elevation: Vec<f32>,
    /// Distance to the nearest sea pixel, in map pixels. 0 at sea, rising inland.
    inland: Vec<f32>,
    /// Distance to the nearest land pixel, in map pixels. 0 on land, rising out
    /// to sea. Subtracted from `inland` this is a signed distance to the coast.
    offshore: Vec<f32>,
    /// Whether the brightness is relief rather than region fill colours.
    carries_elevation: bool,
}

/// How different blue and red must be for an image to count as coloured, and
/// the share of pixels that must manage it.
const COLOUR_EVIDENCE: i16 = 20;
const COLOUR_FRACTION: f32 = 0.02;

impl MapImage {
    fn load(folder: &Path, recipe: &Recipe) -> Option<Self> {
        let road = folder.join(HEIGHTMAP);
        let picture = match image::open(&road) {
            Ok(picture) => picture,
            Err(why) => {
                warn!(
                    "no {HEIGHTMAP} in {} ({why}) - the bench has nothing to shape. \
                     See FORMATS.md",
                    folder.display()
                );
                return None;
            }
        };

        let rgb = picture.to_rgb8();
        let (wide, deep) = (rgb.width() as usize, rgb.height() as usize);
        if wide < 2 || deep < 2 {
            warn!("{} is too small ({wide}x{deep})", road.display());
            return None;
        }

        let pixels: Vec<[u8; 3]> = rgb.pixels().map(|p| p.0).collect();
        let mut elevation: Vec<f32> = pixels.iter().map(|p| luma(*p)).collect();

        // Stretched between the half-percentiles rather than the true extremes.
        // A map export carries things that are not terrain — black label text, a
        // white scale bar, a browser's chrome caught in a screenshot — and one
        // black pixel would otherwise anchor the bottom of the range and squash
        // everything real into the top of it.
        let (low, high) = percentile_range(&elevation, 0.005);
        if high - low > 1.0e-4 {
            let scale = 1.0 / (high - low);
            for v in &mut elevation {
                *v = ((*v - low) * scale).clamp(0.0, 1.0);
            }
        }

        let (sea, carries_elevation) = classify_sea(&pixels, &elevation, recipe);
        let land = clean_mask(&sea, wide, deep, recipe);
        let inland = shore_distance(&land, wide, deep, false);
        let offshore = shore_distance(&land, wide, deep, true);

        let land_share = land.iter().filter(|&&v| v == 1).count() as f32 / land.len() as f32;
        info!(
            "world map {wide}x{deep} from {} ({:.0}% land)",
            road.display(),
            land_share * 100.0
        );

        Some(Self {
            wide,
            deep,
            elevation,
            inland,
            offshore,
            carries_elevation,
        })
    }

    fn aspect(&self) -> f32 {
        self.wide as f32 / self.deep as f32
    }

    fn elevation(&self, u: f32, v: f32) -> f32 {
        self.read(&self.elevation, u, v)
    }

    fn offshore_pixels(&self, u: f32, v: f32) -> f32 {
        self.read(&self.offshore, u, v)
    }

    /// Where the map is furthest from any sea, in image space.
    ///
    /// The heart of the largest landmass, and so where a massif belongs: a
    /// mountain wants the most land around it, and the deepest interior is by
    /// definition the point with the most. Found rather than chosen, so
    /// redrawing the map moves the mountain to the new map's heartland instead
    /// of stranding it in a bay.
    fn deepest_inland(&self) -> (f32, f32) {
        let mut best = 0.0;
        let mut at = (0.5, 0.5);
        for (i, &away) in self.inland.iter().enumerate() {
            if away > best {
                best = away;
                at = (
                    (i % self.wide) as f32 / (self.wide - 1) as f32,
                    (i / self.wide) as f32 / (self.deep - 1) as f32,
                );
            }
        }
        at
    }

    fn inland_pixels(&self, u: f32, v: f32) -> f32 {
        self.read(&self.inland, u, v)
    }

    /// Reads between pixels. Anything outside the image is 0, which is what
    /// makes the world finite: sail far enough and there is only sea.
    fn read(&self, field: &[f32], u: f32, v: f32) -> f32 {
        if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
            return 0.0;
        }
        let fx = (u * self.wide as f32 - 0.5).clamp(0.0, self.wide as f32 - 1.0);
        let fy = (v * self.deep as f32 - 0.5).clamp(0.0, self.deep as f32 - 1.0);

        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1).min(self.wide - 1);
        let y1 = (y0 + 1).min(self.deep - 1);
        let tx = fx - x0 as f32;
        let ty = fy - y0 as f32;

        let cell = |x: usize, y: usize| field[y * self.wide + x];
        let near = cell(x0, y0) * (1.0 - tx) + cell(x1, y0) * tx;
        let far = cell(x0, y1) * (1.0 - tx) + cell(x1, y1) * tx;
        near * (1.0 - ty) + far * ty
    }
}

fn luma(p: [u8; 3]) -> f32 {
    (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0
}

/// Which pixels are sea, and whether the image carries real elevation.
///
/// **A coloured map is read by HUE, not brightness.** Brightness cannot tell
/// open water from a black place name, a road, or a dashed border — every one of
/// them is dark — so a brightness threshold cuts every label on the map into the
/// terrain as a lake. Water is the one thing on a political map that is
/// distinctly BLUE, so that is what gets asked: blue meaningfully greater than
/// red. Labels and borders are neutral or warm and stay land.
///
/// A genuine grayscale heightmap has no hue to ask about, so it is spotted and
/// thresholded on brightness instead.
fn classify_sea(pixels: &[[u8; 3]], elevation: &[f32], recipe: &Recipe) -> (Vec<u8>, bool) {
    let coloured = pixels
        .iter()
        .filter(|p| (p[2] as i16 - p[0] as i16).abs() > COLOUR_EVIDENCE)
        .count() as f32
        / pixels.len() as f32;

    if coloured < COLOUR_FRACTION {
        info!("the map is grayscale - brightness is the waterline and the relief");
        let sea = elevation
            .iter()
            .map(|&v| u8::from(v <= recipe.sea_threshold))
            .collect();
        return (sea, true);
    }

    info!("the map is coloured - blue is the waterline, brightness means nothing");
    let sea = pixels
        .iter()
        .map(|p| u8::from(p[2] as i16 - p[0] as i16 > recipe.sea_blue_margin))
        .collect();
    (sea, false)
}

/// Erases what is drawn on the map but is not the map, and returns land.
fn clean_mask(sea: &[u8], wide: usize, deep: usize, recipe: &Recipe) -> Vec<u8> {
    let mut land: Vec<u8> = sea.iter().map(|&s| 1 - s).collect();
    // Each pixel becomes whatever most of its neighbourhood is. A river or a
    // border a few pixels wide is outvoted by the land around it and goes; a
    // coastline has land one side and sea the other all the way along, so it
    // holds its position exactly.
    for _ in 0..recipe.clean_passes {
        land = majority_filter(&land, wide, deep, recipe.clean_radius);
    }
    drop_small_islands(&mut land, wide, deep, recipe.min_island_pixels);
    land
}

/// Deletes land too small to be land.
///
/// A map image is rarely only a map. A screenshot carries the tool's own
/// furniture — buttons, a scale bar, a legend — and none of it is water-coloured,
/// so it survives every test above and becomes little rectangular islands out at
/// sea. Real islands are far larger, so size alone separates them.
fn drop_small_islands(land: &mut [u8], wide: usize, deep: usize, floor: usize) {
    let mut seen = vec![false; land.len()];
    let mut blob = Vec::new();
    let mut queue = VecDeque::new();

    for start in 0..land.len() {
        if land[start] == 0 || seen[start] {
            continue;
        }
        blob.clear();
        queue.push_back(start);
        seen[start] = true;

        while let Some(at) = queue.pop_front() {
            blob.push(at);
            let (x, y) = (at % wide, at / wide);
            let mut walk = |nx: usize, ny: usize, queue: &mut VecDeque<usize>| {
                let next = ny * wide + nx;
                if land[next] == 1 && !seen[next] {
                    seen[next] = true;
                    queue.push_back(next);
                }
            };
            if x > 0 {
                walk(x - 1, y, &mut queue);
            }
            if x + 1 < wide {
                walk(x + 1, y, &mut queue);
            }
            if y > 0 {
                walk(x, y - 1, &mut queue);
            }
            if y + 1 < deep {
                walk(x, y + 1, &mut queue);
            }
        }

        if blob.len() < floor {
            for &at in &blob {
                land[at] = 0;
            }
        }
    }
}

/// How far every pixel is from the coast, measured one way.
///
/// `offshore` false gives each land pixel its distance from the sea - which is
/// what makes mountains geography rather than noise, since ranges belong in the
/// interior with plains between them and the water. `offshore` true gives each
/// sea pixel its distance from the land, which is what lets the sea floor fall
/// away gradually instead of dropping off a step at the shoreline.
///
/// Together they are a signed distance to the coast, and that is what the
/// terrain is actually built on. One breadth-first sweep each, once, when the
/// world opens.
fn shore_distance(land: &[u8], wide: usize, deep: usize, offshore: bool) -> Vec<f32> {
    let mut away = vec![f32::MAX; land.len()];
    let mut queue = VecDeque::new();

    // The far side is where the measuring starts: sea when measuring inland,
    // land when measuring out to sea.
    let source = u8::from(offshore);
    for y in 0..deep {
        for x in 0..wide {
            let at = y * wide + x;
            // Land running off the image border is not infinitely inland, so
            // the border seeds too - but only when measuring inland, or the
            // open sea at the edge of the map would read as touching land.
            let edge = !offshore && (x == 0 || y == 0 || x == wide - 1 || y == deep - 1);
            if land[at] == source || edge {
                away[at] = 0.0;
                queue.push_back(at);
            }
        }
    }

    while let Some(at) = queue.pop_front() {
        let (x, y) = (at % wide, at / wide);
        let next = away[at] + 1.0;
        let mut walk = |nx: usize, ny: usize, queue: &mut VecDeque<usize>| {
            let to = ny * wide + nx;
            if away[to] > next {
                away[to] = next;
                queue.push_back(to);
            }
        };
        if x > 0 {
            walk(x - 1, y, &mut queue);
        }
        if x + 1 < wide {
            walk(x + 1, y, &mut queue);
        }
        if y > 0 {
            walk(x, y - 1, &mut queue);
        }
        if y + 1 < deep {
            walk(x, y + 1, &mut queue);
        }
    }

    away
}

/// A summed-area table, so every box below is four lookups whatever its size.
fn integral(mask: &[u8], wide: usize, deep: usize) -> Vec<u32> {
    let stride = wide + 1;
    let mut table = vec![0u32; stride * (deep + 1)];
    for y in 0..deep {
        let mut row = 0u32;
        for x in 0..wide {
            row += mask[y * wide + x] as u32;
            table[(y + 1) * stride + x + 1] = table[y * stride + x + 1] + row;
        }
    }
    table
}

fn box_stats(
    table: &[u32],
    wide: usize,
    deep: usize,
    x: usize,
    y: usize,
    radius: usize,
) -> (u32, u32) {
    let stride = wide + 1;
    let x0 = x.saturating_sub(radius);
    let y0 = y.saturating_sub(radius);
    let x1 = (x + radius).min(wide - 1);
    let y1 = (y + radius).min(deep - 1);

    let total = table[(y1 + 1) * stride + x1 + 1] + table[y0 * stride + x0]
        - table[y0 * stride + x1 + 1]
        - table[(y1 + 1) * stride + x0];
    (total, ((x1 - x0 + 1) * (y1 - y0 + 1)) as u32)
}

fn majority_filter(mask: &[u8], wide: usize, deep: usize, radius: usize) -> Vec<u8> {
    let table = integral(mask, wide, deep);
    let mut out = vec![0u8; wide * deep];
    for y in 0..deep {
        for x in 0..wide {
            let (total, area) = box_stats(&table, wide, deep, x, y, radius);
            out[y * wide + x] = u8::from(total * 2 > area);
        }
    }
    out
}

/// The brightness range with `tail` of the pixels left off each end. A
/// 256-bucket count rather than sorting three million numbers.
fn percentile_range(samples: &[f32], tail: f32) -> (f32, f32) {
    const BUCKETS: usize = 256;
    let mut counts = [0usize; BUCKETS];
    for &v in samples {
        let bucket = ((v * (BUCKETS - 1) as f32).round() as usize).min(BUCKETS - 1);
        counts[bucket] += 1;
    }

    let cutoff = (samples.len() as f32 * tail) as usize;
    let value_of = |bucket: usize| bucket as f32 / (BUCKETS - 1) as f32;

    let mut running = 0;
    let mut low = 0.0;
    for (bucket, count) in counts.iter().enumerate() {
        running += count;
        if running > cutoff {
            low = value_of(bucket);
            break;
        }
    }

    let mut running = 0;
    let mut high = 1.0;
    for (bucket, count) in counts.iter().enumerate().rev() {
        running += count;
        if running > cutoff {
            high = value_of(bucket);
            break;
        }
    }

    (low, high)
}
