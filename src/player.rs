use crate::animation::Animation;
use crate::physical::{Physical, Vector2};
use crate::renderable::Renderable;
use crate::block::TerrainGetter;
use crate::input;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, RenderTarget};
use std::time::Duration;
use crate::camera::Camera;

pub struct Player {
    position: Vector2,
    velocity: Vector2,

    // TODO: Multiple animations
    animation: Animation,
}

const MAX_FALL_SPEED: f64 = 16.0;
const GRAVITY: f64 = 0.21875;

impl Player {
    pub fn new() -> Player {
        Player {
            position: Vector2 { x: 0.0, y: 0.0 },
            velocity: Vector2 { x: 0.5, y: 0.1 },
            animation: Animation::new("Tails/Idle".to_string(), 5, Duration::from_millis(200)),
        }
    }

    pub fn update(&mut self, getter: &TerrainGetter) {
        self.position.x += self.velocity.x;
        self.position.y += self.velocity.y;

        self.velocity.y += GRAVITY;
        if self.velocity.y >= MAX_FALL_SPEED {
            self.velocity.y = MAX_FALL_SPEED;
        }

        if input::key_held(Keycode::D) {
            self.velocity.x += 0.05;
        } else if input::key_held(Keycode::A) {
            self.velocity.x -= 0.05;
        } else if self.velocity.x.abs() >= 0.02 {
            self.velocity.x -= self.velocity.x.signum() * 0.02;
        } else {
            self.velocity.x = 0.0;
        }

        if input::key_pressed(Keycode::Space) {
            self.velocity.y = -5.0;
        }

        self.animation.update();

        let (ground, _angle) = find_ground_height(self.get_position(), 5.0, 10.0, getter).map(|(g, a)| (g as f64, a)).unwrap_or((std::f64::INFINITY, 0));

        let floor = if ground < 1000.0 {
            ground
        } else {
            1000.0
        };

        if self.position.y >= floor {
            self.position.y = floor;
            if self.velocity.y > 0.0 {
                self.velocity.y = 0.0;
            }
        }
    }
}

impl Physical for Player {
    fn get_position(&self) -> Vector2 {
        self.position
    }
}

impl Renderable for Player {
    fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>, camera: &Camera) -> Result<(), String> {
        let mut pos = self.position;
        pos.y -= self.animation.height() as f64;
        (pos, &self.animation).render(canvas, camera)
    }
}

/// # Returns
/// The y position and angle of the tile found,
/// if any.
pub fn find_ground_height(position: Vector2, x_radius: f64, _y_radius: f64, getter: &TerrainGetter) -> Option<(u32, u8)> {
    let start_coord = (position.x - x_radius) as u32;
    let end_coord = (position.x + x_radius) as u32;

    let results: Vec<(u32, u8)> = (start_coord..=end_coord)
        .filter_map(|c| collide_line((c as i32, position.y as i32), getter))
        .collect();

    results.iter().min_by_key(|(height, _angle)| height).copied()
}

fn collide_line(position: (i32, i32), getter: &TerrainGetter) -> Option<(u32, u8)> {
    let mut result = None;
    let mut y = position.1 + 17;
    while getter.is_occupied(position.0, y - 1, 0) || (result.is_none() && y >= position.1) {
        result = Some(y);
        y -= 1;
    }

    let (data, angle) = getter.tile_at(position.0, result?, 0)?;
    let top = y as u32 + 16 - data[(position.0 % 8) as usize];
    Some((top, angle))
}