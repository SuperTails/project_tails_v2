use crate::animation::Animation;
use crate::physical::{Physical, Vector2};
use crate::renderable::Renderable;
use sdl2::render::{Canvas, RenderTarget};
use std::time::Duration;

pub struct Player {
    position: Vector2,
    velocity: Vector2,

    // TODO: Physics
    // physics: Physics

    // TODO: Multiple animations
    animation: Animation,
}

const MAX_FALL_SPEED: f64 = 16.0;
const GRAVITY: f64 = 0.21875;

impl Player {
    pub fn new() -> Player {
        Player {
            position: Vector2 { x: 0.0, y: 0.0 },
            velocity: Vector2 { x: 0.1, y: 0.1 },
            animation: Animation::new("Tails/Idle".to_string(), 5, Duration::from_millis(200)),
        }
    }

    pub fn update(&mut self) {
        self.position.x += self.velocity.x;
        self.position.y += self.velocity.y;

        self.velocity.y += GRAVITY;
        if self.velocity.y >= MAX_FALL_SPEED {
            self.velocity.y = MAX_FALL_SPEED;
        }

        self.animation.update();

        if self.position.y >= 500.0 {
            self.position.y = 500.0;
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
    fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>) -> Result<(), String> {
        (self.position, &self.animation).render(canvas)
    }
}
