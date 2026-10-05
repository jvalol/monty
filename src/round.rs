//! One round, start to finish, as arithmetic. See `specs/0001-whose-rule-it-is.md`.
//!
//! The batch the game runs on a keypress is this same function in a loop, which
//! is why playing by hand and watching a thousand go by agree with each other.

use rand::Rng;

use crate::host::{self, Host, Opened};

/// What the player does when the host offers.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Habit {
    Stay,
    Switch,
}

impl Habit {
    pub const ALL: [Habit; 2] = [Habit::Stay, Habit::Switch];
}

/// How a round came out.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Outcome {
    Won,
    Lost,
    /// The host gave the prize away. Nobody won and the round is not counted
    /// against either habit.
    Void,
}

/// Plays one and says how it came out.
///
/// The whole round at once, which is what a batch wants. A round played by hand
/// is the player stepping through the same host one decision at a time, and
/// `stage` holds that. What the two share is `host::opens`, which is where the
/// rules actually are.
pub fn play(host: Host, doors: usize, habit: Habit, rng: &mut impl Rng) -> Outcome {
    let prize = rng.gen_range(0..doors);
    let picked = rng.gen_range(0..doors);

    let landed = match host::opens(host, doors, prize, picked, rng) {
        Opened::Void => return Outcome::Void,
        // he offered nothing, so the pick stands however the player meant to
        // play it. This is the mean host's whole trick.
        Opened::NoOffer => picked,
        Opened::Doors(opened) => match habit {
            Habit::Stay => picked,
            Habit::Switch => {
                host::left_closed(doors, picked, &opened).expect("a door was left to take")
            }
        },
    };

    if landed == prize {
        Outcome::Won
    } else {
        Outcome::Lost
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    /// How often `habit` wins, over enough rounds that the answer settles.
    fn rate(host: Host, doors: usize, habit: Habit, rounds: u32) -> f32 {
        let mut rng = StdRng::seed_from_u64(1);
        let mut played = 0;
        let mut won = 0;

        for _ in 0..rounds {
            match play(host, doors, habit, &mut rng) {
                Outcome::Void => continue,
                Outcome::Won => {
                    played += 1;
                    won += 1;
                }
                Outcome::Lost => played += 1,
            }
        }

        won as f32 / played as f32
    }

    #[test]
    fn switching_on_a_knowing_host_wins_two_thirds() {
        let rate = rate(Host::Knowing, 3, Habit::Switch, 60_000);
        assert!((rate - 2.0 / 3.0).abs() < 0.01, "{}", rate);
    }

    #[test]
    fn staying_on_a_knowing_host_wins_a_third() {
        let rate = rate(Host::Knowing, 3, Habit::Stay, 60_000);
        assert!((rate - 1.0 / 3.0).abs() < 0.01, "{}", rate);
    }

    #[test]
    fn more_doors_make_switching_surer() {
        for (doors, want) in [(3, 2.0 / 3.0), (6, 5.0 / 6.0), (12, 11.0 / 12.0)] {
            let rate = rate(Host::Knowing, doors, Habit::Switch, 40_000);
            assert!(
                (rate - want).abs() < 0.01,
                "{} doors came out {}",
                doors,
                rate
            );
        }
    }

    #[test]
    fn switching_on_a_careless_host_is_a_coin() {
        // the rounds he gave away are thrown out, and what is left is even,
        // which is the result nobody believes about this one
        let rate = rate(Host::Careless, 3, Habit::Switch, 60_000);
        assert!((rate - 0.5).abs() < 0.01, "{}", rate);
    }

    #[test]
    fn switching_on_a_mean_host_never_wins() {
        assert_eq!(rate(Host::Mean, 3, Habit::Switch, 20_000), 0.0);
    }

    #[test]
    fn staying_on_a_mean_host_wins_one_in_three() {
        // he changes nothing for a player who was never going to move
        let rate = rate(Host::Mean, 3, Habit::Stay, 60_000);
        assert!((rate - 1.0 / 3.0).abs() < 0.01, "{}", rate);
    }
}
