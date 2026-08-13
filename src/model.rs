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
    let (low, across) = bounds_of(road).unwrap_or((0.0, 1.0));
    let fit = match tall {
        Some(tall) if across > 1e-4 => tall / across,
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
    let (low, was) = bounds_of(from).ok_or("cannot measure that model, so cannot fit it")?;
    let fit = tall / was;

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
/// Read out of the file rather than guessed: a generated mesh has no idea what size
/// the thing it depicts really is, and two from the same machine differ by a factor
/// of ten. Every `POSITION` accessor carries a `min` and a `max` - the spec requires
/// it - so both are known without decoding a single vertex.
///
/// The LOWEST POINT matters as much as the height, because a generated model's origin
/// is wherever the machine left it, which is usually the middle of the thing. Standing
/// such a model at the floor buries half of it.
///
/// Both are spans of the whole model, not the largest of each primitive: a mesh cut
/// into a body and two wings has each part measuring short, and the model is as tall
/// as the distance from the lowest of them to the highest.
///
/// Node transforms are not applied. Generated models put their geometry at the root
/// with no scaling, and reading the whole node tree to be sure would be a glTF
/// importer - which this bench has no business being when Bevy is already doing it
/// properly two lines further down.
pub fn bounds_of(road: &Path) -> Option<(f32, f32)> {
    let bytes = std::fs::read(road).ok()?;
    let doc = the_json_of(&bytes)?;
    let (mut low, mut high) = (f32::MAX, f32::MIN);
    for mesh in doc.get("meshes")?.as_array()? {
        for prim in mesh.get("primitives")?.as_array()? {
            let at = prim.get("attributes")?.get("POSITION")?.as_u64()? as usize;
            let accessor = doc.get("accessors")?.as_array()?.get(at)?;
            low = low.min(accessor.get("min")?.as_array()?.get(1)?.as_f64()? as f32);
            high = high.max(accessor.get("max")?.as_array()?.get(1)?.as_f64()? as f32);
        }
    }
    (high - low > 1e-6).then_some((low, high - low))
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
            bounds_of(&from),
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
            std::fs::File::open(&road)
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
