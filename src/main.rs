//! monty: three doors, and a host whose rule decides the odds. See `specs/`.

mod bars;
mod doors;
mod host;
mod round;
mod stage;
mod tally;

use blitzkit::camera::Camera;
use blitzkit::geometry::Geometry;
use blitzkit::keyboard::{KeyboardInput, KeyboardKey, KeyboardKeyState};
use blitzkit::lighting::PointLight;
use blitzkit::mesh::{MeshData, Transform};
use blitzkit::notice;
use blitzkit::renderer::render_text::{RenderText, TextRenderer};
use blitzkit::renderer::scene::{MeshId, Scene};
use blitzkit::renderer::Renderer;
use blitzkit::sound::SoundSystem;
use blitzkit::{start, Game};
use glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4};

use host::Host;
use stage::{Phase, Said, Stage};

/// Whether this run is only here to be photographed, for `refresh-screenshots`
/// in the project above.
///
/// An opening frame is three shut doors and two empty bars, which says nothing.
/// The picture wants a host chosen, a batch run and a round part way through.
fn staged() -> bool {
    std::env::args().any(|arg| arg == "--screenshot")
}

const FLOOR: Vec4 = vec4(0.10, 0.10, 0.13, 1.0);
const WALL: Vec4 = vec4(0.17, 0.15, 0.18, 1.0);
/// The wall the doors are set into, lighter than the recess behind them so a
/// gap between two doors reads as wall rather than as more dark.
const PIER: Vec4 = vec4(0.30, 0.26, 0.28, 1.0);
const DOOR: Vec4 = vec4(0.52, 0.34, 0.22, 1.0);
const DOOR_ON: Vec4 = vec4(0.88, 0.66, 0.34, 1.0);
const DOOR_PICKED: Vec4 = vec4(0.74, 0.50, 0.26, 1.0);
const RECESS: Vec4 = vec4(0.025, 0.025, 0.035, 1.0);
/// Past white, so it glows and so it throws nothing. See blitzkit's spec 0039.
const PRIZE: Vec4 = vec4(6.0, 5.2, 2.4, 1.0);

/// The two marks on the floor: where the cursor is, and which door you took.
///
/// Different hues rather than two browns, because three shades of door was the
/// whole trouble. Both past white, so they glow and throw nothing.
const MARK_ON: Vec4 = vec4(3.4, 2.8, 1.3, 1.0);
const MARK_PICKED: Vec4 = vec4(0.9, 1.8, 3.2, 1.0);

const MATTE: f32 = 48.0;

/// How wide and deep the room is, past the row of doors.
const ROOM: f32 = 24.0;

const HUD_LEFT: f32 = 28.0;
const HUD_TOP: f32 = 24.0;
const HUD_APART: f32 = 22.0;
const HUD_SIZE: f32 = 14.0;

struct Monty {
    stage: Stage,
    panel: Option<MeshId>,
    ball: Option<MeshId>,
    window: Vec2,
    held: [bool; 5],
    staged: bool,
    posed: bool,
    quitting: bool,
}

impl Monty {
    fn new() -> Self {
        Self {
            stage: Stage::new(0x4D4F4E54),
            panel: None,
            ball: None,
            window: vec2(900.0, 600.0),
            held: [false; 5],
            staged: staged(),
            posed: false,
            quitting: false,
        }
    }

    /// Plays for the camera: a batch on the board and a round part way through,
    /// so the picture shows what the game is rather than what it looks like
    /// before anyone has touched it. See `refresh-screenshots`.
    fn pose(&mut self) {
        self.stage.run_a_batch();

        // a round carried all the way through, because the prize behind an open
        // door is the thing a still picture can show and an offer is not
        self.stage.on = 1;
        self.stage.take();
        if self.stage.phase == Phase::Offered {
            self.stage.move_by(1);
            self.stage.take();
        }
        self.stage.tick(doors::SWING_SECONDS);
    }

    fn readout(&self) -> Vec<String> {
        let tally = &self.stage.tally;
        // what the host just did, said every time. A round against the mean
        // host is otherwise one keypress and every door open, which reads as
        // the game breaking rather than as him refusing.
        let saying = match self.stage.said {
            Said::Pick => "Arrows move the light. Enter takes the door it is on.",
            Said::Offered => {
                "The blue tab is your door. Move the light across to switch, or take it again to stay."
            }
            Said::GaveItAway => "He opened the prize. That round does not count. Enter deals another.",
            Said::Won => "Yours. Enter deals another.",
            Said::Lost => "Not yours. Enter deals another.",
        };

        vec![
            format!(
                "host: {}   ({} doors, {}, {} thrown away)",
                Host::ALL
                    .iter()
                    .map(|host| if *host == self.stage.host {
                        format!("[{}]", named(*host))
                    } else {
                        String::from(named(*host))
                    })
                    .collect::<Vec<String>>()
                    .join(" "),
                self.stage.doors,
                some(tally.rounds(), "round"),
                tally.thrown_away
            ),
            String::from(
                "The tick on each bar is an even chance. A few rounds say nothing; b runs a thousand.",
            ),
            String::from(saying),
            String::from("1 2 3 pick the host, up and down change the doors, b runs a thousand"),
            String::from("arrows move, enter takes, escape quits"),
        ]
    }
}

/// A count and its noun, with the s only when it earns one.
fn some(many: u32, noun: &str) -> String {
    if many == 1 {
        format!("{} {}", many, noun)
    } else {
        format!("{} {}s", many, noun)
    }
}

fn named(host: Host) -> &'static str {
    match host {
        Host::Knowing => "knowing",
        Host::Careless => "careless",
        Host::Mean => "mean",
    }
}

impl Default for Monty {
    fn default() -> Self {
        Self::new()
    }
}

impl Game for Monty {
    fn load(&mut self, renderer: &mut Renderer) {
        self.panel = Some(renderer.add_mesh(&MeshData::cube()));
        self.ball = Some(renderer.add_mesh(&MeshData::sphere(20, 14)));
    }

    fn initialize(
        &mut self,
        _geometry: &mut Geometry,
        _text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
        window_size: (f32, f32),
    ) {
        self.resized(window_size);
    }

    fn resized(&mut self, window_size: (f32, f32)) {
        self.window = vec2(window_size.0, window_size.1);
    }

    fn update(
        &mut self,
        dt: f32,
        geometry: &mut Geometry,
        text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
    ) {
        if self.staged && !self.posed {
            self.posed = true;
            self.pose();
        }

        self.stage.tick(dt);

        geometry.reset();
        text_renderer.reset();

        for quad in bars::of(&self.stage.tally, self.window) {
            geometry.push_quad(&quad);
        }

        // the bars say what they are, since two unexplained colours are two
        // unexplained colours
        let mut named_bars: Vec<RenderText> = Vec::new();
        for (n, name) in bars::NAMES.iter().enumerate() {
            let (at, ink) = bars::name_at(n);
            named_bars.push(RenderText {
                position: at,
                color: ink,
                size: bars::LABEL_SIZE,
                text: String::from(*name),
                ..Default::default()
            });
            named_bars.push(RenderText {
                position: bars::figure_at(n),
                color: ink,
                size: bars::LABEL_SIZE,
                text: bars::figure(&self.stage.tally, n),
                ..Default::default()
            });
        }

        let lines: Vec<RenderText> = self
            .readout()
            .into_iter()
            .enumerate()
            .map(|(n, text)| RenderText {
                position: vec2(HUD_LEFT, HUD_TOP + n as f32 * HUD_APART),
                color: vec4(1.0, 1.0, 1.0, 1.0),
                size: HUD_SIZE,
                text,
                ..Default::default()
            })
            .collect();

        // the readout goes on a panel, so it reads over the room rather than
        // into it. See blitzkit's spec 0038.
        if let Some(frame) = notice::framing_all(&lines) {
            for quad in frame.iter() {
                geometry.push_quad(quad);
            }
        }
        for line in lines.into_iter().chain(named_bars) {
            text_renderer.push_render_text(line);
        }
    }

    fn process_keyboard(&mut self, input: KeyboardInput) {
        let down = input.state == KeyboardKeyState::Pressed;
        let slot = match input.key {
            KeyboardKey::Left => Some(0),
            KeyboardKey::Right => Some(1),
            KeyboardKey::Up => Some(2),
            KeyboardKey::Down => Some(3),
            KeyboardKey::Return => Some(4),
            _ => None,
        };

        if let Some(slot) = slot {
            // one press per press, so a held arrow does not run down the row
            let was = self.held[slot];
            self.held[slot] = down;
            if !down || was {
                return;
            }
        } else if !down {
            return;
        }

        match input.key {
            KeyboardKey::Left => self.stage.move_by(-1),
            KeyboardKey::Right => self.stage.move_by(1),
            KeyboardKey::Up => self.stage.step_doors(1),
            KeyboardKey::Down => self.stage.step_doors(-1),
            KeyboardKey::Return => {
                if self.stage.settled() {
                    self.stage.take()
                }
            }
            KeyboardKey::Key1 => self.stage.set_host(Host::Knowing),
            KeyboardKey::Key2 => self.stage.set_host(Host::Careless),
            KeyboardKey::Key3 => self.stage.set_host(Host::Mean),
            KeyboardKey::B => self.stage.run_a_batch(),
            KeyboardKey::Escape => self.quitting = true,
            _ => (),
        }
    }

    fn draw(&mut self, scene: &mut Scene, camera: &mut Camera) {
        let (panel, ball) = match (self.panel, self.ball) {
            (Some(panel), Some(ball)) => (panel, ball),
            _ => return,
        };

        camera.position = doors::eye();
        camera.target = doors::looked_at();

        scene.light.direction = vec3(-0.25, -0.9, -0.35).normalize();
        scene.light.intensity = 0.75;
        scene.light.ambient = Vec3::splat(0.12);

        // the floor and the wall the doors are set into
        // centred on the row rather than behind it, so there is floor under the
        // camera as well as under the doors
        scene.push_material(
            panel,
            &Transform::at(vec3(0.0, -0.05, 0.0)).with_scale(vec3(ROOM, 0.1, ROOM)),
            FLOOR,
            MATTE,
        );
        scene.push_material(
            panel,
            &Transform::at(vec3(0.0, doors::TALL * 1.6, -doors::DEEP - 0.1)).with_scale(vec3(
                ROOM,
                doors::TALL * 3.2,
                0.2,
            )),
            WALL,
            MATTE,
        );

        let count = self.stage.doors;

        // the wall the doors are set into, built before them so a door swinging
        // open has something to swing out of
        for pier in 0..doors::piers(count) {
            let (middle, size) = doors::pier(pier, count);
            scene.push_material(panel, &Transform::at(middle).with_scale(size), PIER, MATTE);
        }
        let (over, size) = doors::lintel(count);
        scene.push_material(panel, &Transform::at(over).with_scale(size), PIER, MATTE);

        for door in 0..count {
            let (middle, size) = doors::recess(door, count);
            scene.push_material(
                panel,
                &Transform::at(middle).with_scale(size),
                RECESS,
                MATTE,
            );

            let open = self.stage.open_by(door);
            let (at, turn) = doors::panel(door, count, open);
            let look = if door == self.stage.on && self.stage.phase != Phase::Shown {
                DOOR_ON
            } else if door == self.stage.picked && self.stage.phase != Phase::Picking {
                DOOR_PICKED
            } else {
                DOOR
            };

            scene.push_material(
                panel,
                &Transform::at(at).with_rotation(turn).with_scale(vec3(
                    doors::width(count),
                    doors::TALL,
                    doors::THICK,
                )),
                look,
                MATTE,
            );
        }

        // where the cursor is, and which door was taken. On the floor rather
        // than on the doors, which swing out of the room when they open.
        // the light on the floor is where the cursor is, the tab over the row is
        // the door you took. At opposite ends of the door, so neither hides the
        // other and a door that is both reads as both.
        if self.stage.phase != Phase::Shown {
            let (at, size) = doors::mark(self.stage.on, count);
            scene.push_material(panel, &Transform::at(at).with_scale(size), MARK_ON, MATTE);
        }
        if self.stage.phase != Phase::Picking {
            let (at, size) = doors::tab(self.stage.picked, count);
            scene.push_material(
                panel,
                &Transform::at(at).with_scale(size),
                MARK_PICKED,
                MATTE,
            );
        }

        // the prize, lit from inside its own recess. It is drawn past white, so
        // it glows and casts nothing.
        let prize = doors::prize_at(self.stage.prize, count);
        if self.stage.open_by(self.stage.prize) > 0.3 {
            let size = (doors::width(count) * 0.45).min(doors::TALL * 0.22);
            scene.push_material(
                ball,
                &Transform::at(prize).with_scale(Vec3::splat(size * 2.0)),
                PRIZE,
                MATTE,
            );
            scene.push_light(PointLight::new(
                prize + Vec3::Z * 0.1,
                vec3(1.0, 0.88, 0.5),
                1.4,
                2.2,
            ));
        }
    }

    fn focus_changed(&mut self, _focus: bool) {}

    fn is_quitting(&self) -> bool {
        self.quitting
    }
}

fn main() {
    start("monty", Box::new(Monty::new()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use blitzkit::keyboard::KeyboardInput;

    fn press(game: &mut Monty, key: KeyboardKey) {
        game.process_keyboard(KeyboardInput {
            key,
            state: KeyboardKeyState::Pressed,
            repeat: false,
        });
    }

    #[test]
    fn escape_quits() {
        // the readout promises it does, and for a while it did not: the key was
        // never handled and is_quitting was a hard coded false
        let mut game = Monty::new();
        assert!(!game.is_quitting());

        press(&mut game, KeyboardKey::Escape);

        assert!(game.is_quitting());
    }

    #[test]
    fn nothing_else_quits() {
        let mut game = Monty::new();

        for key in [
            KeyboardKey::Left,
            KeyboardKey::Right,
            KeyboardKey::Up,
            KeyboardKey::Down,
            KeyboardKey::Return,
            KeyboardKey::B,
            KeyboardKey::Key1,
        ] {
            press(&mut game, key);
        }

        assert!(!game.is_quitting());
    }
}
