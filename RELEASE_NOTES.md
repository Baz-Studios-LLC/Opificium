## A terrain bench: ground, brought to the bench and shaped

A fourth bench, for shaping the ground a game's world is made of. **THE TERRAIN**,
in the rail under the rig.

**A world is not a project.** The other benches work on one game's authored content
and are pointed at it when the app opens. This one is a tool you bring ground to,
the way the kiln takes an image — so it opens a world from its own shelf, with
**OPEN A WORLD…**, and any folder holding a `heightmap.png` will do, whoever's it is.
The last one is remembered. A project may *name* its world in `opificium.json` under
`"world"`, which saves a walk across the disk and nothing more.

**Eight tools**, on the number row:

| | | |
| --- | --- | --- |
| `1` Raise | `2` Lower | push the ground up, pull it down |
| `3` Smooth | `4` Flatten | average out what is there, level to where you pressed |
| `5` Path | `6` Roughen | a flat-bottomed cut for roads, fractal detail |
| `7` Erode | `8` Ramp | let steep ground slump, a graded run between two points |

**Erode** is thermal erosion: ground steeper than material can hold — about
thirty-four degrees, where sand and scree settle — sheds downhill and piles at the
foot. Nothing is created or destroyed, so a hill does not shrink, it *settles*.
**Ramp** is clicked rather than dragged: a start, a far end, and a steady climb
between them that can be walked and carted. A preview shows the grade against the
ground it would cut through before any of it is cut.

Undo and redo group **per stroke**, so a drag lasting two hundred frames comes back
in one press. Ground re-meshes live under the brush, with the old mesh left standing
until the new one lands, so nothing blinks out from underfoot. `Ctrl+S` keeps it.

**Shift is the camera at this bench.** Both mouse buttons are tools — the left lays
the brush down and the right takes it back off — so the eye turns on `Shift`+drag,
and the drafting angles move to `Shift`+`1`–`6`. Middle-drag pans and the wheel
zooms, exactly as everywhere else.

**The whole world stands at once.** A bench is for judging the shape of one coastline
against the one across the water, and a disc of ground following the eye about
answers none of that.

It paints in the open game's own ramps — water, sand, grass, foliage, stone, snow —
by height and slope, and lays a moving sea over it with a tide, so a coastline can be
read against the waterline it will actually have.

Three files pass between a game and this bench, and `FORMATS.md` describes each:
`heightmap.png` and `world.json` in, `edits.bin` out. Edits are stored as **signed
height offsets** on top of generated ground, so re-rolling a game's noise or
redrawing its map never moves hand-placed geography.

Nothing about the builder, the kiln or the rig changes.
