//! What happened, counted. See `specs/0001-whose-rule-it-is.md`.
//!
//! One tally per host, and it is thrown away when the host changes. A tally
//! mixing two rules is the mistake this game is about.

use rand::Rng;

use crate::host::Host;
use crate::round::{self, Habit, Outcome};

#[derive(Debug, Default, Copy, Clone)]
pub struct Tally {
    pub stayed: u32,
    pub stayed_won: u32,
    pub switched: u32,
    pub switched_won: u32,
    /// Rounds the host gave away. Kept rather than dropped: they are the
    /// information a knowing host never has to hand over.
    pub thrown_away: u32,
}

impl Tally {
    pub fn record(&mut self, habit: Habit, outcome: Outcome) {
        match (outcome, habit) {
            (Outcome::Void, _) => self.thrown_away += 1,
            (outcome, Habit::Stay) => {
                self.stayed += 1;
                self.stayed_won += u32::from(outcome == Outcome::Won);
            }
            (outcome, Habit::Switch) => {
                self.switched += 1;
                self.switched_won += u32::from(outcome == Outcome::Won);
            }
        }
    }

    /// Every round that happened, the thrown away ones included.
    pub fn rounds(&self) -> u32 {
        self.stayed + self.switched + self.thrown_away
    }

    /// How often staying won, or nothing at all when nobody has stayed yet.
    pub fn stay_rate(&self) -> Option<f32> {
        share(self.stayed_won, self.stayed)
    }

    pub fn switch_rate(&self) -> Option<f32> {
        share(self.switched_won, self.switched)
    }
}

fn share(won: u32, of: u32) -> Option<f32> {
    if of == 0 {
        return None;
    }
    Some(won as f32 / of as f32)
}

/// How many rounds a keypress plays.
pub const BATCH: u32 = 1_000;

/// Plays a batch into `tally`, half staying and half switching, so the two bars
/// move together and can be read against each other.
///
/// The same round the hand plays, run in a loop. Nothing here knows a formula.
pub fn batch(tally: &mut Tally, host: Host, doors: usize, rounds: u32, rng: &mut impl Rng) {
    for n in 0..rounds {
        let habit = Habit::ALL[n as usize % Habit::ALL.len()];
        tally.record(habit, round::play(host, doors, habit, rng));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn the_tally_adds_up() {
        let mut tally = Tally::default();
        tally.record(Habit::Stay, Outcome::Won);
        tally.record(Habit::Stay, Outcome::Lost);
        tally.record(Habit::Switch, Outcome::Won);
        tally.record(Habit::Switch, Outcome::Won);
        tally.record(Habit::Switch, Outcome::Lost);

        assert_eq!(tally.rounds(), 5);
        assert_eq!(tally.stay_rate(), Some(0.5));
        assert!((tally.switch_rate().unwrap() - 2.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn a_void_round_is_counted_as_one() {
        let mut tally = Tally::default();
        tally.record(Habit::Switch, Outcome::Void);

        assert_eq!(tally.thrown_away, 1);
        assert_eq!(tally.rounds(), 1);
        assert_eq!(tally.switched, 0, "a round nobody played is nobody's");
        assert_eq!(tally.switch_rate(), None);
    }

    #[test]
    fn nothing_played_is_no_rate_rather_than_zero() {
        let tally = Tally::default();

        assert_eq!(tally.stay_rate(), None);
        assert_eq!(tally.switch_rate(), None);
    }

    #[test]
    fn a_batch_fills_both_bars() {
        let mut tally = Tally::default();
        let mut rng = StdRng::seed_from_u64(3);
        batch(&mut tally, Host::Knowing, 3, BATCH, &mut rng);

        assert_eq!(tally.stayed + tally.switched, BATCH);
        assert_eq!(tally.stayed, tally.switched, "the halves are not even");
        assert!(tally.stay_rate().is_some());
        assert!(tally.switch_rate().is_some());
    }

    #[test]
    fn a_batch_says_what_the_hand_says() {
        let mut tally = Tally::default();
        let mut rng = StdRng::seed_from_u64(11);
        batch(&mut tally, Host::Knowing, 3, 40_000, &mut rng);

        assert!((tally.switch_rate().unwrap() - 2.0 / 3.0).abs() < 0.02);
        assert!((tally.stay_rate().unwrap() - 1.0 / 3.0).abs() < 0.02);
    }

    #[test]
    fn a_careless_host_throws_a_batch_away_and_says_so() {
        let mut tally = Tally::default();
        let mut rng = StdRng::seed_from_u64(5);
        batch(&mut tally, Host::Careless, 3, BATCH, &mut rng);

        assert!(tally.thrown_away > 0, "he never gave one away");
        assert_eq!(tally.rounds(), BATCH);
    }
}
