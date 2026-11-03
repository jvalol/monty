# 0002 A picture of a round carried through

**Status:** implemented
**Date:** 2026-11-03

## Goal

The frame monty puts on its readme is held against a picture of what that
frame should be.

## Why it belongs here

Spec 0001 is about whose rule the host plays by, and every test of it is
arithmetic: which door he opens, how often staying wins, what a mean host
does when you pick the prize first. None of them can see the stage.

The engine's spec 0076 made the rest one call.

**The staging was already here, and so was the determinism.** `Stage` is
seeded with a fixed number rather than from entropy, so unlike snake,
tessera and starry there was nothing here to pin: this game already came out
the same twice. `--screenshot` runs a batch to put bars on the board and
plays one round all the way through, because the prize behind an open door
is the thing a still picture can show and an offer is not.

## Behavior

**`Monty::new` takes `staged` rather than reading the arguments.** A picture
test asks for the photographed state without `--screenshot` in its own
command line. `main` passes what the arguments say.

**One `update`, at no time at all.** The pose runs inside the first one and
sets `posed`, and a `dt` of nought leaves the doors where the pose swung
them rather than part way through a swing.

**The picture is the 3D.** The engine's harness leaves the overlay empty, so
the readout and the bars are not in shot. Those are what spec 0001's tests
already count.

**It is the engine's size**, 320 by 240, said once in `picture::SIZE`.

## Acceptance criteria

- A round carried through matches its
  picture. — `tests::a_round_carried_through_looks_right`

### Verified by hand

- **The picture itself.** Three doors in the wall, the picked one and one
  of the others swung open, and the prize lit behind the one that holds it.
- **A control.** The pose picking door 0 rather than door 1 failed the test
  on 7.2% of its pixels. Put back and compared.

## Out of scope

**The readout and the bars.** Both are 2D and the engine's harness draws
neither.

**A picture per phase.** Picking, offered and settled are three states spec
0001 already tests, and three references that change together.
