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

/// The account's own purse. NOT under `/v1` - it sits at the root, unlike every other
/// call the bench makes, so it gets its own address rather than a version it does not
/// have.
const PURSE: &str = "https://api.3daistudio.com/account/user/wallet/";

/// Where credits are bought. Opened in the maker's own browser, since a bench has no
/// business handling anybody's card.
///
/// Brett's own link, and it replaces the marketing pricing page I had picked: this is the
/// platform's credit section, which is where somebody with an API key actually tops up.
const TILL: &str = "https://www.3daistudio.com/Platform/API#credit-balance";

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
    /// Roughly how long this machine takes, in seconds - the figure used until this
    /// bench has watched a firing of its own.
    ///
    /// A guess, and only ever the FALLBACK: once a recipe has been fired once, `Firings`
    /// has a real measurement and this number is not consulted again for it. It matters
    /// only on the very first firing of a setting, where being wrong costs nothing - the
    /// bar creeps slower or faster than the truth and still never claims to be finished.
    fn takes(self) -> f32 {
        match self {
            Maker::GameReady => 100.0,
            Maker::Quick => 35.0,
        }
    }

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
    /// What this exact set of choices is called, for filing how long it took.
    ///
    /// The WHOLE recipe, because Brett is right that the settings are what decide the
    /// wait: "maybe over time we could avergae all builds that shared a same settings
    /// build?" A quad remesh cut down to a game's budget is not the same firing as a
    /// plain one, and averaging the two would describe neither.
    ///
    /// Words rather than a hash, since a maker may open the file and should find it
    /// legible.
    pub fn as_a_key(&self) -> String {
        let mut key = match self.maker {
            Maker::GameReady => String::from("game-ready"),
            Maker::Quick => String::from("quick"),
        };
        for (on, word) in [
            (self.quad, "quad"),
            (self.low_poly, "lowpoly"),
            (self.detailed, "detailed"),
        ] {
            if on {
                key.push('+');
                key.push_str(word);
            }
        }
        key
    }

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

/// What the account has left, in credits.
///
/// The number arrives as a STRING - `{"balance": "1490.00", "rate_limit": 3}`, checked
/// against a live account - which is worth reading off their documentation rather than
/// assuming: a float here would have failed to parse the whole answer and reported the
/// account empty, which is a frightening thing to be told wrongly about money. A number is
/// accepted too, in case that ever changes.
///
/// Their answer carries a field the documentation does not mention, so only `balance` is
/// taken and the rest is left alone.
fn the_balance(key: &str) -> Result<f32, String> {
    let answer = ureq::get(PURSE)
        .header("Authorization", &format!("Bearer {key}"))
        .call()
        .map_err(|why| format!("could not ask what the account has: {why}"))?;
    let said: serde_json::Value = answer
        .into_body()
        .read_json()
        .map_err(|why| format!("the purse answered something else: {why}"))?;
    a_balance_in(&said)
}

/// The reading of it, apart from the fetching, so the shape can be pinned by a test.
fn a_balance_in(said: &serde_json::Value) -> Result<f32, String> {
    let balance = said.get("balance").ok_or("the purse named no balance")?;
    balance
        .as_str()
        .and_then(|said| said.trim().parse::<f32>().ok())
        .or_else(|| balance.as_f64().map(|had| had as f32))
        .ok_or_else(|| format!("the purse said {balance}, which is not a number of credits"))
}

/// Opens a page in the maker's own browser.
fn open_in_a_browser(url: &str) {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };
    if let Err(why) = std::process::Command::new(opener).arg(url).spawn() {
        warn!("could not open {url}: {why}");
    }
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
    /// It says FINISHED, has handed over nothing, and has not said why.
    ///
    /// Treated as NOT YET rather than as a failure, and this is the difference between a
    /// bench that works and one that does not. Twice in a row a firing came back
    /// `{"status":"FINISHED","progress":100,"results":[{"asset":null,...}],
    /// "failure_reason":null}` - finished, complete, no model, no complaint. Everything
    /// null at once is not what a refusal looks like; a refusal has a reason, and this
    /// provider does give one when it means it. It is what a record looks like BEFORE the
    /// file is attached to it.
    ///
    /// So the bench keeps asking for a while. The credits are already spent by this point,
    /// so waiting is free and giving up early is the only expensive choice.
    Settling,
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
        None => {
            // The whole answer to the log as well. The panel gets the first 300
            // characters, which is enough to name the trouble, and a payload big enough
            // to be cut off is exactly the one worth having in full.
            error!("the kiln's whole answer: {said}");
            format!(
                "the kiln sent no model back, and said: {}",
                first_words(said)
            )
        }
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
                // FINISHED, nothing handed over, and NO REASON: almost certainly not
                // ready rather than refused. Keep asking - see `Step::Settling`. A
                // finish that names a reason is a real refusal and fails at once,
                // because no amount of waiting fixes a rejected image.
                None if report.failure_reason.is_none() => Ok(Step::Settling),
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
    let home = crate::model::home();
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
    let home = crate::model::home();
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
    // How many empty finishes to sit through before calling it a failure. At five seconds
    // apart that is a minute and a half, which is generous for attaching a file to a
    // record that already says it is complete.
    const SETTLING: u32 = 18;
    let mut settling = 0;
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
            Step::Settling => {
                if settling == 0 {
                    info!("the kiln says finished and has attached nothing yet: {said}");
                }
                settling += 1;
                if settling > SETTLING {
                    error!("the kiln's whole answer: {said}");
                    return Err(format!(
                        "the kiln said it finished, sent no model and gave no reason, \
                         for {} seconds. It said: {}",
                        settling * 5,
                        first_words(&said)
                    ));
                }
                // Honest about which part is outstanding: the making is done by its own
                // account, and what is missing is the file.
                say(Firing::Working("ATTACHING".to_string(), Some(100.0)));
            }
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
        // Finished with NOTHING in it is never a SUCCESS - the one shape that would
        // otherwise leave the bench saying it worked and holding no file. It is not a
        // failure yet either, since no reason came with it: it settles, and fails only if
        // it stays that way. See `Step::Settling`.
        let empty = r#"{"status":"FINISHED","results":[]}"#;
        assert!(
            matches!(how_it_goes(empty), Ok(Step::Settling)),
            "an empty finish must neither succeed nor be given up on"
        );
        assert!(
            !matches!(how_it_goes(empty), Ok(Step::Ready(_))),
            "an empty finish read as a model"
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

        // WITH NO REASON GIVEN it is not a failure yet, and this is the payload two real
        // firings came back with - finished, a hundred percent, every field null. A
        // refusal has a reason; a record with nothing attached looks exactly like this,
        // so the bench keeps asking rather than throwing away a model it has paid for.
        let mute = r#"{"status":"FINISHED","progress":100,
            "results":[{"asset":null,"asset_type":"3D_MODEL","metadata":null}],
            "failure_reason":null}"#;
        assert!(
            matches!(how_it_goes(mute), Ok(Step::Settling)),
            "a mute finish must be waited on, not given up on"
        );
        // Missing entirely says the same as null.
        let bare = r#"{"status":"FINISHED","results":[{"asset":null,"asset_type":"3D_MODEL"}]}"#;
        assert!(matches!(how_it_goes(bare), Ok(Step::Settling)));

        // But a finish that names a reason fails AT ONCE. No amount of waiting fixes an
        // image the far end has already rejected, and pretending otherwise would leave a
        // maker watching a bar for another minute and a half for nothing.
        let refused = r#"{"status":"FINISHED","results":[{"asset":null}],
            "failure_reason":"the image has no clear subject"}"#;
        assert!(how_it_goes(refused).is_err());

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

/// The model standing on the kiln's stage, so it can be swept when another comes.
#[derive(Component)]
struct OnTheStage;

/// The button that names the model and writes it at the height it stands.
#[derive(Component)]
struct KeepAs;

/// A button that changes how tall the model stands.
#[derive(Component, Clone, Copy)]
struct Taller(i32);

/// The line that says how tall it is.
#[derive(Component)]
struct HeightWord;

/// The bar that fills as the kiln works.
#[derive(Component)]
struct KilnBar;

/// The line that says what the next firing will cost.
#[derive(Component)]
struct KilnPrice;

/// The button that actually spends the credits.
#[derive(Component)]
struct FireIt;

/// Its word, dimmed until there is a picture to send.
#[derive(Component)]
struct FireWord;

/// Where the chosen picture hangs.
#[derive(Component)]
struct Thumbnail;

/// The line saying what the account has left.
#[derive(Component)]
struct PurseWord;

/// The button that opens the till.
#[derive(Component)]
struct GetMore;

/// How long past firings took, filed under the settings that made them.
///
/// Brett: "What about recording the last build time and using that as an estimate?" It is
/// a better number than any I could pick - it comes from this maker's own account, their
/// own images and whatever the provider's queue is like these days, none of which I can
/// know from here.
///
/// It lives in the BENCH's support folder rather than in a project, because it is
/// knowledge about somebody else's machine and not about any game. Every project a maker
/// opens benefits from what the others learned, and no game's repository gains a file
/// describing a third party's response times.
#[derive(Resource, Default, serde::Serialize, serde::Deserialize)]
struct Firings(std::collections::BTreeMap<String, Vec<f32>>);

/// How many firings of one setting are remembered.
///
/// The last few rather than all of them: a provider gets faster and slower over months,
/// and a firing from last spring should not still be voting on what the bar does today.
const REMEMBERED: usize = 10;

impl Firings {
    /// Files a firing that WORKED. A failure's duration says nothing about how long a
    /// success takes - most failures come back fast - so recording them would drag every
    /// estimate toward the time it takes to be refused.
    fn note(&mut self, key: &str, seconds: f32) {
        // A firing that finished impossibly fast or hung for an hour is not a sample of
        // anything; it is the network or a queue, and it would sit in the average for ten
        // firings.
        if !(1.0..=15.0 * 60.0).contains(&seconds) {
            return;
        }
        let times = self.0.entry(key.to_string()).or_default();
        times.push(seconds);
        if times.len() > REMEMBERED {
            let over = times.len() - REMEMBERED;
            times.drain(..over);
        }
    }

    /// The mean of what these settings have taken before, or the given fallback when they
    /// have never been fired.
    fn the_usual(&self, key: &str, failing_that: f32) -> f32 {
        match self.0.get(key) {
            Some(times) if !times.is_empty() => times.iter().sum::<f32>() / times.len() as f32,
            _ => failing_that,
        }
    }
}

/// Where the firing times are kept.
fn firings_file() -> PathBuf {
    crate::project::support().join("firings.json")
}

/// Reads what past firings took. A missing or unreadable file is simply no history.
fn read_the_firings() -> Firings {
    std::fs::read_to_string(firings_file())
        .ok()
        .and_then(|said| serde_json::from_str(&said).ok())
        .unwrap_or_default()
}

/// Writes them back. A firing that cannot be filed is not worth interrupting a maker
/// over - the estimate is a convenience, and the model is already kept.
fn write_the_firings(firings: &Firings) {
    let road = firings_file();
    if let Some(under) = road.parent() {
        let _ = std::fs::create_dir_all(under);
    }
    if let Ok(said) = serde_json::to_string_pretty(firings) {
        if let Err(why) = std::fs::write(&road, said) {
            warn!(
                "could not keep the firing times in {}: {why}",
                road.display()
            );
        }
    }
}

/// What the account has left, and the wire while asking.
///
/// Asked on ARRIVAL at the bench and again after every firing, not every frame: it is a
/// call over the network, and the only two moments the number can have changed from the
/// maker's point of view are when they walk up to the bench and when they have just spent
/// some.
#[derive(Resource, Default)]
struct Purse {
    /// Credits, when last asked. `None` means nobody has managed to ask yet.
    left: Option<f32>,
    coming: Option<Mutex<std::sync::mpsc::Receiver<Result<f32, String>>>>,
}

/// The kiln's state, and the thread's end of the wire.
///
/// A `Receiver` is `Send` but not `Sync`, and a resource must be both - hence the
/// lock. There is one reader and it runs on the main thread, so it never waits.
#[derive(Resource)]
struct Kiln {
    firing: Firing,
    coming: Option<Mutex<std::sync::mpsc::Receiver<Firing>>>,
    /// The model on the stage: the file it came out of, and how tall it should be.
    standing: Option<PathBuf>,
    /// When the firing began, on the app's own clock, so a wait can be counted.
    began: f32,
    /// The picture last chosen, and whether it has been put on the shelf yet.
    picture: Option<PathBuf>,
    /// How tall the thing IS, in metres.
    ///
    /// A generated mesh arrives at whatever size the machine felt like, and this
    /// bench's one promise is the sixteenth-metre lattice - so a model is fitted to a
    /// height a maker states rather than trusted. The number is also the only thing
    /// about scale a game could want.
    tall: f32,
}

impl Default for Kiln {
    fn default() -> Self {
        Kiln {
            firing: Firing::Cold,
            coming: None,
            began: 0.0,
            picture: None,
            standing: None,
            // Waist high: a thing you can see the shape of on a bench, and a round
            // number of atoms.
            tall: 1.0,
        }
    }
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
            .insert_resource(read_the_firings())
            .init_resource::<Purse>()
            .add_systems(Startup, hang_the_kiln)
            .add_systems(
                Update,
                (
                    show_the_kiln,
                    take_the_choices,
                    work_the_kiln,
                    say_how_it_goes,
                    ask_the_purse,
                    hear_the_purse,
                    work_the_till,
                    say_what_is_left,
                    hang_the_picture,
                    take_a_picture,
                    dress_the_fire_button,
                    dress_the_choices,
                    stand_the_model,
                    work_the_height,
                    keep_the_model,
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
            font_size: crate::look::text_at(14.0),
            ..default()
        },
        TextColor(theme::accent(&palette)),
        ChildOf(panel),
    ));
    commands.spawn((
        Text::new("an image in, a model out"),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: crate::look::text_at(12.0),
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
                font_size: crate::look::text_at(11.0),
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
            font_size: crate::look::text_at(12.0),
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
            font_size: crate::look::text_at(12.0),
            ..default()
        },
        TextColor(theme::accent(&palette)),
        ChildOf(button),
    ));
    commands.spawn((
        crate::rail::Word(
            "Pick a picture to make a model of. Nothing is sent \
             and nothing is spent until you press GENERATE",
        ),
        ChildOf(button),
    ));

    // THE PICTURE, under the button that chose it. Brett: "When we add an image can we get
    // a preview of the image that loads on the shelf?" - and it is worth more than a
    // courtesy: a firing costs credits and takes minutes, and the one mistake worth
    // catching before spending either is having picked the wrong file.
    //
    // Empty until there is one. A frame standing empty on every launch would be a hole in
    // the panel rather than a place where something goes.
    commands.spawn((
        Thumbnail,
        Node {
            margin: UiRect::top(Val::Px(6.0)),
            width: Val::Percent(100.0),
            // Tall as it is wide, and the picture fits INSIDE that - see `hang_the_picture`.
            // A square keeps the panel from jumping about as pictures of different shapes
            // are chosen, and the buttons underneath from moving out from under the cursor.
            aspect_ratio: Some(1.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
        ChildOf(panel),
    ));

    // GENERATE, its own press. Brett: "you should have a generate button to make it
    // generate instead of auto generating when you add the image."
    //
    // Choosing and spending were one action before, which meant the preview arrived at the
    // same moment as the charge - a picture you could inspect only after paying to use it.
    // Two presses put the look between the choice and the credits, which is the only place
    // it is worth anything.
    let fire = commands
        .spawn((
            FireIt,
            Interaction::default(),
            Node {
                margin: UiRect::top(Val::Px(6.0)),
                padding: UiRect::axes(Val::Px(9.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.18)),
            BorderColor::all(theme::panel_border(&palette)),
            ChildOf(panel),
        ))
        .id();
    commands.spawn((
        FireWord,
        Text::new("GENERATE"),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: crate::look::text_at(12.0),
            ..default()
        },
        TextColor(theme::text_dim(&palette).with_alpha(0.45)),
        ChildOf(fire),
    ));
    commands.spawn((
        crate::rail::Word(
            "Send the picture and make the model. THIS is what \
             spends credits on your 3D AI Studio account",
        ),
        ChildOf(fire),
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

    // WHAT IS LEFT, and where to get more. Under the bar rather than beside the price,
    // because the price is what THIS firing costs and this is what the account holds -
    // two different facts that would read as one number if they shared a line.
    let purse_row = commands
        .spawn((
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                ..default()
            },
            ChildOf(panel),
        ))
        .id();
    commands.spawn((
        PurseWord,
        Text::new(""),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: crate::look::text_at(11.0),
            ..default()
        },
        TextColor(theme::text_dim(&palette)),
        ChildOf(purse_row),
    ));
    let more = commands
        .spawn((
            GetMore,
            Interaction::default(),
            Node {
                padding: UiRect::axes(Val::Px(7.0), Val::Px(3.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.18)),
            BorderColor::all(theme::panel_border(&palette)),
            ChildOf(purse_row),
        ))
        .id();
    commands.spawn((
        Text::new("GET MORE"),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: crate::look::text_at(10.0),
            ..default()
        },
        TextColor(theme::text_dim(&palette)),
        ChildOf(more),
    ));
    commands.spawn((
        crate::rail::Word("Open 3D AI Studio's pricing page in your browser"),
        ChildOf(more),
    ));

    // HOW TALL, and what to call it. Both only mean anything once something stands
    // on the stage, and both are quiet until then rather than absent - a control that
    // appears out of nowhere is a control nobody was looking for.
    commands.spawn((
        HeightWord,
        Text::new("1.00m tall"),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: crate::look::text_at(13.0),
            ..default()
        },
        TextColor(theme::text_dim(&palette).with_alpha(0.5)),
        Node {
            margin: UiRect::top(Val::Px(10.0)),
            ..default()
        },
        ChildOf(panel),
    ));
    let row = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            },
            ChildOf(panel),
        ))
        .id();
    for (by, label, tale) in [
        (-1, "-", "a quarter metre shorter"),
        (1, "+", "a quarter metre taller"),
    ] {
        let button = commands
            .spawn((
                Taller(by),
                Interaction::default(),
                Node {
                    width: Val::Px(38.0),
                    padding: UiRect::vertical(Val::Px(3.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::BLACK.with_alpha(0.18)),
                BorderColor::all(theme::panel_border(&palette)),
                ChildOf(row),
            ))
            .id();
        commands.spawn((
            Text::new(label),
            TextFont {
                font: fonts.display.clone().into(),
                font_size: crate::look::text_at(13.0),
                ..default()
            },
            TextColor(theme::accent(&palette)),
            ChildOf(button),
        ));
        commands.spawn((crate::rail::Word(tale), ChildOf(button)));
    }

    let keep = commands
        .spawn((
            KeepAs,
            Interaction::default(),
            Node {
                margin: UiRect::top(Val::Px(6.0)),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.18)),
            BorderColor::all(theme::accent(&palette).with_alpha(0.5)),
            ChildOf(panel),
        ))
        .id();
    commands.spawn((
        Text::new("KEEP IT AS..."),
        TextFont {
            font: fonts.display.clone().into(),
            font_size: crate::look::text_at(12.0),
            ..default()
        },
        TextColor(theme::accent(&palette)),
        ChildOf(keep),
    ));
    commands.spawn((
        crate::rail::Word(
            "name it, and write it at the height it stands - \
             the game reads it out of the project",
        ),
        ChildOf(keep),
    ));

    commands.spawn((
        KilnWord,
        Text::new(""),
        TextFont {
            font: fonts.text.clone().into(),
            font_size: crate::look::text_at(12.0),
            ..default()
        },
        // BREAK ANYWHERE. What lands here when a firing fails is the machine's own answer,
        // and a JSON payload contains no spaces at all - so word wrapping, which is the
        // default, has nowhere it is willing to break and the line runs off the panel.
        // Brett hit this on a mute failure: the part of the message that says what went
        // wrong was the part past the edge. The same clipping would have swallowed any
        // long reason the machine gave in plain words.
        TextLayout::new(Justify::Left, LineBreak::AnyCharacter),
        TextColor(theme::text_dim(&palette)),
        Node {
            margin: UiRect::top(Val::Px(6.0)),
            ..default()
        },
        ChildOf(panel),
    ));
}

/// Says what the account has left, and warns when a firing would not fit in it.
fn say_what_is_left(
    purse: Res<Purse>,
    recipe: Res<Recipe>,
    palette: Res<Palette>,
    mut words: Query<(&mut Text, &mut TextColor), With<PurseWord>>,
) {
    if !purse.is_changed() && !recipe.is_changed() {
        return;
    }
    let (said, dye) = match purse.left {
        // Not enough for the firing about to be asked for, which is worth saying BEFORE
        // the button is pressed rather than after the machine refuses.
        Some(left) if left < recipe.credits() as f32 => (
            format!("{left:.0} left - not enough"),
            palette.shade("cloth-red", 0.85),
        ),
        Some(left) => (format!("{left:.0} credits left"), theme::text_dim(&palette)),
        // Nothing said until there is something true to say.
        None => (String::new(), theme::text_dim(&palette)),
    };
    for (mut text, mut colour) in &mut words {
        if text.0 != said {
            *text = Text::new(said.clone());
            *colour = TextColor(dye);
        }
    }
}

/// The panel belongs to the kiln bench.
fn show_the_kiln(
    bench: Res<crate::Bench>,
    showing: Res<crate::look::Showing>,
    mut panels: Query<&mut Visibility, With<KilnPanel>>,
) {
    if !bench.is_changed() && !showing.is_changed() {
        return;
    }
    let out = *bench == crate::Bench::Kiln && showing.wanted(crate::look::Tool::Shelf);
    for mut it in &mut panels {
        *it = if out {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Names the model and writes it at the height it stands.
///
/// A save dialog rather than the builder's naming card, for the reason the rig bench
/// keeps its clips the same way: the card was written when the only place a thing
/// could go was the bench's own folder, and a dialog names a file and finds it a home
/// in one gesture. It opens in the project's own `out/models`, which is where a game
/// reads them from.
fn keep_the_model(
    _main_thread: bevy::ecs::system::NonSendMarker,
    bench: Res<crate::Bench>,
    mut kiln: ResMut<Kiln>,
    pressed: Query<&Interaction, (Changed<Interaction>, With<KeepAs>)>,
) {
    if *bench != crate::Bench::Kiln || !pressed.iter().any(|touch| *touch == Interaction::Pressed) {
        return;
    }
    let Some(from) = kiln.standing.clone() else {
        kiln.firing = Firing::Failed("nothing stands on the bench to keep".to_string());
        return;
    };
    let home = crate::model::home();
    let _ = std::fs::create_dir_all(&home);
    let suggested = from
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "model".to_string());
    let Some(to) = rfd::FileDialog::new()
        .set_title("Keep the model")
        .add_filter("glTF binary", &["glb"])
        .set_directory(&home)
        .set_file_name(format!("{suggested}.glb"))
        .save_file()
    else {
        return;
    };
    let tall = kiln.tall;
    match crate::model::keep_at_height(&from, &to, tall) {
        Ok(road) => {
            info!("kept {} at {tall:.2}m", road.display());
            // What was kept is what now stands, so the height reads as one it has
            // rather than one it is waiting for.
            kiln.standing = Some(road.clone());
            kiln.firing = Firing::Done(road);
        }
        Err(why) => kiln.firing = Firing::Failed(why),
    }
}

/// Raises and lowers the model, and says how tall it stands.
///
/// In quarter metres, and on the lattice like everything else on this bench: a model
/// is going into a game whose walls are 2.5 metres, and a fly the size of a longhouse
/// is the first thing anybody notices.
fn work_the_height(
    bench: Res<crate::Bench>,
    mut kiln: ResMut<Kiln>,
    palette: Res<Palette>,
    pressed: Query<(&Interaction, &Taller), Changed<Interaction>>,
    mut words: Query<(&mut Text, &mut TextColor), With<HeightWord>>,
) {
    if *bench == crate::Bench::Kiln {
        for (touch, by) in &pressed {
            if *touch != Interaction::Pressed {
                continue;
            }
            // Four atoms a press, and never nothing: a model of no height is a model
            // nobody can see.
            let step = by.0 as f32 * 0.25;
            kiln.tall = (kiln.tall + step).clamp(0.25, 20.0);
        }
    }
    if !kiln.is_changed() {
        return;
    }
    let said = format!("{:.2}m tall", kiln.tall);
    for (mut text, mut dye) in &mut words {
        if text.0 != said {
            *text = Text::new(said.clone());
        }
        // Gold while something stands to be measured, quiet when the stage is bare.
        *dye = TextColor(if kiln.standing.is_some() {
            theme::accent(&palette)
        } else {
            theme::text_dim(&palette).with_alpha(0.5)
        });
    }
}

/// Stands the model on the stage, and fits it to the height it was told.
///
/// The height a maker states is the one fact that settles the size, because a generated
/// mesh has no idea what size the thing it depicts is - two from the same machine differ
/// by a factor of ten. `model::stand` does the measuring.
fn stand_the_model(
    mut commands: Commands,
    kiln: Res<Kiln>,
    assets: Res<AssetServer>,
    standing: Query<Entity, With<OnTheStage>>,
    mut showing: Local<Option<(PathBuf, f32)>>,
) {
    let wanted = kiln.standing.as_ref().map(|road| (road.clone(), kiln.tall));
    if *showing == wanted {
        return;
    }
    showing.clone_from(&wanted);
    for old in &standing {
        commands.entity(old).despawn();
    }
    let Some((road, tall)) = wanted else {
        return;
    };
    commands.spawn((
        OnTheStage,
        crate::stage::KilnFurniture,
        crate::model::stand(&assets, &road, Some(tall)),
    ));
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
    mut firings: ResMut<Firings>,
    clock: Res<Time>,
    picks: Query<&Interaction, (Changed<Interaction>, With<FireIt>)>,
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
        // What lands, stands. The stage is where a maker judges whether it was
        // worth the credits, which is the whole reason to show it at all.
        if let Firing::Done(road) = &word {
            // What it took, filed under the settings that took it. Timed from the press of
            // GENERATE, which is now purely the machine's own work: choosing the picture is
            // a separate press, so no part of a maker's browsing is counted.
            firings.note(&recipe.as_a_key(), clock.elapsed_secs() - kiln.began);
            write_the_firings(&firings);
            kiln.standing = Some(road.clone());
        }
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
    kiln.picture = Some(image.clone());
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
    kiln.began = clock.elapsed_secs();
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

/// How full the bar should be, and whether the number is the MACHINE'S or the bench's.
///
/// The game-ready machine reports `progress: 0` for the whole firing and then hands over
/// a model, so a bar wired only to its number sits empty for two minutes and then
/// vanishes - Brett: "Right now I am generating a model and it just says In Progress..."
///
/// So when there is a real number it is used and said to be real. When there is not, the
/// bar creeps against how long this machine usually takes: fast at first and slower ever
/// after, approaching but never reaching the end. That curve is chosen for what it CANNOT
/// do - it cannot arrive, so a bar that sits at nine tenths is telling the truth about
/// not knowing, and a slow firing never leaves the bar pinned at a full bar that lies.
///
/// The caller draws the bench's own guess dimmer than the machine's own number, so the
/// two are never mistaken for each other.
fn how_full(how_far: Option<f32>, waited: f32, takes: f32) -> (f32, bool) {
    // A nought from the machine is NOT news that nothing has happened - it is the
    // game-ready machine's way of saying it does not count. Treated as no number at all.
    if let Some(said) = how_far {
        if said > 0.0 {
            return (said.clamp(0.0, 100.0), true);
        }
    }
    let waited = waited.max(0.0);
    let takes = if takes > 1.0 { takes } else { 1.0 };
    // 1 - e^-x: 63% of the way at the expected time, 86% at twice it, never 100%.
    (92.0 * (1.0 - (-waited / takes).exp()), false)
}

/// A wait in minutes and seconds, which is the one thing about it that is certainly true.
fn as_a_clock(seconds: f32) -> String {
    let seconds = seconds.max(0.0) as u32;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// Chooses a picture. Sends nothing and spends nothing.
///
/// Its own press, apart from the firing. Brett: "after you add the image it should load the
/// preview and then you should have a generate button to make it generate instead of auto
/// generating when you add the image." The two used to be one action, so the preview
/// arrived at the same instant as the charge - a picture a maker could inspect only after
/// paying to use it.
///
/// No key is needed to choose one, either. Being told to go and find an API key is a
/// reasonable thing to hear when about to spend credits, and a strange thing to hear when
/// opening a file dialog.
fn take_a_picture(
    _main_thread: bevy::ecs::system::NonSendMarker,
    bench: Res<crate::Bench>,
    mut kiln: ResMut<Kiln>,
    picks: Query<&Interaction, (Changed<Interaction>, With<PickAnImage>)>,
) {
    if *bench != crate::Bench::Kiln || matches!(kiln.firing, Firing::Working(..)) {
        return;
    }
    if !picks.iter().any(|touch| *touch == Interaction::Pressed) {
        return;
    }
    if let Some(image) = rfd::FileDialog::new()
        .set_title("An image for the kiln")
        .add_filter("Pictures", &["png", "jpg", "jpeg", "webp"])
        .pick_file()
    {
        kiln.picture = Some(image);
    }
}

/// GENERATE reads as live only when there is something to send.
fn dress_the_fire_button(
    kiln: Res<Kiln>,
    palette: Res<Palette>,
    mut words: Query<&mut TextColor, With<FireWord>>,
    mut buttons: Query<&mut BorderColor, With<FireIt>>,
) {
    if !kiln.is_changed() {
        return;
    }
    let ready = kiln.picture.is_some() && !matches!(kiln.firing, Firing::Working(..));
    for mut colour in &mut words {
        *colour = TextColor(if ready {
            theme::accent(&palette)
        } else {
            theme::text_dim(&palette).with_alpha(0.45)
        });
    }
    for mut edge in &mut buttons {
        *edge = BorderColor::all(if ready {
            theme::accent(&palette).with_alpha(0.6)
        } else {
            theme::panel_border(&palette)
        });
    }
}

/// Puts the chosen picture on the shelf.
///
/// DECODED HERE rather than loaded by the asset server, because a maker's picture lives
/// wherever they keep their pictures - outside the bench's `assets/` and outside the
/// project, which are the only two places the asset server has been told to look. Reading
/// the bytes and handing over a decoded image asks nothing of the file's whereabouts.
///
/// A picture that cannot be decoded leaves the frame empty and says nothing. The firing is
/// about to say it far more plainly, since whatever the bench cannot read it also cannot
/// send.
fn hang_the_picture(
    mut commands: Commands,
    kiln: Res<Kiln>,
    mut pictures: ResMut<Assets<Image>>,
    frames: Query<Entity, With<Thumbnail>>,
    mut showing: Local<Option<PathBuf>>,
) {
    if *showing == kiln.picture {
        return;
    }
    showing.clone_from(&kiln.picture);
    let Ok(frame) = frames.single() else {
        return;
    };
    commands.entity(frame).despawn_related::<Children>();
    let Some(road) = kiln.picture.clone() else {
        return;
    };
    let Some(picture) = a_picture_from(&road, &mut pictures) else {
        warn!("could not read {} to show it", road.display());
        return;
    };
    commands.spawn((
        ImageNode::new(picture),
        Node {
            // CONTAINED, not stretched: the frame is square and a picture rarely is, so
            // the picture is fitted inside it whole. A stretched preview would misreport
            // the one thing it is being looked at for - the shape of what is being sent.
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        ChildOf(frame),
    ));
}

/// Reads a picture off the disk into something the bench can draw.
fn a_picture_from(road: &Path, pictures: &mut Assets<Image>) -> Option<Handle<Image>> {
    let bytes = std::fs::read(road).ok()?;
    let kind = road
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())?;
    let picture = Image::from_buffer(
        &bytes,
        bevy::image::ImageType::Extension(&kind),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::linear(),
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    )
    .ok()?;
    Some(pictures.add(picture))
}

/// Asks what the account has left, off the frame loop.
///
/// A firing that has just finished spent credits, so the number on the panel is stale the
/// moment it succeeds - which is exactly when a maker looks at it.
fn ask_the_purse(
    bench: Res<crate::Bench>,
    kiln: Res<Kiln>,
    mut purse: ResMut<Purse>,
    mut was: Local<Option<Firing>>,
) {
    let spent = was.as_ref() != Some(&kiln.firing) && matches!(kiln.firing, Firing::Done(_));
    if was.as_ref() != Some(&kiln.firing) {
        *was = Some(kiln.firing.clone());
    }
    let arrived = bench.is_changed() && *bench == crate::Bench::Kiln;
    // One question at a time, and never before there is a key to ask with.
    if (!arrived && !spent) || purse.coming.is_some() {
        return;
    }
    let Some(key) = key() else {
        return;
    };
    let (say, hear) = std::sync::mpsc::channel();
    purse.coming = Some(Mutex::new(hear));
    std::thread::spawn(move || {
        let _ = say.send(the_balance(&key));
    });
}

/// Takes the answer when it comes.
fn hear_the_purse(mut purse: ResMut<Purse>) {
    let heard = purse
        .coming
        .as_ref()
        .and_then(|wire| wire.lock().ok().and_then(|wire| wire.try_recv().ok()));
    let Some(heard) = heard else {
        return;
    };
    purse.coming = None;
    match heard {
        Ok(left) => {
            info!("the account has {left:.0} credits");
            purse.left = Some(left);
        }
        // A purse that cannot be counted is left unsaid rather than reported as a fault.
        // It is a courtesy on the panel, and a maker who cannot reach it has a firing to
        // get on with; the failure of the firing itself will say so far more clearly.
        Err(why) => info!("the kiln could not read the account's balance: {why}"),
    }
}

/// A press on GET MORE opens the till in the maker's own browser.
fn work_the_till(
    bench: Res<crate::Bench>,
    picks: Query<&Interaction, (Changed<Interaction>, With<GetMore>)>,
) {
    if *bench != crate::Bench::Kiln {
        return;
    }
    if picks.iter().any(|touch| *touch == Interaction::Pressed) {
        open_in_a_browser(TILL);
    }
}

/// Says where the firing has got to, in the panel.
fn say_how_it_goes(
    kiln: Res<Kiln>,
    recipe: Res<Recipe>,
    firings: Res<Firings>,
    clock: Res<Time>,
    palette: Res<Palette>,
    mut words: Query<(&mut Text, &mut TextColor), With<KilnWord>>,
    mut bars: Query<(&mut Node, &mut BackgroundColor), With<KilnBar>>,
) {
    // Every frame WHILE WORKING, not only when something changes: the wait itself is what
    // moves, and a bar that only redrew on news would sit still through the whole firing -
    // which is the complaint this is answering.
    let working = matches!(kiln.firing, Firing::Working(..));
    if !working && !kiln.is_changed() && !recipe.is_changed() {
        return;
    }

    let waited = clock.elapsed_secs() - kiln.began;
    // What THIS recipe has taken before, or my rough figure if it has never been fired.
    let expects = firings.the_usual(&recipe.as_a_key(), recipe.maker.takes());
    let (how_far, machines_own) = match &kiln.firing {
        Firing::Working(_, how_far) => how_full(*how_far, waited, expects),
        Firing::Done(_) => (100.0, true),
        _ => (0.0, true),
    };
    for (mut bar, mut dye) in &mut bars {
        bar.width = Val::Percent(how_far);
        // The bench's own guess is drawn dimmer than a number the machine stands behind,
        // so a full-strength bar always means somebody out there counted.
        *dye = BackgroundColor(if machines_own {
            theme::accent(&palette)
        } else {
            theme::accent(&palette).with_alpha(0.45)
        });
    }

    let (said, dye) = match &kiln.firing {
        // What it will cost and how long it has taken BEFORE, which is the pair of facts
        // worth having before spending credits. "About a minute" was a guess printed at
        // every maker regardless of what their own firings did.
        Firing::Cold => (
            format!(
                "{} credits, and about {}",
                recipe.credits(),
                as_a_clock(expects)
            ),
            theme::text_dim(&palette),
        ),
        // The elapsed time rather than a percentage, when the percentage would be ours
        // rather than theirs. A clock is a fact; a made-up percentage is a small lie that
        // a maker would reasonably plan around.
        Firing::Working(word, how_far) => (
            match how_far {
                Some(said) if *said > 0.0 => {
                    format!("{}... {said:.0}%", word.to_lowercase().replace('_', " "))
                }
                _ => format!(
                    "{}... {}",
                    word.to_lowercase().replace('_', " "),
                    as_a_clock(waited)
                ),
            },
            theme::accent(&palette),
        ),
        Firing::Done(road) => (
            format!(
                "kept {} in {}",
                road.file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_default(),
                as_a_clock(waited)
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
mod waiting {
    use super::*;

    /// A number the machine stands behind is used as given, and said to be its own.
    #[test]
    fn the_machines_own_number_wins() {
        let (how_far, machines_own) = how_full(Some(42.0), 5.0, 100.0);
        assert!((how_far - 42.0).abs() < 1e-3, "{how_far}");
        assert!(
            machines_own,
            "a real percentage should be drawn as a real one"
        );
        // Even a silly one, clamped rather than believed.
        assert_eq!(how_full(Some(180.0), 5.0, 100.0).0, 100.0);
    }

    /// A nought is the game-ready machine declining to count, NOT a report of no
    /// progress - which is the whole bug: it reports 0 for the entire firing.
    #[test]
    fn a_nought_is_not_a_report() {
        let (how_far, machines_own) = how_full(Some(0.0), 50.0, 100.0);
        assert!(
            how_far > 1.0,
            "a nought froze the bar at nothing: {how_far}"
        );
        assert!(
            !machines_own,
            "the bench's own guess must not pose as the machine's"
        );
        // And absent says the same thing as nought.
        assert_eq!(
            how_full(None, 50.0, 100.0),
            how_full(Some(0.0), 50.0, 100.0)
        );
    }

    /// The guess NEVER arrives, however long the wait.
    ///
    /// The point of the curve. A bar that reached the end would be claiming the model was
    /// done, and the one thing the bench does not know is when that will be - so it must
    /// be unable to say so, not merely unlikely to.
    #[test]
    fn the_guess_never_claims_to_be_finished() {
        for waited in [0.0, 30.0, 100.0, 600.0, 86_400.0] {
            let (how_far, _) = how_full(None, waited, 100.0);
            assert!(how_far < 100.0, "at {waited}s the bar claimed {how_far}%");
        }
        // And it only ever moves forwards.
        let mut last = -1.0;
        for waited in [0.0, 10.0, 25.0, 60.0, 150.0, 400.0] {
            let (how_far, _) = how_full(None, waited, 100.0);
            assert!(how_far >= last, "it went backwards at {waited}s");
            last = how_far;
        }
        // Empty at the start, rather than a courtesy nudge that means nothing.
        assert!(how_full(None, 0.0, 100.0).0 < 0.01);
    }

    /// A wait reads as a clock, because the seconds are the part that is certainly true.
    #[test]
    fn a_wait_reads_as_a_clock() {
        assert_eq!(as_a_clock(0.0), "0:00");
        assert_eq!(as_a_clock(9.6), "0:09");
        assert_eq!(as_a_clock(60.0), "1:00");
        assert_eq!(as_a_clock(154.0), "2:34");
        // A clock that ran backwards would be a negative-second string.
        assert_eq!(as_a_clock(-5.0), "0:00");
    }
}

#[cfg(test)]
mod the_purse {
    use super::*;

    /// A balance arrives as a STRING, and is read as one.
    ///
    /// Their own documentation shows `{"balance": "150.00"}` - quoted. Typed as a number
    /// this would have failed to parse and the panel would have said the account was
    /// empty, which is an alarming thing to be told wrongly about money.
    #[test]
    fn a_balance_is_quoted() {
        let said = serde_json::json!({ "balance": "150.00" });
        assert_eq!(a_balance_in(&said), Ok(150.0));
        // The answer a live account actually gave, which carries a field their
        // documentation does not mention - so the reading must take what it came for and
        // ignore the rest rather than insisting on a shape.
        let real = serde_json::json!({ "balance": "1490.00", "rate_limit": 3 });
        assert_eq!(a_balance_in(&real), Ok(1490.0));
        // And read as a number too, in case that ever changes under us.
        assert_eq!(
            a_balance_in(&serde_json::json!({ "balance": 150 })),
            Ok(150.0)
        );
        assert_eq!(
            a_balance_in(&serde_json::json!({ "balance": 42.5 })),
            Ok(42.5)
        );
    }

    /// An answer with no balance in it is an error, not a nought.
    ///
    /// Nought would be indistinguishable from a spent account, and would put "not enough"
    /// on the panel of somebody with plenty.
    #[test]
    fn a_missing_balance_is_not_nothing() {
        assert!(a_balance_in(&serde_json::json!({})).is_err());
        assert!(a_balance_in(&serde_json::json!({ "credits": "10" })).is_err());
        assert!(a_balance_in(&serde_json::json!({ "balance": "plenty" })).is_err());
        assert!(a_balance_in(&serde_json::json!({ "balance": null })).is_err());
    }
}

#[cfg(test)]
mod remembering {
    use super::*;

    /// Firings are filed under the WHOLE recipe, so unlike settings never share an average.
    #[test]
    fn settings_are_filed_apart() {
        let plain = Recipe {
            maker: Maker::GameReady,
            quad: false,
            low_poly: false,
            detailed: false,
        };
        let heavy = Recipe {
            maker: Maker::GameReady,
            quad: true,
            low_poly: true,
            detailed: true,
        };
        assert_eq!(plain.as_a_key(), "game-ready");
        assert_eq!(heavy.as_a_key(), "game-ready+quad+lowpoly+detailed");
        assert_ne!(
            plain.as_a_key(),
            Recipe {
                maker: Maker::Quick,
                ..plain
            }
            .as_a_key(),
            "two machines must not share one average"
        );
        // The same settings always name themselves the same way, or yesterday's firings
        // are filed where nothing will look for them.
        assert_eq!(plain.as_a_key(), plain.as_a_key());
    }

    /// The estimate is the mean of what those settings took before.
    #[test]
    fn it_averages_what_went_before() {
        let mut firings = Firings::default();
        assert!(
            (firings.the_usual("game-ready", 100.0) - 100.0).abs() < 1e-3,
            "with no history it must fall back to the rough figure"
        );
        firings.note("game-ready", 60.0);
        firings.note("game-ready", 90.0);
        assert!((firings.the_usual("game-ready", 100.0) - 75.0).abs() < 1e-3);
        // And another setting's firings do not leak into it.
        firings.note("quick", 20.0);
        assert!((firings.the_usual("game-ready", 100.0) - 75.0).abs() < 1e-3);
        assert!((firings.the_usual("quick", 100.0) - 20.0).abs() < 1e-3);
    }

    /// Only the last few are kept, and the OLDEST are the ones that go.
    #[test]
    fn it_forgets_the_oldest_first() {
        let mut firings = Firings::default();
        for i in 0..REMEMBERED + 5 {
            firings.note("game-ready", 10.0 + i as f32);
        }
        let times = firings.0.get("game-ready").expect("some history");
        assert_eq!(times.len(), REMEMBERED, "it kept more than it promised");
        assert_eq!(
            times.first(),
            Some(&15.0),
            "it dropped the newest instead of the oldest"
        );
        assert_eq!(times.last(), Some(&(10.0 + (REMEMBERED + 4) as f32)));
    }

    /// A firing that took no time or hung for an hour is not a sample of anything.
    #[test]
    fn nonsense_is_not_remembered() {
        let mut firings = Firings::default();
        firings.note("game-ready", 0.0);
        firings.note("game-ready", -12.0);
        firings.note("game-ready", 60.0 * 60.0);
        assert!(
            firings
                .0
                .get("game-ready")
                .is_none_or(|times| times.is_empty()),
            "it believed a nonsense duration, which would skew ten firings"
        );
        firings.note("game-ready", 95.0);
        assert!((firings.the_usual("game-ready", 1.0) - 95.0).abs() < 1e-3);
    }

    /// What is written can be read back, which is the whole point of writing it.
    #[test]
    fn a_history_survives_the_round_trip() {
        let mut firings = Firings::default();
        firings.note("game-ready+lowpoly", 104.0);
        let said = serde_json::to_string(&firings).expect("written");
        let back: Firings = serde_json::from_str(&said).expect("read back");
        assert!((back.the_usual("game-ready+lowpoly", 1.0) - 104.0).abs() < 1e-3);
        // And a file of rubbish is no history rather than a panic.
        assert!(serde_json::from_str::<Firings>("{{not json").is_err());
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
