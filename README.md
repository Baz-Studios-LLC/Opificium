# Opificium

A maker's bench for buildings and models. Draw a building by hand out of
boxes, ramps and roofs; commission a model from a picture and keep it at
the size you want; export both as plain files for a game to read.

Opificium is a Baz Studios tool, built with Rust and Bevy. It grew up
inside [Divus Factus](https://github.com/Baz-Studios-LLC/Divus-Factus)
and now stands on its own, so any game the studio makes can use it.

## It holds no game's content

The bench is the program. The buildings, the palette, the bodies and the
templates all belong to whichever game asked for them, and they live in
that game's own repository — a **project**, described by an
`opificium.json` at its root:

```json
{
  "format": 1,
  "name": "Divus Factus"
}
```

Every path has a sensible default — including where baked work is carried,
which is the game's own `assets/buildings` — so the manifest above is
complete, and a folder with no manifest at all is still a project. Point
Opificium at an empty directory and start working.

```bash
opificium /path/to/your-game/opificium
```

With no argument it ASKS which game to open, listing the ones you have
worked in before. A path names one outright and skips the question, which
is what scripts, the launcher and the bench's own project switcher all do.

[`FORMATS.md`](FORMATS.md) is the single word on every file that passes
between a game and the bench — the palette going in, the blueprints and
clips coming out. The two programs share no code, only files.

## Running it

```bash
cargo run --release
```

`OPIFICIUM_BENCH=rig` opens on the animation bench instead of the
builder.

## Licence

© Baz Studios LLC. The bundled fonts carry their own licences, kept
beside them in `assets/fonts`.
