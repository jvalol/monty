# monty

Three doors and a prize behind one. You pick, the host opens a door that is not
yours, and you decide whether to switch.

![Three doors in a grey wall, all swung open to the left, with a pale blue tab on the lintel over the middle one. A glowing ball sits in the middle alcove and the other two are dark and empty. Two bars across the top left: a short red one marked stayed at 30 percent of 500, and a longer green one marked switched at 67 percent of 501](media/screenshot.png)

```
cargo run --release
```

Everyone has had this argument and it never gets as far as the part that settles
it, which is how the host chose which door to open. So that is the part you set.

The **knowing** host never opens the prize, and switching wins two thirds of the
time. The **careless** host opens at random and the round is thrown away when he
gives it away, and then both staying and switching win half the time. The
**mean** host offers the switch only when you were already right, and switching
wins nothing at all.

From where you sit those rounds look the same. The bars do not. Play twenty by
hand, then hold a key and watch a thousand go by.

The doors run from three to a hundred. The knowing host opening ninety eight of
them is the same fact that is hard to believe with three.

Built on [blitzkit](https://github.com/jvalol/blitzkit).

Specs are in [`specs/`](specs/), written before the code.

## Licence

MIT or Apache-2.0, at your option.

---

I asked AI to draft this for me. I've edited it. Any surviving AI smells are my oversight.
