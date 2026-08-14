//! A MODEL as a file: where they live, how big they are, and how to stand one up.
//!
//! Shared, because a model is nobody's bench in particular. The kiln commissions one
//! and the rig looks at one closely, and both need the same three answers - what is in
//! the folder, how tall is this file, and what does Bevy need to draw it. Putting them
//! here rather than in whichever bench happened to need them first is the same rule
//! the rest of the bench follows: a tool that two benches use belongs to neither.
//!
//! A model is NOT a part. A part is a name resolved into boxes on the 1/16 m lattice
//! and painted from a ramp; a model is arbitrary triangles wearing their own materials,
//! which cannot be painted, cut to the lattice, or written into a baked building's
//! `boxes`. So models are kept as FILES and a game loads them whole.

use std::path::{Path, PathBuf};

use bevy::prelude::*;

/// Where a finished model is kept: the project's own folder.
///
/// Beside the drawings rather than among them - a `.glb` is not a work the bench can
/// reopen and change, it is the finished thing. The bake carries it into the game
/// from here.
pub fn home() -> PathBuf {
    crate::project::root().join("out/models")
}

/// Every model this project holds, newest first.
///
/// Newest first because the one a maker wants is nearly always the one just made, and
/// a folder that has been worked in for a month sorted by name buries it.
///
/// A folder that cannot be read is an EMPTY shelf rather than an error: a project that
/// has never fired the kiln has no `out/models` at all, which is not a fault.
pub fn all() -> Vec<PathBuf> {
    all_in(&home())
}

/// The pure half, so the order can be pinned without a project.
pub fn all_in(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut found: Vec<(std::time::SystemTime, PathBuf)> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|road| road.extension().is_some_and(|kind| kind == "glb"))
        .filter_map(|road| {
            let when = road.metadata().and_then(|it| it.modified()).ok()?;
            Some((when, road))
        })
        .collect();
    found.sort_by(|a, b| b.0.cmp(&a.0));
    found.into_iter().map(|(_, road)| road).collect()
}

/// What a model is called on a shelf: its file name without the `.glb`.
pub fn name_of(road: &Path) -> String {
    road.file_stem()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Stands a model up, with its feet on the floor.
///
/// `tall` is the height in metres to fit it to, or `None` to show it at whatever size
/// the file itself says - which is the right answer for a model the bench already
/// fitted and kept, since the fit is baked into the file.
///
/// A generated model's origin is wherever the machine left it, usually the middle of
/// the thing, so standing that origin on the floor buries half of it. The lift is by
/// the model's own lowest point, AFTER scaling, that being the distance in the stage's
/// metres rather than the model's own units.
///
/// A file that cannot be measured is stood unscaled rather than not at all: seeing it
/// wrong beats not seeing it.
pub fn stand(assets: &AssetServer, road: &Path, tall: Option<f32>) -> impl Bundle {
    let size = bounds_of(road);
    let low = size.map(|size| size.low.y).unwrap_or(0.0);
    let fit = match (tall, size) {
        (Some(tall), Some(size)) if size.tall() > 1e-4 => tall / size.tall(),
        _ => 1.0,
    };
    let name = road
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    (
        // `WorldAssetRoot` in this Bevy, renamed from `SceneRoot` when the next scene
        // system took the word "scene" for itself. The label is the glTF convention:
        // the file's first scene.
        //
        // `project://` is the second asset root - see `main` - so this loads out of
        // whichever game is open rather than out of the bench's own folder.
        bevy::world_serialization::WorldAssetRoot(
            assets.load(format!("project://out/models/{name}#Scene0")),
        ),
        Transform::from_xyz(0.0, -low * fit, 0.0).with_scale(Vec3::splat(fit)),
    )
}

/// Writes a GLB out at a chosen height, under a chosen name.
///
/// The SCALE GOES INTO THE FILE, rather than beside it in a note the game has to
/// read. A model that is the right size is a model a game can load and forget; a
/// model plus a number is two things to keep together, and one of them will go
/// missing.
///
/// It is done by wrapping, not by editing: a new node carries the scale and adopts
/// whatever the scene's roots were. Rewriting the existing nodes would mean
/// composing with transforms already on them, and a bench that starts composing
/// glTF node trees has become a glTF editor by accident.
pub fn keep_at_height(from: &Path, to: &Path, tall: f32) -> Result<PathBuf, String> {
    let bytes = std::fs::read(from).map_err(|why| format!("{}: {why}", from.display()))?;
    let mut doc = the_json_of(&bytes).ok_or("that file is not a GLB")?;
    let size = bounds_of(from).ok_or("cannot measure that model, so cannot fit it")?;
    let (low, fit) = (size.low.y, tall / size.tall());

    let nodes = doc
        .get_mut("nodes")
        .and_then(|nodes| nodes.as_array_mut())
        .ok_or("that GLB has no nodes")?;
    let wrapper = nodes.len();
    let scenes = doc
        .get("scenes")
        .and_then(|scenes| scenes.as_array())
        .ok_or("that GLB has no scenes")?;
    let roots: Vec<serde_json::Value> = scenes
        .first()
        .and_then(|scene| scene.get("nodes"))
        .and_then(|nodes| nodes.as_array())
        .cloned()
        .unwrap_or_default();

    doc["nodes"]
        .as_array_mut()
        .expect("just read")
        .push(serde_json::json!({
            "name": "opificium-fit",
            // Standing ON its origin, not straddling it, for the same reason the
            // height is baked in: a game that has to know a model's mesh sits 40cm
            // below its own origin is a game keeping a second fact about the file.
            // Everything else this bench makes is authored from the ground up, and a
            // model it keeps should load the same way.
            //
            // glTF applies scale before translation, so this is in fitted metres.
            "translation": [0.0, -low * fit, 0.0],
            "scale": [fit, fit, fit],
            "children": roots,
        }));
    doc["scenes"].as_array_mut().expect("just read")[0]["nodes"] = serde_json::json!([wrapper]);

    // A GLB is chunks with their own lengths, each padded to four bytes - the JSON
    // with spaces, the binary with zeroes - and a total length in the header that has
    // to agree with the file. Every one of those is recomputed rather than adjusted:
    // a length that disagrees by one byte is a file every loader refuses.
    let mut json = serde_json::to_vec(&doc).map_err(|why| format!("{why}"))?;
    while json.len() % 4 != 0 {
        json.push(b' ');
    }
    let binary = the_binary_of(&bytes).unwrap_or_default();
    let mut out = Vec::with_capacity(28 + json.len() + binary.len());
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    let total = 12
        + 8
        + json.len()
        + if binary.is_empty() {
            0
        } else {
            8 + binary.len()
        };
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(json.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&json);
    if !binary.is_empty() {
        out.extend_from_slice(&(binary.len() as u32).to_le_bytes());
        out.extend_from_slice(b"BIN\0");
        out.extend_from_slice(&binary);
    }

    let road = to.to_path_buf();
    // The folder, first. A firing makes it on the way past, but keeping a model is
    // not always preceded by one - a project opened fresh and handed a file has
    // nowhere to put it yet.
    if let Some(under) = road.parent() {
        std::fs::create_dir_all(under).map_err(|why| format!("{}: {why}", under.display()))?;
    }
    std::fs::write(&road, out).map_err(|why| format!("{}: {why}", road.display()))?;
    Ok(road)
}

/// The binary chunk of a GLB, if it has one. Padded to four bytes already, by
/// whoever wrote it.
fn the_binary_of(bytes: &[u8]) -> Option<Vec<u8>> {
    let json = u32::from_le_bytes(bytes.get(12..16)?.try_into().ok()?) as usize;
    let at = 20 + json;
    let len = u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?) as usize;
    if bytes.get(at + 4..at + 8)? != b"BIN\0" {
        return None;
    }
    bytes.get(at + 8..at + 8 + len).map(<[u8]>::to_vec)
}

/// Where a GLB's own geometry sits and how tall it is, in its own units: the lowest
/// point, and the height above it.
///
/// Read out of the file rather than guessed. Every `POSITION` accessor carries a `min`
/// and a `max` - the spec requires it - so both are known without decoding a vertex.
///
/// What comes back is NORMALISED: the provider fits every model into a unit box, so the
/// longest axis is exactly 1.0 whatever the subject. Measured across three real firings, a
/// housefly and a two-seater sofa both arrived the same size. So a model's own numbers say
/// what SHAPE it is and nothing whatever about how big the thing is - which is why the
/// height is a maker's to state and cannot be read off the file or left to the game.
///
/// The LOWEST POINT matters as much as the height, because a generated model's origin
/// is wherever the machine left it, which is usually the middle of the thing. Standing
/// such a model at the floor buries half of it.
///
/// Both are spans of the whole model, not the largest of each primitive: a mesh cut
/// into a body and two wings has each part measuring short, and the model is as tall
/// as the distance from the lowest of them to the highest.
///
/// NODE TRANSFORMS ARE APPLIED, which they must be: this bench writes some itself. A model
/// kept at a stated height carries a wrapper node holding the scale and the lift, so
/// reading the raw accessors would report a fitted couch as half a metre tall and stand it
/// floating, lifted once by its wrapper and again by the bench.
fn bounds_of_doc(doc: &serde_json::Value) -> Option<Size> {
    let mut low = Vec3::splat(f32::MAX);
    let mut high = Vec3::splat(f32::MIN);
    let scene = doc.get("scene").and_then(|at| at.as_u64()).unwrap_or(0) as usize;
    let roots = doc
        .get("scenes")?
        .as_array()?
        .get(scene)?
        .get("nodes")?
        .as_array()?;
    for root in roots {
        walk_a_node(
            doc,
            root.as_u64()? as usize,
            Mat4::IDENTITY,
            0,
            &mut low,
            &mut high,
        );
    }
    let size = Size { low, high };
    (size.tall() > 1e-6).then_some(size)
}

pub fn bounds_of(road: &Path) -> Option<Size> {
    let bytes = std::fs::read(road).ok()?;
    bounds_of_doc(&the_json_of(&bytes)?)
}

/// Grows the box to hold whatever this node and its children draw.
///
/// The eight CORNERS of each primitive's own box are carried through the transform rather
/// than its two extremes, because a rotation turns a box into a box with different extremes
/// and taking the min and max of two rotated corners describes neither.
fn walk_a_node(
    doc: &serde_json::Value,
    at: usize,
    above: Mat4,
    deep: u32,
    low: &mut Vec3,
    high: &mut Vec3,
) {
    // A file may name itself its own child. Nothing legitimate is nested this far.
    if deep > 32 {
        return;
    }
    let Some(node) = doc
        .get("nodes")
        .and_then(|nodes| nodes.as_array())
        .and_then(|nodes| nodes.get(at))
    else {
        return;
    };
    let here = above * its_own_transform(node);
    if let Some(mesh) = node.get("mesh").and_then(|mesh| mesh.as_u64())
        && let Some(prims) = doc
            .get("meshes")
            .and_then(|meshes| meshes.as_array())
            .and_then(|meshes| meshes.get(mesh as usize))
            .and_then(|mesh| mesh.get("primitives"))
            .and_then(|prims| prims.as_array())
    {
        for prim in prims {
            let Some(corners) = the_box_of(doc, prim) else {
                continue;
            };
            for corner in corners {
                let point = here.transform_point3(corner);
                *low = low.min(point);
                *high = high.max(point);
            }
        }
    }
    if let Some(children) = node.get("children").and_then(|kids| kids.as_array()) {
        for child in children {
            if let Some(child) = child.as_u64() {
                walk_a_node(doc, child as usize, here, deep + 1, low, high);
            }
        }
    }
}

/// The eight corners of a primitive's own bounding box, from the accessor the spec
/// requires to carry them.
fn the_box_of(doc: &serde_json::Value, prim: &serde_json::Value) -> Option<[Vec3; 8]> {
    let at = prim.get("attributes")?.get("POSITION")?.as_u64()? as usize;
    let accessor = doc.get("accessors")?.as_array()?.get(at)?;
    let corner = |which: &str| -> Option<Vec3> {
        let said = accessor.get(which)?.as_array()?;
        Some(Vec3::new(
            said.first()?.as_f64()? as f32,
            said.get(1)?.as_f64()? as f32,
            said.get(2)?.as_f64()? as f32,
        ))
    };
    let (low, high) = (corner("min")?, corner("max")?);
    Some([
        Vec3::new(low.x, low.y, low.z),
        Vec3::new(high.x, low.y, low.z),
        Vec3::new(low.x, high.y, low.z),
        Vec3::new(low.x, low.y, high.z),
        Vec3::new(high.x, high.y, low.z),
        Vec3::new(high.x, low.y, high.z),
        Vec3::new(low.x, high.y, high.z),
        Vec3::new(high.x, high.y, high.z),
    ])
}

/// A node's own transform: either a matrix outright, or the scale-rotate-translate the
/// spec says to compose in that order.
fn its_own_transform(node: &serde_json::Value) -> Mat4 {
    if let Some(said) = node.get("matrix").and_then(|m| m.as_array())
        && said.len() == 16
    {
        let mut cells = [0.0f32; 16];
        for (cell, said) in cells.iter_mut().zip(said) {
            *cell = said.as_f64().unwrap_or(0.0) as f32;
        }
        // glTF writes a matrix in column-major order, which is how `Mat4` reads one.
        return Mat4::from_cols_array(&cells);
    }
    let three = |which: &str, fallback: f32| -> Vec3 {
        node.get(which)
            .and_then(|it| it.as_array())
            .filter(|it| it.len() == 3)
            .map(|it| {
                Vec3::new(
                    it[0].as_f64().unwrap_or(fallback as f64) as f32,
                    it[1].as_f64().unwrap_or(fallback as f64) as f32,
                    it[2].as_f64().unwrap_or(fallback as f64) as f32,
                )
            })
            .unwrap_or(Vec3::splat(fallback))
    };
    let turn = node
        .get("rotation")
        .and_then(|it| it.as_array())
        .filter(|it| it.len() == 4)
        .map(|it| {
            // glTF orders a quaternion x, y, z, w.
            Quat::from_xyzw(
                it[0].as_f64().unwrap_or(0.0) as f32,
                it[1].as_f64().unwrap_or(0.0) as f32,
                it[2].as_f64().unwrap_or(0.0) as f32,
                it[3].as_f64().unwrap_or(1.0) as f32,
            )
        })
        .unwrap_or(Quat::IDENTITY);
    Mat4::from_scale_rotation_translation(three("scale", 1.0), turn, three("translation", 0.0))
}

/// The box a model's geometry fills, in the model's own units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    pub low: Vec3,
    pub high: Vec3,
}

impl Size {
    /// How tall, which is the measurement a maker states and everything else follows from.
    pub fn tall(&self) -> f32 {
        self.high.y - self.low.y
    }

    /// The other two, which are worth SHOWING even though nobody sets them: a couch stated
    /// at a metre tall comes out two and a half metres long, and that is the number a maker
    /// recognises as right or wrong at a glance. Height alone is easy to get wrong by half.
    pub fn wide(&self) -> f32 {
        self.high.x - self.low.x
    }

    pub fn deep(&self) -> f32 {
        self.high.z - self.low.z
    }

    /// What everything measures once the model is fitted to `tall` metres.
    pub fn at(&self, tall: f32) -> Vec3 {
        let fit = if self.tall() > 1e-6 {
            tall / self.tall()
        } else {
            1.0
        };
        Vec3::new(self.wide(), self.tall(), self.deep()) * fit
    }
}

/// The JSON chunk of a GLB.
///
/// A GLB is "glTF", a version, a total length, and then chunks: the first is always
/// the JSON. Twelve bytes of header, eight of chunk header, then the document.
fn the_json_of(bytes: &[u8]) -> Option<serde_json::Value> {
    if bytes.len() < 20 || &bytes[..4] != b"glTF" {
        return None;
    }
    let chunk = u32::from_le_bytes(bytes[12..16].try_into().ok()?) as usize;
    if &bytes[16..20] != b"JSON" || 20 + chunk > bytes.len() {
        return None;
    }
    serde_json::from_slice(&bytes[20..20 + chunk]).ok()
}

#[cfg(test)]
mod fitting {
    use super::*;

    /// A GLB written at a height is still a GLB, and says it is that tall.
    ///
    /// Built here rather than fetched: a minimal glTF with a known bounding box, so
    /// the arithmetic can be checked without a network or a fixture. The chunk
    /// lengths and the total are the part worth pinning - a GLB whose header
    /// disagrees with its body by one byte is refused by every loader, and nothing
    /// about the file looks wrong until something tries to open it.
    #[test]
    fn a_model_can_be_written_at_a_height() {
        // Two metres tall in its own units, so a fit to 0.5 must scale by a quarter.
        let doc = serde_json::json!({
            "asset": { "version": "2.0" },
            "scenes": [ { "nodes": [0] } ],
            "nodes": [ { "mesh": 0 } ],
            "meshes": [ { "primitives": [ { "attributes": { "POSITION": 0 } } ] } ],
            "accessors": [ { "min": [0.0, -1.0, 0.0], "max": [1.0, 1.0, 1.0] } ],
        });
        let mut json = serde_json::to_vec(&doc).expect("json");
        while json.len() % 4 != 0 {
            json.push(b' ');
        }
        let mut glb = Vec::new();
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&((12 + 8 + json.len()) as u32).to_le_bytes());
        glb.extend_from_slice(&(json.len() as u32).to_le_bytes());
        glb.extend_from_slice(b"JSON");
        glb.extend_from_slice(&json);

        let home = std::env::temp_dir().join("opificium-test-kiln");
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).expect("a folder");
        let from = home.join("two-metres.glb");
        std::fs::write(&from, &glb).expect("write");

        // Two units tall, and its lowest point one unit BELOW its own origin - which
        // is where a generated model usually leaves it.
        assert_eq!(
            bounds_of(&from).map(|size| (size.low.y, size.tall())),
            Some((-1.0, 2.0)),
            "it measured the wrong bounds"
        );

        // Fitted to half a metre: a quarter of what it was.
        let out = keep_at_height(&from, &home.join("fitted.glb"), 0.5).expect("kept");
        let kept = std::fs::read(&out).expect("read it back");
        assert_eq!(&kept[..4], b"glTF", "what came out is not a GLB");
        let declared = u32::from_le_bytes(kept[8..12].try_into().unwrap()) as usize;
        assert_eq!(declared, kept.len(), "the header disagrees with the file");

        let doc = the_json_of(&kept).expect("its json");
        let scene_roots = doc["scenes"][0]["nodes"].as_array().expect("roots");
        assert_eq!(
            scene_roots.len(),
            1,
            "the scene should point at the one wrapper"
        );
        let wrapper = &doc["nodes"][scene_roots[0].as_u64().unwrap() as usize];
        let scale = wrapper["scale"].as_array().expect("a scale");
        assert!(
            (scale[1].as_f64().unwrap() - 0.25).abs() < 1e-6,
            "{scale:?}"
        );
        // And STANDING on its origin rather than straddling it. Its lowest point was a
        // unit under the origin, so at a quarter scale it has to be lifted a quarter of
        // a metre - not the whole unit, because glTF scales before it translates.
        let up = wrapper["translation"].as_array().expect("a translation");
        assert!(
            (up[1].as_f64().unwrap() - 0.25).abs() < 1e-6,
            "the kept model does not stand on the ground: {up:?}"
        );
        // And it adopted what the scene used to hold, rather than orphaning it.
        assert_eq!(wrapper["children"], serde_json::json!([0]));

        // AND IT MEASURES AS WHAT IT WAS KEPT AS. The whole point of baking the fit into
        // the file is that the file then says how big the thing is - so reading it back has
        // to give the stated height, not the raw geometry under the wrapper. Measuring
        // without applying node transforms reported a fitted couch as half a metre tall and
        // stood it floating, lifted once by its own wrapper and once again by the bench.
        let kept = bounds_of(&out).expect("a kept model measures");
        assert!(
            (kept.tall() - 0.5).abs() < 1e-4,
            "kept at half a metre, measures {:.4}",
            kept.tall()
        );
        assert!(
            kept.low.y.abs() < 1e-4,
            "a kept model should stand ON the ground, not at {:.4}",
            kept.low.y
        );
    }

    /// The shelf shows the newest model first, and only models.
    ///
    /// Newest first is the whole reason the order exists - the model a maker wants is
    /// nearly always the one just made - so it is worth a test that would notice the
    /// comparison being written the other way round, which reads correctly either way.
    #[test]
    fn the_newest_model_comes_first() {
        let home = std::env::temp_dir().join("opificium-test-shelf");
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).expect("a folder");

        // Written oldest first, and stamped apart so the order cannot come down to
        // whatever the filesystem hands back.
        for (name, ago) in [("oldest.glb", 90), ("middle.glb", 60), ("newest.glb", 30)] {
            let road = home.join(name);
            std::fs::write(&road, b"glTF").expect("write");
            let when = std::time::SystemTime::now() - std::time::Duration::from_secs(ago);
            // Opened FOR WRITING to set a timestamp, which Windows requires and Unix does
            // not care about: `File::open` hands back a read-only handle, and this test
            // passed on macOS and failed the release build with "Access is denied".
            std::fs::OpenOptions::new()
                .write(true)
                .open(&road)
                .and_then(|file| file.set_modified(when))
                .expect("stamp it");
        }
        // Not a model, and not to be listed however new it is.
        std::fs::write(home.join("notes.txt"), b"not a model").expect("write");

        let found: Vec<String> = all_in(&home).iter().map(|road| name_of(road)).collect();
        assert_eq!(
            found,
            ["newest", "middle", "oldest"],
            "wrong order or wrong files"
        );

        // And a folder that does not exist is an empty shelf, not a panic: a project
        // that never fired the kiln has no models folder at all.
        assert!(all_in(&home.join("nowhere")).is_empty());
    }
}
