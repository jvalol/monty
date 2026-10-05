# 0001 Whose rule it is

**Status:** draft
**Date:** 2026-10-04

## Goal

Three doors, a prize behind one. You pick, the host opens a door that is not
yours, and you decide whether to switch. Everybody has an opinion about the
odds and the argument never gets as far as the part that settles it, which is
how the host chose which door to open. Here you set the host, and the same
sequence of events comes out at two thirds, a half, or nothing at all.

## Behavior

### A round

There are `doors` doors, three unless you say otherwise. The prize is behind one
of them, drawn evenly. You pick a door. The host opens doors that are not yours
until one unopened door is left beside your own. You stay or you switch. Then
everything opens.

### The three hosts

They differ in one thing only, which is how the host picks what to open. Every
one of them opens doors and offers the switch, so a round is the same shape
whoever is on and the rule is the only variable.

- **Knowing.** He knows where the prize is and never opens it. Switching wins
  `(doors - 1) / doors`, which is two thirds of the time on three doors.
- **Careless.** He opens at random among the doors that are not yours. When the
  prize turns up the round is void and another is dealt. Switching wins half the
  time. From where you sit a round that was not void looks exactly like a round
  with the knowing host, which is the whole trouble.

  A void round is shown before it is thrown away: what he opened stands open,
  the prize among it, and your own door stays shut because the round did not
  happen. It used to be discarded between one keypress and the next with nothing
  drawn, and that read as the game resetting itself rather than as the host
  giving the game away.
- **Mean.** He opens the prize the moment you pick wrong, and leaves you an
  empty door to move to. Switching wins nothing, and staying wins `1 / doors`.

  He used to offer nothing at all on a losing pick, which is the same arithmetic
  and a worse round: two in three had no decision in them and ended the instant
  you pressed a key. Opening the prize costs nothing, since you had already lost
  when you picked, and it makes his round the same shape as the other two. You
  simply get to watch it happen.

### The tally

Per host: rounds played, rounds won by staying, rounds won by switching, and
rounds thrown away. Drawn as two bars against each other with the counts beside
them.

The thrown-away rounds are kept and shown rather than quietly dropped. They are
exactly the information the knowing host never has to give you, and on many
doors the careless host throws away nearly everything, which is worth seeing
rather than waiting through.

### Playing a lot of them

By hand you play one door and one decision at a time, which fills one bar.

A key runs a batch of rounds under the current host, half of them staying and
half switching, so both bars move together and can be read against each other.
The batch is play sped up, not a different calculation: the same round code with
the same host runs it.

### More doors

The door count runs from three to twelve. The knowing host opens all but
one of the rest, and switching climbs from two thirds towards certainty, which
is the same lesson walking in from the other side. The careless host voids
almost every round once the count is past a handful, and the tally says so.

A hundred was the first thought and it cannot be drawn. The row is a fixed width
however many doors divide it, so at a hundred each door is a sliver, the gaps
and the piers fall below a pixel and the wall reads as a fence. Standing the
camera back does not help, because the row shrinks with it.

Twenty five was the second and it is a picket fence. The doors draw, but the row
cannot be used: twenty five identical shut doors is a dozen presses to cross and
nothing to count your place by. Twelve reads as doors, is six presses end to
end, and switching there wins ninety two percent of the time, which makes the
same point ninety nine does in a row a person can work.

The door the cursor is on stands ajar, a crack and no more. A colour and a light
on the floor are both easy to lose in a row of identical doors; a door off its
frame is not, and it needs no reading. It is well short of the point at which a
door counts as open, so standing ajar gives nothing away.

Nothing that marks a door is sized off the gap between two of them. The lintel,
the tab on it and the light on the floor all have a floor under their size, so
no count can make any of them disappear.

### What you see

The doors in a row, yours lit. The ones the host opens stand open. The prize is
a light behind its door and the rest are dark and empty.

## Acceptance criteria

- The knowing host never opens the prize. — `host::tests::the_knowing_host_keeps_the_prize_shut`
- Nor the door you picked. — `host::tests::no_host_opens_your_own_door`
- He leaves exactly one door beside yours. — `host::tests::one_door_is_left_to_switch_to`
- Switching against him wins two thirds on three doors. — `round::tests::switching_on_a_knowing_host_wins_two_thirds`
- And ninety two percent on twelve. — `round::tests::more_doors_make_switching_surer`
- Staying against him wins a third. — `round::tests::staying_on_a_knowing_host_wins_a_third`
- The careless host voids the rounds where he opens the prize. — `host::tests::the_careless_host_voids_what_he_gives_away`
- And it is shown before it is thrown away. — `stage::tests::a_careless_giveaway_is_shown_before_it_is_thrown_away`
- Switching against him wins half. — `round::tests::switching_on_a_careless_host_is_a_coin`
- The mean host shows you the prize the moment you are wrong. — `host::tests::the_mean_host_shows_you_the_prize_the_moment_you_are_wrong`
- And still leaves a door to move to. — `host::tests::the_mean_host_always_leaves_a_door_to_move_to`
- So his round is the same shape as anyone's. — `stage::tests::a_mean_host_still_offers_a_door_to_move_to`
- Switching against him never wins. — `round::tests::switching_on_a_mean_host_never_wins`
- And staying wins one door in three. — `round::tests::staying_on_a_mean_host_wins_one_in_three`
- A batch plays the same rounds the hand does. — `round::tests::a_batch_is_the_same_round_repeated`
- And splits itself evenly between staying and switching. — `tally::tests::a_batch_fills_both_bars`
- The tally counts what happened and nothing else. — `tally::tests::the_tally_adds_up`
- Thrown away rounds are counted, not dropped. — `tally::tests::a_void_round_is_counted_as_one`

- The door under the cursor stands ajar. — `stage::tests::the_door_under_the_cursor_stands_ajar`
- And not far enough to give the prize away. — `stage::tests::standing_ajar_gives_nothing_away`
- Nothing holds back once the round is over. — `stage::tests::nothing_stands_ajar_once_it_is_all_over`
- Nothing over the row vanishes as the doors multiply. — `doors::tests::nothing_over_the_row_vanishes_as_the_doors_multiply`
- The prize stands in the open part of its alcove at every count. — `doors::tests::the_prize_stands_in_the_open_part_of_the_alcove`

### Verified by hand

- Playing twenty rounds against the knowing host and twenty against the careless
  one, the two look the same and the bars do not.
- A batch of a thousand separates the bars far enough to read across the room.
- Twelve doors against the careless host voids nearly everything, and the count
  of thrown away rounds says why.

## Out of scope

Goats. A goat is a modelling project and this is a game about a rule. An empty
door is empty.

A host you write yourself. Three is enough to make the point and the three are
the ones people actually argue about.

Any explanation on screen beyond the tally. The bars are the argument.

Betting, money, or a score carried between hosts. The tally resets when the
host changes, because a tally mixing two rules is the mistake this game exists
to show.
