//! THE KILN — an image in, a model out.
//!
//! Every other bench makes a thing out of parts the bench already holds. This one
//! hands an image to somebody else's machine and keeps what comes back: a GLB, mesh
//! and textures and all. Brett: "add an image and pass that image to the 3daistudio
//! app to get a model generated using their api an then have that model in the
//! bench. that model can then be saved as a glb file in the game."
//!
//! # What a model is NOT
//!
//! Not a part. A part is a NAME the bench resolves into boxes on a sixteenth-metre
//! lattice, painted from a ramp; a generated mesh is arbitrary triangles carrying
//! its own PBR materials. It cannot be painted, cut to the lattice or written into a
//! baked building's `boxes`, and pretending otherwise would break the brush, the
//! cutaway, the resize handles and the bake at once. So the kiln keeps FILES: a GLB
//! lands in the project and is carried into the game like any other asset, and every
//! game already knows how to load one.
//!
//! # It costs money and it leaves the building
//!
//! Each firing spends credits on somebody's account and uploads the image to a third
//! party. So it happens on a press and never on its own: nothing here retries, polls
//! ahead of being asked, or fires twice for one image.
//!
//! # The contract
//!
//! ```text
//! POST /v1/3d-models/trellis2/generate/   { "image": "data:image/png;base64,..." }
//!   -> { "task_id": "..." }
//! GET  /v1/generation-request/<id>/status/
//!   -> PENDING | IN_PROGRESS | FINISHED | FAILED
//!   -> FINISHED: { "results": [ { "asset": "https://...", "asset_type": "3D_MODEL" } ] }
//! ```
//!
//! GLB always, PBR always, one to three minutes. See <https://docs.3daistudio.com>.

use serde::Deserialize;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Where the machine lives.
const HOUSE: &str = "https://api.3daistudio.com/v1";

/// Which machine does the work.
///
/// Two, not a zoo. One is built for what this bench is for and the other is cheap
/// enough to ask a question with.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Maker {
    /// Tripo 3.1: the one meant for GAME assets - it can remesh to quads and cut a
    /// mesh down to a game's budget, which no other provider here offers. Brett:
    /// "they have a model specific to game assets."
    GameReady,
    /// TRELLIS.2, at ten credits: not game topology, but the cheapest way to find
    /// out whether an image makes any sense to a generator at all. Worth keeping for
    /// exactly that - a failed firing costs the same as a good one.
    Quick,
}

impl Maker {
    /// The path it answers on.
    fn road(self) -> &'static str {
        match self {
            // The version is named outright. Unversioned defaults to 3.0, and a
            // silent upgrade under a maker's feet is the kind of change that turns
            // "the kiln got worse" into an afternoon.
            Maker::GameReady => "3d-models/tripo/image-to-3d/3.1/",
            Maker::Quick => "3d-models/trellis2/generate/",
        }
    }
}

/// What to ask the kiln for.
///
/// Only the choices that change the THING, and every one of them changes the price -
/// which is why the panel adds them up out loud. The rest of Tripo's options
/// (seeds, alignment, autofix, geometry quality) are knobs for a person tuning one
/// image, and a bench that offered all of them would be a worse version of their own
/// website.
#[derive(Resource, Clone, Copy)]
pub struct Recipe {
    pub maker: Maker,
    /// Remesh to quads: what a modeller can actually edit afterwards.
    pub quad: bool,
    /// Cut the mesh down to something a game can carry.
    pub low_poly: bool,
    /// Twice the texture, twice that part of the price.
    pub detailed: bool,
}

impl Default for Recipe {
    fn default() -> Self {
        // Low poly, and NOT quads.
        //
        // Quads were on here, in the name of being game-ready, and they are the one
        // option that guarantees a file the game cannot open. glTF has no quad
        // primitive - its modes are points, lines and triangles, and quads are
        // triangulated on export - so a quad mesh cannot be a GLB at all, and the
        // machine hands back an FBX instead. Bevy has no FBX loader. Every firing
        // before this discovery came back unusable for that reason, and the FBX
        // magic ("Kaydara") was the only clue.
        //
        // Quads remain worth asking for when a model is bound for Blender rather
        // than for a game. That is a choice a maker makes, not a default.
        Recipe {
            maker: Maker::GameReady,
            quad: false,
            low_poly: true,
            detailed: false,
        }
    }
}

impl Recipe {
    /// What this firing will cost, in the machine's own credits.
    ///
    /// Added up here so a panel can say it BEFORE the press. Their published prices:
    /// Tripo is forty to begin with, plus twenty for a standard texture or forty for
    /// a detailed one, plus ten to remesh to quads and twenty to cut it down.
    /// TRELLIS.2 is ten, whole.
    pub fn credits(&self) -> u32 {
        match self.maker {
            Maker::Quick => 10,
            Maker::GameReady => {
                40 + if self.detailed { 40 } else { 20 }
                    + if self.quad { 10 } else { 0 }
                    + if self.low_poly { 20 } else { 0 }
            }
        }
    }

    /// The request, as that machine wants it.
    fn body(&self, uri: String) -> serde_json::Value {
        match self.maker {
            // TRELLIS.2 takes the picture and nothing else.
            Maker::Quick => serde_json::json!({ "image": uri }),
            Maker::GameReady => serde_json::json!({
                "image": uri,
                "texture": true,
                "texture_quality": if self.detailed { "detailed" } else { "standard" },
                "pbr": true,
                "quad": self.quad,
                "smart_low_poly": self.low_poly,
                // The provider refuses some images outright - "an error occurred
                // while processing the input image" - and a transparent PNG is the
                // one that does it: the same picture flattened to a JPEG went
                // straight through. This is their own remedy for it, it costs
                // nothing in the price list, and a maker should not have to know
                // that a logo with an alpha channel is a different kind of file.
                "enable_image_autofix": true,
            }),
        }
    }
}

/// Where the key is kept: the BENCH's own folder, never a game's.
///
/// It is the maker's credential, not any game's content. In a project it would be
/// committed into that game's repository the first time anybody ran `git add`, and
/// pushed to wherever that game is published.
fn key_file() -> PathBuf {
    crate::project::support().join("3daistudio.key")
}

/// The API key, if this bench has one.
///
/// A file the maker writes, or the environment - never asked for at a prompt and
/// never written to a log. The bench only ever hands it to the one host above.
pub fn key() -> Option<String> {
    if let Ok(said) = std::env::var("OPIFICIUM_3DAI_KEY") {
        let said = said.trim().to_string();
        if !said.is_empty() {
            return Some(said);
        }
    }
    let said = std::fs::read_to_string(key_file()).ok()?;
    let said = said.trim().to_string();
    (!said.is_empty()).then_some(said)
}

/// Where to tell a maker to put the key, when there is none.
pub fn where_the_key_goes() -> String {
    key_file().display().to_string()
}

/// Where a finished model is kept: the project's own folder.
///
/// Beside the drawings rather than among them - a `.glb` is not a work the bench can
/// reopen and change, it is the finished thing. The bake carries it into the game
/// from here.
pub fn models_home() -> PathBuf {
    crate::project::root().join("out/models")
}

/// What the machine says when a job is taken.
#[derive(Deserialize)]
struct Taken {
    task_id: String,
}

/// What it says while it works, and when it is done.
#[derive(Deserialize)]
struct Progress {
    status: String,
    /// How far along, as a percentage, when the machine cares to say. Absent on some
    /// providers and early in a job, which is why the bar it drives has an
    /// unmeasured state rather than a nought.
    #[serde(default)]
    progress: Option<f32>,
    #[serde(default)]
    results: Vec<Made>,
    /// Why it went wrong, when the machine says - and it says this on a report that
    /// calls itself FINISHED, which is how the first real firing was lost.
    #[serde(default)]
    failure_reason: Option<String>,
}

#[derive(Deserialize)]
struct Made {
    /// NULL on a job that failed. It arrives inside a FINISHED report, beside an
    /// `asset_type` that still claims to be a 3D model - so this cannot be a plain
    /// `String`, or the whole report fails to parse and the reason beside it is lost
    /// with it. That is exactly what happened to the first firing: a real diagnosis
    /// reported as "the kiln answered something else".
    #[serde(default)]
    asset: Option<String>,
    #[serde(default)]
    asset_type: String,
}

/// How far along a firing is, in the bench's own words.
#[derive(Clone, PartialEq, Debug)]
pub enum Firing {
    /// Nothing has been asked for.
    Cold,
    /// Sent, and waiting. The word is the machine's own; the number is how far it
    /// says it has got, when it says.
    Working(String, Option<f32>),
    /// A file, kept.
    Done(PathBuf),
    /// It did not work, and why.
    Failed(String),
}

/// An image as the machine wants it: a data URI, base64, with its own type named.
///
/// The type is read from the NAME rather than the bytes. A PNG announced as a JPEG
/// is refused by the far end with a message about the encoding, which is a confusing
/// thing to be told about a file that opens perfectly well here.
pub fn data_uri(image: &Path) -> Result<String, String> {
    let kind = match image
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        other => {
            return Err(format!(
                "{} is a {}, and the kiln takes a png, a jpeg or a webp",
                image.display(),
                other.unwrap_or("file with no extension")
            ));
        }
    };
    let bytes = std::fs::read(image).map_err(|why| format!("{}: {why}", image.display()))?;
    Ok(format!("data:{kind};base64,{}", base64(&bytes)))
}

/// RFC 4648 base64, written out rather than taken as a dependency.
///
/// Twenty lines against a crate in the tree, for a bench whose manifest justifies
/// every line of it. It is checked against the RFC's own vectors, which is the only
/// reason writing it is defensible at all.
fn base64(bytes: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for lot in bytes.chunks(3) {
        let (a, b, c) = (
            lot[0] as u32,
            lot.get(1).copied().unwrap_or(0) as u32,
            lot.get(2).copied().unwrap_or(0) as u32,
        );
        let three = (a << 16) | (b << 8) | c;
        out.push(ABC[(three >> 18 & 63) as usize] as char);
        out.push(ABC[(three >> 12 & 63) as usize] as char);
        // The tail is padded, not truncated: a decoder counts in fours.
        out.push(if lot.len() > 1 {
            ABC[(three >> 6 & 63) as usize] as char
        } else {
            '='
        });
        out.push(if lot.len() > 2 {
            ABC[(three & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// Reads what the machine said when it took the job.
fn took_it(said: &str) -> Result<String, String> {
    serde_json::from_str::<Taken>(said)
        .map(|taken| taken.task_id)
        .map_err(|_| format!("the kiln answered something else: {}", first_words(said)))
}

/// Where a job has got to: still going, with the machine's own word for it, or
/// ready, with somewhere to fetch the model from.
///
/// Two variants rather than one carrying either, which is what this was at first: a
/// caller handed `Working(String)` cannot tell a status word from a URL, and the one
/// that guesses wrong downloads "IN_PROGRESS".
#[derive(Debug)]
enum Step {
    Waiting(String, Option<f32>),
    Ready(String),
}

/// Reads a progress report: where it is, and the model if it is finished.
///
/// A FINISHED report with no 3D model in it is a failure, not a success - it is the
/// one shape that would otherwise slip through and leave a maker looking at a bench
/// that says it worked and holds nothing.
fn how_it_goes(said: &str) -> Result<Step, String> {
    let report = serde_json::from_str::<Progress>(said)
        .map_err(|_| format!("the kiln answered something else: {}", first_words(said)))?;
    // A report that says FINISHED and carries no model is a FAILURE, whatever it
    // calls itself - and the machine's own reason for it is the one thing worth
    // saying out loud, since it is the only part that tells a maker what to change.
    let blame = || match &report.failure_reason {
        Some(why) => why.clone(),
        // No reason given: hand back what it actually SAID instead of a summary of
        // it. A sentence of mine that leaves out the payload is a sentence that
        // costs a firing to get past.
        None => format!(
            "the kiln sent no model back, and said: {}",
            first_words(said)
        ),
    };
    match report.status.as_str() {
        "FINISHED" => {
            // BY FORMAT, not by order. A finished job hands back the same model in
            // several formats, all of them typed `3D_MODEL` - and taking the first
            // one fetched an FBX: `Kaydara FBX Binary`, which is where the four
            // bytes "Kayd" came from. The docs say every endpoint outputs GLB, and
            // they do; they output others beside it.
            let models: Vec<&str> = report
                .results
                .iter()
                .filter(|made| made.asset_type.is_empty() || made.asset_type == "3D_MODEL")
                .filter_map(|made| made.asset.as_deref())
                .collect();
            let glb = models
                .iter()
                .find(|url| road_of(url).ends_with(".glb"))
                .or_else(|| models.iter().find(|url| road_of(url).ends_with(".gltf")));
            match glb {
                Some(url) => Ok(Step::Ready((*url).to_string())),
                // Something came back and none of it was a glTF. Say WHAT came back:
                // "no model" would be a lie, and the formats name the problem.
                None if !models.is_empty() => Err(format!(
                    "the kiln sent no glTF, only: {}",
                    models
                        .iter()
                        .map(|url| road_of(url).rsplit('.').next().unwrap_or("?").to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
                None => Err(blame()),
            }
        }
        "FAILED" => Err(blame()),
        waiting => Ok(Step::Waiting(waiting.to_string(), report.progress)),
    }
}

/// A URL's own path, without the query it is signed with.
///
/// The format has to be read off the path: these arrive pre-signed, and the
/// signature's parameters end in things like `aws4_request` that would confound any
/// test for an extension.
fn road_of(url: &str) -> &str {
    url.split('?').next().unwrap_or(url)
}

/// Enough of an unexpected answer to recognise it, and no more.
///
/// A whole HTML error page in a log is a log nobody reads, and an API key is never in
/// one of these - the bench sends the key and never receives it.
///
/// Three hundred rather than a hundred and twenty. The first real firing failed with
/// its reason written in the payload, and the old cut fell exactly where
/// `failure_reason` began: the bench quoted the useless half of the sentence and
/// dropped the half that said what was wrong.
fn first_words(said: &str) -> String {
    let brief: String = said.chars().take(300).collect();
    brief.replace('\n', " ")
}

/// A name a file can have, out of whatever the maker typed.
pub fn a_plain_name(said: &str) -> String {
    let kept: String = said
        .trim()
        .to_lowercase()
        .chars()
        .map(|letter| {
            if letter.is_ascii_alphanumeric() || letter == '-' {
                letter
            } else {
                '-'
            }
        })
        .collect();
    let kept = kept.trim_matches('-').to_string();
    if kept.is_empty() {
        "model".to_string()
    } else {
        kept
    }
}

/// Where this model will be kept, without treading on one already there.
pub fn a_free_road(name: &str) -> PathBuf {
    let home = models_home();
    let first = home.join(format!("{name}.glb"));
    if !first.exists() {
        return first;
    }
    // A second model of the same name stands beside the first rather than over it. A
    // firing costs credits, and overwriting one that has just been paid for is the
    // one mistake worth designing out.
    for n in 2..1000 {
        let road = home.join(format!("{name}-{n}.glb"));
        if !road.exists() {
            return road;
        }
    }
    home.join(format!("{name}-many.glb"))
}

/// Commissions one model, start to finish, and keeps the file.
///
/// BLOCKING, and meant for a thread of its own: the machine takes one to three
/// minutes, and a bench that waited for it on the frame loop would be a bench nobody
/// could move. `say` is how it reports back - a channel, in practice.
///
/// It spends credits the moment the first request lands, so everything that can be
/// checked is checked before then: the image is a kind the machine takes, there is a
/// key, and the folder can be written.
pub fn commission(
    image: &Path,
    name: &str,
    key: &str,
    recipe: Recipe,
    say: &dyn Fn(Firing),
) -> Result<PathBuf, String> {
    // Before any credit: the picture, the folder, the name.
    let uri = data_uri(image)?;
    let home = models_home();
    std::fs::create_dir_all(&home).map_err(|why| format!("{}: {why}", home.display()))?;
    let road = a_free_road(name);

    say(Firing::Working("SENDING".to_string(), None));
    let taken = ureq::post(format!("{HOUSE}/{}", recipe.maker.road()))
        .header("Authorization", format!("Bearer {key}"))
        .send_json(recipe.body(uri))
        .map_err(|why| format!("the kiln would not take it: {}", plainly(&why)))?
        .body_mut()
        .read_to_string()
        .map_err(|why| format!("the kiln said nothing back: {}", plainly(&why)))?;
    let job = took_it(&taken)?;

    // Now it is paid for. Poll until it is one thing or the other - and give up
    // rather than poll for ever, since a job that has not finished in a quarter of an
    // hour is a job that has gone wrong at the far end.
    let began = std::time::Instant::now();
    let url = loop {
        if began.elapsed() > std::time::Duration::from_secs(15 * 60) {
            return Err("the kiln never finished; the job may still be on your account".into());
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
        let said = ureq::get(format!("{HOUSE}/generation-request/{job}/status/"))
            .header("Authorization", format!("Bearer {key}"))
            .call()
            .map_err(|why| format!("lost the kiln: {}", plainly(&why)))?
            .body_mut()
            .read_to_string()
            .map_err(|why| format!("lost the kiln: {}", plainly(&why)))?;
        match how_it_goes(&said)? {
            Step::Waiting(word, how_far) => say(Firing::Working(word, how_far)),
            Step::Ready(url) => break url,
        }
    };

    // A hundred, because the making is done and only the wire is left.
    say(Firing::Working("FETCHING".to_string(), Some(100.0)));
    // STREAMED to the file, not gathered in memory first.
    //
    // `read_to_vec` carries a ten-megabyte cap, and a textured GLB goes past it
    // without trying: the first real firing died on "the response body is larger than
    // request limit: 10485760" - AFTER the model had been made and paid for, which is
    // the worst place to fail. Raising the cap would only move the number; a mesh has
    // no size a bench should be guessing at.
    eprintln!("  fetching {url}");
    let answer = ureq::get(&url)
        .call()
        .map_err(|why| format!("could not fetch the model: {}", plainly(&why)))?;
    // What the far end SAYS it is, kept for the message below: a mismatch between the
    // magic and the type is the difference between "they sent something else" and
    // "we read it wrong", and guessing between those two costs a firing each time.
    let claimed = answer
        .headers()
        .get("content-type")
        .and_then(|said| said.to_str().ok())
        .unwrap_or("nothing at all")
        .to_string();
    let mut reader = answer
        .into_body()
        // Unlimited by default, so a stop is set on purpose: half a gigabyte is far
        // past any model and still a stop, rather than trusting whatever is at the
        // other end of the wire to stop on its own.
        .into_reader()
        .take(512 * 1024 * 1024);

    // A GLB begins with "glTF". The first four bytes are checked BEFORE a file
    // exists, because a proxy handing back an error page with a 200 would otherwise
    // be written to disk as a model and fail later, in the game, where nobody would
    // connect it to this.
    let mut magic = [0u8; 4];
    reader
        .read_exact(&mut magic)
        .map_err(|why| format!("the model came back empty: {why}"))?;
    if &magic != b"glTF" {
        // The bytes AND what the server called them. A GLB begins "glTF"; anything
        // else is either a different format or an error dressed as a download, and
        // the type tells which.
        return Err(format!(
            "what came back is not a GLB: it says it is {claimed} and begins {:?} ({})",
            String::from_utf8_lossy(&magic),
            magic
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }
    let mut file =
        std::fs::File::create(&road).map_err(|why| format!("{}: {why}", road.display()))?;
    file.write_all(&magic)
        .map_err(|why| format!("{}: {why}", road.display()))?;
    let carried = std::io::copy(&mut reader, &mut file)
        .map_err(|why| format!("{}: {why}", road.display()))?;
    info!("the kiln kept {} ({} bytes)", road.display(), carried + 4);
    say(Firing::Done(road.clone()));
    Ok(road)
}

/// An error, briefly, and without the key in it.
///
/// The key rides in a header on every request, and a library's own error text can
/// carry the request back out with it. This keeps the sentence and drops the rest.
fn plainly(why: &ureq::Error) -> String {
    let said = why.to_string();
    let brief: String = said.chars().take(160).collect();
    brief.replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Base64, against the RFC's own vectors.
    ///
    /// The whole reason it is written here rather than taken from a crate: it is
    /// short enough to check completely, so it is checked completely.
    #[test]
    fn base64_says_what_the_rfc_says() {
        // RFC 4648 section 10, every one of them.
        for (raw, said) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(raw.as_bytes()), said, "base64({raw:?})");
        }
        // And bytes that are not letters: every value, in fours, padded.
        let all: Vec<u8> = (0u8..=255).collect();
        let said = base64(&all);
        assert_eq!(
            said.len() % 4,
            0,
            "the tail was truncated rather than padded"
        );
        assert!(
            said.chars()
                .all(|c| c.is_ascii_alphanumeric() || "+/=".contains(c)),
            "something outside the alphabet got in"
        );
    }

    /// An image the kiln cannot send is refused HERE, before any credit is spent.
    #[test]
    fn only_the_pictures_it_can_send() {
        for bad in ["model.glb", "notes.txt", "sketch"] {
            let why = data_uri(Path::new(bad)).expect_err("should refuse");
            assert!(
                why.contains("png") && why.contains("jpeg"),
                "the refusal never says what it would take: {why}"
            );
        }
    }

    /// The machine's answers, read.
    #[test]
    fn what_the_kiln_says_is_understood() {
        assert_eq!(
            took_it(r#"{"task_id":"abc-123"}"#).expect("a job"),
            "abc-123"
        );
        // Waiting, in the machine's own words, so the bench can show them.
        for waiting in ["PENDING", "IN_PROGRESS"] {
            let said = format!("{{\"status\":\"{waiting}\"}}");
            assert!(matches!(how_it_goes(&said), Ok(Step::Waiting(word, _)) if word == waiting));
        }
        // Finished, with a model.
        let done = r#"{"status":"FINISHED","results":[{"asset":"https://x/y.glb","asset_type":"3D_MODEL"}]}"#;
        assert!(matches!(how_it_goes(done), Ok(Step::Ready(url)) if url == "https://x/y.glb"));
        // Finished with NOTHING in it is a failure, not a success. The one shape
        // that would otherwise leave the bench saying it worked and holding no file.
        let empty = r#"{"status":"FINISHED","results":[]}"#;
        assert!(
            how_it_goes(empty).is_err(),
            "an empty finish read as success"
        );
        assert!(how_it_goes(r#"{"status":"FAILED"}"#).is_err());
        // And something that is not the API at all - a proxy's error page, say -
        // says so briefly rather than pasting a screenful into the log.
        let html = "<html><body>".to_string() + &"x".repeat(4000) + "</body></html>";
        let why = how_it_goes(&html).expect_err("not json");
        // Far shorter than what came in, rather than under some particular number:
        // the point is that a page is not quoted whole, and the cut has been moved
        // once already - it used to fall across the only useful part of a real answer.
        assert!(
            why.len() < html.len() / 4,
            "an error page was quoted whole: {} chars",
            why.len()
        );
    }

    /// A FINISHED report with a null asset is a failure, and it says WHY.
    ///
    /// The shape of the first real firing, kept word for word. The machine answered
    /// `FINISHED` with `progress: 100`, an `asset_type` of `3D_MODEL`, and
    /// `asset: null` - a failure in a success's clothes. Two things went wrong at
    /// once: `asset` was a plain `String`, so the whole report failed to parse, and
    /// the parse error quoted the first hundred and twenty characters, which stopped
    /// exactly where the reason began. A maker was told "the kiln answered something
    /// else" about a job whose cause was written in the payload.
    #[test]
    fn a_finish_with_no_model_says_why() {
        let said = r#"{"status":"FINISHED","progress":100,"results":[{"asset":null,
            "asset_type":"3D_MODEL","metadata":null}],
            "failure_reason":"the image has no clear subject"}"#;
        let why = how_it_goes(said).expect_err("a null asset is not a success");
        assert!(
            why.contains("no clear subject"),
            "the machine's own reason never reached the bench: {why}"
        );

        // With no reason given it still reads as a failure, rather than as a model at
        // a URL of "null".
        let mute = r#"{"status":"FINISHED","results":[{"asset":null,"asset_type":"3D_MODEL"}]}"#;
        assert!(how_it_goes(mute).is_err());

        // A FAILED report's reason comes through the same door.
        let failed = r#"{"status":"FAILED","failure_reason":"content policy"}"#;
        let why = how_it_goes(failed).expect_err("failed");
        assert!(why.contains("content policy"), "{why}");
    }

    /// A typed name becomes a name a file can have.
    #[test]
    fn a_name_becomes_a_filename() {
        assert_eq!(a_plain_name("Old Oak Tree"), "old-oak-tree");
        assert_eq!(a_plain_name("  barrel/2  "), "barrel-2");
        assert_eq!(a_plain_name("../../etc/passwd"), "etc-passwd");
        assert_eq!(a_plain_name(""), "model");
        assert_eq!(a_plain_name("!!!"), "model");
        // Nothing that could climb out of the folder it is written into.
        for said in ["../up", "a/b", "a\\b", "a:b"] {
            let name = a_plain_name(said);
            assert!(!name.contains('/') && !name.contains('\\') && !name.contains(".."));
        }
    }

    /// A key that IS there is read cleanly.
    ///
    /// Skipped where there is none - a build machine has no key, and a test that
    /// demanded one would fail for everybody but its author. Where there is one it
    /// catches the classic: a file read whole, newline and all, which the far end
    /// refuses with a message about authorisation that says nothing about whitespace.
    #[test]
    fn a_key_that_is_there_is_read_cleanly() {
        let Some(said) = key() else {
            return;
        };
        assert!(
            !said.chars().any(char::is_whitespace),
            "the key came back with the file's own whitespace still on it"
        );
        assert!(said.len() > 8, "that is too short to be a key");
    }

    /// The key is looked for in the bench's own house, never in a game's.
    #[test]
    fn the_key_is_the_benchs_own() {
        let road = where_the_key_goes().to_lowercase();
        assert!(road.contains("opificium"), "{road}");
        // Not in a project: a credential in a game's folder is a credential in that
        // game's repository.
        for game in ["divus", "fly on the wall", "out/", "data/"] {
            assert!(
                !road.contains(game),
                "the key would live in a game's folder: {road}"
            );
        }
    }
}

// ---------------------------------------------------------------- the bench

use crate::look::{Fonts, Palette, theme};
use bevy::prelude::*;
use bevy::text::FontSize;
use std::sync::Mutex;

/// The kiln's own panel.
#[derive(Component)]
struct KilnPanel;

/// The button that asks for an image.
#[derive(Component)]
struct PickAnImage;

/// The line that says how the firing goes.
#[derive(Component)]
struct KilnWord;

/// One thing about the firing a maker can change.
#[derive(Component, Clone, Copy, PartialEq)]
enum Choice {
    Maker(Maker),
    Quad,
    LowPoly,
    Detailed,
}

/// The bar that fills as the kiln works.
#[derive(Component)]
struct KilnBar;

/// The line that says what the next firing will cost.
#[derive(Component)]
struct KilnPrice;

/// The kiln's state, and the thread's end of the wire.
///
/// A `Receiver` is `Send` but not `Sync`, and a resource must be both - hence the
/// lock. There is one reader and it runs on the main thread, so it never waits.
#[derive(Resource, Default)]
struct Kiln {
    firing: Firing,
    coming: Option<Mutex<std::sync::mpsc::Receiver<Firing>>>,
}

impl Default for Firing {
    fn default() -> Self {
        Firing::Cold
    }
}

pub struct KilnPlugin;

impl Plugin for KilnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Kiln>()
            .init_resource::<Recipe>()
            .add_systems(Startup, hang_the_kiln)
            .add_systems(
                Update,
                (
                    show_the_kiln,
                    take_the_choices,
                    work_the_kiln,
                    say_how_it_goes,
                    dress_the_choices,
                )
                    .chain(),
            );
    }
}

fn hang_the_kiln(mut commands: Commands, fonts: Res<Fonts>, palette: Res<Palette>) {
    // The shelf's own edge and width: the three benches flank one stage, and a panel
    // that sat differently would read as a different program.
    let panel = commands
        .spawn((
            KilnPanel,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(crate::menu::BAR_HIGH),
                bottom: Val::Px(0.0),
                width: Val::Px(crate::look::PANEL_WIDE),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(6.0),
                border: UiRect::left(Val::Px(1.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(theme::panel_bg()),
            BorderColor::all(theme::panel_border(&palette)),
            crate::look::Scrollable,
            bevy::ui::ScrollPosition::default(),
            Visibility::Hidden,
        ))
        .id();

    commands.spawn((
        Text::new("THE KILN"),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(theme::accent(&palette)),
        ChildOf(panel),
    ));
    commands.spawn((
        Text::new("an image in, a model out"),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: FontSize::Px(12.0),
            ..default()
        },
        TextColor(theme::text_dim(&palette)),
        ChildOf(panel),
    ));

    // WHAT to ask for. Two machines and three options, and every one of them moves
    // the price - so the price is added up under them rather than found later on
    // somebody's billing page.
    for (choice, label, tale) in [
        (
            Choice::Maker(Maker::GameReady),
            "GAME-READY",
            "Tripo: the one that can make game topology",
        ),
        (
            Choice::Maker(Maker::Quick),
            "QUICK LOOK",
            "TRELLIS.2, ten credits: is this image any good?",
        ),
        (
            Choice::Quad,
            "QUADS",
            "editable topology, for Blender - comes back as FBX, \
             which glTF cannot carry and no game here can load",
        ),
        (
            Choice::LowPoly,
            "LOW POLY",
            "cut it down to something a game can carry",
        ),
        (
            Choice::Detailed,
            "FINE TEXTURE",
            "twice the texture, and twice that part of the price",
        ),
    ] {
        let row = commands
            .spawn((
                choice,
                Interaction::default(),
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::BLACK.with_alpha(0.18)),
                BorderColor::all(theme::panel_border(&palette)),
                ChildOf(panel),
            ))
            .id();
        commands.spawn((
            Text::new(label),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: FontSize::Px(11.0),
                ..default()
            },
            TextColor(theme::text_dim(&palette)),
            ChildOf(row),
        ));
        commands.spawn((crate::rail::Word(tale), ChildOf(row)));
    }

    commands.spawn((
        KilnPrice,
        Text::new(""),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: FontSize::Px(12.0),
            ..default()
        },
        TextColor(theme::accent(&palette).with_alpha(0.9)),
        Node {
            margin: UiRect::top(Val::Px(4.0)),
            ..default()
        },
        ChildOf(panel),
    ));

    let button = commands
        .spawn((
            PickAnImage,
            Interaction::default(),
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                padding: UiRect::axes(Val::Px(9.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.18)),
            BorderColor::all(theme::accent(&palette).with_alpha(0.6)),
            ChildOf(panel),
        ))
        .id();
    commands.spawn((
        Text::new("AN IMAGE..."),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: FontSize::Px(12.0),
            ..default()
        },
        TextColor(theme::accent(&palette)),
        ChildOf(button),
    ));
    // What it costs, said before it is spent rather than after.
    commands.spawn((
        crate::rail::Word(
            "Pick a picture and the kiln makes a model of it. \
             It spends credits on your 3D AI Studio account and \
             sends the picture to them",
        ),
        ChildOf(button),
    ));

    // The bar fills on the machine's OWN number - the status report carries a
    // percentage - so it is never a decoration pretending to know something. Before
    // the first number arrives it sits empty rather than inventing a nought.
    let trough = commands
        .spawn((
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                width: Val::Percent(100.0),
                height: Val::Px(6.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.35)),
            BorderColor::all(theme::panel_border(&palette)),
            ChildOf(panel),
        ))
        .id();
    commands.spawn((
        KilnBar,
        Node {
            width: Val::Percent(0.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(theme::accent(&palette)),
        ChildOf(trough),
    ));

    commands.spawn((
        KilnWord,
        Text::new(""),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: FontSize::Px(12.0),
            ..default()
        },
        TextColor(theme::text_dim(&palette)),
        Node {
            margin: UiRect::top(Val::Px(6.0)),
            ..default()
        },
        ChildOf(panel),
    ));
}

/// The panel belongs to the kiln bench.
fn show_the_kiln(bench: Res<crate::Bench>, mut panels: Query<&mut Visibility, With<KilnPanel>>) {
    if !bench.is_changed() {
        return;
    }
    for mut showing in &mut panels {
        *showing = if *bench == crate::Bench::Kiln {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// A press on a choice changes what the NEXT firing asks for.
///
/// Never during one: the recipe is what was sent, and letting it change mid-firing
/// would make the price on the panel a lie about the charge already made.
fn take_the_choices(
    bench: Res<crate::Bench>,
    kiln: Res<Kiln>,
    mut recipe: ResMut<Recipe>,
    chosen: Query<(&Interaction, &Choice), Changed<Interaction>>,
) {
    if *bench != crate::Bench::Kiln || matches!(kiln.firing, Firing::Working(..)) {
        return;
    }
    for (touch, choice) in &chosen {
        if *touch != Interaction::Pressed {
            continue;
        }
        match choice {
            Choice::Maker(which) => recipe.maker = *which,
            Choice::Quad => recipe.quad = !recipe.quad,
            Choice::LowPoly => recipe.low_poly = !recipe.low_poly,
            Choice::Detailed => recipe.detailed = !recipe.detailed,
        }
    }
}

/// The standing choices wear the gold, and the price says what they add up to.
fn dress_the_choices(
    recipe: Res<Recipe>,
    palette: Res<Palette>,
    children: Query<&Children>,
    mut rows: Query<(Entity, &Choice, &mut BorderColor, &mut BackgroundColor)>,
    mut words: Query<&mut TextColor>,
    mut prices: Query<&mut Text, With<KilnPrice>>,
) {
    if !recipe.is_changed() {
        return;
    }
    for (row, choice, mut border, mut fill) in &mut rows {
        // A quick look takes no options, so the three of them go quiet with it -
        // greyed rather than hidden, since they are still what the other machine
        // would do and a row that vanishes is a row a maker has to go looking for.
        let idle = recipe.maker == Maker::Quick && !matches!(choice, Choice::Maker(_));
        let standing = match choice {
            Choice::Maker(which) => recipe.maker == *which,
            Choice::Quad => recipe.quad && !idle,
            Choice::LowPoly => recipe.low_poly && !idle,
            Choice::Detailed => recipe.detailed && !idle,
        };
        *border = BorderColor::all(if standing {
            theme::accent(&palette)
        } else {
            theme::panel_border(&palette).with_alpha(if idle { 0.15 } else { 0.35 })
        });
        *fill = BackgroundColor(if standing {
            Color::srgb(0.075, 0.082, 0.102)
        } else {
            Color::BLACK.with_alpha(0.18)
        });
        let dye = if standing {
            theme::accent(&palette)
        } else {
            theme::text_dim(&palette).with_alpha(if idle { 0.3 } else { 1.0 })
        };
        if let Ok(kids) = children.get(row) {
            for kid in kids.iter() {
                if let Ok(mut colour) = words.get_mut(kid) {
                    *colour = TextColor(dye);
                }
            }
        }
    }
    let said = format!("{} credits", recipe.credits());
    for mut text in &mut prices {
        if text.0 != said {
            *text = Text::new(said.clone());
        }
    }
}

/// Takes the press, and hears back from the thread.
fn work_the_kiln(
    _main_thread: bevy::ecs::system::NonSendMarker,
    bench: Res<crate::Bench>,
    mut kiln: ResMut<Kiln>,
    recipe: Res<Recipe>,
    picks: Query<&Interaction, (Changed<Interaction>, With<PickAnImage>)>,
) {
    // Whatever the thread has said since last frame, in order.
    let mut heard = Vec::new();
    if let Some(wire) = kiln.coming.as_ref()
        && let Ok(wire) = wire.lock()
    {
        while let Ok(word) = wire.try_recv() {
            heard.push(word);
        }
    }
    for word in heard {
        let ended = matches!(word, Firing::Done(_) | Firing::Failed(_));
        kiln.firing = word;
        if ended {
            // The wire is done with; dropping it lets the thread's end close.
            kiln.coming = None;
        }
    }

    if *bench != crate::Bench::Kiln {
        return;
    }
    if !picks.iter().any(|touch| *touch == Interaction::Pressed) {
        return;
    }
    // One firing at a time. Two presses would be two charges, and the second would
    // land on the same name.
    if matches!(kiln.firing, Firing::Working(..)) {
        return;
    }
    let Some(key) = key() else {
        kiln.firing = Firing::Failed(format!("no key. Put one in {}", where_the_key_goes()));
        return;
    };
    let Some(image) = rfd::FileDialog::new()
        .set_title("An image for the kiln")
        .add_filter("Pictures", &["png", "jpg", "jpeg", "webp"])
        .pick_file()
    else {
        return;
    };
    let name = a_plain_name(
        &image
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_default(),
    );

    // Off the frame loop: one to three minutes of somebody else's work.
    let recipe = *recipe;
    let (say, hear) = std::sync::mpsc::channel();
    kiln.coming = Some(Mutex::new(hear));
    kiln.firing = Firing::Working("SENDING".to_string(), None);
    std::thread::spawn(move || {
        let told = say.clone();
        let out = commission(&image, &name, &key, recipe, &move |word| {
            let _ = told.send(word);
        });
        if let Err(why) = out {
            let _ = say.send(Firing::Failed(why));
        }
    });
}

/// Says where the firing has got to, in the panel.
fn say_how_it_goes(
    kiln: Res<Kiln>,
    recipe: Res<Recipe>,
    palette: Res<Palette>,
    mut words: Query<(&mut Text, &mut TextColor), With<KilnWord>>,
    mut bars: Query<&mut Node, With<KilnBar>>,
) {
    if !kiln.is_changed() && !recipe.is_changed() {
        return;
    }
    let how_far = match &kiln.firing {
        Firing::Working(_, how_far) => how_far.unwrap_or(0.0),
        _ => 0.0,
    };
    for mut bar in &mut bars {
        bar.width = Val::Percent(how_far.clamp(0.0, 100.0));
    }
    let (said, dye) = match &kiln.firing {
        Firing::Cold => (
            format!("{} credits, and about a minute", recipe.credits()),
            theme::text_dim(&palette),
        ),
        Firing::Working(word, _) => (
            format!("{}...", word.to_lowercase().replace('_', " ")),
            theme::accent(&palette),
        ),
        Firing::Done(road) => (
            format!(
                "kept {}",
                road.file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_default()
            ),
            theme::text(&palette),
        ),
        Firing::Failed(why) => (why.clone(), palette.shade("cloth-red", 0.85)),
    };
    for (mut text, mut colour) in &mut words {
        if text.0 != said {
            *text = Text::new(said.clone());
            *colour = TextColor(dye);
        }
    }
}

#[cfg(test)]
mod prices {
    use super::*;

    /// The panel's arithmetic matches their published prices.
    ///
    /// Worth pinning because it is money, and because the sum is spread over four
    /// booleans: Tripo is forty to begin with, plus twenty for a standard texture or
    /// forty for a detailed one, plus ten to remesh to quads and twenty to cut it
    /// down. TRELLIS.2 is ten, whole, and takes no options.
    #[test]
    fn a_firing_costs_what_they_charge() {
        let quick = Recipe {
            maker: Maker::Quick,
            quad: true,
            low_poly: true,
            detailed: true,
        };
        assert_eq!(quick.credits(), 10, "options must not price a quick look");

        let bare = Recipe {
            maker: Maker::GameReady,
            quad: false,
            low_poly: false,
            detailed: false,
        };
        assert_eq!(
            bare.credits(),
            60,
            "forty, and twenty for a standard texture"
        );
        assert_eq!(Recipe { quad: true, ..bare }.credits(), 70);
        assert_eq!(
            Recipe {
                low_poly: true,
                ..bare
            }
            .credits(),
            80
        );
        assert_eq!(
            Recipe {
                detailed: true,
                ..bare
            }
            .credits(),
            80
        );
        let lot = Recipe {
            maker: Maker::GameReady,
            quad: true,
            low_poly: true,
            detailed: true,
        };
        assert_eq!(lot.credits(), 110, "everything on");

        // And the bench opens on something a game can actually load.
        let opens = Recipe::default();
        assert_eq!(opens.maker, Maker::GameReady);
        assert!(
            !opens.quad,
            "quads cannot be a default: glTF has no quad primitive, so the machine \
             answers with an FBX and no game here can load it"
        );
        assert!(opens.low_poly, "a game still wants the polygon count cut");
        assert_eq!(opens.credits(), 80);
    }

    /// Each machine is asked in its own words.
    #[test]
    fn each_machine_is_asked_its_own_way() {
        let uri = "data:image/png;base64,AAAA".to_string();
        let quick = Recipe {
            maker: Maker::Quick,
            ..Recipe::default()
        }
        .body(uri.clone());
        assert_eq!(
            quick.as_object().expect("an object").len(),
            1,
            "a quick look takes only the image"
        );

        let game = Recipe::default().body(uri);
        assert_eq!(game["quad"], false, "see the default's own reasoning");
        assert_eq!(game["smart_low_poly"], true);
        // Their remedy for an input the provider will not take. A transparent PNG is
        // refused outright without it, and the same image flattened goes through.
        assert_eq!(game["enable_image_autofix"], true);
        assert_eq!(game["texture_quality"], "standard");
        assert_eq!(
            Recipe {
                detailed: true,
                ..Recipe::default()
            }
            .body(String::new())["texture_quality"],
            "detailed"
        );
        // The version is named, or an unversioned path would drift to 3.0 under us.
        assert!(
            Maker::GameReady.road().ends_with("/3.1/"),
            "{}",
            Maker::GameReady.road()
        );
    }
}

/// Fires the kiln from the command line: `opificium --kiln <image>`.
///
/// The same reason `--bake` exists. A button cannot be driven by a script, a build,
/// or anybody checking that the thing still works - and this one talks to a machine
/// across the internet, which is exactly the part worth being able to exercise
/// without a hand on a mouse.
///
/// Returns the process's exit status.
pub fn from_the_command_line() -> Option<i32> {
    let mut args = std::env::args().skip(1);
    let mut image = None;
    let mut wanted = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--kiln" => wanted = true,
            other if other.starts_with('-') => {}
            other if wanted && image.is_none() => image = Some(std::path::PathBuf::from(other)),
            _ => {}
        }
    }
    if !wanted {
        return None;
    }
    let project = crate::project::open_quietly();
    println!(
        "the kiln is standing in {}",
        project.map_or_else(|| "no project".to_string(), |p| p.name)
    );
    let Some(image) = image else {
        eprintln!("opificium --kiln <image.png>: name a picture to fire");
        return Some(2);
    };
    let Some(key) = key() else {
        eprintln!("no key. Put one in {}", where_the_key_goes());
        return Some(2);
    };
    // The recipe, from the command line, so a script can ask for what a panel can.
    let said: Vec<String> = std::env::args().collect();
    let asked = |flag: &str| said.iter().any(|arg| arg == flag);
    // The bench's OWN default, then whatever the flags change about it. It used to
    // build a recipe of its own with `quad: !asked("--no-quad")`, which quietly put
    // quads back on for every headless firing however the default read - and quads
    // are what force an FBX. A second place that decides what a firing asks for is a
    // second place to be wrong.
    let recipe = Recipe {
        maker: if asked("--quick") {
            Maker::Quick
        } else {
            Maker::GameReady
        },
        // Opt IN, both of them, exactly like the panel.
        quad: asked("--quad"),
        detailed: asked("--fine"),
        low_poly: !asked("--no-lowpoly"),
        ..Recipe::default()
    };
    let name = a_plain_name(
        &image
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_default(),
    );
    println!(
        "firing {} as {name}, {} credits",
        image.display(),
        recipe.credits()
    );
    match commission(&image, &name, &key, recipe, &|word| match word {
        Firing::Working(said, how_far) => match how_far {
            Some(pct) => println!("  {said} {pct:.0}%"),
            None => println!("  {said}"),
        },
        Firing::Done(road) => println!("  kept {}", road.display()),
        _ => {}
    }) {
        Ok(road) => {
            let size = std::fs::metadata(&road).map(|m| m.len()).unwrap_or(0);
            println!("{} ({size} bytes)", road.display());
            Some(0)
        }
        Err(why) => {
            eprintln!("the kiln: {why}");
            Some(1)
        }
    }
}
