# The file contract

Opificium and the games that use it share no code. Everything that passes
between them is a file described here, and this page is the single word
on what those files mean. A game exports its truth for Opificium
(`data/`); the maker exports work for the game (`out/`), and it is
carried into the world from there.

## A project

Opificium holds no game's content. It works on a PROJECT: one game's own
folder, living in that game's repository, described by an
`opificium.json` at its root. Everything below is relative to that
folder.

```json
{
  "format": 1,
  "name": "Divus Factus"
}
```

Every field has a default, `install` included, so that is a COMPLETE project - and
a folder with no manifest at all is one too. Set a path only where a game differs
from the table below.

| field       | default             | what it is                                      |
| ----------- | ------------------- | ----------------------------------------------- |
| `name`      | the folder's name   | what the bench calls this project                |
| `palette`   | `data/palette.json` | the game's colour ramps                          |
| `templates` | `templates`         | starting shapes to draw from                     |
| `work`      | `out/buildings`     | the maker's own saved work                       |
| `baked`     | `out/baked`         | exported work, ready for the game                |
| `install`   | `../assets/buildings` | where baked work is carried so the game reads it. Empty means nowhere: the bake stops at `baked` |

A folder with no manifest at all is still a project, and takes every
default — point the bench at an empty directory and start working.

Opificium opens the project named on its command line, or the last one
worked in, and asks for a folder only when it has never been opened
before. Its own settings live apart from any game, under
`Opificium/` in the machine's usual place for such things.

## Game → Opificium

### `data/palette.json`

Written by the game. In Divus Factus that is
`cargo test export_palette_for_opificium -- --ignored`; every game does
it its own way. Re-run whenever the game's palette changes.

```json
{ "ramps": [ { "name": "wood", "steps": [[26,28,36], ...5 RGB steps...] } ] }
```

Colour in every other file is spoken as `{ "ramp": "wood", "shade": 0.7 }` —
a name and a 0..1 step, never raw RGB — so authored work inherits palette
changes for free.

## Opificium → game

### Blueprints: `out/buildings/<name>.json`

**The lattice**: the bench speaks in UNITS, and one unit - 1/16 m - is
the smallest measure that exists. Every coordinate and dimension is a
whole number of them. The files below still carry metres (divide by 16)
so the game's world and old exports stay true. Coarse work steps 4/16, joints land on
2/16, fine work on 1/16. Binary fractions carry exactly in floats, so
two parts that should meet, meet. STRUCTURAL DIMENSIONS obey it too:
walls 0.25 x 2.5, floors and roofs 0.125 thick, foundations 0.375,
trim 0.3125 high, door openings 1.25 x 2.125, windows 1.25 x 1.25
with the sill at 0.75. Furniture stays organic - nothing joins
against a stool.

Local space: +Y up, metres. Position and orientation on the bench are
FREE: the import rebases the building on its own bounds, and the door
widget defines the front - the whole blueprint is turned so the (first)
door faces the village. The gold sill (+X) is a working aid, not a law.
The bench saves to `out/buildings/workbench.baz` and reloads it on launch;
rename a finished piece in Finder to keep it. A saved building is a `.baz` —
JSON inside, as it always was, but named for the studio whose bench made it
rather than for the notation it happens to be written in. Files saved as `.json`
before the rename still open, and always will.

The file is a list of PARTS, not raw boxes — every entry is something the
game already understands, so the carrying-in is mechanical:

```json
{
  "format": 1,
  "name": "workbench",
  "parts": [
    { "part": "wall-2",       "at": [3.0, 0.0, 1.0], "yaw": 0.0, "tilt": 0.0,
      "ramp": null, "shade": 0.7, "stage": "walls" },
    { "part": "prop:bed",     "at": [-2.1, 0.0, 1.4], "yaw": 1.5708, "tilt": 0.0,
      "ramp": null, "shade": 0.7, "stage": "furnishing" },
    { "part": "widget:sleep", "at": [-2.1, 0.0, 1.4], "yaw": 1.5708, "tilt": 0.0,
      "ramp": null, "shade": 0.7, "stage": "widget" }
  ]
}
```

- `part`: `wall-<len>` (0.25 thick, 2.4 high — Opificium's truth; the game conforms),
  `ridge-<len>` (the cap over a ridge line, half a metre across at the
  bench's pitch), `gable-<len>` (a wedge - a real triangular prism, peak at the
  bench's 45 degree pitch, so it stands half as tall as it is wide),
  `floor`, `roof`, `prop:<name>` (bed, table, stool, hearth, chair, bench,
  chest, barrel, crate, shelves, cupboard, pot, basket, rug, woodpile,
  candle, sack, trough — the shelf grows on request), or `widget:<kind>`.
  `prop:mannequin` is the scale reference and is SKIPPED on import.
  `prop:doorway` is an opening with no leaf, for interior walls, and it
  carries NO widget: a gap in a wall run is a portal by itself, found by
  walking the segments. A door widget means a real door - an entrance -
  and the first one defines the building's front.
  `prop:bed` and `prop:bed-double` carry their own sleep semantics when
  imported: a double becomes the household's marriage bed - the wedded
  pair claims it together, each to their own side, and children never
  do, whatever the shortage.
- EXPORT A COPY writes `out/buildings/build-<n>.json`, never overwriting;
  the SAVED WORK drawer lists everything in that folder on launch.
- `yaw` turns about the part's centre; `tilt` pitches roof panels.
- `flip`: mirrored - the body reflected across its own length and the
  tilt leaning the other way (the far half of a gable).
- `ramp`/`shade`: a repaint, or `null` for the part's authored colours.
- `stage`: `footing | frame | walls | roof | furnishing` — the order the
  village raises it; `widget` entries never become boxes at all.

### Models: `out/models/<name>.glb`

A glTF binary, made at the kiln from an image and kept by the maker under a name.
Loaded WHOLE - it is not parts, not boxes, and carries its own materials and
textures, so there is nothing in it for a game to translate.

- **Metres, standing on its own origin.** The bench bakes both into the file when
  it keeps one: the maker states a height and the mesh is scaled to it, and the
  model is lifted so its lowest point sits at y=0. So `translation` is a place to
  put it on the ground and needs no offset of the game's own.
- **+X is the front**, the same as a building's, when the maker points it that way.
  Nothing in the file enforces it.
- Textures are usually JPEG. Engines that leave JPEG out of their image formats
  fail the whole file rather than the texture - see this bench's own `Cargo.toml`.
- The bench never reopens a `.glb` to change it. A model is the finished thing; the
  image it was made from is the source, and it lives wherever the maker keeps it.

There was a clip format promised here - `out/anim/<name>.json`, rotations on a
timeline - for a body-and-clips bench that has since been retired. It was never
written by anything, so nothing read it. It is not deprecated; it never shipped.

### The baked building: `assets/buildings/<name>.json`

What the bench hands the game, written by `cargo test bake_the_works --
--ignored`. Not parts any more but the plain boxes they resolve to, each with
its colour already looked up, plus the marks that say what the place is FOR.

```json
{
  "format": 2, "name": "longhouse1-10people",
  "half_w": 3.65, "half_d": 6.7, "high": 5.2,
  "boxes": [ { "at": [0,1.25,0], "size": [4,2.5,0.25], "turn": [0,0,0,1],
               "form": "box", "rgb": [110,92,70], "alpha": 1.0, "cloth": "wood",
               "stage": "walls" } ],
  "marks": [ { "mark": "door", "at": [3.65,0.375,0.0], "yaw": 0.0 } ],
  "levels": [
    { "name": "", "half_w": 3.65, "half_d": 6.7, "high": 5.2,
      "phases": [ { "boxes": [ "...the footings only..." ] },
                  { "boxes": [ "...and the frame..." ] } ],
      "marks": [ { "mark": "door", "at": [3.65,0.375,0.0], "yaw": 0.0 } ] }
  ]
}
```

### LEVELS, and the two axes of a build

A building is not one thing for ever, and there are two different sequences in it.
The format used to have a word for neither, and the bench used one word - "stage" -
for both.

- A **PHASE** is a step of raising ONE building: footings, then frame, then walls.
- A **LEVEL** is a form the building takes over its life: the original, then each
  upgrade. Every level is itself a build, so every level has its own phases.

`levels` carries both. Each entry is one level, in order - the first is the
original - and holds:

- `phases`: one COMPLETE set of boxes per step of raising that level. Complete and
  not additive, which is what lets a maker draw a step that is not simply the last
  one plus more: a frame is a picture of a frame, and by the time the walls are up
  it should be gone.
- `half_w`/`half_d`/`high`: the FINISHED footprint at that level - the plot to
  clear, and the shell when it is done. A level may reach further than the one
  before it.
- `marks`: what the place is FOR once that level is finished.

Every level is measured from ONE origin: the first level's finished footprint. An
upgrade has to land on the building it upgrades, so it is never recentred on its
own bounds - a wing added to one end would otherwise shunt the whole building
sideways the day it was built.

`boxes` and `marks` at the top level are the **first level, finished**: exactly
what a format 1 file held. A game that wants nothing to do with levels reads only
those and needs no change. A game that wants upgrades reads `levels` instead.

The per-box `stage` says what a box IS, and still does. It is enough to raise a
level without reading its phases at all, which is how the older readers work; the
phases are there for when the sequence a maker actually drew matters more than the
one a tag can imply.

- `form` is the box's shape, and there are four. Both programs draw each one
  from its own code — they share none — so a shape is only the same shape in
  both because it is written out twice and named here:
  - `box`: the plain cuboid, which is most of everything.
  - `wedge`: a GABLE's prism. A triangle with its peak in the middle, standing
    across the part's length. Symmetric.
  - `ridge`: the same prism turned to run lengthwise, apex up — a ridge cap.
  - `cut:<low>x<high>`: a box with a face cut back at each end. The two numbers
    are RUNS as fractions of the piece's own length — how far along it the saw
    travels while crossing its full height — at the -X end and the +X end.
    Nought is a square end. A POSITIVE run cuts the TOP face back; a NEGATIVE
    one cuts the BOTTOM.

    The signs are what let a brace exist. `cut:0.2500x-0.2500` cuts the top at
    one end and the bottom at the other, so the two ends come out PARALLEL — a
    parallelogram, which is what a diagonal brace is, both of its ends meeting
    horizontal timber. Cut the top at both ends and the ends converge instead,
    and a brace sits in its bay like a wedge.

    A run rather than an angle, because a run is the number everything already
    has: a roof hands over the difference between where its slope crosses the
    top of a beam and where it crosses the bottom, and nobody needs
    trigonometry to say what they mean.
  - `mitre` and `mitre-back`: WHAT `cut` REPLACED, still read so that older
    drawings open. They were two whole shapes rather than one property, each
    able to say only that ALL of one end was gone — so no beam could be cut at
    both ends, and every part that wanted an angled end grew a shape of its own
    instead. They are exactly `cut:0.0000x1.0000` and `cut:1.0000x0.0000`, and
    the bench no longer writes either.
  - `hip:<x>x<z>`: a truncated pyramid - four faces sloping in from the foot to
    a flat top, which is a hip roof with a deck. The two numbers are how much of
    the box's width and depth the top keeps, so `hip:0.5000x0.6250` slopes in a
    quarter of the width on each side and three sixteenths of the depth. They
    ride in the form because the mesh is different at every deck size and a
    name alone could not say which.
- `kind` is what the village raises it AS: one of `house`, `longhouse`, `sawmill`,
  `blacksmith`, `tavern`, `townhall`, `storehouse`, `granary`, `well`,
  `smokehouse`, `mill`, `bakery`, `weaver`, `herbalist`, `watchtower`, `shrine`,
  `dock`, `mine`. The bench asks for it when a work is carried in, so it is a
  fact rather than a guess. A file without one falls back to the older reading -
  the longest of those words that begins its NAME - which is what every drawing
  baked before the card existed relies on.
- `half_w`/`half_d` are the FINISHED building's footprint: the plot the village
  clears, the obstacle while it is being raised, and the walkable shell when it
  is done.
- `cloth` names the ramp a box was painted from. The bench writes it and the
  game no longer reads it: villages used to re-dye a drawing's dominant wall and
  roof cloth so a street of one blueprint was a street of different houses,
  which was the right answer while a drawing wore whatever the catalogue handed
  it. There is a brush on the bench now, so the colour in `rgb` is a choice, and
  the game paints exactly what it is given. Variety comes from the mirror
  instead: half of every kind is raised as its own reflection along z, which the
  game does to the numbers rather than by scaling, so nothing turns inside out.
  Along z no shape changes hands, and the front door - which is what a
  building's +X means to the village - stays in the front wall.

## Versioning

Every file carries a `format`. When a format grows, the number moves and this page
says what changed; the game keeps reading old numbers.

**Baked buildings are at `2`**, which added `levels`. A `2` is a superset of a `1`:
`boxes` and `marks` still hold the first level finished, so a reader that only knows
`1` needs no change and sees what it always saw.

**Saved works are at `2`**, which added `levels` to the `.baz`. All three shapes a
work has ever had still open: `levels`, the older `stages` without them, and the
flat `parts` list from before either existed. A maker's buildings are not something
to lose to a format change.
