//! Who opens which door. It is the only thing that differs between the three,
//! and it is the whole of the answer. See `specs/0001-whose-rule-it-is.md`.
//!
//! Nothing here draws anything or reads a clock, so all of it is checked on the
//! CPU by playing a great many rounds against a seeded source of chance.

use rand::seq::SliceRandom;
use rand::Rng;

/// How the host decides what to open.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Host {
    /// He knows where the prize is and never opens it.
    Knowing,
    /// He opens at random and sometimes gives it away.
    Careless,
    /// He opens the prize the moment you pick wrong.
    Mean,
}

impl Host {
    pub const ALL: [Host; 3] = [Host::Knowing, Host::Careless, Host::Mean];
}

/// What the host did.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Opened {
    /// The doors he opened. One door is left closed beside your own.
    Doors(Vec<usize>),
    /// He opened the prize, and these are the doors he opened. The round is
    /// thrown away, but it is shown first: a round that vanishes between one
    /// keypress and the next reads as the game resetting itself.
    Gave(Vec<usize>),
}

/// The doors, numbered from zero.
///
/// Three is the argument everyone has had. A hundred is the same argument with
/// the answer showing.
pub const FEWEST: usize = 3;
/// A hundred was the first thought and it cannot be drawn. The row is a fixed
/// width however many doors divide it, so at a hundred each one is a twentieth
/// of a degree across and the wall reads as a fence. Pulling the camera back
/// does not help: the row shrinks with it.
///
/// Twenty five was the second thought and it is a picket fence. The doors are
/// drawable but the row is not navigable: twenty five identical shut doors is
/// a dozen arrow presses to cross and no way to count where you are.
///
/// Twelve reads as doors, is six presses end to end, and switching against the
/// knowing host wins 92 percent of the time there. That makes the same point
/// 99 does, and it makes it in a row a person can use.
pub const MOST: usize = 12;

/// What the host does, given where the prize is and which door you picked.
///
/// Opens every door but two: yours, and one more. Which one more is the whole
/// question.
pub fn opens(host: Host, doors: usize, prize: usize, picked: usize, rng: &mut impl Rng) -> Opened {
    match host {
        Host::Knowing => Opened::Doors(all_but(doors, picked, keeper(doors, prize, picked, rng))),
        Host::Careless => {
            // he does not look first, so the doors he opens are drawn from
            // everything that is not yours, prize included
            let others: Vec<usize> = (0..doors).filter(|door| *door != picked).collect();
            let opening: Vec<usize> = others
                .choose_multiple(rng, doors.saturating_sub(2))
                .copied()
                .collect();

            if opening.contains(&prize) {
                Opened::Gave(opening)
            } else {
                Opened::Doors(opening)
            }
        }
        // the same shape of round as the other two: a door opens and you
        // decide. He used to open nothing at all when you picked wrong, which
        // was two rounds in three with no decision in them and read as the
        // game ending on one keypress.
        Host::Mean => {
            if picked == prize {
                Opened::Doors(all_but(doors, picked, keeper(doors, prize, picked, rng)))
            } else {
                // he opens the prize and leaves you an empty one to move to, so
                // you can see you have lost before you choose
                let empty: Vec<usize> = (0..doors)
                    .filter(|door| *door != picked && *door != prize)
                    .collect();
                let keeper = *empty.choose(rng).expect("a losing pick leaves two others");

                Opened::Doors(all_but(doors, picked, keeper))
            }
        }
    }
}

/// The one door the host leaves closed beside yours.
///
/// The prize, unless you are already standing in front of it, in which case any
/// other door will do and he picks one.
fn keeper(doors: usize, prize: usize, picked: usize, rng: &mut impl Rng) -> usize {
    if prize != picked {
        return prize;
    }

    let rest: Vec<usize> = (0..doors).filter(|door| *door != picked).collect();
    *rest.choose(rng).expect("there is more than one door")
}

fn all_but(doors: usize, picked: usize, keeper: usize) -> Vec<usize> {
    (0..doors)
        .filter(|door| *door != picked && *door != keeper)
        .collect()
}

/// The door the host left closed, out of what he opened.
pub fn left_closed(doors: usize, picked: usize, opened: &[usize]) -> Option<usize> {
    (0..doors).find(|door| *door != picked && !opened.contains(door))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn rng(seed: u64) -> StdRng {
        StdRng::seed_from_u64(seed)
    }

    #[test]
    fn the_knowing_host_keeps_the_prize_shut() {
        for seed in 0..200 {
            let mut rng = rng(seed);
            let prize = rng.gen_range(0..3);
            let picked = rng.gen_range(0..3);

            if let Opened::Doors(opened) = opens(Host::Knowing, 3, prize, picked, &mut rng) {
                assert!(!opened.contains(&prize), "he gave it away on seed {}", seed);
            } else {
                panic!("the knowing host always opens something");
            }
        }
    }

    #[test]
    fn no_host_opens_your_own_door() {
        for seed in 0..200 {
            for host in Host::ALL {
                let mut rng = rng(seed);
                let prize = rng.gen_range(0..3);
                let picked = rng.gen_range(0..3);

                if let Opened::Doors(opened) = opens(host, 3, prize, picked, &mut rng) {
                    assert!(!opened.contains(&picked), "{:?} opened yours", host);
                }
            }
        }
    }

    #[test]
    fn one_door_is_left_to_switch_to() {
        for doors in [3, 4, 10, 100] {
            for seed in 0..50 {
                let mut rng = rng(seed);
                let prize = rng.gen_range(0..doors);
                let picked = rng.gen_range(0..doors);

                if let Opened::Doors(opened) = opens(Host::Knowing, doors, prize, picked, &mut rng)
                {
                    assert_eq!(opened.len(), doors - 2);
                    assert!(left_closed(doors, picked, &opened).is_some());
                }
            }
        }
    }

    #[test]
    fn the_careless_host_voids_what_he_gives_away() {
        let mut voids = 0;
        let mut rounds = 0;

        for seed in 0..2_000 {
            let mut rng = rng(seed);
            let prize = rng.gen_range(0..3);
            let picked = rng.gen_range(0..3);

            match opens(Host::Careless, 3, prize, picked, &mut rng) {
                Opened::Gave(opened) => {
                    assert!(opened.contains(&prize), "he voided without giving it away");
                    voids += 1;
                }
                Opened::Doors(opened) => assert!(!opened.contains(&prize)),
            }
            rounds += 1;
        }

        // he opens one of the two that are not yours, and the prize is behind
        // one of them two times in three, so a third of all rounds go in the bin
        let rate = voids as f32 / rounds as f32;
        assert!((rate - 1.0 / 3.0).abs() < 0.03, "{} voided", rate);
    }

    #[test]
    fn the_mean_host_shows_you_the_prize_the_moment_you_are_wrong() {
        for seed in 0..500 {
            let mut rng = rng(seed);
            let prize = rng.gen_range(0..3);
            let picked = rng.gen_range(0..3);

            let Opened::Doors(opened) = opens(Host::Mean, 3, prize, picked, &mut rng) else {
                panic!("he always opens something now");
            };

            if picked == prize {
                assert!(!opened.contains(&prize), "he opened the one you are on");
            } else {
                assert!(opened.contains(&prize), "he kept it back on a losing pick");
            }
        }
    }

    #[test]
    fn the_mean_host_always_leaves_a_door_to_move_to() {
        // every host offers now, so a round is the same shape whoever is on
        for seed in 0..300 {
            for host in Host::ALL {
                let mut rng = rng(seed);
                let prize = rng.gen_range(0..3);
                let picked = rng.gen_range(0..3);

                if let Opened::Doors(opened) = opens(host, 3, prize, picked, &mut rng) {
                    assert!(left_closed(3, picked, &opened).is_some(), "{:?}", host);
                }
            }
        }
    }
}
