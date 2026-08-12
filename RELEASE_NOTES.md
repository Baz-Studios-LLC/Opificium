## A baked building says what it is again

**If you baked anything with 0.4.1, bake it again.** Those files do not record
what KIND of building they are, so the game falls back to guessing from the
drawing's name — right for a drawing called `longhouse-corner`, and silently
wrong for one called `guard-tower`, which the village will raise as nothing at
all. The bake said "carried in as a watchtower" while writing a file that never
said so. Fixed, and there is now a test that reads the file back, which is the
thing that was missing.

**The bench has a face.** The `.app` carries its own icon, so macOS shows it in
the Dock and in Finder instead of the generic one.

Everything 0.4.1 brought is unchanged: opening a game from the rail, the bench's
own twenty-four colour ramps, the menu bar, building kinds and marks as the
game's own vocabulary, levels and phases, and baked work landing in the game's
`assets/buildings` without being told.

Under the floor, the builder was one twelve-thousand-line file and is now
twenty-one, which changes nothing you can see and a great deal about what can be
built next.

See `FORMATS.md` for every file that passes between a game and the bench.
