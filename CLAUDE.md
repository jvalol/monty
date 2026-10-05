# monty

Three doors and a prize behind one. You pick, the host opens a door that is not
yours, and you decide whether to switch. The odds depend entirely on how the
host chose which door to open, so the host is the thing you set.

The thirteenth game on blitzkit, and the first that is not about space or
physics at all. Starry is a puzzle you solve; this is an argument you settle.

## Building and running

```
cargo run --release
```

Inside `blitzkit-project` this builds against the engine checkout rather than
the published crate, because of the `[patch.crates-io]` in
`blitzkit-project/.cargo/config.toml`. Cloned on its own it builds against
whatever is on crates.io, which is what a stranger gets.

```
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check
```

Or `./check-all` from `blitzkit-project`, which runs that for every crate and
then checks the copy markers, the spec citations and the listings.

## The spec flow

A spec before the code, in `specs/`, numbered in the order written. Each
acceptance criterion names the test that proves it, and `check-specs` fails the
gate when an implemented spec cites a test that does not exist. A draft is not
checked, which is what makes it safe to write one before building it.

## Decisions worth defending

**The host is the game.** Stay or switch is one decision with a known answer,
and a player who has heard of this problem is done in forty seconds. What people
actually get wrong is why the odds are what they are, and that lives entirely in
how the host picks what to open. So the host is what the player sets, and the
doors are what the setting is visible in.

**Both of the careless host's bars land at a half.** Switching drops from two
thirds to a half, which is the result people argue about, and staying climbs
from a third to a half, which they do not. The pair sitting on top of each other
is a stronger picture than either number, and it is the reason that host is in
the game rather than in a footnote.

**Thrown away rounds are counted and drawn.** When the careless host opens the
prize the round does not count, and it would be easy to drop it silently. Those
rounds are exactly the information the knowing host never has to hand over, and
with the count on screen the difference between the two hosts has somewhere to
come from. On many doors he throws away nearly everything, which is worth seeing
rather than waiting through.

**A batch is the same round in a loop.** The thousand rounds a keypress plays
run through `round::play`, not through a formula that happens to agree with it.
`round::tests::a_batch_says_what_the_hand_says` pins that. A closed form would
be faster and would stop being evidence.

**The tally resets when the host changes.** A tally mixing two rules is the
mistake this game exists to show, so it cannot be made here by accident.

**No goats.** A goat is a modelling project. An empty door is empty, and the
prize is a light.

## User-facing text

Every string this game draws carries `[COPY - Jake]` until Jake rewrites it into
his own voice and strikes the marker himself. `check-all` reads `src/` as well
as the readmes and holds the gate while one remains.
