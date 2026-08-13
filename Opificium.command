#!/bin/zsh
#
# Double-click in Finder to open the bench.
#
# Two things about .command files bite, and both are handled below: Finder hands
# the script the HOME directory rather than the one it lives in, and the login
# shell it opens has not always picked up the Rust toolchain.
#
#   ./Opificium.command                       the last project worked in
#   ./Opificium.command /path/to/game/opificium   that project
#   ./Opificium.command --release             the tuned build, once it exists
#
# Anything after --release is still passed on, so the two combine.

cd "${0:A:h}" || exit 1

# rustup installs here. Harmless if cargo is already on PATH, and Finder's
# Terminal session often has not sourced it.
export PATH="$HOME/.cargo/bin:$PATH"
if ! command -v cargo > /dev/null 2>&1; then
  [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
fi
if ! command -v cargo > /dev/null 2>&1; then
  echo "Cargo is not on PATH and ~/.cargo/env does not exist."
  echo "Install Rust from https://rustup.rs and try again."
  echo
  echo "Press return to close."
  read -r
  exit 1
fi

profile=()
if [[ "$1" == "--release" ]]; then
  profile=(--release)
  shift
fi

# Which project the bench will open.
#
# Only what was NAMED, the way `project::named_outright` reads it: a path, then
# OPIFICIUM_PROJECT. With neither, the bench asks which game to open, and a script
# that guessed the answer to a question would be worse than one that says a
# question is coming.
#
# Worked out again here ONLY to print it — the bench itself decides, so if these
# two ever disagree the wrong thing is a line of text rather than a folder. Worth
# printing because the bench holds no game's content, so "which project" is the
# difference between one game's palette and another's.
opening=""
if [[ -n "$1" ]]; then
  opening="$1"
elif [[ -n "$OPIFICIUM_PROJECT" ]]; then
  opening="$OPIFICIUM_PROJECT"
fi

print -r -- ""
print -r -- "  THE OPIFICIUM — the maker's own bench"
print -r -- "  ─────────────────────────────────────────────────────────────"
if [[ -n "$opening" ]]; then
  print -r -- "  project    $opening"
  if [[ ! -f "$opening/data/palette.json" ]]; then
    print -r -- "             no data/palette.json — painting in the bench's"
    print -r -- "             own 24 ramps until the game exports its own"
  fi
else
  print -r -- "  project    the bench will ask which game to open"
fi
print -r -- ""
print -r -- "  another    ./Opificium.command /path/to/game/opificium"
print -r -- "  the rig    OPIFICIUM_BENCH=rig ./Opificium.command"
print -r -- "  the keys   the gear at the foot of the rail
  the log    ~/Library/Application Support/Opificium/opificium.log"
print -r -- "  ─────────────────────────────────────────────────────────────"
print -r -- ""

# The dev profile on purpose. Every dependency is already built at opt-level 3
# and the bench's own code at 1 — see Cargo.toml — so this is a bench that opens
# NOW, while --release would spend ten minutes rebuilding Bevy to speed up a
# program that draws a few hundred boxes and some panels. --release is there for
# the day that matters; it is a poor thing to make somebody wait for on a
# double-click.
cargo run "${profile[@]}" -- "$@"
# NOT `status`. In zsh that name is a read-only alias for `$?`, so assigning to
# it fails the script on its very last line - after a clean run, which is the
# one time nobody is looking for an error.
code=$?

# A build error scrolls away before it can be read, so hold the window — but
# close cleanly when the bench was simply shut.
if [ $code -ne 0 ]; then
  echo
  echo "Exited with status $code."
  echo "Press return to close."
  read -r
fi
