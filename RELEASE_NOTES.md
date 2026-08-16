## A ceiling raises the roof, and a part says what it is made of

**⚠ SAVED BUILDINGS LOSE THEIR WALLS.** A wall is spelled differently now, and the
bench skips a part it cannot read — so a `.baz` from before this opens with
everything but its walls. Nothing crashes and nothing else is lost, but a building
you care about is worth rebuilding rather than reopening.

### Roofs are raised from a ceiling

Placing a roof was the worst job on the bench. Now you place a **CEILING** — a
rectangle you drag out like a floor — and right-click it for **GENERATE ROOF**.

The ceiling stays where it is, because that is what a room needs. It **wears the
ridge it will raise**, so you can see which way the roof runs and whether it is
gabled or hipped before you commit to either: a gable's ridge runs the whole length,
a hip's stops short at both ends. **R swings the ridge** rather than turning the
ceiling — a rectangle spun a quarter is the same rectangle, and the flip is what a
cross wing wants, its gable facing the street while the main range runs behind.

**A HIPPED ROOF / A GABLE ROOF** chooses which. The roof lands sized to the ceiling
with its ridge the long way, seats on the wall the ceiling sits on, keeps its own
eaves and pitch handles, and ungroups into slopes and gables like any other.

### One wall, framed or not

A half-timbered wall used to be a different *kind of thing* from a plain one. Now
there is one **WALL**, and **ADD FRAMING** on its right-click menu — its length, its
height and its doors all stay where they are. Every wall has a **height you can
pull**, where only framed ones did.

**Windows glaze themselves.** A pane is a size, the way a bay is: one mullion makes
two lights, two make three. A 2.5 m cottage wall comes out two panes by two; a
three-metre hall wall comes out two by three. **BARS IN BLACK** for dark joinery
against pale plaster.

### What a part is MADE OF

New on every part's menu, beside PART OF. Wood, stone, clay — or whatever your game
knows, from `data/materials.json`, with **+ ANOTHER** to type one in.

**It is not a colour.** `rgb` and `cloth` on a baked box are what you painted;
`material` is what the thing is built of, and only one of those should cost your game
anything to gather. A part nobody has spoken for writes no material at all — absent
is not `wood`, and what that costs is your decision. See `FORMATS.md`.

### A slimmer shelf

Twenty-seven lines down to eighteen. **STRETCH** is gone from every label — a thing
that stretches is the ordinary case here. **STAIRS**, **RAIL** and **TRIM** each
choose their material after they stand, a flight from a drawer of four looks since
its treads and its rail are separate. The whole-roof entries are gone, replaced by
the ceiling; the pieces they break into stay, for lean-tos, dormers and junctions.

### The modifiers mean one thing each

| | |
| --- | --- |
| **shift** | more of the same — gather, paint the whole, keep placing |
| **alt** | the other way — the dropper, and the fine 1/16 m snap |
| **cmd** | the commands — undo, redo, copy, paste |

**One part per pick**: setting a foundation down used to leave another stuck to the
cursor. Hold **shift** while placing to keep the tool in hand.

The part menu has **drawers** that pop out to the right, so PART OF is one line
rather than five.

### Smaller things

The note Opificium writes into your game's folder is **brought up to date every time
the project is opened**, so a folder made six versions ago stops describing a bench
that no longer exists.

Nothing about the terrain bench, the kiln or the rig changes.
