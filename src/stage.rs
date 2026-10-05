//! What is happening right now, and what a key does about it.
//!
//! The rules live in `host`, `round` and `tally`. This is the order a player
//! walks through them in, and it is kept apart from the drawing so it can be
//! played without a window. See `specs/0001-whose-rule-it-is.md`.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::doors;
use crate::host::{self, Host, Opened, FEWEST, MOST};
use crate::round::{Habit, Outcome};
use crate::tally::{self, Tally};

/// The door counts a press steps between. Three is the argument; a hundred is
/// the argument with the answer showing.
pub const COUNTS: [usize; 6] = [3, 4, 5, 6, 8, MOST];

/// How long the doors take to finish swinging before a key does anything.
pub const SETTLES: f32 = doors::SWING_SECONDS;

/// How far off its frame the door under the cursor stands.
///
/// A crack, and well under the point at which a door counts as open, so nothing
/// behind it shows and it cannot give the prize away.
pub const AJAR: f32 = 0.09;

/// What the host just did, so the readout can say it.
///
/// Without this a round against the mean host is one keypress and then every
/// door open, which reads as the game breaking rather than as the host
/// refusing. What he did is the whole of what this game is about, so it is said
/// out loud every time.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Said {
    Pick,
    Offered,
    /// He opened the prize. The round did not count and another is dealt.
    GaveItAway,
    Won,
    Lost,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Phase {
    /// Choosing a door.
    Picking,
    /// The host has opened what he is going to. Stay or move across.
    Offered,
    /// Everything is open and the answer is on the floor.
    Shown,
    /// The careless host opened the prize. What he opened stands open, your own
    /// door does not, and the round is not counted. It is a phase of its own
    /// because throwing the round away between one keypress and the next, with
    /// nothing drawn, read as the game resetting itself.
    Gave,
}

pub struct Stage {
    pub host: Host,
    pub doors: usize,
    pub tally: Tally,

    pub phase: Phase,
    pub prize: usize,
    pub picked: usize,
    /// The door the cursor is on, which is what enter takes.
    pub on: usize,
    pub opened: Vec<usize>,
    pub outcome: Option<Outcome>,
    pub said: Said,
    /// How the last round was played, for the tally.
    pub habit: Habit,

    /// Seconds since the phase changed, so the doors have time to swing.
    pub since: f32,
    rng: StdRng,
}

impl Stage {
    pub fn new(seed: u64) -> Self {
        let mut stage = Self {
            host: Host::Knowing,
            doors: FEWEST,
            tally: Tally::default(),
            phase: Phase::Picking,
            prize: 0,
            picked: 0,
            on: 0,
            opened: Vec::new(),
            outcome: None,
            said: Said::Pick,
            habit: Habit::Stay,
            since: 0.0,
            rng: StdRng::seed_from_u64(seed),
        };
        stage.deal();
        stage
    }

    /// A fresh round, everything shut.
    pub fn deal(&mut self) {
        self.prize = self.rng.gen_range(0..self.doors);
        self.picked = 0;
        self.on = self.doors / 2;
        self.opened.clear();
        self.outcome = None;
        self.said = Said::Pick;
        self.phase = Phase::Picking;
        self.since = 0.0;
    }

    pub fn tick(&mut self, dt: f32) {
        self.since += dt;
    }

    /// How far open a door is, from 0 to 1.
    pub fn open_by(&self, door: usize) -> f32 {
        let shown = self.phase == Phase::Shown;
        let over = shown || self.phase == Phase::Gave;
        let open = shown || self.opened.contains(&door);

        if !open {
            // the one the cursor is on stands ajar. A colour and a light on the
            // floor are both easy to lose in a long row; a door off its frame
            // is not, and it needs no reading.
            return if door == self.on && !over { AJAR } else { 0.0 };
        }
        (self.since / doors::SWING_SECONDS).clamp(0.0, 1.0)
    }

    /// Whether a key should be listened to yet.
    pub fn settled(&self) -> bool {
        self.since >= SETTLES
    }

    /// Which doors the cursor may be on in this phase.
    pub fn reachable(&self) -> Vec<usize> {
        match self.phase {
            Phase::Picking => (0..self.doors).collect(),
            Phase::Offered => {
                let mut pair = vec![self.picked];
                if let Some(other) = host::left_closed(self.doors, self.picked, &self.opened) {
                    pair.push(other);
                }
                pair.sort_unstable();
                pair
            }
            Phase::Shown | Phase::Gave => Vec::new(),
        }
    }

    pub fn move_by(&mut self, step: i32) {
        let reachable = self.reachable();
        if reachable.is_empty() {
            return;
        }

        let at = reachable
            .iter()
            .position(|door| *door == self.on)
            .unwrap_or(0);
        let next = (at as i32 + step).rem_euclid(reachable.len() as i32) as usize;
        self.on = reachable[next];
    }

    /// Enter. Takes whatever the cursor is on, and moves the round along.
    pub fn take(&mut self) {
        match self.phase {
            Phase::Picking => {
                self.picked = self.on;

                match host::opens(
                    self.host,
                    self.doors,
                    self.prize,
                    self.picked,
                    &mut self.rng,
                ) {
                    // he gave it away, so nobody played. It is shown before it
                    // is thrown away, and the count of these is the point.
                    Opened::Gave(opened) => {
                        self.tally.record(self.habit, Outcome::Void);
                        self.opened = opened;
                        self.outcome = Some(Outcome::Void);
                        self.phase = Phase::Gave;
                        self.said = Said::GaveItAway;
                        self.since = 0.0;
                    }
                    // he offered nothing, so the pick stands and everything opens
                    Opened::Doors(opened) => {
                        self.opened = opened;
                        self.phase = Phase::Offered;
                        self.said = Said::Offered;
                        self.since = 0.0;
                    }
                }
            }
            Phase::Offered => {
                self.habit = if self.on == self.picked {
                    Habit::Stay
                } else {
                    Habit::Switch
                };
                self.settle(self.on);
            }
            Phase::Shown | Phase::Gave => self.deal(),
        }
    }

    fn settle(&mut self, landed: usize) {
        let outcome = if landed == self.prize {
            Outcome::Won
        } else {
            Outcome::Lost
        };

        self.tally.record(self.habit, outcome);
        self.outcome = Some(outcome);
        self.said = if outcome == Outcome::Won {
            Said::Won
        } else {
            Said::Lost
        };
        self.on = landed;
        self.phase = Phase::Shown;
        self.since = 0.0;
    }

    /// A batch, which is the same round in a loop. Leaves the table as it was.
    pub fn run_a_batch(&mut self) {
        tally::batch(
            &mut self.tally,
            self.host,
            self.doors,
            tally::BATCH,
            &mut self.rng,
        );
    }

    /// Changing the host throws the tally away, because a tally mixing two
    /// rules is the mistake this game is about.
    pub fn set_host(&mut self, host: Host) {
        if self.host == host {
            return;
        }
        self.host = host;
        self.tally = Tally::default();
        self.deal();
    }

    /// The next door count up or down the list. Also throws the tally away:
    /// two thirds and ninety nine percent are not the same measurement.
    pub fn step_doors(&mut self, step: i32) {
        let at = COUNTS
            .iter()
            .position(|count| *count == self.doors)
            .unwrap_or(0);
        let next = (at as i32 + step).clamp(0, COUNTS.len() as i32 - 1) as usize;

        if COUNTS[next] == self.doors {
            return;
        }
        self.doors = COUNTS[next];
        self.tally = Tally::default();
        self.deal();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage() -> Stage {
        Stage::new(4)
    }

    #[test]
    fn a_fresh_round_has_everything_shut() {
        let stage = stage();

        assert_eq!(stage.phase, Phase::Picking);
        assert!(stage.opened.is_empty());
        assert!(stage.outcome.is_none());
        // every door but the one the cursor is on, which stands ajar
        assert!((0..stage.doors)
            .filter(|door| *door != stage.on)
            .all(|door| stage.open_by(door) == 0.0));
    }

    #[test]
    fn picking_leaves_one_door_to_move_to() {
        let mut stage = stage();
        stage.on = 1;
        stage.take();

        assert_eq!(stage.phase, Phase::Offered);
        assert_eq!(stage.reachable().len(), 2);
        assert!(stage.reachable().contains(&stage.picked));
    }

    #[test]
    fn the_cursor_only_reaches_doors_that_are_still_shut() {
        let mut stage = stage();
        stage.take();

        for door in stage.reachable() {
            assert!(!stage.opened.contains(&door), "door {} is open", door);
        }
    }

    #[test]
    fn moving_across_counts_as_switching() {
        let mut stage = stage();
        stage.take();
        let other = stage
            .reachable()
            .into_iter()
            .find(|door| *door != stage.picked)
            .expect("a door to move to");

        stage.on = other;
        stage.take();

        assert_eq!(stage.habit, Habit::Switch);
        assert_eq!(stage.phase, Phase::Shown);
        assert_eq!(stage.tally.switched, 1);
    }

    #[test]
    fn staying_put_counts_as_staying() {
        let mut stage = stage();
        stage.take();
        stage.on = stage.picked;
        stage.take();

        assert_eq!(stage.habit, Habit::Stay);
        assert_eq!(stage.tally.stayed, 1);
    }

    #[test]
    fn everything_opens_once_it_is_over() {
        let mut stage = stage();
        stage.take();
        stage.take();
        stage.tick(doors::SWING_SECONDS);

        assert!((0..stage.doors).all(|door| stage.open_by(door) == 1.0));
    }

    #[test]
    fn a_careless_giveaway_is_shown_before_it_is_thrown_away() {
        let mut stage = stage();
        stage.set_host(Host::Careless);

        for _ in 0..200 {
            stage.take();
            if stage.phase == Phase::Gave {
                // the prize he opened stands open and the round is counted
                assert!(stage.opened.contains(&stage.prize));
                assert!(stage.tally.thrown_away > 0);
                stage.tick(doors::SWING_SECONDS);
                assert_eq!(stage.open_by(stage.prize), 1.0, "he hid what he gave");

                // and the player's own door stays shut, because the round did
                // not happen
                assert_eq!(stage.open_by(stage.picked), 0.0);

                // only then is another dealt
                stage.take();
                assert_eq!(stage.phase, Phase::Picking);
                assert!(stage.opened.is_empty());
                return;
            }
            stage.deal();
        }

        panic!("he never gave one away in two hundred rounds");
    }

    #[test]
    fn a_mean_host_still_offers_a_door_to_move_to() {
        let mut stage = stage();
        stage.set_host(Host::Mean);

        for _ in 0..200 {
            stage.take();
            // whatever he did, the round is a round: he opened something and
            // there is a decision to make
            assert_eq!(stage.phase, Phase::Offered, "he ended it on one press");
            assert_eq!(stage.reachable().len(), 2);
            stage.deal();
        }
    }

    #[test]
    fn a_mean_host_shows_the_prize_when_the_pick_was_wrong() {
        let mut stage = stage();
        stage.set_host(Host::Mean);

        for _ in 0..200 {
            stage.take();
            if stage.picked != stage.prize {
                assert!(stage.opened.contains(&stage.prize), "he kept it back");
                return;
            }
            stage.deal();
        }

        panic!("two hundred rounds and every pick was right");
    }

    #[test]
    fn changing_the_host_throws_the_tally_away() {
        let mut stage = stage();
        stage.run_a_batch();
        assert!(stage.tally.rounds() > 0);

        stage.set_host(Host::Mean);

        assert_eq!(stage.tally.rounds(), 0);
    }

    #[test]
    fn changing_the_door_count_throws_it_away_too() {
        let mut stage = stage();
        stage.run_a_batch();

        stage.step_doors(1);

        assert_eq!(stage.doors, COUNTS[1]);
        assert_eq!(stage.tally.rounds(), 0);
    }

    #[test]
    fn the_door_count_stops_at_both_ends() {
        let mut stage = stage();

        stage.step_doors(-1);
        assert_eq!(stage.doors, FEWEST);

        for _ in 0..20 {
            stage.step_doors(1);
        }
        assert_eq!(stage.doors, MOST);
    }

    #[test]
    fn a_batch_fills_the_tally_without_touching_the_table() {
        let mut stage = stage();
        let before = stage.prize;

        stage.run_a_batch();

        assert_eq!(stage.tally.rounds(), tally::BATCH);
        assert_eq!(stage.prize, before, "the batch moved the prize");
        assert_eq!(stage.phase, Phase::Picking);
    }

    #[test]
    fn a_door_swings_rather_than_jumping() {
        let mut stage = stage();
        stage.take();

        let door = stage.opened[0];
        assert_eq!(stage.open_by(door), 0.0);

        stage.tick(doors::SWING_SECONDS * 0.5);
        let half = stage.open_by(door);
        assert!(half > 0.0 && half < 1.0, "it jumped to {}", half);

        stage.tick(doors::SWING_SECONDS);
        assert_eq!(stage.open_by(door), 1.0);
    }

    #[test]
    fn a_fresh_round_asks_for_a_pick() {
        assert_eq!(stage().said, Said::Pick);
    }

    #[test]
    fn the_careless_host_says_when_he_gave_it_away() {
        let mut stage = stage();
        stage.set_host(Host::Careless);

        for _ in 0..200 {
            stage.take();
            if stage.said == Said::GaveItAway {
                assert_eq!(stage.phase, Phase::Gave, "it vanished without showing");
                assert!(stage.tally.thrown_away > 0);
                return;
            }
            stage.deal();
        }

        panic!("he never gave one away in two hundred rounds");
    }

    #[test]
    fn a_finished_round_says_how_it_came_out() {
        let mut stage = stage();
        stage.take();
        if stage.phase == Phase::Offered {
            stage.take();
        }

        assert!(matches!(stage.said, Said::Won | Said::Lost));
        assert_eq!(stage.said == Said::Won, stage.outcome == Some(Outcome::Won));
    }

    #[test]
    fn the_door_under_the_cursor_stands_ajar() {
        let mut stage = stage();
        stage.on = 1;

        assert!(
            stage.open_by(1) > 0.0,
            "the cursor door is flush with the rest"
        );
        assert_eq!(stage.open_by(0), 0.0);
        assert_eq!(stage.open_by(2), 0.0);
    }

    #[test]
    fn standing_ajar_gives_nothing_away() {
        let mut stage = stage();
        stage.on = stage.prize;

        // the draw shows the prize past a third open, and a crack is well short
        assert!(stage.open_by(stage.prize) < 0.3);
    }

    #[test]
    fn nothing_stands_ajar_once_it_is_all_over() {
        let mut stage = stage();
        stage.take();
        if stage.phase == Phase::Offered {
            stage.take();
        }
        stage.tick(doors::SWING_SECONDS);

        for door in 0..stage.doors {
            assert_eq!(stage.open_by(door), 1.0, "door {} held back", door);
        }
    }
}
