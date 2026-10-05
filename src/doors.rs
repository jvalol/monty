//! Where the doors stand and how they swing. See `specs/0001-whose-rule-it-is.md`.
//!
//! All of it is arithmetic on a row, so the layout is checked without a window.
//! A door swings about its own hinge rather than turning on the spot, which
//! means its drawn middle moves as it opens and has to be worked out rather
//! than stored.

use glam::{vec3, Quat, Vec3};

/// How wide the whole row is, whatever the count. Three doors are wide and a
/// hundred are slivers, which is the picture.
pub const SPAN: f32 = 7.0;

pub const TALL: f32 = 1.7;
pub const THICK: f32 = 0.07;

/// How much of a door's slot is the gap beside it.
pub const GAP: f32 = 0.16;

/// How far open a door swings, and how long it takes.
pub const SWINGS_TO: f32 = 1.9;
pub const SWING_SECONDS: f32 = 0.45;

/// How deep the recess behind a door is, and how far back the prize sits in it.
pub const DEEP: f32 = 0.5;

/// How wide one door is.
pub fn width(doors: usize) -> f32 {
    (SPAN / doors as f32) * (1.0 - GAP)
}

/// The middle of a door's slot, on the floor.
pub fn slot(door: usize, doors: usize) -> Vec3 {
    let step = SPAN / doors as f32;
    vec3((door as f32 + 0.5) * step - SPAN * 0.5, 0.0, 0.0)
}

/// The edge a door hangs from, which is its left one.
pub fn hinge(door: usize, doors: usize) -> Vec3 {
    slot(door, doors) - Vec3::X * width(doors) * 0.5
}

/// Where a door's middle is when it is `open` of the way open, from 0 to 1.
///
/// The panel hangs off the hinge, so opening carries its middle round an arc
/// rather than leaving it where it was.
pub fn panel(door: usize, doors: usize, open: f32) -> (Vec3, Quat) {
    let turn = Quat::from_rotation_y(-SWINGS_TO * open.clamp(0.0, 1.0));
    let reach = Vec3::X * width(doors) * 0.5;

    (
        hinge(door, doors) + turn * reach + Vec3::Y * TALL * 0.5,
        turn,
    )
}

/// The pier of wall between one slot and the next, and the two at the ends.
///
/// Without them the doors float in front of a flat wall and the gaps between
/// them read as more dark recess. With them it is a wall with doors in it.
///
/// `pier` runs from 0 to `doors`, so there is one to the left of every door and
/// one more past the last.
pub fn pier(pier: usize, doors: usize) -> (Vec3, Vec3) {
    let step = SPAN / doors as f32;
    let wide = step - width(doors);
    let x = pier as f32 * step - SPAN * 0.5;

    (
        vec3(x, TALL * 0.5, -DEEP * 0.5),
        vec3(wide, TALL + wide, DEEP),
    )
}

/// How many piers a row of `doors` has.
pub fn piers(doors: usize) -> usize {
    doors + 1
}

/// How tall the lintel is, and how much room a tab has on it.
///
/// It used to be the gap between two doors, which is fine at three doors and
/// nothing at a hundred: the lintel vanished and took the tab marking your own
/// door with it. A floor under it, so no count can make either disappear.
pub fn headroom(doors: usize) -> f32 {
    (SPAN / doors as f32 - width(doors)).max(TALL * 0.1)
}

/// The lintel over the whole row, so the doors are set into something rather
/// than standing under open sky.
pub fn lintel(doors: usize) -> (Vec3, Vec3) {
    let over = headroom(doors);

    (
        vec3(0.0, TALL + over * 0.5, -DEEP * 0.5),
        vec3(SPAN + over, over, DEEP),
    )
}

/// The dark recess behind a door, which is what an open one shows.
pub fn recess(door: usize, doors: usize) -> (Vec3, Vec3) {
    let middle = slot(door, doors) + Vec3::Y * TALL * 0.5 - Vec3::Z * (DEEP - BACK * 0.5);
    (middle, vec3(width(doors), TALL, BACK))
}

/// How thick the back of an alcove is.
///
/// A back wall and not a filled block. It was a block, and the prize behind a
/// door was buried inside it: at three doors the ball was big enough that its
/// front cap poked out through the doorway and it looked fine, and at ten it
/// was wholly inside the slab and the door opened on nothing. The alcove's
/// sides are the piers and its top is the lintel, both of which are already
/// `DEEP` deep, so the only piece missing was this.
pub const BACK: f32 = 0.06;

/// How deep the light on the floor in front of a door is, and how thick.
///
/// Deep, because the camera looks across the floor rather than down at it and a
/// thin strip there is a few pixels tall.
pub const MARK_DEEP: f32 = 0.42;
pub const MARK_HIGH: f32 = 0.02;

/// The slab on the floor in front of a door, which is where a mark goes.
///
/// On the floor rather than on the door itself, because a door swings and a
/// mark that swings with it leaves the room every time the host opens it.
///
pub fn mark(door: usize, doors: usize) -> (Vec3, Vec3) {
    (
        slot(door, doors) + vec3(0.0, MARK_HIGH * 0.5, MARK_DEEP * 0.6),
        // at least as wide as it is deep, so it reads as a light on the floor
        // rather than a scratch when the doors are narrow
        vec3(width(doors).max(MARK_DEEP), MARK_HIGH, MARK_DEEP),
    )
}

/// The tab over a door, on the face of the lintel, which is where the door you
/// took is marked.
///
/// Above the row rather than beside the light on the floor. The camera looks
/// across that floor at a shallow angle, so two marks lying on it read as one
/// bar with a coloured edge. Up here it is seen square on and it cannot be
/// stood in front of.
pub fn tab(door: usize, doors: usize) -> (Vec3, Vec3) {
    let over = headroom(doors);

    (
        slot(door, doors) + vec3(0.0, TALL + over * 0.5, 0.012),
        // never narrower than it is tall, so one door in a long row is still a
        // mark and not a hairline
        vec3((width(doors) * 0.45).max(over * 0.8), over * 0.5, 0.02),
    )
}

/// Where the prize sits behind its door.
pub fn prize_at(door: usize, doors: usize) -> Vec3 {
    slot(door, doors) + Vec3::Y * TALL * 0.4 - Vec3::Z * DEEP * 0.6
}

/// How far back the camera has to stand to take the whole row in, with room
/// above the doors for the readout and floor below them to stand on.
pub fn eye() -> Vec3 {
    vec3(0.0, TALL * 0.95, SPAN * 1.05)
}

/// What the camera looks at. Below the middle of a door, so the row sits in the
/// lower half of the window and the readout has the upper half to itself.
pub fn looked_at() -> Vec3 {
    vec3(0.0, TALL * 0.62, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_row_is_the_same_width_whatever_the_count() {
        for doors in [3usize, 4, 10, 100] {
            let first = slot(0, doors).x - width(doors) * 0.5;
            let last = slot(doors - 1, doors).x + width(doors) * 0.5;

            // the outer gaps are half a gap each, so the doors themselves stop
            // just inside the span
            assert!(first > -SPAN * 0.5 - 1e-5, "{} doors spill left", doors);
            assert!(last < SPAN * 0.5 + 1e-5, "{} doors spill right", doors);
            assert!(last - first > SPAN * 0.8, "{} doors are huddled", doors);
        }
    }

    #[test]
    fn more_doors_are_thinner_doors() {
        assert!(width(3) > width(10));
        assert!(width(10) > width(100));
        assert!(width(100) > 0.0);
    }

    #[test]
    fn doors_do_not_overlap() {
        for doors in [3usize, 4, 10, 100] {
            for door in 1..doors {
                let left = slot(door - 1, doors).x + width(doors) * 0.5;
                let right = slot(door, doors).x - width(doors) * 0.5;
                assert!(right > left, "{} doors touch at {}", doors, door);
            }
        }
    }

    #[test]
    fn a_shut_door_fills_its_slot() {
        let (middle, turn) = panel(1, 3, 0.0);

        assert!((middle.x - slot(1, 3).x).abs() < 1e-5);
        assert!((middle.z).abs() < 1e-5);
        assert!(turn.to_axis_angle().1.abs() < 1e-5);
    }

    #[test]
    fn an_open_door_swings_out_of_the_way() {
        let shut = panel(1, 3, 0.0).0;
        let open = panel(1, 3, 1.0).0;

        // it comes forward and across, and it leaves the slot it was filling
        assert!(open.z > shut.z + 0.1, "it did not come forward");
        assert!((open.x - shut.x).abs() > 0.05, "it did not move across");
    }

    #[test]
    fn a_door_hangs_off_its_hinge_the_whole_way() {
        let hinge = hinge(2, 5);

        for step in 0..=10 {
            let open = step as f32 / 10.0;
            let (middle, _) = panel(2, 5, open);
            let reach = (middle - Vec3::Y * TALL * 0.5) - hinge;

            assert!(
                (reach.length() - width(5) * 0.5).abs() < 1e-4,
                "the panel came off its hinge at {}",
                open
            );
        }
    }

    #[test]
    fn the_prize_stands_in_the_open_part_of_the_alcove() {
        // at every door count, because the ball shrinks with the doors and a
        // filled recess hid it the moment it stopped poking out of the doorway
        for doors in [3usize, 5, 10, 25, 100] {
            let (back, thick) = recess(0, doors);
            let prize = prize_at(0, doors);

            assert!(
                prize.z > back.z + thick.z * 0.5,
                "{} doors buries the prize in the back wall",
                doors
            );
            assert!(prize.z < 0.0, "{} doors puts it out in the room", doors);
            assert!(prize.y > 0.0 && prize.y < TALL);
            assert!((prize.x - back.x).abs() < width(doors) * 0.5);
        }
    }

    #[test]
    fn the_alcove_is_open_between_the_door_and_its_back() {
        let (back, thick) = recess(0, 3);
        let open = (0.0 - (back.z + thick.z * 0.5)).abs();

        assert!(open > DEEP * 0.8, "there is nowhere to stand a prize");
    }

    #[test]
    fn the_camera_stands_back_far_enough_to_see_the_ends() {
        let eye = eye();
        let end = slot(0, 3).x - width(3) * 0.5;

        // the whole row has to fall inside a sixty degree view, which is about
        // what the engine's camera has
        let across = (end.abs() / eye.z).atan() * 2.0;
        assert!(across < 1.0, "the row is {} radians wide", across);
        assert!(eye.y > TALL * 0.5, "the camera is below the doors");
    }

    #[test]
    fn a_pier_fills_the_gap_between_two_doors() {
        for doors in [3usize, 4, 10] {
            let left = slot(0, doors).x + width(doors) * 0.5;
            let right = slot(1, doors).x - width(doors) * 0.5;
            let (middle, size) = pier(1, doors);

            assert!((middle.x - (left + right) * 0.5).abs() < 1e-4);
            assert!((size.x - (right - left)).abs() < 1e-4, "{} doors", doors);
        }
    }

    #[test]
    fn there_is_a_pier_at_each_end() {
        let doors = 3;
        assert_eq!(piers(doors), 4);

        let first = pier(0, doors).0.x;
        let last = pier(doors, doors).0.x;

        assert!(first < slot(0, doors).x);
        assert!(last > slot(doors - 1, doors).x);
    }

    #[test]
    fn a_pier_is_as_deep_as_the_alcove_it_stands_beside() {
        // the piers are the alcove's sides, so they run its whole depth. The
        // back wall is only the back.
        let (_, size) = pier(1, 3);

        assert!((size.z - DEEP).abs() < 1e-5);
    }

    #[test]
    fn the_lintel_reaches_past_both_ends() {
        let (middle, size) = lintel(3);

        assert!((middle.x).abs() < 1e-5);
        assert!(size.x > SPAN);
        assert!(middle.y > TALL);
    }

    #[test]
    fn a_mark_lies_on_the_floor_in_front_of_its_door() {
        let (at, size) = mark(1, 3);

        assert!((at.x - slot(1, 3).x).abs() < 1e-5);
        assert!(at.z > 0.0, "it is behind the door");
        assert!(at.y - size.y * 0.5 >= 0.0, "it is sunk into the floor");
        assert!(at.y < TALL * 0.1, "it is floating");
    }

    #[test]
    fn a_tab_sits_on_the_front_of_the_lintel_over_its_door() {
        let (at, size) = tab(1, 3);
        let (over, lintel) = lintel(3);

        assert!((at.x - slot(1, 3).x).abs() < 1e-5);
        assert!(
            (at.y - over.y).abs() < lintel.y * 0.5,
            "it is off the lintel"
        );
        assert!(at.z > over.z + lintel.z * 0.5, "it is buried in the lintel");
        assert!(size.x < width(3), "it is as wide as the door");
        assert!(size.x > width(3) * 0.2, "it is a hairline");
    }

    #[test]
    fn the_tab_and_the_light_are_at_opposite_ends_of_the_door() {
        let (tab_at, _) = tab(1, 3);
        let (mark_at, _) = mark(1, 3);

        // one over the row and one on the floor, so neither can hide the other
        assert!(tab_at.y > TALL);
        assert!(mark_at.y < TALL * 0.1);
    }

    #[test]
    fn marks_on_neighbouring_doors_do_not_touch() {
        for doors in [3usize, 5, 10] {
            let left = mark(0, doors);
            let right = mark(1, doors);

            let gap = (right.0.x - right.1.x * 0.5) - (left.0.x + left.1.x * 0.5);
            assert!(gap > 0.0, "{} doors run their marks together", doors);
        }
    }

    #[test]
    fn nothing_over_the_row_vanishes_as_the_doors_multiply() {
        for doors in [3usize, 5, 10, 25, 100] {
            let (_, lintel) = lintel(doors);
            let (_, tab) = tab(doors - 1, doors);
            let (_, mark) = mark(0, doors);

            assert!(lintel.y > TALL * 0.05, "{} doors flatten the lintel", doors);
            assert!(tab.x > 0.05 && tab.y > 0.05, "{} doors lose the tab", doors);
            assert!(mark.x > 0.1, "{} doors thin the light to a scratch", doors);
        }
    }

    #[test]
    fn a_tab_still_sits_over_its_own_door() {
        // widened at high counts, so it has to stay nearer its own door than
        // either neighbour
        for doors in [3usize, 10, 25] {
            for door in [0, doors / 2, doors - 1] {
                let (at, _) = tab(door, doors);
                assert!((at.x - slot(door, doors).x).abs() < 1e-5);
            }
        }
    }
}
