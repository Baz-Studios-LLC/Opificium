## The bench serves any game

Opificium stood on its own last time. Now it stands in front of whichever game
you point it at, and it arrives furnished rather than empty.

**Open a game from inside the bench.** THE PROJECT sits at the top of the rail:
it says which game you are working for, and it opens as a drawer holding every
game you have worked in, with OPEN A GAME at the foot of it. Pick a game's own
folder — the game, not a folder inside it — and the bench makes its `opificium`
folder there, writes a manifest, and reopens pointed at it. Whatever was standing
on the bench is kept before it goes.

**It brings its own colours.** A game that has not exported a palette yet is the
ordinary case rather than a broken one: the bench now carries twenty-four ramps of
its own, so a new project draws in wood and stone from the first minute. A game's
own `palette.json` still wins, name for name, the moment it exists. Before this,
a fresh project painted almost everything magenta.

**A menu bar.** FILE, EDIT, VIEW, BENCH and HELP along the top, each item showing
the key that does the same thing, so the shortcuts are learnable from the menu you
were already reaching for. Commands that only mean something at the building bench
grey out at the rig.

**The game's own words.** What a drawing may be baked AS, and the marks you can
place, are the game's vocabulary now and live in the project — `data/kinds.json`
and `data/widgets.json`. A game that raises no sawmills is never offered one, and
the bake card can add a kind you type, which the project then remembers. A game
can generate both files from its own code so they cannot drift.

**Buildings have levels.** A work holds LEVELS — the original, then each upgrade —
and every level is a build with its own PHASES. The phases you draw now reach the
game: before this the bake kept only the finished building and threw the rest away.
Baked buildings are format 2, and a 2 is a superset of a 1, so a game reads the new
`levels` when it is ready and needs no change until then.

**Every game gets a note.** A new project is given a README explaining what each
file is, which side writes it, what shape a baked building has, and what not to
commit — so anyone, or anything, opening that folder later can work it out without
this page.

**Baked work knows where to go.** It lands in the game's own `assets/buildings`
without being told. A game that keeps its assets elsewhere says so in one line.

`OPIFICIUM_BENCH=rig` still opens on the animation bench, and `opificium <project>
--bake` still bakes every drawing without opening a window.

See `FORMATS.md` for every file that passes between a game and the bench.
