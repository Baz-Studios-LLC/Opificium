## A kiln for models, and the rig looks at them

**A new bench: THE KILN.** Give it a picture and it commissions a 3D model, stands
the model on the bench, and keeps it as a `.glb` in the game. About seven minutes
and eighty credits for a game-ready one; a quick look is ten.

It needs a key of your own from 3D AI Studio. Put it in

    ~/Library/Application Support/Opificium/3daistudio.key

or in `OPIFICIUM_3DAI_KEY` — **never in a project folder**, which is a game's own
repository and usually a public one.

**What the panel tells you.** What the firing will cost and roughly how long it
will take, learned from your own past firings of those same settings rather than
guessed. What your account has left, and a warning in red BEFORE you press the
button if the firing would not fit in it. The picture you chose, so a wrong file is
caught while it is still free to change. And a bar that fills on the machine's own
number when there is one and creeps dimly when there is not — the game-ready
machine says nothing at all for the whole firing, so a dim bar is honest and a full
one would be a lie.

**The size is yours to state, and it goes into the file.** Every generated model
arrives normalised into a unit box — a housefly and a two-seater sofa come back the
same size — so nothing in the file, and nothing a game could work out from it, says
how big the thing is. You say how tall it is, the panel shows what that makes the
other two dimensions, and KEEP IT AS… writes a model standing on its own origin at
that size. A game loads it and forgets; there is no second number to keep beside it.

**THE RIG is a model viewer now.** It was a body-and-clips bench built for one
game's animation, and that has gone: no game ever read the clips it saved. Pick any
model the project holds, stand it on the bench, walk around it. Rigging generated
models is where it is heading, which is why it keeps the name.

**A ruler.** TOOLS → THE RULER stands a measuring post on the bench, banded every
decimetre with a wider gold collar at each metre. Off until you reach for it, and it
answers at every bench — a building wants measuring as much as a model does.

**Put the furniture away.** A TOOLS menu switches the top bar, the step bar, the
shelf and the rail on and off. None of them are needed everywhere, and every row of
buttons covers a strip of the work.

**Bigger words.** Every size of text on the bench went up together, which lifts the
small labels by a fifth and leaves the headings much as they were. The side panels
widened to match rather than crowding.

**The bench keeps a log**, at `~/Library/Application Support/Opificium/opificium.log`.
The bench you actually use is a child process whose launching window has usually
closed, so everything it said used to go nowhere — and a firing was lost to exactly
that, silently, after the model was made and paid for.

**Smaller things.** The benches are ordered the way work flows: builder, kiln, rig.
A model on the rig's shelf shows all three of its dimensions, not just its height.
The kiln's profile — which machine, quads, low poly, fine texture — folds into one
button that says what is set, since it is set once and then left alone.

See `FORMATS.md` for every file that passes between a game and the bench.
