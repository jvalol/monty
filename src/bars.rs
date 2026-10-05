//! The tally drawn as two bars against each other. See `specs/0001-whose-rule-it-is.md`.
//!
//! Quads and nothing else, so where they land is checked without a window. The
//! bars are the argument this game makes, so they are worth a test.

use blitzkit::geometry::quad::Quad;
use glam::{vec2, vec4, Vec2, Vec4};

use crate::tally::Tally;

/// Where the pair sits and how big it is, in pixels from the window's corner.
///
/// Under the readout rather than off in a corner, because the line that says
/// what the numbers are and the bars that show them are one thing to read.
pub const LABEL_LEFT: f32 = 28.0;
pub const LABEL_SIZE: f32 = 14.0;
/// The track starts past the widest label, so a bar never runs under its name.
pub const LEFT: f32 = 152.0;
pub const TOP: f32 = 148.0;
pub const LONGEST: f32 = 340.0;
pub const THICK: f32 = 26.0;
pub const APART: f32 = 38.0;

/// The track a bar is drawn in, so an empty bar is still a shape rather than
/// nothing at all.
pub const TRACK: Vec4 = vec4(0.05, 0.05, 0.07, 1.0);
pub const STAY: Vec4 = vec4(0.58, 0.18, 0.16, 1.0);
pub const SWITCH: Vec4 = vec4(0.12, 0.42, 0.26, 1.0);

/// A mark at the length a bar would reach at an even chance, so a bar sitting
/// on a half is visibly sitting on a half rather than somewhere near it.
pub const EVEN: Vec4 = vec4(0.75, 0.76, 0.82, 1.0);
pub const EVEN_WIDE: f32 = 2.0;

/// What each bar is, said beside it. Two unexplained coloured bars are two
/// unexplained coloured bars.
pub const NAMES: [&str; 2] = ["stayed", "switched"];

/// The shortest a bar gets once its habit has been played at all.
///
/// A habit played twice and lost twice is worth nothing, and drawing nothing
/// for it made it look exactly like a habit nobody had tried. The stub says
/// "this has been played and it is winning none of them", which is a different
/// sentence.
pub const STUB: f32 = 4.0;

/// The names are written brighter than the bars they belong to, because a bar
/// dark enough to sit under a pale one is too dark to read as text.
pub const STAY_INK: Vec4 = vec4(0.95, 0.52, 0.46, 1.0);
pub const SWITCH_INK: Vec4 = vec4(0.42, 0.88, 0.60, 1.0);

/// Where a bar's name goes, and in what. `n` is 0 for staying and 1 for
/// switching, the order they are drawn in.
///
/// The y is the top of the line rather than its middle, because that is where
/// the renderer hangs a string from.
/// What a bar is worth, written at its far end: how often it won, and out of
/// how many.
///
/// The count matters as much as the share. Two rounds and five hundred draw the
/// same length and mean nothing like the same thing.
pub fn figure(tally: &Tally, n: usize) -> String {
    let (rate, of) = if n == 0 {
        (tally.stay_rate(), tally.stayed)
    } else {
        (tally.switch_rate(), tally.switched)
    };

    match rate {
        Some(rate) => format!("{:.0}% of {}", rate * 100.0, of),
        None => String::from("none yet"),
    }
}

/// Where that figure goes, past the end of the track.
pub fn figure_at(n: usize) -> Vec2 {
    let line = blitzkit::text::room_for("0", LABEL_SIZE).y;

    vec2(LEFT + LONGEST + 12.0, TOP + n as f32 * APART - line * 0.5)
}

pub fn name_at(n: usize) -> (Vec2, Vec4) {
    let line = blitzkit::text::room_for(NAMES[n], LABEL_SIZE).y;
    let middle_y = TOP + n as f32 * APART;

    (
        vec2(LABEL_LEFT, middle_y - line * 0.5),
        if n == 0 { STAY_INK } else { SWITCH_INK },
    )
}

/// The two bars and their tracks, in the order they are to be pushed.
///
/// A habit nobody has played yet draws its track and no bar, rather than a bar
/// of length nothing, which would read as a loss.
pub fn of(tally: &Tally, _window: Vec2) -> Vec<Quad> {
    let mut quads = Vec::new();

    for (n, rate) in [tally.stay_rate(), tally.switch_rate()].iter().enumerate() {
        let colour = if n == 0 { STAY } else { SWITCH };
        let middle_y = TOP + n as f32 * APART;

        quads.push(Quad::colored(
            vec2(LEFT + LONGEST * 0.5, middle_y),
            vec2(LONGEST, THICK),
            TRACK,
        ));

        if let Some(rate) = rate {
            let long = (LONGEST * rate.clamp(0.0, 1.0)).max(STUB);
            quads.push(Quad::colored(
                vec2(LEFT + long * 0.5, middle_y),
                vec2(long, THICK),
                colour,
            ));
        }

        quads.push(Quad::colored(
            vec2(LEFT + LONGEST * 0.5, middle_y),
            vec2(EVEN_WIDE, THICK),
            EVEN,
        ));
    }

    quads
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::round::{Habit, Outcome};

    const WINDOW: Vec2 = Vec2::new(900.0, 600.0);

    fn tally(stayed: u32, stayed_won: u32, switched: u32, switched_won: u32) -> Tally {
        let mut tally = Tally::default();
        for n in 0..stayed {
            let outcome = if n < stayed_won {
                Outcome::Won
            } else {
                Outcome::Lost
            };
            tally.record(Habit::Stay, outcome);
        }
        for n in 0..switched {
            let outcome = if n < switched_won {
                Outcome::Won
            } else {
                Outcome::Lost
            };
            tally.record(Habit::Switch, outcome);
        }
        tally
    }

    fn bar_of(quads: &[Quad], colour: Vec4) -> Option<Quad> {
        quads.iter().find(|quad| quad.color == colour).copied()
    }

    #[test]
    fn a_longer_bar_is_the_one_that_won_more() {
        // two thirds against one third, which is the knowing host
        let quads = of(&tally(300, 100, 300, 200), WINDOW);

        let stay = bar_of(&quads, STAY).expect("a stay bar");
        let switch = bar_of(&quads, SWITCH).expect("a switch bar");

        assert!(switch.size.x > stay.size.x);
        assert!((switch.size.x / stay.size.x - 2.0).abs() < 0.05);
    }

    #[test]
    fn two_halves_draw_the_same_length() {
        // the careless host, where the pair sitting on top of each other is the
        // whole of what the player is being shown
        let quads = of(&tally(400, 200, 400, 200), WINDOW);

        let stay = bar_of(&quads, STAY).expect("a stay bar");
        let switch = bar_of(&quads, SWITCH).expect("a switch bar");

        assert!((stay.size.x - switch.size.x).abs() < 1e-3);
        assert!((stay.size.x - LONGEST * 0.5).abs() < 1e-3);
    }

    #[test]
    fn a_habit_nobody_played_draws_a_track_and_no_bar() {
        let quads = of(&tally(10, 4, 0, 0), WINDOW);

        assert!(bar_of(&quads, STAY).is_some());
        assert!(
            bar_of(&quads, SWITCH).is_none(),
            "an unplayed habit drew a bar of nothing, which reads as a loss"
        );
    }

    #[test]
    fn a_bar_that_never_wins_is_still_drawn() {
        // ten played and ten lost is not the same as none played, and drawing
        // nothing for both said it was
        let quads = of(&tally(10, 4, 10, 0), WINDOW);
        let switch = bar_of(&quads, SWITCH).expect("a played habit draws something");

        assert!((switch.size.x - STUB).abs() < 1e-3);
    }

    #[test]
    fn a_figure_says_how_many_rounds_are_behind_it() {
        let tally = tally(0, 0, 2, 0);

        assert_eq!(figure(&tally, 1), "0% of 2");
        assert_eq!(figure(&tally, 0), "none yet");
    }

    #[test]
    fn a_figure_sits_past_the_end_of_its_track() {
        for n in 0..NAMES.len() {
            assert!(figure_at(n).x > LEFT + LONGEST);
        }
    }

    #[test]
    fn the_even_mark_is_halfway_along_the_track() {
        let quads = of(&tally(10, 5, 10, 5), WINDOW);
        let mark = bar_of(&quads, EVEN).expect("an even mark");

        assert!((mark.position.x - (LEFT + LONGEST * 0.5)).abs() < 1e-3);
    }

    #[test]
    fn the_bars_sit_under_the_readout() {
        let quads = of(&tally(10, 5, 10, 5), WINDOW);

        for quad in &quads {
            assert!(quad.position.y > TOP - THICK, "a bar climbed into the text");
            assert!(quad.position.y < TOP + APART * 2.0, "a bar drifted off");
        }
    }

    #[test]
    fn staying_is_drawn_above_switching() {
        let quads = of(&tally(10, 5, 10, 5), WINDOW);
        let stay = bar_of(&quads, STAY).expect("a stay bar");
        let switch = bar_of(&quads, SWITCH).expect("a switch bar");

        // the readout says stayed first, so the bars read in the same order
        assert!(stay.position.y < switch.position.y);
    }

    #[test]
    fn a_name_sits_clear_of_the_track_it_belongs_to() {
        for (n, name) in NAMES.iter().enumerate() {
            let (at, _) = name_at(n);
            let wide = blitzkit::text::room_for(name, LABEL_SIZE).x;

            assert!(at.x >= LABEL_LEFT - 1e-3);
            assert!(at.x + wide < LEFT, "{} runs under the bar", name);
        }
    }

    #[test]
    fn a_name_lines_up_with_its_own_bar() {
        let quads = of(&tally(10, 5, 10, 5), WINDOW);
        let stay = bar_of(&quads, STAY).expect("a stay bar");
        let switch = bar_of(&quads, SWITCH).expect("a switch bar");

        let line = blitzkit::text::room_for(NAMES[0], LABEL_SIZE).y;
        assert!((name_at(0).0.y + line * 0.5 - stay.position.y).abs() < 1.0);
        assert!((name_at(1).0.y + line * 0.5 - switch.position.y).abs() < 1.0);
    }

    #[test]
    fn the_names_are_brighter_than_the_bars() {
        // dark enough to sit under a pale bar is too dark to read as text
        assert!(STAY_INK.x > STAY.x);
        assert!(SWITCH_INK.y > SWITCH.y);
    }
}
