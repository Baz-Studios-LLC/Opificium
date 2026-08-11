## Opificium stands on its own

The maker's bench has left Divus Factus and become its own program, its
own repository and its own download. It was always a separate crate that
shared no code with the game; now it shares no content either.

**It works on projects.** A project is one game's own folder — palette,
bodies, templates and authored work — living in that game's repository,
described by an `opificium.json` at its root. Every path in that file has
a sensible default, and a folder with no manifest at all is still a
project, so a new game needs nothing but an empty directory.

Opificium opens the project named on its command line, or the last one
you worked in, and asks for a folder only the first time it is ever run.
Its own settings live apart from any game now, under `Opificium/`, rather
than inside Divus Factus's application-support folder.

**Nothing in the bench knows which game it is working for.** That is the
whole of what makes it serve more than one.

See `FORMATS.md` for every file that passes between a game and the bench.
