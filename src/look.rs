//! The game's look, carried in as data.
//!
//! The palette arrives from the game via the project's `data/palette.json` — written by
//! the game's own `export_palette_for_opificium` test — so nothing here can
//! drift from what the game actually draws.
//!
//! A game that has not written one yet is the ordinary case rather than the
//! broken one: a bench is where a game's look is DECIDED, so it must open and
//! paint before any game has an opinion. Without a file it wears its own ramps —
//! see [`BENCH_RAMPS`] — and says so. A project's file wins name for name the
//! moment it exists.

use bevy::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct PaletteFile {
    ramps: Vec<RampEntry>,
}

#[derive(Deserialize)]
struct RampEntry {
    name: String,
    steps: [[u8; 3]; 5],
}

/// The game's ramps, by name.
#[derive(Resource, Default)]
pub struct Palette {
    ramps: Vec<(String, [[u8; 3]; 5])>,
}

impl Palette {
    pub fn ramp(&self, name: &str) -> Option<&[[u8; 3]; 5]> {
        self.ramps
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, ramp)| ramp)
    }

    /// A step from a named ramp, 0 shadow to 1 bright — the game's own
    /// `palette::shade`, re-derived from the data.
    pub fn shade(&self, name: &str, t: f32) -> Color {
        let Some(ramp) = self.ramp(name) else {
            return Color::srgb(0.8, 0.2, 0.8); // the classic missing-colour
        };
        let idx = ((t.clamp(0.0, 1.0) * 4.0).round() as usize).min(4);
        let [r, g, b] = ramp[idx];
        Color::srgb_u8(r, g, b)
    }

    #[allow(dead_code)] // the Builder bench's ramp picker reads this
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.ramps.iter().map(|(n, _)| n.as_str())
    }
}

/// Opificium wears the codex's colours.
pub mod theme {
    use super::Palette;
    use bevy::prelude::*;

    pub fn panel_bg() -> Color {
        Color::srgb(0.045, 0.05, 0.062).with_alpha(0.985)
    }

    pub fn panel_border(palette: &Palette) -> Color {
        palette.shade("cloth-gold", 0.55).with_alpha(0.35)
    }

    pub fn accent(palette: &Palette) -> Color {
        palette.shade("cloth-gold", 0.85)
    }

    #[allow(dead_code)] // body text, once the benches carry prose
    pub fn text(palette: &Palette) -> Color {
        palette.shade("bone", 0.97)
    }

    pub fn text_dim(palette: &Palette) -> Color {
        palette.shade("bone", 0.78)
    }
}

/// How wide the panels down either edge stand.
///
/// One number for both, because they flank the same stage: a bench with mismatched
/// margins reads as a bench that was assembled rather than drawn. It was written out
/// in four places - the rail, the shelf, the palette and the body shelf - which was
/// harmless while nothing else cared, and stopped being harmless the moment
/// something had to know where the VIEW begins. It begins here.
pub const PANEL_WIDE: f32 = 232.0;

/// One piece of the window's furniture, that a maker may put away.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    /// The row across the top: the modes at the builder, the model's name at the rig.
    TopBar,
    /// The phases along the bottom.
    StageBar,
    /// The panel down the right, whichever bench's it is.
    Shelf,
    /// The rail down the left.
    Rail,
    /// The measuring post on the stage.
    Ruler,
}

/// Which pieces of furniture the maker wants on the bench.
///
/// Brett: "tools like the top bar and the stage bar that you could toggle them on or off?
/// They arent needed in some places." A bench is for looking at the work, and every row of
/// buttons is a strip of the work it covers.
///
/// This says what the MAKER wants. Whether a piece is actually on screen is that AND
/// whether it belongs to the bench they are standing at - a builder's mode row switched on
/// is still no business of the kiln's. Every system that hides furniture reads both.
#[derive(Resource)]
pub struct Showing {
    top_bar: bool,
    stage_bar: bool,
    shelf: bool,
    rail: bool,
    ruler: bool,
}

impl Default for Showing {
    fn default() -> Self {
        // All of it, because a bench that opened with its tools put away would look
        // broken to somebody who never asked for them to be.
        Showing {
            top_bar: true,
            stage_bar: true,
            shelf: true,
            rail: true,
            // Except the ruler, which is a thing you REACH FOR. A measuring post standing
            // on the bench at all times is furniture in the way of the work.
            ruler: false,
        }
    }
}

impl Showing {
    /// Whether the maker wants this piece at all.
    pub fn wanted(&self, tool: Tool) -> bool {
        match tool {
            Tool::TopBar => self.top_bar,
            Tool::StageBar => self.stage_bar,
            Tool::Shelf => self.shelf,
            Tool::Rail => self.rail,
            Tool::Ruler => self.ruler,
        }
    }

    /// Puts it away, or takes it back out.
    pub fn flip(&mut self, tool: Tool) {
        let now = !self.wanted(tool);
        match tool {
            Tool::TopBar => self.top_bar = now,
            Tool::StageBar => self.stage_bar = now,
            Tool::Shelf => self.shelf = now,
            Tool::Rail => self.rail = now,
            Tool::Ruler => self.ruler = now,
        }
    }
}

/// How much bigger every word on the bench is than it was first drawn.
///
/// ONE KNOB for the whole app. Brett: "this small text is used throughout the app, is
/// there a way we can make it bigger?" - and the honest answer was no, there wasn't: sixty
/// call sites each named their own number, so making the small text bigger meant sixty
/// edits and the next adjustment would mean sixty more.
///
/// ADDED rather than multiplied, which is the whole point. The complaint is about the
/// SMALL text, and adding two lifts a nine-pixel swatch label by a fifth while leaving a
/// twenty-six-pixel title almost as it was. Multiplying would have grown the headings most
/// and the labels that are actually hard to read least.
///
/// The sizes at the call sites keep their own numbers, because the difference between a
/// title and a label is real design and not something to flatten into one name. This only
/// moves them all together.
pub const BIGGER: f32 = 2.0;

/// A size of text, in pixels, grown by [`BIGGER`].
pub fn text_at(px: f32) -> bevy::text::FontSize {
    bevy::text::FontSize::Px(px + BIGGER)
}

/// A pane the wheel scrolls when the cursor is over it.
#[derive(Component)]
pub struct Scrollable;

/// Wheel over a scrollable pane walks it, and says so, so the camera
/// knows to keep its hands off the zoom.
pub fn scroll_panes(
    wheel: Res<bevy::input::mouse::AccumulatedMouseScroll>,
    windows: Query<&Window>,
    mut panes: Query<
        (
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            &mut ScrollPosition,
        ),
        With<Scrollable>,
    >,
    mut over_pane: ResMut<OverPane>,
) {
    let cursor = windows
        .iter()
        .next()
        .and_then(|window| window.cursor_position());
    over_pane.0 = false;
    let Some(cursor) = cursor else {
        return;
    };
    // Hit-tested by geometry, not by hover: a button inside the pane
    // would otherwise swallow the wheel.
    for (computed, transform, visibility, mut scroll) in &mut panes {
        if !visibility.get() {
            continue;
        }
        let scale = computed.inverse_scale_factor();
        let centre = Vec2::new(transform.translation.x, transform.translation.y) * scale;
        let half = computed.size() * scale * 0.5;
        if (cursor.x - centre.x).abs() <= half.x && (cursor.y - centre.y).abs() <= half.y {
            over_pane.0 = true;
            if wheel.delta.y != 0.0 {
                // A line's worth per notch, and trackpads send small
                // fractions of one - both read the same speed.
                let notches = match wheel.unit {
                    bevy::input::mouse::MouseScrollUnit::Line => wheel.delta.y * 3.0,
                    bevy::input::mouse::MouseScrollUnit::Pixel => wheel.delta.y * 0.14,
                };
                scroll.0.y -= notches * 22.0;
            }
        }
    }
}

/// Whether the cursor rests over a scrollable pane - the camera reads
/// this and leaves the wheel alone.
#[derive(Resource, Default)]
pub struct OverPane(pub bool);

/// Display and body faces, the game's own.
///
/// `Default` is two handles to nothing, which is meaningless on a bench and exactly right
/// in a headless test: a system that hangs a menu wants the resource to EXIST, and a test
/// that never draws a glyph does not care which face it would have used.
#[derive(Resource, Default)]
pub struct Fonts {
    pub display: Handle<Font>,
    pub text: Handle<Font>,
}

pub struct LookPlugin;

impl Plugin for LookPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(load_palette())
            .insert_resource(ClearColor(Color::srgb(0.035, 0.04, 0.05)))
            .init_resource::<OverPane>()
            .init_resource::<Showing>()
            .add_systems(PreStartup, load_fonts)
            .add_systems(Update, scroll_panes);
    }
}

fn load_fonts(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Fonts {
        display: assets.load("fonts/Cinzel.ttf"),
        text: assets.load("fonts/EBGaramond.ttf"),
    });
}

/// The palette, for the bake to resolve colours with.
///
/// No longer test-only: the headless bake in [`crate::bake`] is ordinary code
/// and needs the same colours the button does.
pub fn load_palette_for_bake() -> Palette {
    load_palette()
}

/// Reads the palette relative to the crate first (so `cargo run` works from
/// anywhere), then the working directory, then gives up gracefully.
fn load_palette() -> Palette {
    // The open project's own palette first: every game paints in its own
    // colours, and that file is the one thing the game must hand the
    // bench before any authored work means anything.
    let mut roads = vec![crate::project::palette()];
    if let Ok(exe) = std::env::current_exe()
        && let Some(beside) = exe.parent()
    {
        roads.push(beside.join("data/palette.json"));
        roads.push(beside.join("../Resources/data/palette.json"));
    }
    roads.push("data/palette.json".into());

    for road in roads {
        if let Ok(text) = std::fs::read_to_string(&road)
            && let Ok(file) = serde_json::from_str::<PaletteFile>(&text)
        {
            info!(
                "palette: {} ramps from {}",
                file.ramps.len(),
                road.display()
            );
            return Palette {
                ramps: file
                    .ramps
                    .into_iter()
                    .map(|entry| (entry.name, entry.steps))
                    .collect(),
            };
        }
    }
    warn!(
        "no palette in {} - painting in the bench's own {} ramps. A game that \
         wants its own exports one; see FORMATS.md",
        crate::project::palette().display(),
        BENCH_RAMPS.len(),
    );
    bench_palette()
}

/// The colours the bench wears when no game has handed it any.
///
/// A game's palette is that game's own truth and arrives as data. These are not
/// that. They are the BENCH's own dress: what it paints with while a project is
/// still empty — which, for a game being started from nothing, is every day of
/// the first week.
///
/// It used to be two ramps, bone and gold. They dressed the rail and the grid,
/// and left everything else MAGENTA: `shade` answers a ramp it does not know
/// with the classic missing-colour, and `body_of` names fourteen. So a maker who
/// pointed the bench at a fresh project got a shelf of parts that all drew in
/// magenta, a paint palette holding two rows, and nothing on the screen to say
/// why. Brett, on exactly that: "This used to have way more colors before I
/// separated it. What happened?" — the colours had gone home with the game they
/// belonged to, correctly, and nothing had taken their place.
///
/// A standalone bench carries its own. Every one of these loses, name for name,
/// to the same name in a project's `palette.json` the moment one exists, so a
/// game is never stuck with them — but a bench with no game is never stuck
/// either.
///
/// The studio's own five-step convention: shadow, low, mid, high, bright.
///
/// One line per ramp, and the formatter is told to leave it that way. A ramp is a
/// ROW of a table — its whole meaning is how its five steps run, and how they run
/// beside the ramp above it — and rustfmt would give each number a line of its
/// own, turning twenty-four readable rows into two hundred lines nobody can scan.
#[rustfmt::skip]
pub(crate) const BENCH_RAMPS: [(&str, [[u8; 3]; 5]); 24] = [
    // The ground and what grows on it.
    ("stone", [[26, 28, 36], [43, 47, 59], [66, 70, 84], [93, 98, 112], [126, 131, 145]]),
    ("earth", [[31, 23, 18], [51, 37, 26], [74, 56, 38], [101, 78, 53], [136, 107, 74]]),
    ("grass", [[23, 36, 28], [37, 58, 42], [54, 84, 58], [77, 115, 70], [112, 154, 82]]),
    ("foliage", [[17, 29, 25], [27, 48, 36], [40, 70, 48], [58, 95, 56], [86, 128, 68]]),
    ("sand", [[61, 47, 32], [92, 70, 48], [129, 99, 63], [168, 133, 81], [203, 171, 116]]),
    ("water", [[13, 26, 38], [19, 41, 58], [28, 64, 84], [42, 94, 115], [74, 142, 156]]),
    ("sky", [[27, 36, 64], [43, 61, 99], [67, 98, 147], [106, 145, 192], [166, 197, 224]]),
    // What a building is made of. `wood` and `stone` carry most of the bench:
    // between them they are a hundred and fifty of the slabs in `body_of`.
    ("wood", [[28, 19, 16], [46, 33, 26], [69, 47, 34], [93, 66, 44], [125, 91, 60]]),
    ("bone", [[43, 38, 32], [69, 62, 51], [102, 92, 75], [138, 126, 105], [179, 166, 140]]),
    ("snow", [[109, 117, 134], [139, 147, 163], [168, 177, 190], [198, 205, 215], [232, 236, 242]]),
    ("scrub", [[42, 38, 22], [68, 61, 34], [100, 89, 48], [135, 119, 66], [171, 153, 92]]),
    // The people who live in it.
    ("skin-pale", [[58, 32, 24], [90, 51, 37], [125, 76, 54], [160, 106, 75], [196, 143, 107]]),
    ("skin-mid", [[44, 24, 16], [71, 40, 26], [102, 64, 38], [138, 92, 58], [173, 128, 88]]),
    ("skin-deep", [[28, 15, 10], [48, 26, 17], [74, 43, 27], [102, 64, 39], [138, 92, 60]]),
    // And what they dye things with. `cloth-gold` is the bench's own accent -
    // the rail, the sill, the handles - so it is never absent whatever else is.
    ("cloth-gold", [[50, 36, 12], [77, 56, 19], [111, 82, 32], [149, 116, 64], [193, 156, 98]]),
    ("cloth-red", [[42, 16, 21], [69, 24, 34], [107, 37, 48], [147, 53, 58], [189, 90, 82]]),
    ("cloth-blue", [[18, 26, 44], [29, 43, 70], [44, 66, 102], [64, 96, 140], [99, 137, 181]]),
    ("cloth-green", [[19, 33, 15], [31, 52, 25], [47, 77, 36], [69, 106, 51], [102, 143, 73]]),
    ("cloth-purple", [[31, 18, 40], [51, 29, 66], [77, 44, 96], [107, 65, 130], [143, 99, 164]]),
    ("cloth-wine", [[38, 14, 28], [62, 21, 45], [92, 32, 66], [125, 48, 88], [161, 77, 116]]),
    ("cloth-teal", [[13, 32, 32], [20, 51, 51], [31, 76, 74], [46, 106, 101], [73, 143, 134]]),
    ("cloth-rust", [[46, 20, 10], [76, 32, 14], [112, 49, 20], [150, 71, 30], [189, 102, 51]]),
    ("cloth-sable", [[12, 12, 16], [21, 21, 26], [32, 33, 39], [45, 47, 54], [62, 65, 73]]),
    ("cloth-pink", [[56, 20, 36], [94, 34, 61], [140, 55, 91], [184, 81, 121], [221, 120, 155]]),
];

/// The bench's own ramps, as a palette.
pub(crate) fn bench_palette() -> Palette {
    Palette {
        ramps: BENCH_RAMPS
            .iter()
            .map(|(name, steps)| ((*name).to_string(), *steps))
            .collect(),
    }
}
